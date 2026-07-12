//! `s2bt` diagnostic CLI entry point.

use std::{fs, process::ExitCode};

use clap::Parser;
use switch2_gamecube_bt::cli::{Args, run};

fn main() -> ExitCode {
    let args = Args::parse();
    let result_file = args.result_file.clone();
    let result = run(args);
    if let Some(path) = result_file {
        if let Err(error) = fs::write(&path, &result.output) {
            eprintln!("could not write sanitized result: {error}");
            return ExitCode::from(8);
        }
        return ExitCode::from(result.exit_code);
    }
    if result.exit_code == 0 {
        println!("{}", result.output.trim_end());
    } else {
        eprintln!("{}", result.output.trim_end());
    }
    ExitCode::from(result.exit_code)
}
