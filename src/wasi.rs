//! # sys (WASI)
//!
//! WASI-specific structs and functions. Will be imported as `sys` on WASI
//! systems.

use crate::Error;
pub use crate::xdg::*;

pub struct TermMode {}

/// Return the current window size as (rows, columns).
/// By returning an error we cause kibi to fall back to another method of
/// getting the window size
pub const fn get_window_size() -> Result<(usize, usize), Error> { Err(Error::InvalidWindowSize) }

/// Register a signal handler that sets a global variable when the window size
/// changes. On WASI platforms, this does nothing.
#[allow(clippy::unnecessary_wraps)] // Result required on other platforms
pub const fn register_winsize_change_signal_handler() -> Result<(), Error> { Ok(()) }

/// Check if the windows size has changed since the last call to this function.
/// On WASI platforms, this always return false.
pub const fn has_window_size_changed() -> bool { false }

/// Set the terminal mode. On WASI platforms, this does nothing.
#[allow(clippy::unnecessary_wraps)] // Result required on other platforms
pub const fn set_term_mode(_term: &TermMode) -> Result<(), Error> { Ok(()) }

// Opening the file /dev/tty is effectively the same as `raw_mode`
#[allow(clippy::unnecessary_wraps)] // Result required on other platforms
pub const fn enable_raw_mode() -> Result<TermMode, Error> { Ok(TermMode {}) }

pub fn stdin() -> std::io::Result<std::fs::File> { std::fs::File::open("/dev/tty") }

pub fn path(filename: &str) -> std::path::PathBuf {
    // If the filename is absolute then it starts with a forward slash and we
    // can just open the file however if it lacks a forwrad slash then its
    // relative to the current working directory. As WASI does not have an ABI
    // for current directory we are using the PWD environment variable as a
    // defacto standard
    if filename.starts_with('/') {
        std::path::PathBuf::from(filename)
    } else {
        std::env::current_dir().unwrap_or_else(|_| "/".into()).join(filename)
    }
}
