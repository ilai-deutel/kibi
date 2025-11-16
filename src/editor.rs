use std::fmt::{Display, Write as _};
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Seek, Write};
use std::iter::{self, repeat, successors};
use std::{fs::File, path::Path, process::Command, time::Instant};

use crate::row::{HlState, Row};
use crate::{Config, Error, ansi_escape::*, syntax::Conf as SyntaxConf, sys, terminal};

const fn ctrl_key(key: u8) -> u8 { key & 0x1f }
const EXIT: u8 = ctrl_key(b'Q');
const DELETE_BIS: u8 = ctrl_key(b'H');
const REFRESH_SCREEN: u8 = ctrl_key(b'L');
const SAVE: u8 = ctrl_key(b'S');
const FIND: u8 = ctrl_key(b'F');
const GOTO: u8 = ctrl_key(b'G');
const CUT: u8 = ctrl_key(b'X');
const COPY: u8 = ctrl_key(b'C');
const PASTE: u8 = ctrl_key(b'V');
const DUPLICATE: u8 = ctrl_key(b'D');
const EXECUTE: u8 = ctrl_key(b'E');
const REMOVE_LINE: u8 = ctrl_key(b'R');
const BACKSPACE: u8 = 127;

const HELP_MESSAGE: &str = "^S save | ^Q quit | ^F find | ^G go to | ^D duplicate | ^E execute | \
                            ^C copy | ^X cut | ^V paste";

/// `set_status!` sets a formatted status message for the editor.
/// Example usage: `set_status!(editor, "{file_size} written to {file_name}")`
macro_rules! set_status { ($editor:expr, $($arg:expr),*) => ($editor.status_msg = Some(StatusMessage::new(format!($($arg),*)))) }

/// Enum of input keys
#[cfg_attr(test, derive(Debug, PartialEq))]
enum Key {
    Arrow(AKey),
    CtrlArrow(AKey),
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    Escape,
    Char(u8),
}

/// Enum of arrow keys
#[cfg_attr(test, derive(Debug, PartialEq))]
enum AKey {
    Left,
    Right,
    Up,
    Down,
}

/// Describes the cursor position and the screen offset
#[derive(Debug, Default, Clone, PartialEq)]
struct CursorState {
    /// x position (indexing the characters, not the columns)
    x: usize,
    /// y position (row number, 0-indexed)
    y: usize,
    /// Row offset
    roff: usize,
    /// Column offset
    coff: usize,
}

impl CursorState {
    const fn move_to_next_line(&mut self) { (self.x, self.y) = (0, self.y + 1); }

    /// Scroll the terminal window vertically and horizontally (i.e. adjusting
    /// the row offset and the column offset) so that the cursor can be
    /// shown.
    fn scroll(&mut self, rx: usize, screen_rows: usize, screen_cols: usize) {
        self.roff = self.roff.clamp(self.y.saturating_sub(screen_rows.saturating_sub(1)), self.y);
        self.coff = self.coff.clamp(rx.saturating_sub(screen_cols.saturating_sub(1)), rx);
    }
}

/// The `Editor` struct, contains the state and configuration of the text
/// editor.
#[derive(Default)]
pub struct Editor {
    /// If not `None`, the current prompt mode (`Save`, `Find`, `GoTo`, or
    /// `Execute`). If `None`, we are in regular edition mode.
    prompt_mode: Option<PromptMode>,
    /// The current state of the cursor.
    cursor: CursorState,
    /// The padding size used on the left for line numbering.
    ln_pad: usize,
    /// The width of the current window. Will be updated when the window is
    /// resized.
    window_width: usize,
    /// The number of rows that can be used for the editor, excluding the status
    /// bar and the message bar
    screen_rows: usize,
    /// The number of columns that can be used for the editor, excluding the
    /// part used for line numbers
    screen_cols: usize,
    /// The collection of rows, including the content and the syntax
    /// highlighting information.
    rows: Vec<Row>,
    /// Whether the document has been modified since it was open.
    dirty: bool,
    /// The configuration for the editor.
    config: Config,
    /// The number of consecutive times the user has tried to quit without
    /// saving. After `config.quit_times`, the program will exit.
    quit_times: usize,
    /// The file name. If None, the user will be prompted for a file name the
    /// first time they try to save.
    // TODO: It may be better to store a PathBuf instead
    file_name: Option<String>,
    /// The current status message being shown.
    status_msg: Option<StatusMessage>,
    /// The syntax configuration corresponding to the current file's extension.
    syntax: SyntaxConf,
    /// The number of bytes contained in `rows`. This excludes new lines.
    n_bytes: u64,
    /// The copied buffer of a row
    copied_row: Vec<u8>,
}

/// Describes a status message, shown at the bottom at the screen.
struct StatusMessage {
    /// The message to display.
    msg: String,
    /// The `Instant` the status message was first displayed.
    time: Instant,
}

impl StatusMessage {
    /// Create a new status message and set time to the current date/time.
    fn new(msg: String) -> Self { Self { msg, time: Instant::now() } }
}

/// Pretty-format a size in bytes.
fn format_size(n: u64) -> String {
    if n < 1024 {
        return format!("{n}B");
    }
    // i is the largest value such that 1024 ^ i < n
    let i = n.ilog2() / 10;

    // Compute the size with two decimal places (rounded down) as the last two
    // digits of q This avoid float formatting reducing the binary size
    let q = 100 * n / (1024 << ((i - 1) * 10));
    format!("{}.{:02}{}B", q / 100, q % 100, b" kMGTPEZ"[i as usize] as char)
}

/// Return an Arrow Key given an ANSI code.
///
/// The argument must be a valide arrow key ANSI code (`a`, `b`, `c` or `d`),
/// case-insensitive).
fn get_akey(c: u8) -> AKey {
    match c {
        b'a' | b'A' => AKey::Up,
        b'b' | b'B' => AKey::Down,
        b'c' | b'C' => AKey::Right,
        b'd' | b'D' => AKey::Left,
        _ => unreachable!("Invalid ANSI code for arrow key {}", c),
    }
}

impl Editor {
    /// Return the current row if the cursor points to an existing row, `None`
    /// otherwise.
    fn current_row(&self) -> Option<&Row> { self.rows.get(self.cursor.y) }

    /// Return the position of the cursor, in terms of rendered characters (as
    /// opposed to `self.cursor.x`, which is the position of the cursor in
    /// terms of bytes).
    fn rx(&self) -> usize { self.current_row().map_or(0, |r| r.cx2rx[self.cursor.x]) }

