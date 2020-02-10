//! # Errors

use std::fmt::{self, Display, Formatter};

/// These errors can used in the program.
pub enum Error {
    IO(std::io::Error),
    Nix(nix::Error),
    MPSCTryRecv(std::sync::mpsc::TryRecvError),
    CursorPosition,
    Config(String, String),
    TooManyArguments(usize),
}

impl Display for Error {
    /// Format the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {}", err),
            Self::Nix(err) => write!(f, "System call error: {}", err),
            Self::MPSCTryRecv(err) => write!(f, "MSPC try_recv error: {}", err),
            Self::CursorPosition => write!(f, "Could not obtain cursor position"),
            Self::Config(line, reason) => write!(f, "Could not parse config {}: {}", line, reason),
            Self::TooManyArguments(n) => write!(f, "Expected 0 or 1 argument, got {}", n),
        }
    }
}

impl From<std::io::Error> for Error {
    /// Convert an IO Error into a Kibi Error.
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}
impl From<nix::Error> for Error {
    /// Convert a nix IO Error into a Kibi Error.
    fn from(err: nix::Error) -> Self {
        Self::Nix(err)
    }
}
