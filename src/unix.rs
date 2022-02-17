//! # sys (UNIX)
//!
//! UNIX-specific structs and functions. Will be imported as `sys` on UNIX systems.

use std::env::var;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

// On UNIX systems, termios represents the terminal mode.
pub use libc::termios as TermMode;
use libc::{c_int, c_void, sigaction, sighandler_t, siginfo_t, winsize};
use libc::{SA_SIGINFO, STDIN_FILENO, STDOUT_FILENO, TCSADRAIN, TIOCGWINSZ, VMIN, VTIME};

use crate::Error;

fn cerr(err: c_int) -> Result<(), Error> {
    match err {
        0..=c_int::MAX => Ok(()),
        _ => Err(std::io::Error::last_os_error().into()),
    }
}

/// Return directories following the XDG Base Directory Specification
///
/// See `conf_dirs()` and `data_dirs()` for usage example.
///
/// The XDG Base Directory Specification is defined here:
/// <https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html>
fn xdg_dirs(xdg_type: &str, def_home_suffix: &str, def_dirs: &str) -> Vec<String> {
    let (home_key, dirs_key) = (format!("XDG_{}_HOME", xdg_type), format!("XDG_{}_DIRS", xdg_type));

    let mut dirs = Vec::new();

    // If environment variable `home_key` (e.g. `$XDG_CONFIG_HOME`) is set, add its value to `dirs`.
    // Otherwise, if environment variable `$HOME` is set, add `$HOME{def_home_suffix}`
    // (e.g. `$HOME/.config`) to `dirs`.
    dirs.extend(var(home_key).or_else(|_| var("HOME").map(|d| d + def_home_suffix)).into_iter());

    // If environment variable `dirs_key` (e.g. `XDG_CONFIG_DIRS`) is set, split by `:` and add the
    // parts to `dirs`.
    // Otherwise, add the split `def_dirs` (e.g. `/etc/xdg:/etc`) and add the parts to `dirs`.
    dirs.extend(var(dirs_key).unwrap_or_else(|_| def_dirs.into()).split(':').map(String::from));

    dirs.into_iter().map(|p| p + "/kibi").collect()
}

/// Return configuration directories for UNIX systems
pub fn conf_dirs() -> Vec<String> { xdg_dirs("CONFIG", "/.config", "/etc/xdg:/etc") }

/// Return syntax directories for UNIX systems
pub fn data_dirs() -> Vec<String> {
    xdg_dirs("DATA", "/.local/share", "/usr/local/share/:/usr/share/")
}

/// Return the current window size as (rows, columns).
///
/// We use the `TIOCGWINSZ` ioctl to get window size. If it succeeds, a `Winsize` struct will be
/// populated.
/// This ioctl is described here: <http://man7.org/linux/man-pages/man4/tty_ioctl.4.html>
pub fn get_window_size() -> Result<(usize, usize), Error> {
    let mut maybe_ws = std::mem::MaybeUninit::<winsize>::uninit();
    cerr(unsafe { libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, maybe_ws.as_mut_ptr()) })
        .map_or(None, |_| unsafe { Some(maybe_ws.assume_init()) })
        .filter(|ws| ws.ws_col != 0 && ws.ws_row != 0)
        .map_or(Err(Error::InvalidWindowSize), |ws| Ok((ws.ws_row as usize, ws.ws_col as usize)))
}

/// Stores whether the window size has changed since last call to has_window_size_changed.
static WSC: AtomicBool = AtomicBool::new(false);

/// Handle a change in window size.
extern "C" fn handle_wsize(_: c_int, _: *mut siginfo_t, _: *mut c_void) { WSC.store(true, Relaxed) }

/// Register a signal handler that sets a global variable when the window size changes.
/// After calling this function, use has_window_size_changed to query the global variable.
pub fn register_winsize_change_signal_handler() -> Result<(), Error> {
    unsafe {
        let mut maybe_sa = std::mem::MaybeUninit::<sigaction>::uninit();
        cerr(libc::sigemptyset(&mut (*maybe_sa.as_mut_ptr()).sa_mask))?;
        // We could use sa_handler here, however, sigaction defined in libc does not have
        // sa_handler field, so we use sa_sigaction instead.
        (*maybe_sa.as_mut_ptr()).sa_flags = SA_SIGINFO;
        (*maybe_sa.as_mut_ptr()).sa_sigaction = handle_wsize as sighandler_t;
        cerr(libc::sigaction(libc::SIGWINCH, maybe_sa.as_ptr(), std::ptr::null_mut()))
    }
}

/// Check if the windows size has changed since the last call to this function.
/// The register_winsize_change_signal_handler needs to be called before this function.
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
    // Set the minimum number of characters for non-canonical reads
    term.c_cc[VMIN] = 0;
    // Set the timeout in deciseconds for non-canonical reads
    term.c_cc[VTIME] = 1;
    set_term_mode(&term)?;
    Ok(orig_term)
}

pub fn stdin() -> Box<dyn std::io::Read> { Box::new(std::io::stdin()) }

pub fn path(filename: &String) -> std::path::PathBuf { std::path::PathBuf::from(filename) }