    /// Move the cursor following an arrow key (← → ↑ ↓).
    fn move_cursor(&mut self, key: &AKey, ctrl: bool) {
        match (key, self.current_row()) {
            (AKey::Left, Some(row)) if self.cursor.x > 0 => {
                let mut cursor_x = self.cursor.x - row.get_char_size(row.cx2rx[self.cursor.x] - 1);
                // ← moving to previous word
                while ctrl && cursor_x > 0 && row.chars[cursor_x - 1] != b' ' {
                    cursor_x -= row.get_char_size(row.cx2rx[cursor_x] - 1);
                }
                self.cursor.x = cursor_x;
            }
            // ← at the beginning of the line: move to the end of the previous line. The x
            // position will be adjusted after this `match` to accommodate the current row
            // length, so we can just set here to the maximum possible value here.
            (AKey::Left, _) if self.cursor.y > 0 =>
                (self.cursor.y, self.cursor.x) = (self.cursor.y - 1, usize::MAX),
            (AKey::Right, Some(row)) if self.cursor.x < row.chars.len() => {
                let mut cursor_x = self.cursor.x + row.get_char_size(row.cx2rx[self.cursor.x]);
                // → moving to next word
                while ctrl && cursor_x < row.chars.len() && row.chars[cursor_x] != b' ' {
                    cursor_x += row.get_char_size(row.cx2rx[cursor_x]);
                }
                self.cursor.x = cursor_x;
            }
            (AKey::Right, Some(_)) => self.cursor.move_to_next_line(),
            // TODO: For Up and Down, move self.cursor.x to be consistent with tabs and UTF-8
            //  characters, i.e. according to rx
            (AKey::Up, _) if self.cursor.y > 0 => self.cursor.y -= 1,
            (AKey::Down, Some(_)) => self.cursor.y += 1,
            _ => (),
        }
        self.update_cursor_x_position();
    }

    /// Update the cursor x position. If the cursor y position has changed, the
    /// current position might be illegal (x is further right than the last
    /// character of the row). If that is the case, clamp `self.cursor.x`.
    fn update_cursor_x_position(&mut self) {
        self.cursor.x = self.cursor.x.min(self.current_row().map_or(0, |row| row.chars.len()));
    }

    /// Run a loop to obtain the key that was pressed. At each iteration of the
    /// loop (until a key is pressed), we listen to the `ws_changed` channel
    /// to check if a window size change signal has been received. When
    /// bytes are received, we match to a corresponding `Key`. In particular,
    /// we handle ANSI escape codes to return `Key::Delete`, `Key::Home` etc.
    fn loop_until_keypress(&mut self, input: &mut impl BufRead) -> Result<Key, Error> {
        let mut bytes = input.bytes();
        loop {
            // Handle window size if a signal has be received
            if sys::has_window_size_changed() {
                self.update_window_size()?;
                self.refresh_screen()?;
            }
            if let Some(a) = bytes.next().transpose()? {
                // Match on the next byte received or, if the first byte is <ESC> ('\x1b'), on
                // the next few bytes.
                if a != b'\x1b' {
                    return Ok(Key::Char(a));
                }
                return Ok(match bytes.next().transpose()? {
                    Some(b @ (b'[' | b'O')) => match (b, bytes.next().transpose()?) {
                        (b'[', Some(c @ b'A'..=b'D')) => Key::Arrow(get_akey(c)),
                        (b'[' | b'O', Some(b'H')) => Key::Home,
                        (b'[' | b'O', Some(b'F')) => Key::End,
                        (b'[', mut c @ Some(b'0'..=b'8')) => {
                            let mut d = bytes.next().transpose()?;
                            if (c, d) == (Some(b'1'), Some(b';')) {
                                // 1 is the default modifier value. Therefore, <ESC>[1;5C is
                                // equivalent to <ESC>[5C, etc.
                                c = bytes.next().transpose()?;
                                d = bytes.next().transpose()?;
                            }
                            match (c, d) {
                                (Some(c), Some(b'~')) if c == b'1' || c == b'7' => Key::Home,
                                (Some(c), Some(b'~')) if c == b'4' || c == b'8' => Key::End,
                                (Some(b'3'), Some(b'~')) => Key::Delete,
                                (Some(b'5'), Some(b'~')) => Key::PageUp,
                                (Some(b'6'), Some(b'~')) => Key::PageDown,
                                (Some(b'5'), Some(d @ b'A'..=b'D')) => Key::CtrlArrow(get_akey(d)),
                                _ => Key::Escape,
                            }
                        }
                        (b'O', Some(c @ b'a'..=b'd')) => Key::CtrlArrow(get_akey(c)),
                        _ => Key::Escape,
                    },
                    _ => Key::Escape,
                });
            }
        }
    }

    /// Update the `screen_rows`, `window_width`, `screen_cols` and `ln_padding`
    /// attributes.
    fn update_window_size(&mut self) -> Result<(), Error> {
        let wsize = sys::get_window_size().or_else(|_| terminal::get_window_size_using_cursor())?;
        // Make room for the status bar and status message
        (self.screen_rows, self.window_width) = (wsize.0.saturating_sub(2), wsize.1);
        self.update_screen_cols();
        Ok(())
    }

    /// Update the `screen_cols` and `ln_padding` attributes based on the
    /// maximum number of digits for line numbers (since the left padding
    /// depends on this number of digits).
    fn update_screen_cols(&mut self) {
        // The maximum number of digits to use for the line number is the number of
        // digits of the last line number. This is equal to the number of times
        // we can divide this number by ten, computed below using `successors`.
        let n_digits =
            successors(Some(self.rows.len()), |u| Some(u / 10).filter(|u| *u > 0)).count();
        let show_line_num = self.config.show_line_num && n_digits + 2 < self.window_width / 4;
        self.ln_pad = if show_line_num { n_digits + 2 } else { 0 };
        self.screen_cols = self.window_width.saturating_sub(self.ln_pad);
    }

    /// Update a row, given its index. If `ignore_following_rows` is `false` and
    /// the highlight state has changed during the update (for instance, it
    /// is now in "multi-line comment" state, keep updating the next rows
    fn update_row(&mut self, y: usize, ignore_following_rows: bool) {
        let mut hl_state = if y > 0 { self.rows[y - 1].hl_state } else { HlState::Normal };
        for row in self.rows.iter_mut().skip(y) {
            let previous_hl_state = row.hl_state;
            hl_state = row.update(&self.syntax, hl_state, self.config.tab_stop);
            if ignore_following_rows || hl_state == previous_hl_state {
                return;
            }
            // If the state has changed (for instance, a multi-line comment
            // started in this row), continue updating the following
            // rows
        }
    }

    /// Update all the rows.
    fn update_all_rows(&mut self) {
        let mut hl_state = HlState::Normal;
        for row in &mut self.rows {
            hl_state = row.update(&self.syntax, hl_state, self.config.tab_stop);
        }
    }

