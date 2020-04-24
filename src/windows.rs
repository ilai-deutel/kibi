//! # sys (Windows)
//!
//! Windows-specific structs and functions. Will be imported as `sys` on Windows systems.

#![allow(clippy::wildcard_imports)]

use std::{convert::TryInto, env::var, io, sync::mpsc::Receiver};

use winapi::um::wincon::*;
use winapi_util::{console as cons, HandleRef};

use crate::Error;

// On Windows systems, the terminal mode is represented as 2 unsigned integers (one for stdin, one
// for stdout).
pub type TermMode = (u32, u32);

/// Return configuration directories for Windows systems
pub fn conf_dirs() -> Vec<String> { var("APPDATA").map(|d| d + "/Kibi").into_iter().collect() }

/// Return data directories for Windows systems
pub fn data_dirs() -> Vec<String> { conf_dirs() }

/// Return the current window size as (rows, columns).
pub fn get_window_size() -> Result<(usize, usize), Error> {
    let rect = cons::screen_buffer_info(HandleRef::stdout())?.window_rect();
    match ((rect.bottom - rect.top + 1).try_into(), (rect.right - rect.left + 1).try_into()) {
        (Ok(rows), Ok(cols)) => Ok((rows, cols)),
        _ => Err(Error::InvalidWindowSize),
    }
}

pub fn get_window_size_update_receiver() -> Result<Option<Receiver<()>>, Error> { Ok(None) }

/// Set the terminal mode.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn set_term_mode((stdin_mode, stdout_mode): &TermMode) -> Result<(), io::Error> {
    cons::set_mode(HandleRef::stdin(), *stdin_mode)?;
    cons::set_mode(HandleRef::stdout(), *stdout_mode)
}

/// Enable raw mode, and return the original terminal mode.
///
/// Documentation for console modes is available at:
/// <https://docs.microsoft.com/en-us/windows/console/setconsolemode>
pub fn enable_raw_mode() -> Result<TermMode, Error> {
    // (mode_in0, mode_out0) are the original terminal modes
    let (mode_in0, mode_out0) = (cons::mode(HandleRef::stdin())?, cons::mode(HandleRef::stdout())?);

    // (mode_in, mode_out) are the new terminal modes
    let mode_in = (mode_in0 | ENABLE_VIRTUAL_TERMINAL_INPUT)
        & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
    let mode_out = (mode_out0 | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
        | (DISABLE_NEWLINE_AUTO_RETURN | ENABLE_PROCESSED_OUTPUT);

    set_term_mode(&(mode_in, mode_out))?;
    Ok((mode_in0, mode_out0))
}
