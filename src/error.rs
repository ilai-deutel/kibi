//! # Errors

/// Kibi error type.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `std::io::Error`
    Io(std::io::Error),
    /// Wrapper around `std::fmt::Error`
    Fmt(std::fmt::Error),
    /// Error returned when the window size obtained through a system call is
    /// invalid.
    InvalidWindowSize,
    /// Error setting or retrieving the cursor position.
    CursorPosition,
    /// Too many arguments given to kibi. The attribute corresponds to the total
    /// number of command line arguments.
    TooManyArguments(Vec<String>),
    /// Unrecognized option given as a command line argument.
    UnrecognizedOption(String),
}

impl From<std::io::Error> for Error {
    /// Convert an IO Error into a Kibi Error.
    fn from(err: std::io::Error) -> Self { Self::Io(err) }
}

impl From<std::fmt::Error> for Error {
    /// Convert an Fmt Error into a Kibi Error.
    fn from(err: std::fmt::Error) -> Self { Self::Fmt(err) }
}
