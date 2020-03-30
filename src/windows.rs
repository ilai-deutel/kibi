//! # sys (Windows)
//!
//! Windows-specific structs and functions. Will be imported as `sys` on Windows systems.

use std::{io, sync::mpsc::Receiver};

use winapi::um::wincon::*;
use winapi_util::{console, HandleRef};

use crate::Error;

// On Windows systems, the terminal mode is represented as an unsigned int.
pub(crate) type TermMode = u32;

/// Return configuration directories for Windows systems
pub(crate) fn conf_dirs() -> [Option<String>; 1] { [std::env::var("APPDATA").ok()] }

pub(crate) fn get_window_size() -> Result<(usize, usize), Error> {
    let w_rect = console::screen_buffer_info(HandleRef::stdout())?.window_rect();
    Ok(((w_rect.Bottom - w_rect.Top + 1) as usize, (w_rect.Right - w_rect.Left + 1) as usize))
}

pub(crate) fn get_window_size_update_receiver() -> Result<Option<Receiver<()>>, Error> { Ok(None) }

/// Set the terminal mode.
pub(crate) fn set_term_mode(stdin_term_mode: &TermMode) -> Result<(), io::Error> {
    console::set_mode(HandleRef::stdin(), *stdin_term_mode)
}

/// Enable raw mode, and return the original terminal mode.
///
/// Documentation for console modes is available at:
/// <https://docs.microsoft.com/en-us/windows/console/setconsolemode>
pub(crate) fn enable_raw_mode() -> Result<TermMode, Error> {
    let orig_mode_in = console::mode(HandleRef::stdin())?;
    let mode_in = (orig_mode_in | ENABLE_VIRTUAL_TERMINAL_INPUT)
        & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
    set_term_mode(&mode_in)?;
    Ok(orig_mode_in)
}
