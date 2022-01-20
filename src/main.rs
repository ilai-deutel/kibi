//! # Kibi

use kibi::{Config, Editor, Error};

/// Load the configuration, initialize the editor and run the program, optionally opening a file if
/// an argument is given.
///
/// # Errors
///
/// Any error that occur during the execution of the program will be returned by this function.
fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    match (args.nth(1), /*remaining_args=*/ args.len()) {
        (Some(arg), 0) if arg == "--version" => println!("kibi, v{}", env!("CARGO_PKG_VERSION")),
        (Some(arg), 0) if arg.starts_with('-') => return Err(Error::UnrecognizedOption(arg)),
        (file_name, 0) => Editor::new(Config::load()?)?.run(file_name)?,
        (_, n_remaining_args) => return Err(Error::TooManyArguments(n_remaining_args + 1)),
    }
    Ok(())
}
