use std::io::{self, BufRead, Read, Write};

use crate::{ansi_escape::DEVICE_STATUS_REPORT, ansi_escape::REPOSITION_CURSOR_END, Error};

/// Obtain the window size using the cursor position.
///
/// This function moves the cursor to the bottom-right using ANSI escape sequence
/// `\x1b[999C\x1b[999B`, then requests the cursor position using ANSI escape sequence `\x1b[6n`.
/// After this sequence is sent, the next characters on stdin should be `\x1b[{row};{column}R`.
pub(crate) fn get_window_size_using_cursor() -> Result<(usize, usize), Error> {
    print!("{}{}", REPOSITION_CURSOR_END, DEVICE_STATUS_REPORT);
    io::stdout().flush()?;
    let mut prefix_buffer = [0_u8; 2];
    io::stdin().read_exact(&mut prefix_buffer)?;
    if prefix_buffer != [b'\x1b', b'['] {
        return Err(Error::CursorPosition);
    }
    Ok((read_value_until(b';')?, read_value_until(b'R')?))
}

/// Read value until a certain stop byte is reached, and parse the result (pre-stop byte).
fn read_value_until<T: std::str::FromStr>(stop_byte: u8) -> Result<T, Error> {
    let mut buf = Vec::new();
    io::stdin().lock().read_until(stop_byte, &mut buf)?;
    // Check that we have reached `stop_byte`, not EOF.
    if buf.pop().filter(|u| *u == stop_byte).is_none() {
        return Err(Error::CursorPosition);
    }
    std::str::from_utf8(&buf).or(Err(Error::CursorPosition))?.parse().or(Err(Error::CursorPosition))
}
