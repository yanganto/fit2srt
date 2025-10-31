use chrono::{DateTime, Local, NaiveTime, TimeDelta, TimeZone, Timelike};
use fitparser::Value;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;

use crate::summary::Summary;

type SrtString = String;

#[derive(Copy, Clone)]
pub struct SrtGenerator {
    field: &'static str,
    tick: f64,

    // These are used when a video recording before under water
    start_time_secs: u32,
    after_time_secs: u32,
    before_time_secs: u32,
}

impl Default for SrtGenerator {
    fn default() -> Self {
        Self {
            field: "depth",
            tick: 0.1,
            start_time_secs: 0,
            after_time_secs: 0,
            before_time_secs: 0,
        }
    }
}

impl SrtGenerator {
    pub fn after_hour(&mut self, h: u32) {
        self.after_time_secs += h * 60 * 60;
    }

    pub fn after_minute(&mut self, m: u32) {
        self.after_time_secs += m * 60;
    }

    pub fn after_second(&mut self, s: u32) {
        self.after_time_secs += s;
    }

    pub fn before_hour(&mut self, h: u32) {
        self.before_time_secs += h * 60 * 60;
    }

    pub fn before_minute(&mut self, m: u32) {
        self.before_time_secs += m * 60;
    }

    pub fn before_second(&mut self, s: u32) {
        self.before_time_secs += s;
    }

    pub fn starting_hour(&mut self, h: u32) {
        self.start_time_secs += h * 60 * 60;
    }

    pub fn starting_minute(&mut self, m: u32) {
        self.start_time_secs += m * 60;
    }

    pub fn starting_second(&mut self, s: u32) {
        self.start_time_secs += s;
    }

    pub fn open<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<SrtIter, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let mut fp = File::open(path)?;
        let mut start_time: Option<DateTime<Local>> = None;
        let mut previous_value = f64::NAN;
        let mut data = VecDeque::new();
        let mut unit = "".to_string();
        let mut before = true;
        let mut previous_time = None;
        let mut summary = Summary::default();

