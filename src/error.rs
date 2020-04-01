//! # Errors

/// Kibi error type.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `std::io::Error`
    IO(std::io::Error),
    /// Wrapper around `nix::Error`
    #[cfg(unix)]
    Nix(nix::Error),
    /// Wrapper around `std::sync::mpsc::TryRecvError`
    MPSCTryRecv(std::sync::mpsc::TryRecvError),
    /// Error returned when the window size obtained through a system call is invalid.
    InvalidWindowSize,
    /// Error setting or retrieving the cursor position.
    CursorPosition,
    /// Configuration error. The three attributes correspond the file path, the line number and the
    /// error message.
    Config(std::path::PathBuf, usize, String),
    /// Too many arguments given to kibi. The attribute corresponds to the total number of command
    /// line armuments.
    TooManyArguments(usize),
}

impl From<std::io::Error> for Error {
    /// Convert an IO Error into a Kibi Error.
    fn from(err: std::io::Error) -> Self { Self::IO(err) }
}

#[cfg(unix)]
impl From<nix::Error> for Error {
    /// Convert a nix Error into a Kibi Error.
    fn from(err: nix::Error) -> Self { Self::Nix(err) }
}
