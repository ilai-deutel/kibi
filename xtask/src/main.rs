#![expect(clippy::multiple_crate_versions)]
use std::process::ExitCode;

mod count_loc;

type Result<T, E=Box<dyn std::error::Error>> = std::result::Result<T, E>;

const RESET: &str = "\x1b[m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";

const USAGE: &str = "Usage: cargo xtask count-loc";

fn execute(command: Option<&str>, args: &[String]) -> Result<()> {
    match (command, args) {
        (None, _) => Err(format!("no command provided\n\n{USAGE}"))?,
        (Some("count-loc"), []) => count_loc::count_loc(),
        (Some("count-loc"), [argument, ..]) =>
            Err(format!("invalid argument \"{YELLOW}{argument}{RESET}\"\n\n{USAGE}"))?,
        (Some(command), _) =>
            Err(format!("invalid command \"{YELLOW}{command}{RESET}\"\n\n{USAGE}"))?,
    }
}

fn main() -> ExitCode {
    let (command, args) = {
        let mut args = std::env::args();
        (args.nth(1), args.collect::<Vec<_>>())
    };
    match execute(command.as_deref(), &args[..]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{RED}error{RESET}: {error}");
            ExitCode::FAILURE
        }
    }
}