        for record in fitparser::from_reader(&mut fp)? {
            let mut timestamp: Option<DateTime<Local>> = None;
            let mut value = 0f64;
            let mut has_depth = false;
            match record.kind() {
                fitparser::profile::field_types::MesgNum::DiveSummary => {
                    for field in record.fields() {
                        match field.name() {
                            "avg_depth" => {
                                if let fitparser::Value::Float64(d) = field.value() {
                                    summary.avg_depth = Some(*d);
                                    summary.set_unit(field.units())?
                                }
                            }
                            "max_depth" => {
                                if let fitparser::Value::Float64(d) = field.value() {
                                    summary.max_depth = Some(*d);
                                    summary.set_unit(field.units())?
                                }
                            }
                            _ => (),
                        }
                    }
                }
                fitparser::profile::field_types::MesgNum::Session => {
                    for field in record.fields() {
                        match field.name() {
                            "start_position_lat" | "end_position_lat" => {
                                if let fitparser::Value::SInt32(lat) = field.value() {
                                    if let Some(old_lat) = summary.location.0 {
                                        let avg_lat = old_lat / 2 + *lat / 2;
                                        summary.location.0 = Some(avg_lat);
                                    } else {
                                        summary.location.0 = Some(*lat);
                                    }
                                }
                            }
                            "start_position_long" | "end_position_long" => {
                                if let fitparser::Value::SInt32(long) = field.value() {
                                    if let Some(old_long) = summary.location.1 {
                                        let avg_long = old_long / 2 + *long / 2;
                                        summary.location.1 = Some(avg_long);
                                    } else {
                                        summary.location.1 = Some(*long);
                                    }
                                }
                            }
                            "total_elapsed_time" => {
                                if let fitparser::Value::Float64(t) = field.value() {
                                    summary.time = *t
                                }
                            }
                            "avg_temperature" => {
                                if let fitparser::Value::SInt8(t) = field.value() {
                                    summary.avg_temperature = Some(*t);
                                    summary.set_unit(field.units())?;
                                }
                            }
                            "min_temperature" => {
                                if let fitparser::Value::SInt8(t) = field.value() {
                                    summary.min_temperature = Some(*t);
                                    summary.set_unit(field.units())?
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => {
                    for field in record.fields() {
                        if field.name() == "timestamp" {
                            if let Value::Timestamp(ts) = field.value() {
                                if before
                                    && self.after_time_secs
                                        >= ts.hour() * 60 * 60 + ts.minute() * 60 + ts.second()
                                {
                                    // TODO DEBUG print here
                                    // println!("skip {}:{}:{}", ts.hour(), ts.minute(), ts.second());
                                    continue;
                                } else if self.before_time_secs > 0
                                    && self.before_time_secs
                                        < ts.hour() * 60 * 60 + ts.minute() * 60 + ts.second()
                                {
                                    // TODO DEBUG print here
                                    // println!("skip record after {}:{}:{}", ts.hour(), ts.minute(), ts.second());
                                    break;
                                } else {
                                    before = false;
                                    timestamp = Some(*ts);
                                }
                            }
                        } else if field.name() == self.field {
                            if let Value::Float64(v) = field.value() {
                                has_depth = true;
                                value = *v;
                            }
                            if unit.is_empty() {
                                unit = field.units().to_string();
                            }
                        }
                    }
                    if timestamp.is_some() && has_depth {
                        if let Some(start_time) = start_time {
                            let rounded_value = (value / self.tick).round() * self.tick;
                            if (rounded_value - previous_value).abs() > self.tick {
                                data.push_back((
                                    timestamp.unwrap() - start_time,
                                    format!("{rounded_value:.1}{unit}"),
                                ));
                                previous_value = rounded_value;
                            }
                        } else {
                            if self.start_time_secs != 0 {
                                let date =
                                    unsafe { timestamp.as_ref().unwrap_unchecked().date_naive() };
                                let time = NaiveTime::from_num_seconds_from_midnight_opt(
                                    self.start_time_secs,
                                    0,
                                )
                                .expect("Invalid start time!");
                                let naive_datetime = date.and_time(time);
                                let st = Local.from_local_datetime(&naive_datetime).unwrap();
                                previous_time = unsafe { Some(timestamp.unwrap_unchecked() - st) };
                                start_time = Some(st);
                            } else {
                                start_time = timestamp;
                            }
                            previous_value = value;
                        }
                    }
                }
            }
        }

        Ok(SrtIter {
            summary,
            count: 0,
            data,
            previous_time: previous_time.unwrap_or_default(),
            previous_iter_previours_time: TimeDelta::default(),
        })
    }

    pub fn concat<P: AsRef<Path>>(
        self,
        previous_iter_count: usize,
        previous_iter_timedelta: TimeDelta,
        path: P,
    ) -> Result<SrtIter, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let mut it = self.open(path)?;
        it.count = previous_iter_count;
        it.previous_iter_previours_time = previous_iter_timedelta;
        Ok(it)
    }
}

pub struct SrtIter {
    pub summary: Summary,
    pub count: usize,
    previous_time: TimeDelta,
    data: VecDeque<(TimeDelta, String)>,
    previous_iter_previours_time: TimeDelta,
}

impl SrtIter {
    pub fn previous_time(&self) -> TimeDelta {
        self.previous_time.clone()
    }
}

impl std::iter::Iterator for SrtIter {
    type Item = (usize, TimeDelta, SrtString);
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        let time = self.previous_time + self.previous_iter_previours_time;
        let previous_time_str = delta_srt_format(&time);
        self.data.pop_front().map(|i| {
            self.previous_time = i.0;
            (
                self.count,
                time,
                format!(
                    "{}\n{} --> {}\n{}",
                    self.count,
                    previous_time_str,
                    delta_srt_format(&i.0),
                    i.1
                ),
            )
        })
    }
}

pub fn delta_srt_format(delta: &TimeDelta) -> String {
    format!(
        "{:0>2}:{:0>2}:{:0>2},{:0>3}",
        delta.num_hours(),
        delta.num_minutes() % 60,
        delta.num_seconds() % 60,
        delta.num_milliseconds() % 1000
    )
}

#[test]
fn srt_time_string() {
    assert_eq!("00:00:00,000", delta_srt_format(&TimeDelta::default()));
}

#[test]
fn parse_garmin_g1() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    let mut iter = SrtGenerator::default().open("../assets/garmin_g1.fit")?;
    let (_idx, _time_delta, srt) = iter.next().unwrap();
    assert_eq!(srt, "1\n00:00:00,000 --> 00:00:01,000\n1.5m".to_string());
    let (_idx, _time_delta, srt) = iter.next().unwrap();
    assert_eq!(srt, "2\n00:00:01,000 --> 00:00:02,000\n1.7m".to_string());
    let (_idx, _time_delta, srt) = iter.next().unwrap();
    assert_eq!(srt, "3\n00:00:02,000 --> 00:00:03,000\n2.0m".to_string());
    let (_idx, _time_delta, srt) = iter.next().unwrap();
    assert_eq!(srt, "4\n00:00:03,000 --> 00:00:06,000\n1.8m".to_string());
    Ok(())
}
