use std::io::Write;

use anstyle::{AnsiColor, Reset, Style};
use clap::{Parser, Subcommand};
mod count_loc;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

const BOLD: Style = Style::new().bold();
const RESET: Reset = Reset;
const RED: Style = AnsiColor::Red.on_default();
const GREEN: Style = AnsiColor::Green.on_default();

fn main() {
    let args = Args::parse();
    if let Err(err) = args.command.execute() {
        let _ = std::io::stdout().flush();
        eprintln!("{RED}{err}{RESET}");
        std::process::exit(1);
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
            Command::CountLoc => count_loc::count_loc(),
        }
    }
}
