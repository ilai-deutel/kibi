//! # Kibi

use kibi::{Error, run, stdin};

/// Load the configuration, initialize the editor and run the program,
/// optionally opening a file if an argument is given.
///
/// # Errors
///
/// Any error that occur during the execution of the program will be returned by
/// this function.
fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    match (args.nth(1).as_deref(), args.next().as_deref(), /* remaining_args= */ args.len()) {
        (Some("--version"), None | Some("--"), 0) => println!("kibi {}", env!("CARGO_PKG_VERSION")),
        (Some(o), ..) if o.starts_with('-') && o != "--" =>
            return Err(Error::UnrecognizedOption(o.into())),
        (Some("--"), p, 0) | (p, Some("--") | None, 0) => run(p, &mut stdin()?)?,
        _ => return Err(Error::TooManyArguments(std::env::args().collect())),
    }
    Ok(())
}