    /// Insert a byte at the current cursor position. If there is no row at the
    /// current cursor position, add a new row and insert the byte.
    fn insert_byte(&mut self, c: u8) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            row.chars.insert(self.cursor.x, c);
        } else {
            self.rows.push(Row::new(vec![c]));
            // The number of rows has changed. The left padding may need to be updated.
            self.update_screen_cols();
        }
        self.update_row(self.cursor.y, false);
        (self.cursor.x, self.n_bytes, self.dirty) = (self.cursor.x + 1, self.n_bytes + 1, true);
    }

    /// Insert a new line at the current cursor position and move the cursor to
    /// the start of the new line. If the cursor is in the middle of a row,
    /// split off that row.
    fn insert_new_line(&mut self) {
        let (position, new_row_chars) = if self.cursor.x == 0 {
            (self.cursor.y, Vec::new())
        } else {
            // self.rows[self.cursor.y] must exist, since cursor.x = 0 for any cursor.y ≥
            // row.len()
            let new_chars = self.rows[self.cursor.y].chars.split_off(self.cursor.x);
            self.update_row(self.cursor.y, false);
            (self.cursor.y + 1, new_chars)
        };
        self.rows.insert(position, Row::new(new_row_chars));
        self.update_row(position, false);
        self.update_screen_cols();
        self.cursor.move_to_next_line();
        self.dirty = true;
    }

    /// Delete a character at the current cursor position. If the cursor is
    /// located at the beginning of a row that is not the first or last row,
    /// merge the current row and the previous row. If the cursor is located
    /// after the last row, move up to the last character of the previous row.
    fn delete_char(&mut self) {
        if self.cursor.x > 0 {
            let row = &mut self.rows[self.cursor.y];
            // Obtain the number of bytes to be removed: could be 1-4 (UTF-8 character
            // size).
            let n_bytes_to_remove = row.get_char_size(row.cx2rx[self.cursor.x] - 1);
            row.chars.splice(self.cursor.x - n_bytes_to_remove..self.cursor.x, iter::empty());
            self.update_row(self.cursor.y, false);
            self.cursor.x -= n_bytes_to_remove;
            self.dirty = if self.is_empty() { self.file_name.is_some() } else { true };
            self.n_bytes -= n_bytes_to_remove as u64;
        } else if self.cursor.y < self.rows.len() && self.cursor.y > 0 {
            let row = self.rows.remove(self.cursor.y);
            let previous_row = &mut self.rows[self.cursor.y - 1];
            self.cursor.x = previous_row.chars.len();
            previous_row.chars.extend(&row.chars);
            self.update_row(self.cursor.y - 1, true);
            self.update_row(self.cursor.y, false);
            // The number of rows has changed. The left padding may need to be updated.
            self.update_screen_cols();
            (self.dirty, self.cursor.y) = (true, self.cursor.y - 1);
        } else if self.cursor.y == self.rows.len() {
            // If the cursor is located after the last row, pressing backspace is equivalent
            // to pressing the left arrow key.
            self.move_cursor(&AKey::Left, false);
        }
    }

    fn delete_current_row(&mut self) {
        if self.cursor.y < self.rows.len() {
            self.rows[self.cursor.y].chars.clear();
            self.cursor.x = 0;
            self.cursor.y = std::cmp::min(self.cursor.y + 1, self.rows.len() - 1);
            self.delete_char();
            self.cursor.x = 0;
        }
    }

    fn duplicate_current_row(&mut self) {
        self.copy_current_row();
        self.paste_current_row();
    }

    fn copy_current_row(&mut self) {
        if let Some(row) = self.current_row() {
            self.copied_row = row.chars.clone();
        }
    }

    fn paste_current_row(&mut self) {
        if self.copied_row.is_empty() {
            return;
        }
        self.n_bytes += self.copied_row.len() as u64;
        if self.cursor.y == self.rows.len() {
            self.rows.push(Row::new(self.copied_row.clone()));
        } else {
            self.rows.insert(self.cursor.y + 1, Row::new(self.copied_row.clone()));
        }
        self.update_row(self.cursor.y + usize::from(self.cursor.y + 1 != self.rows.len()), false);
        (self.cursor.y, self.dirty) = (self.cursor.y + 1, true);
        // The line number has changed
        self.update_screen_cols();
    }

    /// Try to load a file. If found, load the rows and update the render and
    /// syntax highlighting. If not found, do not return an error.
    fn load(&mut self, path: &Path) -> Result<(), Error> {
        let mut file = match File::open(path) {
            Err(e) if e.kind() == ErrorKind::NotFound => {
                self.rows.push(Row::new(Vec::new()));
                return Ok(());
            }
            r => r,
        }?;
        let ft = file.metadata()?.file_type();
        if !(ft.is_file() || ft.is_symlink()) {
            return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid input file type").into());
        }
        for line in BufReader::new(&file).split(b'\n') {
            self.rows.push(Row::new(line?));
        }
        // If the file ends with an empty line or is empty, we need to append an empty
        // row to `self.rows`. Unfortunately, BufReader::split doesn't yield an
        // empty Vec in this case, so we need to check the last byte directly.
        file.seek(io::SeekFrom::End(0))?;
        #[expect(clippy::unbuffered_bytes)]
        if file.bytes().next().transpose()?.is_none_or(|b| b == b'\n') {
            self.rows.push(Row::new(Vec::new()));
        }
        self.update_all_rows();
        // The number of rows has changed. The left padding may need to be updated.
        self.update_screen_cols();
        self.n_bytes = self.rows.iter().map(|row| row.chars.len() as u64).sum();
        Ok(())
    }

    /// Save the text to a file, given its name.
    fn save(&self, file_name: &str) -> Result<usize, io::Error> {
        let mut file = File::create(file_name)?;
        let mut written = 0;
        for (i, row) in self.rows.iter().enumerate() {
            file.write_all(&row.chars)?;
            written += row.chars.len();
            if i != (self.rows.len() - 1) {
                file.write_all(b"\n")?;
                written += 1;
            }
        }
        file.sync_all()?;
        Ok(written)
    }

    /// Save the text to a file and handle all errors. Errors and success
    /// messages will be printed to the status bar. Return whether the file
    /// was successfully saved.
    fn save_and_handle_io_errors(&mut self, file_name: &str) -> bool {
        let saved = self.save(file_name);
        // Print error or success message to the status bar
        match saved.as_ref() {
            Ok(w) => set_status!(self, "{} written to {}", format_size(*w as u64), file_name),
            Err(err) => set_status!(self, "Can't save! I/O error: {err}"),
        }
        // If save was successful, set dirty to false.
        self.dirty &= saved.is_err();
        saved.is_ok()
    }

    /// Save to a file after obtaining the file path from the prompt. If
    /// successful, the `file_name` attribute of the editor will be set and
    /// syntax highlighting will be updated.
    fn save_as(&mut self, file_name: String) {
        if self.save_and_handle_io_errors(&file_name) {
            // If save was successful
            self.syntax = SyntaxConf::find(&file_name, &sys::data_dirs());
            self.file_name = Some(file_name);
            self.update_all_rows();
        }
    }

    /// Draw the left part of the screen: line numbers and vertical bar.
    fn draw_left_padding<T: Display>(&self, buffer: &mut String, val: T) -> Result<(), Error> {
        if self.ln_pad >= 2 {
            // \x1b[38;5;240m: Dark grey color; \u{2502}: pipe "│"
            write!(buffer, "\x1b[38;5;240m{:>2$} \u{2502}{}", val, RESET_FMT, self.ln_pad - 2)?;
        }
        Ok(())
    }

    /// Return whether the file being edited is empty or not. If there is more
    /// than one row, even if all the rows are empty, `is_empty` returns
    /// `false`, since the text contains new lines.
    const fn is_empty(&self) -> bool { self.rows.len() <= 1 && self.n_bytes == 0 }

    /// Draw rows of text and empty rows on the terminal, by adding characters
    /// to the buffer.
    fn draw_rows(&self, buffer: &mut String) -> Result<(), Error> {
        let row_it = self.rows.iter().map(Some).chain(repeat(None)).enumerate();
        for (i, row) in row_it.skip(self.cursor.roff).take(self.screen_rows) {
            buffer.push_str(CLEAR_LINE_RIGHT_OF_CURSOR);
            if let Some(row) = row {
                // Draw a row of text
                self.draw_left_padding(buffer, i + 1)?;
                row.draw(self.cursor.coff, self.screen_cols, buffer)?;
            } else {
                // Draw an empty row
                self.draw_left_padding(buffer, '~')?;
                if self.is_empty() && i == self.screen_rows / 3 {
                    let welcome_message = concat!("Kibi ", env!("KIBI_VERSION"));
                    write!(buffer, "{:^1$.1$}", welcome_message, self.screen_cols)?;
                }
            }
            buffer.push_str("\r\n");
        }
        Ok(())
    }

    /// Draw the status bar on the terminal, by adding characters to the buffer.
    fn draw_status_bar(&self, buffer: &mut String) -> Result<(), Error> {
        // Left part of the status bar
        let modified = if self.dirty { " (modified)" } else { "" };
        let mut left =
            format!("{:.30}{modified}", self.file_name.as_deref().unwrap_or("[No Name]"));
        left.truncate(self.window_width);

        // Right part of the status bar
        let size = format_size(self.n_bytes + self.rows.len().saturating_sub(1) as u64);
        let right =
            format!("{} | {size} | {}:{}", self.syntax.name, self.cursor.y + 1, self.rx() + 1);

        // Draw
        let rw = self.window_width.saturating_sub(left.len());
        write!(buffer, "{REVERSE_VIDEO}{left}{right:>rw$.rw$}{RESET_FMT}\r\n")?;
        Ok(())
    }

    /// Draw the message bar on the terminal, by adding characters to the
    /// buffer.
    fn draw_message_bar(&self, buffer: &mut String) {
        buffer.push_str(CLEAR_LINE_RIGHT_OF_CURSOR);
        let msg_duration = self.config.message_dur;
        if let Some(sm) = self.status_msg.as_ref().filter(|sm| sm.time.elapsed() < msg_duration) {
            buffer.push_str(&sm.msg[..sm.msg.len().min(self.window_width)]);
        }
    }

    /// Refresh the screen: update the offsets, draw the rows, the status bar,
    /// the message bar, and move the cursor to the correct position.
    fn refresh_screen(&mut self) -> Result<(), Error> {
        self.cursor.scroll(self.rx(), self.screen_rows, self.screen_cols);
        let mut buffer = format!("{HIDE_CURSOR}{MOVE_CURSOR_TO_START}");
        self.draw_rows(&mut buffer)?;
        self.draw_status_bar(&mut buffer)?;
        self.draw_message_bar(&mut buffer);
        let (cursor_x, cursor_y) = if self.prompt_mode.is_none() {
            // If not in prompt mode, position the cursor according to the `cursor`
            // attributes.
            (self.rx() - self.cursor.coff + 1 + self.ln_pad, self.cursor.y - self.cursor.roff + 1)
        } else {
            // If in prompt mode, position the cursor on the prompt line at the end of the
            // line.
            (self.status_msg.as_ref().map_or(0, |sm| sm.msg.len() + 1), self.screen_rows + 2)
        };
        // Finally, print `buffer` and move the cursor
        print!("{buffer}\x1b[{cursor_y};{cursor_x}H{SHOW_CURSOR}");
        io::stdout().flush().map_err(Error::from)
    }

    /// Process a key that has been pressed, when not in prompt mode. Returns
    /// whether the program should exit, and optionally the prompt mode to
    /// switch to.
    fn process_keypress(&mut self, key: &Key) -> (bool, Option<PromptMode>) {
        // This won't be mutated, unless key is Key::Character(EXIT)
        let mut reset_quit_times = true;
        let mut prompt_mode = None;

        match key {
            Key::Arrow(arrow) => self.move_cursor(arrow, false),
            Key::CtrlArrow(arrow) => self.move_cursor(arrow, true),
            Key::PageUp => {
                self.cursor.y = self.cursor.roff.saturating_sub(self.screen_rows);
                self.update_cursor_x_position();
            }
            Key::PageDown => {
                self.cursor.y = (self.cursor.roff + 2 * self.screen_rows - 1).min(self.rows.len());
                self.update_cursor_x_position();
            }
            Key::Home => self.cursor.x = 0,
            Key::End => self.cursor.x = self.current_row().map_or(0, |row| row.chars.len()),
            Key::Char(b'\r' | b'\n') => self.insert_new_line(), // Enter
            Key::Char(BACKSPACE | DELETE_BIS) => self.delete_char(), // Backspace or Ctrl + H
            Key::Char(REMOVE_LINE) => self.delete_current_row(),
            Key::Delete => {
                self.move_cursor(&AKey::Right, false);
                self.delete_char();
            }
            Key::Escape | Key::Char(REFRESH_SCREEN) => (),
            Key::Char(EXIT) => {
                if !self.dirty || self.quit_times + 1 >= self.config.quit_times {
                    return (true, None);
                }
                let r = self.config.quit_times - self.quit_times - 1;
                set_status!(self, "Press Ctrl+Q {0} more time{1:.2$} to quit.", r, "s", r - 1);
                reset_quit_times = false;
            }
            Key::Char(SAVE) => match self.file_name.take() {
                // TODO: Can we avoid using take() then reassigning the value to file_name?
                Some(file_name) => {
                    self.save_and_handle_io_errors(&file_name);
                    self.file_name = Some(file_name);
                }
                None => prompt_mode = Some(PromptMode::Save(String::new())),
            },
            Key::Char(FIND) =>
                prompt_mode = Some(PromptMode::Find(String::new(), self.cursor.clone(), None)),
            Key::Char(GOTO) => prompt_mode = Some(PromptMode::GoTo(String::new())),
            Key::Char(DUPLICATE) => self.duplicate_current_row(),
            Key::Char(CUT) => {
                self.copy_current_row();
                self.delete_current_row();
            }
            Key::Char(COPY) => self.copy_current_row(),
            Key::Char(PASTE) => self.paste_current_row(),
            Key::Char(EXECUTE) => prompt_mode = Some(PromptMode::Execute(String::new())),
            Key::Char(c) => self.insert_byte(*c),
        }
        self.quit_times = if reset_quit_times { 0 } else { self.quit_times + 1 };
        (false, prompt_mode)
    }

    /// Try to find a query, this is called after pressing Ctrl-F and for each
    /// key that is pressed. `last_match` is the last row that was matched,
    /// `forward` indicates whether to search forward or backward. Returns
    /// the row of a new match, or `None` if the search was unsuccessful.
    fn find(&mut self, query: &str, last_match: Option<usize>, forward: bool) -> Option<usize> {
        // Number of rows to search
        let num_rows = if query.is_empty() { 0 } else { self.rows.len() };
        let mut current = last_match.unwrap_or_else(|| num_rows.saturating_sub(1));
        // TODO: Handle multiple matches per line
        for _ in 0..num_rows {
            current = (current + if forward { 1 } else { num_rows - 1 }) % num_rows;
            let row = &mut self.rows[current];
            if let Some(cx) = row.chars.windows(query.len()).position(|w| w == query.as_bytes()) {
                // self.cursor.coff: Try to reset the column offset; if the match is after the
                // offset, this will be updated in self.cursor.scroll() so that
                // the result is visible
                (self.cursor.x, self.cursor.y, self.cursor.coff) = (cx, current, 0);
                let rx = row.cx2rx[cx];
                row.match_segment = Some(rx..rx + query.len());
                return Some(current);
            }
        }
        None
    }

    /// If `file_name` is not None, load the file. Then run the text editor.
    ///
    /// # Errors
    ///
    /// Will Return `Err` if any error occur.
    pub fn run<I: BufRead>(&mut self, file_name: Option<&str>, input: &mut I) -> Result<(), Error> {
        self.update_window_size()?;
        set_status!(self, "{HELP_MESSAGE}");
        self.refresh_screen()?;

        if let Some(path) = file_name.map(sys::path) {
            self.syntax = SyntaxConf::find(&path.to_string_lossy(), &sys::data_dirs());
            self.load(path.as_path())?;
            self.file_name = Some(path.to_string_lossy().to_string());
        } else {
            self.rows.push(Row::new(Vec::new()));
            self.file_name = None;
        }
        loop {
            if let Some(mode) = &self.prompt_mode {
                set_status!(self, "{}", mode.status_msg());
            }
            self.refresh_screen()?;
            let key = self.loop_until_keypress(input)?;
            // TODO: Can we avoid using take()?
            self.prompt_mode = match self.prompt_mode.take() {
                // process_keypress returns (should_quit, prompt_mode)
                None => match self.process_keypress(&key) {
                    (true, _) => return Ok(()),
                    (false, prompt_mode) => prompt_mode,
                },
                Some(prompt_mode) => prompt_mode.process_keypress(self, &key),
            }
        }
    }
}

