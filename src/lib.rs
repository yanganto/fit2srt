use chrono::{DateTime, Local, TimeDelta};
use fitparser::{FitDataField, FitDataRecord, Value};
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;

type SrtString = String;

pub struct SrtGenerator {
    field: &'static str,
    tick: f64,
    unit: Option<String>,
}

impl Default for SrtGenerator {
    fn default() -> Self {
        Self {
            field: "depth",
            tick: 0.1,
            unit: Some("M".to_string()),
        }
    }
}

impl SrtGenerator {
    fn open<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<SrtIter, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let mut fp = File::open(path)?;
        let mut start_time: Option<DateTime<Local>> = None;
        let mut previous_value = f64::NAN;
        let mut data = VecDeque::new();
        let mut unit = self.unit;

        for record in fitparser::from_reader(&mut fp)? {
            let mut timestamp: Option<DateTime<Local>> = None;
            let mut value = 0f64;
            let mut has_depth = false;
            for field in record.fields() {
                if field.name() == "timestamp" {
                    if let Value::Timestamp(ts) = field.value() {
                        timestamp = Some(*ts);
                    }
                } else if field.name() == "depth" {
                    if let Value::Float64(v) = field.value() {
                        has_depth = true;
                        value = *v;
                    }
                    if unit.is_none() {
                        unit = Some(field.units().to_string())
                    }
                }
            }
            if timestamp.is_some() && has_depth {
                if let Some(start_time) = start_time {
                    let rounded_value = (value / self.tick).round() * self.tick;
                    if (rounded_value - previous_value).abs() > self.tick {
                        data.push_back((
                            timestamp.unwrap() - start_time,
                            format!("{rounded_value:.1}{}", unit.as_ref().unwrap()),
                        ));
                        previous_value = rounded_value;
                    }
                } else {
                    start_time = timestamp;
                    previous_value = value;
                }
            }
        }

        Ok(SrtIter {
            count: 0,
            data,
            previous_time: start_time.unwrap() - start_time.unwrap(), // TODO
        })
    }
}

struct SrtIter {
    count: usize,
    data: VecDeque<(TimeDelta, String)>,
    previous_time: TimeDelta,
}

impl std::iter::Iterator for SrtIter {
    type Item = SrtString;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        let previous_time_str = delta_srt_format(&self.previous_time);
        self.data.pop_front().map(|i| {
            self.previous_time = i.0;
            format!(
                "{}\n{} --> {}\n{}",
                self.count,
                previous_time_str,
                delta_srt_format(&i.0),
                i.1
            )
        })
    }
}

fn delta_srt_format(delta: &TimeDelta) -> String {
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
    let mut generator = SrtGenerator::default().open("asset/garmin_g1.fit")?;
    assert_eq!(
        generator.next(),
        Some("1\n00:00:00,000 --> 00:00:01,000\n1.5M".to_string())
    );
    assert_eq!(
        generator.next(),
        Some("2\n00:00:01,000 --> 00:00:02,000\n1.7M".to_string())
    );
    assert_eq!(
        generator.next(),
        Some("3\n00:00:02,000 --> 00:00:03,000\n2.0M".to_string())
    );
    assert_eq!(
        generator.next(),
        Some("4\n00:00:03,000 --> 00:00:06,000\n1.8M".to_string())
    );
    Ok(())
}
