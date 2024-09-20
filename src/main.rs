#![allow(special_module_name)]
use clap::Parser;
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

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    let mut generator = SrtGenerator::default();

    if let Some(after_str) = cli.after {
        let mut valid = true;
        let after_time: Vec<u32> = after_str
            .split(":")
            .map(|s| {
                if let Ok(i) = s.parse() {
                    i
                } else {
                    valid = false;
                    0
                }
            })
            .collect();

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
        let before_time: Vec<u32> = before_str
            .split(":")
            .map(|s| {
                if let Ok(i) = s.parse() {
                    i
                } else {
                    valid = false;
                    0
                }
            })
            .collect();

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
