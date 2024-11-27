//! # sys (UNIX)
//!
//! UNIX-specific structs and functions. Will be imported as `sys` on UNIX
//! systems.
#![allow(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

// On UNIX systems, termios represents the terminal mode.
pub use libc::termios as TermMode;
use libc::{SA_SIGINFO, STDIN_FILENO, STDOUT_FILENO, TCSADRAIN, TIOCGWINSZ, VMIN, VTIME};
use libc::{c_int, c_void, sigaction, sighandler_t, siginfo_t, winsize};

use crate::Error;
pub use crate::xdg::*;

fn cerr(err: c_int) -> Result<(), Error> {
    match err {
        0..=c_int::MAX => Ok(()),
        _ => Err(std::io::Error::last_os_error().into()),
    }
}

/// Return the current window size as (rows, columns).
///
/// We use the `TIOCGWINSZ` ioctl to get window size. If it succeeds, a
/// `Winsize` struct will be populated.
/// This ioctl is described here: <http://man7.org/linux/man-pages/man4/tty_ioctl.4.html>
pub fn get_window_size() -> Result<(usize, usize), Error> {
    let mut maybe_ws = std::mem::MaybeUninit::<winsize>::uninit();
    cerr(unsafe { libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, maybe_ws.as_mut_ptr()) })
        .map_or(None, |()| unsafe { Some(maybe_ws.assume_init()) })
        .filter(|ws| ws.ws_col != 0 && ws.ws_row != 0)
        .map_or(Err(Error::InvalidWindowSize), |ws| Ok((ws.ws_row as usize, ws.ws_col as usize)))
}

/// Stores whether the window size has changed since last call to
/// `has_window_size_changed`.
static WSC: AtomicBool = AtomicBool::new(false);

/// Handle a change in window size.
extern "C" fn handle_wsize(_: c_int, _: *mut siginfo_t, _: *mut c_void) { WSC.store(true, Relaxed) }

#[allow(clippy::fn_to_numeric_cast_any)]
/// Register a signal handler that sets a global variable when the window size
/// changes. After calling this function, use `has_window_size_changed` to query
/// the global variable.
pub fn register_winsize_change_signal_handler() -> Result<(), Error> {
    unsafe {
        let mut maybe_sa = std::mem::MaybeUninit::<sigaction>::uninit();
        cerr(libc::sigemptyset(&mut (*maybe_sa.as_mut_ptr()).sa_mask))?;
        // We could use sa_handler here, however, sigaction defined in libc does not
        // have sa_handler field, so we use sa_sigaction instead.
        (*maybe_sa.as_mut_ptr()).sa_flags = SA_SIGINFO;
        (*maybe_sa.as_mut_ptr()).sa_sigaction = handle_wsize as sighandler_t;
        cerr(sigaction(libc::SIGWINCH, maybe_sa.as_ptr(), std::ptr::null_mut()))
    }
}

/// Check if the windows size has changed since the last call to this function.
/// The `register_winsize_change_signal_handler` needs to be called before this
/// function.
pub fn has_window_size_changed() -> bool { WSC.swap(false, Relaxed) }

/// Set the terminal mode.
pub fn set_term_mode(term: &TermMode) -> Result<(), Error> {
    cerr(unsafe { libc::tcsetattr(STDIN_FILENO, TCSADRAIN, term) })
}

/// Setup the termios to enable raw mode, and return the original termios.
///
/// termios manual is available at: <http://man7.org/linux/man-pages/man3/termios.3.html>
pub fn enable_raw_mode() -> Result<TermMode, Error> {
    let mut maybe_term = std::mem::MaybeUninit::<TermMode>::uninit();
    cerr(unsafe { libc::tcgetattr(STDIN_FILENO, maybe_term.as_mut_ptr()) })?;
    let orig_term = unsafe { maybe_term.assume_init() };
    let mut term = orig_term;
    unsafe { libc::cfmakeraw(&mut term) };
    // First sets the minimum number of characters for non-canonical reads
    // Second sets the timeout in deciseconds for non-canonical reads
    (term.c_cc[VMIN], term.c_cc[VTIME]) = (0, 1);
    set_term_mode(&term)?;
    Ok(orig_term)
}

#[allow(clippy::unnecessary_wraps)] // Result required on other platforms
pub fn stdin() -> std::io::Result<std::io::Stdin> { Ok(std::io::stdin()) }

pub fn path(filename: &str) -> std::path::PathBuf { std::path::PathBuf::from(filename) }
