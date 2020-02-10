use std::io::{self, BufRead, Read, Write};

use libc::{STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ, VMIN, VTIME};
use nix::{pty::Winsize, sys::termios};

use crate::{ansi_escape::*, Error};

pub(super) fn set_termios(term: &termios::Termios) -> Result<(), nix::Error> {
    termios::tcsetattr(STDIN_FILENO, termios::SetArg::TCSAFLUSH, term)
}

/// Setup the termios to enable raw mode, and return the original termios.
///
/// termios manual is available at: <http://man7.org/linux/man-pages/man3/termios.3.html>
pub(super) fn enable_raw_mode() -> Result<termios::Termios, Error> {
    let orig_termios = termios::tcgetattr(STDIN_FILENO)?;
    let mut term = orig_termios.clone();
    termios::cfmakeraw(&mut term);
    // Set the minimum number of characters for non-canonical reads
    term.control_chars[VMIN] = 0;
    // Set the timeout in deciseconds for non-canonical reads
    term.control_chars[VTIME] = 1;
    set_termios(&term)?;
    Ok(orig_termios)
}

/// Return the current window size as (rows, columns).
///
/// Two methods are used to get the window size:
/// 1. Use the TIOCGWINSZ to get window size. If it succeeds, a Winsize struct will be populated
///    This ioctl is described here: <http://man7.org/linux/man-pages/man4/tty_ioctl.4.html>
/// 2. If the first method fails, we reposition the cursor at the end of the terminal and get the
///    cursor position.
pub(super) fn get_window_size() -> Result<(usize, usize), Error> {
    nix::ioctl_read_bad!(get_ws, TIOCGWINSZ, Winsize);
    let mut maybe_ws = std::mem::MaybeUninit::<Winsize>::uninit();

    // Alternate method to get the window size, if TIOCGWINSZ ioctl calls fails: reposition the
    // cursor at the end and obtain the cursor position.
    let get_window_size_v2 =
        || print_and_flush(REPOSITION_CURSOR_END).and_then(|_| get_cursor_position());

    unsafe { get_ws(STDOUT_FILENO, maybe_ws.as_mut_ptr()).ok().map(|_| maybe_ws.assume_init()) }
        .filter(|ws| ws.ws_col != 0 && ws.ws_row != 0)
        // If the IOCTL method fails, use the alternate method
        .map_or_else(get_window_size_v2, |ws| Ok((ws.ws_row as usize, ws.ws_col as usize)))
}

/// Read value until a certain stop byte is reached, and parse the result (pre-stop byte).
fn read_value_until<T: std::str::FromStr>(stop_byte: u8) -> Result<T, Error> {
    let mut buffer = Vec::new();
    io::stdin().lock().read_until(stop_byte, &mut buffer)?;
    // Check that we have reached `stop_byte`, not EOF.
    if buffer.pop().filter(|u| *u == stop_byte).is_none() {
        return Err(Error::CursorPosition);
    }
    std::str::from_utf8(&buffer)
        .or(Err(Error::CursorPosition))?
        .parse()
        .or(Err(Error::CursorPosition))
}

/// Return the position (row, column) of the cursor.
fn get_cursor_position() -> Result<(usize, usize), Error> {
    // DEVICE_STATUS_REPORT reports the cursor position as ESC[{row};{column}R
    print_and_flush(DEVICE_STATUS_REPORT)?;
    let mut prefix_buffer = [0_u8; 2];
    io::stdin().read_exact(&mut prefix_buffer)?;
    if prefix_buffer != [b'\x1b', b'['] {
        return Err(Error::CursorPosition);
    }
    Ok((read_value_until(b';')?, read_value_until(b'R')?))
}

/// Print a string to stdout and flush.
pub(crate) fn print_and_flush(s: &str) -> Result<(), Error> {
    io::stdout().write_all(s.as_bytes())?;
    io::stdout().flush().map_err(Error::from)
}
