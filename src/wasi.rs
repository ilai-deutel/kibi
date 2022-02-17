//! # sys (WASI)
//!
//! WASI-specific structs and functions. Will be imported as `sys` on WASI systems.

use std::env::var;

use crate::Error;

#[allow(unused)]
pub struct TermMode {}

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
/// By returning an error we cause kibi to fall back to another method of getting the window size
pub fn get_window_size() -> Result<(usize, usize), Error> { Err(Error::InvalidWindowSize) }

/// Register a signal handler that sets a global variable when the window size changes.
/// After calling this function, use has_window_size_changed to query the global variable.
pub fn register_winsize_change_signal_handler() -> Result<(), Error> { Ok(()) }

/// Check if the windows size has changed since the last call to this function.
/// The register_winsize_change_signal_handler needs to be called before this function.
pub fn has_window_size_changed() -> bool { false }

/// Set the terminal mode (this does nothing)
pub fn set_term_mode(_term: &TermMode) -> Result<(), Error> { Ok(()) }

/// Opening the file /dev/tty is effectively the same as raw_mode
pub fn enable_raw_mode() -> Result<TermMode, Error> { Ok(TermMode {}) }

pub fn stdin() -> Box<dyn std::io::Read> { Box::new(std::fs::File::open("/dev/tty").unwrap()) }

pub fn path(filename: &String) -> std::path::PathBuf {
    if filename.starts_with("/") {
        std::path::PathBuf::from(filename)
    } else {
        let cur_dir = var("PWD").unwrap_or("/".to_string());
        let path = std::path::PathBuf::from(cur_dir);
        path.join(filename)
    }
}