/// Set up the terminal and run the text editor. If `file_name` is not None,
/// load the file.
///
/// Update the panic hook to restore the terminal on panic.
///
/// # Errors
///
/// Will Return `Err` if any error occur when registering the window size signal
/// handler, enabling raw mode, or running the editor.
pub fn run<I: BufRead>(file_name: Option<&str>, input: &mut I) -> Result<(), Error> {
    sys::register_winsize_change_signal_handler()?;
    let orig_term_mode = sys::enable_raw_mode()?;
    print!("{USE_ALTERNATE_SCREEN}");

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal::restore_terminal(&orig_term_mode).unwrap_or_else(|e| eprintln!("{e}"));
        prev_hook(info);
    }));

    let mut editor = Editor { config: Config::load(), ..Default::default() };
    let result = editor.run(file_name, input);

    // Restore the original terminal mode.
    terminal::restore_terminal(&orig_term_mode)?;

    result
}

/// The prompt mode.
#[cfg_attr(test, derive(Debug, PartialEq))]
enum PromptMode {
    /// Save(prompt buffer)
    Save(String),
    /// Find(prompt buffer, saved cursor state, last match)
    Find(String, CursorState, Option<usize>),
    /// GoTo(prompt buffer)
    GoTo(String),
    /// Execute(prompt buffer)
    Execute(String),
}

