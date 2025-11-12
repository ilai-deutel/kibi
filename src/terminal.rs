use std::io::{self, BufRead, Read, Write};

use crate::{Error, ansi_escape::*, sys};

/// Obtain the window size using the cursor position.
///
/// This function moves the cursor to the bottom-right using ANSI escape
/// sequence `\x1b[999C\x1b[999B`, then requests the cursor position using ANSI
/// escape sequence `\x1b[6n`. After this sequence is sent, the next characters
/// on stdin should be `\x1b[{row};{column}R`.
///
/// It is used as an alternative method if `sys::get_window_size()` returns an
/// error.
pub fn get_window_size_using_cursor() -> Result<(usize, usize), Error> {
    print!("{REPOSITION_CURSOR_END}{DEVICE_STATUS_REPORT}");
    io::stdout().flush()?;
    let mut prefix_buffer = [0u8; 2];
    sys::stdin()?.read_exact(&mut prefix_buffer)?;
    if prefix_buffer != [b'\x1b', b'['] {
        return Err(Error::CursorPosition);
    }
    Ok((read_value_until(b';')?, read_value_until(b'R')?))
}

/// Read value until a certain stop byte is reached, and parse the result
/// (pre-stop byte).
fn read_value_until<T: std::str::FromStr>(stop_byte: u8) -> Result<T, Error> {
    let mut buf = Vec::new();
    sys::stdin()?.read_until(stop_byte, &mut buf)?;
    // Check that we have reached `stop_byte`, not EOF.
    buf.pop().filter(|u| *u == stop_byte).ok_or(Error::CursorPosition)?;
    std::str::from_utf8(&buf).or(Err(Error::CursorPosition))?.parse().or(Err(Error::CursorPosition))
}

#[cfg_attr(any(windows, target_os = "wasi"), expect(clippy::trivially_copy_pass_by_ref))]
pub fn restore_terminal(orig_term_mode: &sys::TermMode) -> io::Result<()> {
    // Restore the original terminal mode.
    sys::set_term_mode(orig_term_mode)?;
    print!("{USE_MAIN_SCREEN}");
    io::stdout().flush()
}
