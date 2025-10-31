#![allow(special_module_name)]
use clap::Parser;
use std::num::ParseIntError;
use std::path::PathBuf;

use fit2srt_core::SrtGenerator;
use fit2srt_core::Summary;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Generate subtitles after 00:00:00
    #[arg(short, long)]
    after: Option<String>,

    /// Generate subtitles before 00:00:00
    #[arg(short, long)]
    before: Option<String>,

    /// Generate subtitles start from 00:00:00
    /// This option is used if you recording a video before the dive computer started
    /// If you record after dive computer started, you do not need this.
    #[arg(short, long)]
    start: Option<String>,

    /// Generate dive summary in the end of srt
    #[arg(short, long)]
    no_summary: bool,

    fit_files: Vec<PathBuf>,
}

fn time_to_vec(time_str: &str) -> Result<Vec<u32>, ParseIntError> {
    if time_str.contains(':') {
        let mut err = None;
        let time = time_str
            .split(':')
            .map(|s| match s.parse() {
                Ok(i) => i,
                Err(e) => {
                    err = Some(e);
                    0
                }
            })
            .collect();
        if let Some(e) = err {
            Err(e)
        } else {
            Ok(time)
        }
    } else {
        Ok(vec![
            time_str[0..2].parse()?,
            time_str[2..4].parse()?,
            time_str[4..6].parse()?,
        ])
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    let mut generator = SrtGenerator::default();

    if let Some(after_str) = cli.after {
        let mut valid = true;
        let after_time: Vec<u32> = time_to_vec(&after_str)?;

        if after_time.len() == 3 && after_time[0] <= 24 && after_time[1] < 60 || after_time[2] < 60
        {
            generator.after_hour(after_time[0]);
            generator.after_minute(after_time[1]);
            generator.after_second(after_time[2]);
        } else {
            valid = false;
        }

        if !valid {
            // TODO
            eprintln!("after invalid");
            return Ok(());
        }
    }

    if let Some(before_str) = cli.before {
        let mut valid = true;
        let before_time: Vec<u32> = time_to_vec(&before_str)?;

        if before_time.len() == 3 && before_time[0] <= 24 && before_time[1] < 60
            || before_time[2] < 60
        {
            generator.before_hour(before_time[0]);
            generator.before_minute(before_time[1]);
            generator.before_second(before_time[2]);
        } else {
            valid = false;
        }

        if !valid {
            // TODO
            eprintln!("before invalid");
            return Ok(());
        }
    }

    if let Some(start_str) = cli.start {
        let mut valid = true;
        let start_time: Vec<u32> = time_to_vec(&start_str)?;

        if start_time.len() == 3 && start_time[0] <= 24 && start_time[1] < 60 || start_time[2] < 60
        {
            generator.starting_hour(start_time[0]);
            generator.starting_minute(start_time[1]);
            generator.starting_second(start_time[2]);
        } else {
            valid = false;
        }

        if !valid {
            // TODO
            eprintln!("start invalid");
            return Ok(());
        }
    }

    let mut previous_iter_info: Option<(usize, chrono::TimeDelta)> = None;
    let mut summary = Summary::default();

    for fit_file in cli.fit_files.iter() {
        let iter = if let Some(info) = previous_iter_info {
            generator.concat(info.0, info.1, fit_file)?
        } else {
            generator.open(fit_file)?
        };
        summary = summary.merge(&iter.summary)?;
        for (count, time_delta, srt) in iter.into_iter() {
            println!("{srt:}\n");
            // TODO find other way to keep state of iterator
            previous_iter_info = Some((count, time_delta));
        }
    }
    if !summary.is_empty() && !cli.no_summary {
        if let Some((mut count, previous_time)) = previous_iter_info {
            count = count + 1;
            let previous_time = previous_time
                .checked_add(&chrono::TimeDelta::try_seconds(5).unwrap())
                .unwrap();
            let previous_time_str = fit2srt_core::srt_iter::delta_srt_format(&previous_time);
            let end_time = previous_time
                .checked_add(&chrono::TimeDelta::try_seconds(10).unwrap())
                .unwrap();
            let mut summary_str = "Summary:\n".to_string();
            if let Some((lat, long)) = summary.location() {
                summary_str += &format!("Location: {lat:}, {long:}\n");
            }
            if let Some(avg_t) = summary.avg_temperature {
                summary_str += &format!("Temperature: {avg_t:}{}", summary.temp_unit());
                if let Some(min_t) = summary.min_temperature {
                    if avg_t != min_t {
                        summary_str += &format!(" (min: {min_t:}{})", summary.temp_unit());
                    }
                }
                summary_str += &format!("\n");
            }
            if let Some(avg_d) = summary.avg_depth {
                summary_str += &format!("Depth: {avg_d:}{}", summary.depth_unit());
                if let Some(max_d) = summary.max_depth {
                    summary_str += &format!(" (max: {max_d:}{})", summary.depth_unit());
                }
                summary_str += &format!("\n");
            }

            println!(
                "{}\n{} --> {}\n{}",
                count,
                previous_time_str,
                fit2srt_core::srt_iter::delta_srt_format(&end_time),
                summary_str
            );
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        println!("{e:?}");
    }
}

#[test]
fn test_time_to_vec() {
    assert_eq!(time_to_vec("15:01:30"), Ok(vec![15, 1, 30]));
    assert_eq!(time_to_vec("150130"), Ok(vec![15, 1, 30]));
}
