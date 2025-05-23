#![allow(special_module_name)]
use clap::Parser;
use std::num::ParseIntError;
use std::path::PathBuf;

mod lib;
use lib::SrtGenerator;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Generate subtitles after 00:00:00
    #[arg(short, long)]
    after: Option<String>,

    /// Generate subtitles before 00:00:00
    #[arg(short, long)]
    before: Option<String>,

    fit_file: PathBuf,
}

fn time_to_vec(time_str: &str) -> Result<Vec<u32>, ParseIntError> {
    if time_str.contains(':') {
        let mut err = None;
        let time = time_str
            .split(":")
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

    for srt in generator.open(cli.fit_file)? {
        println!("{srt:}\n");
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