// TODO: Use trait with mode_status_msg and process_keypress, implement the
// trait for separate  structs for Save and Find?
impl PromptMode {
    /// Return the status message to print for the selected `PromptMode`.
    fn status_msg(&self) -> String {
        match self {
            Self::Save(buffer) => format!("Save as: {buffer}"),
            Self::Find(buffer, ..) => format!("Search (Use ESC/Arrows/Enter): {buffer}"),
            Self::GoTo(buffer) => format!("Enter line number[:column number]: {buffer}"),
            Self::Execute(buffer) => format!("Command to execute: {buffer}"),
        }
    }

    /// Process a keypress event for the selected `PromptMode`.
    fn process_keypress(self, ed: &mut Editor, key: &Key) -> Option<Self> {
        ed.status_msg = None;
        match self {
            Self::Save(b) => match process_prompt_keypress(b, key) {
                PromptState::Active(b) => return Some(Self::Save(b)),
                PromptState::Cancelled => set_status!(ed, "Save aborted"),
                PromptState::Completed(file_name) => ed.save_as(file_name),
            },
            Self::Find(b, saved_cursor, last_match) => {
                if let Some(row_idx) = last_match {
                    ed.rows[row_idx].match_segment = None;
                }
                match process_prompt_keypress(b, key) {
                    PromptState::Active(query) => {
                        #[expect(clippy::wildcard_enum_match_arm)]
                        let (last_match, forward) = match key {
                            Key::Arrow(AKey::Right | AKey::Down) | Key::Char(FIND) =>
                                (last_match, true),
                            Key::Arrow(AKey::Left | AKey::Up) => (last_match, false),
                            _ => (None, true),
                        };
                        let curr_match = ed.find(&query, last_match, forward);
                        return Some(Self::Find(query, saved_cursor, curr_match));
                    }
                    // The prompt was cancelled. Restore the previous position.
                    PromptState::Cancelled => ed.cursor = saved_cursor,
                    // Cursor has already been moved, do nothing
                    PromptState::Completed(_) => (),
                }
            }
            Self::GoTo(b) => match process_prompt_keypress(b, key) {
                PromptState::Active(b) => return Some(Self::GoTo(b)),
                PromptState::Cancelled => (),
                PromptState::Completed(b) => {
                    let mut split = b.splitn(2, ':')
                        // saturating_sub: Lines and cols are 1-indexed
                        .map(|u| u.trim().parse().map(|s: usize| s.saturating_sub(1)));
                    match (split.next().transpose(), split.next().transpose()) {
                        (Ok(Some(y)), Ok(x)) => {
                            ed.cursor.y = y.min(ed.rows.len());
                            if let Some(rx) = x {
                                ed.cursor.x = ed.current_row().map_or(0, |r| r.rx2cx[rx]);
                            } else {
                                ed.update_cursor_x_position();
                            }
                        }
                        (Err(e), _) | (_, Err(e)) => set_status!(ed, "Parsing error: {e}"),
                        (Ok(None), _) => (),
                    }
                }
            },
            Self::Execute(b) => match process_prompt_keypress(b, key) {
                PromptState::Active(b) => return Some(Self::Execute(b)),
                PromptState::Cancelled => (),
                PromptState::Completed(b) => {
                    let mut args = b.split_whitespace();
                    match Command::new(args.next().unwrap_or_default()).args(args).output() {
                        Ok(out) if !out.status.success() =>
                            set_status!(ed, "{}", String::from_utf8_lossy(&out.stderr).trim_end()),
                        Ok(out) => out.stdout.into_iter().for_each(|c| match c {
                            b'\n' => ed.insert_new_line(),
                            c => ed.insert_byte(c),
                        }),
                        Err(e) => set_status!(ed, "{e}"),
                    }
                }
            },
        }
        None
    }
}

