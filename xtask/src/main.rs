#![expect(clippy::multiple_crate_versions)]
use std::process::ExitCode;

use anstyle::{AnsiColor, Reset, Style};
use clap::{Parser, Subcommand};
mod count_loc;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

const BOLD: Style = Style::new().bold();
const RESET: Reset = Reset;
const RED: Style = AnsiColor::Red.on_default();
const GREEN: Style = AnsiColor::Green.on_default();

fn main() -> ExitCode {
    let args = Args::parse();
    match args.command.execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{RED}{err}{RESET}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    CountLoc,
}

impl Command {
    fn execute(&self) -> Result<()> {
        match self {
            Self::CountLoc => count_loc::count_loc(),
        }
    }
}
