//! # Errors

/// Kibi error type.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `std::io::Error`
    Io(std::io::Error),
    /// Error returned when the window size obtained through a system call is invalid.
    InvalidWindowSize,
    /// Error setting or retrieving the cursor position.
    CursorPosition,
    /// Configuration error. The three attributes correspond the file path, the line number and the
    /// error message.
    Config(std::path::PathBuf, usize, String),
    /// Too many arguments given to kibi. The attribute corresponds to the total number of command
    /// line arguments.
    TooManyArguments(usize),
    /// Unrecognized option given as a command line argument.
    UnrecognizedOption(String),
}

impl From<std::io::Error> for Error {
    /// Convert an IO Error into a Kibi Error.
    fn from(err: std::io::Error) -> Self { Self::Io(err) }
}