/// The state of the prompt after processing a keypress event.
#[cfg_attr(test, derive(Debug, PartialEq))]
enum PromptState {
    // Active contains the current buffer
    Active(String),
    // Completed contains the final string
    Completed(String),
    Cancelled,
}

/// Process a prompt keypress event and return the new state for the prompt.
fn process_prompt_keypress(mut buffer: String, key: &Key) -> PromptState {
    #[expect(clippy::wildcard_enum_match_arm)]
    match key {
        Key::Char(b'\r') => return PromptState::Completed(buffer),
        Key::Escape | Key::Char(EXIT) => return PromptState::Cancelled,
        Key::Char(BACKSPACE | DELETE_BIS) => _ = buffer.pop(),
        Key::Char(c @ 0..=126) if !c.is_ascii_control() => buffer.push(*c as char),
        // No-op
        _ => (),
    }
    PromptState::Active(buffer)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rstest::rstest;

    use super::*;
    use crate::syntax::HlType;

    fn assert_row_chars_equal(editor: &Editor, expected: &[&[u8]]) {
        assert_eq!(
            editor.rows.len(),
            expected.len(),
            "editor has {} rows, expected {}",
            editor.rows.len(),
            expected.len()
        );
        for (i, (row, expected)) in editor.rows.iter().zip(expected).enumerate() {
            assert_eq!(
                row.chars,
                *expected,
                "comparing characters for row {}\n  left: {}\n  right: {}",
                i,
                String::from_utf8_lossy(&row.chars),
                String::from_utf8_lossy(expected)
            );
        }
    }

    fn assert_row_synthax_highlighting_types_equal(editor: &Editor, expected: &[&[HlType]]) {
        assert_eq!(
            editor.rows.len(),
            expected.len(),
            "editor has {} rows, expected {}",
            editor.rows.len(),
            expected.len()
        );
        for (i, (row, expected)) in editor.rows.iter().zip(expected).enumerate() {
            assert_eq!(row.hl, *expected, "comparing HlTypes for row {i}",);
        }
    }

    #[rstest]
    #[case(0, "0B")]
    #[case(1, "1B")]
    #[case(1023, "1023B")]
    #[case(1024, "1.00kB")]
    #[case(1536, "1.50kB")]
    // round down!
    #[case(21 * 1024 - 11, "20.98kB")]
    #[case(21 * 1024 - 10, "20.99kB")]
    #[case(21 * 1024 - 3, "20.99kB")]
    #[case(21 * 1024, "21.00kB")]
    #[case(21 * 1024 + 3, "21.00kB")]
    #[case(21 * 1024 + 10, "21.00kB")]
    #[case(21 * 1024 + 11, "21.01kB")]
    #[case(1024 * 1024 - 1, "1023.99kB")]
    #[case(1024 * 1024, "1.00MB")]
    #[case(1024 * 1024 + 1, "1.00MB")]
    #[case(100 * 1024 * 1024 * 1024, "100.00GB")]
    #[case(313 * 1024 * 1024 * 1024 * 1024, "313.00TB")]
    fn format_size_output(#[case] input: u64, #[case] expected_output: &str) {
        assert_eq!(format_size(input), expected_output);
    }

    #[test]
    fn editor_insert_byte() {
        let mut editor = Editor::default();
        let editor_cursor_x_before = editor.cursor.x;

        editor.insert_byte(b'X');
        editor.insert_byte(b'Y');
        editor.insert_byte(b'Z');

        assert_eq!(editor.cursor.x, editor_cursor_x_before + 3);
        assert_eq!(editor.rows.len(), 1);
        assert_eq!(editor.n_bytes, 3);
        assert_eq!(editor.rows[0].chars, [b'X', b'Y', b'Z']);
    }

    #[test]
    fn editor_insert_new_line() {
        let mut editor = Editor::default();
        let editor_cursor_y_before = editor.cursor.y;

        for _ in 0..3 {
            editor.insert_new_line();
        }

        assert_eq!(editor.cursor.y, editor_cursor_y_before + 3);
        assert_eq!(editor.rows.len(), 3);
        assert_eq!(editor.n_bytes, 0);

        for row in &editor.rows {
            assert_eq!(row.chars, []);
        }
    }

    #[test]
    fn editor_delete_char() {
        let mut editor = Editor::default();
        for b in b"Hello world!" {
            editor.insert_byte(*b);
        }
        editor.delete_char();
        assert_row_chars_equal(&editor, &[b"Hello world"]);
        editor.move_cursor(&AKey::Left, true);
        editor.move_cursor(&AKey::Left, false);
        editor.move_cursor(&AKey::Left, false);
        editor.delete_char();
        assert_row_chars_equal(&editor, &[b"Helo world"]);
    }

    #[test]
    fn editor_delete_next_char() {
        let mut editor = Editor::default();
        for &b in b"Hello world!\nHappy New Year!" {
            editor.process_keypress(&Key::Char(b));
        }
        editor.process_keypress(&Key::Delete);
        assert_row_chars_equal(&editor, &[b"Hello world!", b"Happy New Year!"]);
        editor.move_cursor(&AKey::Left, true);
        editor.process_keypress(&Key::Delete);
        assert_row_chars_equal(&editor, &[b"Hello world!", b"Happy New ear!"]);
        editor.move_cursor(&AKey::Left, true);
        editor.move_cursor(&AKey::Left, true);
        editor.move_cursor(&AKey::Left, true);
        editor.process_keypress(&Key::Delete);
        assert_row_chars_equal(&editor, &[b"Hello world!Happy New ear!"]);
    }

    #[test]
    fn editor_move_cursor_left() {
        let mut editor = Editor::default();
        for &b in b"Hello world!\nHappy New Year!" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 15);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Left, true);
        assert_eq!(editor.cursor.x, 10);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Left, false);
        assert_eq!(editor.cursor.x, 9);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Left, true);
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Left, true);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Left, false);
        assert_eq!(editor.cursor.x, 12);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Left, true);
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Left, true);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Left, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn editor_move_cursor_up() {
        let mut editor = Editor::default();
        for &b in b"abcdefgh\nij\nklmnopqrstuvwxyz" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 16);
        assert_eq!(editor.cursor.y, 2);

        editor.move_cursor(&AKey::Up, false);
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Up, true);
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Up, false);
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn editor_move_cursor_right() {
        let mut editor = Editor::default();
        for &b in b"Hello world\nHappy New Year" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 14);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Right, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 2);

        editor.move_cursor(&AKey::Right, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 2);

        editor.move_cursor(&AKey::Up, true);
        editor.move_cursor(&AKey::Up, true);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Right, true);
        assert_eq!(editor.cursor.x, 5);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Right, true);
        assert_eq!(editor.cursor.x, 11);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Right, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn editor_move_cursor_down() {
        let mut editor = Editor::default();
        for &b in b"abcdefgh\nij\nklmnopqrstuvwxyz" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 16);
        assert_eq!(editor.cursor.y, 2);

        editor.move_cursor(&AKey::Down, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 3);

        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Right, true);
        assert_eq!(editor.cursor.x, 8);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Down, true);
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);

        editor.move_cursor(&AKey::Down, true);
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 2);

        editor.move_cursor(&AKey::Down, true);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 3);

        editor.move_cursor(&AKey::Down, false);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 3);
    }

    #[test]
    fn editor_press_home_key() {
        let mut editor = Editor::default();
        for &b in b"Hello\nWorld\nand\nFerris!" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 7);
        assert_eq!(editor.cursor.y, 3);

        editor.process_keypress(&Key::Home);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 3);

        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);

        editor.move_cursor(&AKey::Right, true);
        assert_eq!(editor.cursor.x, 5);
        assert_eq!(editor.cursor.y, 0);

        editor.process_keypress(&Key::Home);
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn editor_press_end_key() {
        let mut editor = Editor::default();
        for &b in b"Hello\nWorld\nand\nFerris!" {
            editor.process_keypress(&Key::Char(b));
        }

        // check current position
        assert_eq!(editor.cursor.x, 7);
        assert_eq!(editor.cursor.y, 3);

        editor.process_keypress(&Key::End);
        assert_eq!(editor.cursor.x, 7);
        assert_eq!(editor.cursor.y, 3);

        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);
        editor.move_cursor(&AKey::Up, false);

        assert_eq!(editor.cursor.x, 3);
        assert_eq!(editor.cursor.y, 0);

        editor.process_keypress(&Key::End);
        assert_eq!(editor.cursor.x, 5);
        assert_eq!(editor.cursor.y, 0);
    }

    #[rstest]
    #[case::beginning_of_first_row(b"Hello\nWorld!\n", (0, 0), &[&b"World!"[..], &b""[..]], 0)]
    #[case::middle_of_first_row(b"Hello\nWorld!\n", (3, 0), &[&b"World!"[..], &b""[..]], 0)]
    #[case::end_of_first_row(b"Hello\nWorld!\n", (5, 0), &[&b"World!"[..], &b""[..]], 0)]
    #[case::empty_first_row(b"\nHello", (0, 0), &[&b"Hello"[..]], 0)]
    #[case::beginning_of_only_row(b"Hello", (0, 0), &[&b""[..]], 0)]
    #[case::middle_of_only_row(b"Hello", (3, 0), &[&b""[..]], 0)]
    #[case::end_of_only_row(b"Hello", (5, 0), &[&b""[..]], 0)]
    #[case::beginning_of_middle_row(b"Hello\nWorld!\n", (0, 1), &[&b"Hello"[..], &b""[..]], 1)]
    #[case::middle_of_middle_row(b"Hello\nWorld!\n", (3, 1), &[&b"Hello"[..], &b""[..]], 1)]
    #[case::end_of_middle_row(b"Hello\nWorld!\n", (6, 1), &[&b"Hello"[..], &b""[..]], 1)]
    #[case::empty_middle_row(b"Hello\n\nWorld!", (0, 1), &[&b"Hello"[..], &b"World!"[..]], 1)]
    #[case::beginning_of_last_row(b"Hello\nWorld!", (0, 1), &[&b"Hello"[..]], 0)]
    #[case::middle_of_last_row(b"Hello\nWorld!", (3, 1), &[&b"Hello"[..]], 0)]
    #[case::end_of_last_row(b"Hello\nWorld!", (6, 1), &[&b"Hello"[..]], 0)]
    #[case::empty_last_row(b"Hello\n", (0, 1), &[&b"Hello"[..]], 0)]
    #[case::after_last_row(b"Hello\nWorld!", (0, 2), &[&b"Hello"[..], &b"World!"[..]], 2)]
    fn delete_current_row_updates_buffer_and_position(
        #[case] initial_buffer: &[u8], #[case] cursor_position: (usize, usize),
        #[case] expected_rows: &[&[u8]], #[case] expected_cursor_row: usize,
    ) {
        let mut editor = Editor::default();
        for &b in initial_buffer {
            editor.process_keypress(&Key::Char(b));
        }
        (editor.cursor.x, editor.cursor.y) = cursor_position;

        editor.delete_current_row();

        assert_row_chars_equal(&editor, expected_rows);
        assert_eq!(
            (editor.cursor.x, editor.cursor.y),
            (0, expected_cursor_row),
            "cursor is at {}:{}, expected {}:0",
            editor.cursor.y,
            editor.cursor.x,
            expected_cursor_row
        );
    }

    #[rstest]
    #[case::first_row(0)]
    #[case::middle_row(5)]
    #[case::last_row(9)]
    fn delete_current_row_updates_screen_cols_and_ln_pad(#[case] current_row: usize) {
        let mut editor = Editor { window_width: 100, ..Default::default() };
        for _ in 0..10 {
            editor.insert_new_line();
        }
        assert_eq!(editor.screen_cols, 96);
        assert_eq!(editor.ln_pad, 4);

        editor.cursor.y = current_row;
        editor.delete_current_row();

        assert_eq!(editor.screen_cols, 97);
        assert_eq!(editor.ln_pad, 3);
    }

    #[test]
    fn editor_save_and_load() -> Result<(), Error> {
        let mut editor = Editor::default();
        let content = b"first line\nsecond line\nthird line\nfourth line";
        for &b in content {
            editor.process_keypress(&Key::Char(b));
        }
        let temp_dir = if cfg!(target_arch = "wasm32") {
            std::path::PathBuf::from("/")
        } else {
            std::env::temp_dir()
        };
        let file_path = temp_dir.join("kibi_test.tmp");
        let file_name = file_path.to_str().unwrap();

        let bytes_written = editor.save(file_name)?;
        assert_eq!(bytes_written, content.len());

        let mut new_editor = Editor::default();
        new_editor.load(&file_path)?;

        assert_row_chars_equal(&new_editor, &[
            b"first line",
            b"second line",
            b"third line",
            b"fourth line",
            b"",
        ]);

        std::fs::remove_file(file_path)?;

        Ok(())
    }

    #[test]
    fn delete_current_row_updates_syntax_highlighting() {
        let mut editor = Editor {
            syntax: SyntaxConf {
                ml_comment_delims: Some(("/*".to_owned(), "*/".to_owned())),
                ..Default::default()
            },
            ..Default::default()
        };
        for &b in b"A\nb/*c\nd\ne\nf*/g\nh" {
            editor.process_keypress(&Key::Char(b));
        }

        assert_row_chars_equal(&editor, &[b"A", b"b/*c", b"d", b"e", b"f*/g", b"h"]);
        assert_row_synthax_highlighting_types_equal(&editor, &[
            &[HlType::Normal],
            &[HlType::Normal, HlType::MlComment, HlType::MlComment, HlType::MlComment],
            &[HlType::MlComment],
            &[HlType::MlComment],
            &[HlType::MlComment, HlType::MlComment, HlType::MlComment, HlType::Normal],
            &[HlType::Normal],
        ]);

        (editor.cursor.x, editor.cursor.y) = (0, 4);
        editor.delete_current_row();

        assert_row_chars_equal(&editor, &[b"A", b"b/*c", b"d", b"e", b"h"]);
        assert_row_synthax_highlighting_types_equal(&editor, &[
            &[HlType::Normal],
            &[HlType::Normal, HlType::MlComment, HlType::MlComment, HlType::MlComment],
            &[HlType::MlComment],
            &[HlType::MlComment],
            &[HlType::MlComment],
        ]);

        (editor.cursor.x, editor.cursor.y) = (0, 1);
        editor.delete_current_row();

        assert_row_chars_equal(&editor, &[b"A", b"d", b"e", b"h"]);
        assert_row_synthax_highlighting_types_equal(&editor, &[
            &[HlType::Normal],
            &[HlType::Normal],
            &[HlType::Normal],
            &[HlType::Normal],
        ]);
    }

    #[test]
    fn loop_until_keypress() -> Result<(), Error> {
        let mut editor = Editor::default();
        let mut fake_stdin = Cursor::new(
            b"abc\x1b[A\x1b[B\x1b[C\x1b[D\x1b[H\x1bOH\x1b[F\x1bOF\x1b[1;5C\x1b[5C\x1b[99",
        );
        for expected_key in [
            Key::Char(b'a'),
            Key::Char(b'b'),
            Key::Char(b'c'),
            Key::Arrow(AKey::Up),
            Key::Arrow(AKey::Down),
            Key::Arrow(AKey::Right),
            Key::Arrow(AKey::Left),
            Key::Home,
            Key::Home,
            Key::End,
            Key::End,
            Key::CtrlArrow(AKey::Right),
            Key::CtrlArrow(AKey::Right),
            Key::Escape,
        ] {
            assert_eq!(editor.loop_until_keypress(&mut fake_stdin)?, expected_key);
        }
        Ok(())
    }

    #[rstest]
    #[case::ascii_completed(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(b'\r')], &PromptState::Completed(String::from("Hi")))]
    #[case::escape(&[Key::Char(b'H'), Key::Char(b'i'), Key::Escape], &PromptState::Cancelled)]
    #[case::exit(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(EXIT)], &PromptState::Cancelled)]
    #[case::skip_ascii_control(&[Key::Char(b'\x0A')], &PromptState::Active(String::new()))]
    #[case::unsupported_non_ascii(&[Key::Char(b'\xEF')], &PromptState::Active(String::new()))]
    #[case::backspace(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(BACKSPACE), Key::Char(BACKSPACE)], &PromptState::Active(String::new()))]
    #[case::delete_bis(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(DELETE_BIS), Key::Char(DELETE_BIS), Key::Char(DELETE_BIS)], &PromptState::Active(String::new()))]
    fn process_prompt_keypresses(#[case] keys: &[Key], #[case] expected_final_state: &PromptState) {
        let mut prompt_state = PromptState::Active(String::new());
        for key in keys {
            if let PromptState::Active(buffer) = prompt_state {
                prompt_state = process_prompt_keypress(buffer, key);
            } else {
                panic!("Prompt state: {prompt_state:?} is not active")
            }
        }
        assert_eq!(prompt_state, *expected_final_state);
    }

    #[rstest]
    #[case(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(BACKSPACE), Key::Char(b'e'), Key::Char(b'l'), Key::Char(b'l'), Key::Char(b'o')], "Hello")]
    #[case(&[Key::Char(b'H'), Key::Char(b'i'), Key::Char(BACKSPACE), Key::Char(BACKSPACE), Key::Char(BACKSPACE)], "")]
    fn process_find_keypress_completed(#[case] keys: &[Key], #[case] expected_final_value: &str) {
        let mut ed: Editor = Editor::default();
        ed.insert_new_line();
        let mut prompt_mode = Some(PromptMode::Find(String::new(), CursorState::default(), None));
        for key in keys {
            prompt_mode = prompt_mode
                .take()
                .and_then(|prompt_mode| prompt_mode.process_keypress(&mut ed, key));
        }
        assert_eq!(
            prompt_mode,
            Some(PromptMode::Find(
                String::from(expected_final_value),
                CursorState::default(),
                None
            ))
        );
        prompt_mode = prompt_mode
            .take()
            .and_then(|prompt_mode| prompt_mode.process_keypress(&mut ed, &Key::Char(b'\r')));
        assert_eq!(prompt_mode, None);
    }
}
