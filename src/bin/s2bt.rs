//! `s2bt` diagnostic CLI entry point.

use std::process::ExitCode;

use clap::Parser;
use switch2_gamecube_bt::cli::{Args, run};

fn main() -> ExitCode {
    let result = run(Args::parse());
    if result.exit_code == 0 {
        println!("{}", result.output.trim_end());
    } else {
        eprintln!("{}", result.output.trim_end());
    }
    ExitCode::from(result.exit_code)
}
