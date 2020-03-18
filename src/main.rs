//! # Kibi

use kibi::{ansi_escape::CLEAR_SCREEN, ansi_escape::MOVE_CURSOR_TO_START, Config, Editor, Error};

/// Load the configuration, initialize the editor and run the program, optionally opening a file if
/// an argument is given.
///
/// # Errors
///
/// Any error that occur during the execution of the program will be returned by this function.
fn run(mut args: std::env::Args) -> Result<(), Error> {
    if args.len() > 2 {
        return Err(Error::TooManyArguments(args.len() - 1));
    }
    Editor::new(Config::load()?)?.run(args.nth(1))
}

fn main() {
    let err_str = run(std::env::args()).err().map_or("".into(), |e| e.to_string() + "\n");
    print!("{}{}{}", CLEAR_SCREEN, MOVE_CURSOR_TO_START, err_str);
}
