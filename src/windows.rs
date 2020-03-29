//! # sys (Windows)
//!
//! Windows-specific structs and functions. Will be imported as `sys` on Windows systems.

use std::sync::mpsc::Receiver;

use winapi::um::wincon::*;

pub(crate) use crate::{terminal::get_window_size_using_cursor as get_window_size, Error};

// On Windows systems, the terminal mode is represented as an unsigned int.
pub(crate) type TermMode = u32;

/// Return configuration directories for Windows systems
pub(crate) fn conf_dirs() -> [Option<String>; 1] { [std::env::var("APPDATA").ok()] }

pub(crate) fn get_window_size_update_receiver() -> Result<Option<Receiver<()>>, Error> { Ok(None) }

/// Set the terminal mode.
pub(crate) fn set_term_mode(stdin_term_mode: &TermMode) -> Result<(), std::io::Error> {
    winapi_util::console::set_mode(winapi_util::HandleRef::stdin(), *stdin_term_mode)
}

/// Enable raw mode, and return the original terminal mode.
///
/// Documentation for console modes is available at:
/// <https://docs.microsoft.com/en-us/windows/console/setconsolemodel>
pub(crate) fn enable_raw_mode() -> Result<TermMode, Error> {
    let orig_mode_in = winapi_util::console::mode(winapi_util::HandleRef::stdin())?;
    let mode_in = (orig_mode_in | ENABLE_VIRTUAL_TERMINAL_INPUT)
        & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
    set_term_mode(&mode_in)?;
    Ok(orig_mode_in)
}
