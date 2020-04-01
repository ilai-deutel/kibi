//! # sys (UNIX)
//!
//! UNIX-specific structs and functions. Will be imported as `sys` on UNIX systems.

use std::env::var;
use std::sync::mpsc::{self, Receiver};

use libc::{STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ, VMIN, VTIME};
use nix::{pty::Winsize, sys::termios};
use signal_hook::{iterator::Signals, SIGWINCH};

// On UNIX systems, Termios represents the terminal mode.
pub(crate) use nix::sys::termios::Termios as TermMode;

use crate::Error;

/// Return configuration directories for UNIX systems
pub(crate) fn conf_dirs() -> [Option<String>; 3] {
    [Some("/etc".into()), var("XDG_CONFIG_HOME").ok(), var("HOME").map(|d| d + "/.config").ok()]
}

/// Return the current window size as (rows, columns).
///
/// We use the `TIOCGWINSZ` ioctl to get window size. If it succeeds, a `Winsize` struct will be
/// populated.
/// This ioctl is described here: <http://man7.org/linux/man-pages/man4/tty_ioctl.4.html>
pub(crate) fn get_window_size() -> Result<(usize, usize), Error> {
    nix::ioctl_read_bad!(get_ws, TIOCGWINSZ, Winsize);

    let mut maybe_ws = std::mem::MaybeUninit::<Winsize>::uninit();

    unsafe { get_ws(STDOUT_FILENO, maybe_ws.as_mut_ptr()).ok().map(|_| maybe_ws.assume_init()) }
        .filter(|ws| ws.ws_col != 0 && ws.ws_row != 0)
        .map_or(Err(Error::InvalidWindowSize), |ws| Ok((ws.ws_row as usize, ws.ws_col as usize)))
}

/// Return a MPSC receiver that receives a message whenever the window size is updated.
pub(crate) fn get_window_size_update_receiver() -> Result<Option<Receiver<()>>, Error> {
    // Create a channel for receiving window size update requests
    let (ws_changed_tx, ws_changed_rx) = mpsc::sync_channel(1);
    // Spawn a new thread that will push to the aforementioned channel every time the SIGWINCH
    // signal is received
    let signals = Signals::new(&[SIGWINCH])?;
    std::thread::spawn(move || signals.forever().for_each(|_| ws_changed_tx.send(()).unwrap()));
    Ok(Some(ws_changed_rx))
}

/// Set the terminal mode.
pub(crate) fn set_term_mode(term: &TermMode) -> Result<(), nix::Error> {
    termios::tcsetattr(STDIN_FILENO, termios::SetArg::TCSAFLUSH, term)
}

/// Setup the termios to enable raw mode, and return the original termios.
///
/// termios manual is available at: <http://man7.org/linux/man-pages/man3/termios.3.html>
pub(crate) fn enable_raw_mode() -> Result<TermMode, Error> {
    let orig_termios = termios::tcgetattr(STDIN_FILENO)?;
    let mut term = orig_termios.clone();
    termios::cfmakeraw(&mut term);
    // Set the minimum number of characters for non-canonical reads
    term.control_chars[VMIN] = 0;
    // Set the timeout in deciseconds for non-canonical reads
    term.control_chars[VTIME] = 1;
    set_term_mode(&term)?;
    Ok(orig_termios)
}
