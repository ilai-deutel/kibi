use std::io::{self, BufRead, BufReader, ErrorKind::NotFound, Read, Seek, Write};
use std::iter::{self, repeat, successors};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::{fmt::Display, fs::File, path::Path, time::Instant};

use nix::sys::termios::Termios;
use signal_hook::{iterator::Signals, SIGWINCH};

use crate::row::{HLState, Row};
use crate::{ansi_escape::*, syntax::SyntaxConf, terminal, Config, Error};

const fn ctrl_key(key: u8) -> u8 { key & 0x1f }
const EXIT: u8 = ctrl_key(b'Q');
const DELETE_BIS: u8 = ctrl_key(b'H');
const REFRESH_SCREEN: u8 = ctrl_key(b'L');
const SAVE: u8 = ctrl_key(b'S');
const FIND: u8 = ctrl_key(b'F');
const GOTO: u8 = ctrl_key(b'G');
const DUPLICATE: u8 = ctrl_key(b'D');
const BACKSPACE: u8 = 127;

const HELP_MESSAGE: &str =
    "Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find | Ctrl-G = go to | Ctrl-D = duplicate";

/// set_status! sets a formatted status message for the editor.
/// Example usage: `set_status!(editor, "{} written to {}", file_size, file_name)`
macro_rules! set_status {
    ($editor:expr, $($arg:expr),*) => ($editor.status_msg = Some(StatusMessage::new(format!($($arg),*))))
}

/// Enum of input keys
enum Key {
    Arrow(AKey),
    CtrlArrow(AKey),
    Page(PageKey),
    Home,
    End,
    Delete,
    Escape,
    Char(u8),
}

/// Enum of arrow keys
enum AKey {
    Left,
    Right,
    Up,
    Down,
}

/// Enum of page keys
enum PageKey {
    Up,
    Down,
}

/// Describes the cursor position and the screen offset
#[derive(Default, Clone)]
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

/// The `Editor` struct, contains the state and configuration of the text editor.
pub struct Editor<'a> {
    /// If not `None`, the current prompt mode (Save, Find, GoTo). If `None`, we are in regular
    /// edition mode.
    prompt_mode: Option<PromptMode>,
    /// The current state of the cursor.
    cursor: CursorState,
    /// The padding size used on the left for line numbering.
    ln_pad: usize,
    /// The width of the current window. Will be updated when the window is resized.
    window_width: usize,
    /// The number of rows that can be used for the editor, excluding the status bar and the message
    /// bar
    screen_rows: usize,
    /// The number of columns that can be used for the editor, excluding the part used for line numbers
    screen_cols: usize,
    /// The collection of rows, including the content and the syntax highlighting information.
    rows: Vec<Row>,
    /// Whether the document has been modified since it was open.
    dirty: bool,
    /// The configuration for the editor.
    config: &'a Config,
    /// The number of warnings remaining before we can quit without saving. Defaults to
    /// `config.quit_times`, then decreases to 0.
    quit_times: usize,
    /// The file name. If None, the user will be prompted for a file name the first time they try to
    /// save.
    // TODO: It may be better to store a PathBuf instead
    file_name: Option<String>,
    /// The current status message being shown.
    status_msg: Option<StatusMessage>,
    /// The syntax configuration corresponding to the current file's extension.
    syntax: SyntaxConf,
    /// The number of bytes contained in `rows`. This excludes new lines.
    n_bytes: u64,
    /// A channel receiver for the "window size changed" message. A message is received shortly
    /// after a SIGWINCH signal is received.
    ws_changed_receiver: Receiver<()>,
    /// The original termios configuration. It will be restored when the `Editor` is dropped.
    orig_termios: Termios,
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
    let quo_rem = successors(Some((n, 0)), |(q, _)| Some((q / 1024, q % 1024)).filter(|u| u.0 > 0));
    // unwrap(): quo_rem is never empty (since `successors` has an initial value), so _.last()
    // cannot be None
    let ((q, r), prefix) = quo_rem.zip(&["", "k", "M", "G", "T", "P", "E", "Z"]).last().unwrap();
    format!("{:.2$}{}B", q as f32 + r as f32 / 1024., prefix, p = if *prefix == "" { 0 } else { 2 })
}

impl<'a> Editor<'a> {
    /// Initialize the text editor.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an error occurs when enabling termios raw mode, creating the signal hook
    /// or when obtaining the terminal window size.
    pub fn new(config: &'a Config) -> Result<Self, Error> {
        // Enable termios raw mode and store the original (non-raw) termios.
        let orig_termios = terminal::enable_raw_mode()?;

        // Create a channel for receiving window size update requests
        let (ws_changed_tx, ws_changed_rx) = mpsc::sync_channel(1);
        // Spawn a new thread that will push to the aforementioned channel every time the SIGWINCH
        // signal is received
        let signals = Signals::new(&[SIGWINCH])?;
        std::thread::spawn(move || signals.forever().for_each(|_| ws_changed_tx.send(()).unwrap()));

        let mut editor = Self {
            prompt_mode: None,
            cursor: CursorState::default(),
            // Will be updated with update_window_size() below
            ln_pad: 0,
            window_width: 0,
            screen_rows: 0,
            screen_cols: 0,
            rows: Vec::new(),
            dirty: false,
            quit_times: config.quit_times,
            config,
            file_name: None,
            status_msg: Some(StatusMessage::new(HELP_MESSAGE.to_string())),
            syntax: SyntaxConf::default(),
            n_bytes: 0,
            ws_changed_receiver: ws_changed_rx,
            orig_termios,
        };
        editor.update_window_size()?;

        Ok(editor)
    }

    /// Return the current row if the cursor points to an existing row, `None` otherwise.
    fn current_row(&self) -> Option<&Row> { self.rows.get(self.cursor.y) }

    /// Return the position of the cursor, in terms of rendered characters (as opposed to
    /// `self.cursor.x`, which is the position of the cursor in terms of bytes.
    fn rx(&self) -> usize { self.current_row().map_or(0, |r| r.cx2rx[self.cursor.x]) }

    /// Move the cursor following an arrow key (← → ↑ ↓).
    fn move_cursor(&mut self, key: &AKey) {
        match (key, self.current_row()) {
            (AKey::Left, Some(row)) if self.cursor.x > 0 => {
                self.cursor.x -= row.get_char_size(row.cx2rx[self.cursor.x] - 1)
            }
            (AKey::Left, _) if self.cursor.y > 0 => {
                // ← at the beginning of the line: move to the end of the previous line. The x
                // position will be adjusted after this `match` to accommodate the current row
                // length, so we can just set here to the maximum possible value here.
                self.cursor.y -= 1;
                self.cursor.x = usize::max_value();
            }
            (AKey::Right, Some(row)) if self.cursor.x < row.chars.len() => {
                self.cursor.x += row.get_char_size(row.cx2rx[self.cursor.x])
            }
            (AKey::Right, Some(_)) => {
                // Move to the next line
                self.cursor.y += 1;
                self.cursor.x = 0;
            }
            // TODO: For Up and Down, move self.cursor.x to be consistent with tabs and UTF-8
            //  characters, i.e. according to rx
            (AKey::Up, _) if self.cursor.y > 0 => self.cursor.y -= 1,
            (AKey::Down, Some(_)) => self.cursor.y += 1,
            _ => (),
        }
        self.update_cursor_x_position()
    }

    /// Update the cursor x position. If the cursor y position has changed, the current position
    /// might be illegal (x is further right than the last character of the row). If that is the
    /// case, clamp `self.cursor.x`.
    fn update_cursor_x_position(&mut self) {
        self.cursor.x = self.cursor.x.min(self.current_row().map_or(0, |row| row.chars.len()))
    }

    /// Run a loop to obtain the key that was pressed. At each iteration of the loop (until a key is
    /// pressed), we listen to the `ws_changed` channel to check if a window size change signal has
    /// been received. When bytes are received, we match to a corresponding `Key`. In particular,
    /// we handle ANSI escape codes to return `Key::Delete`, `Key::Home` etc.
    fn loop_until_keypress(&mut self) -> Result<Key, Error> {
        loop {
            // Handle window size if a signal has be received
            match self.ws_changed_receiver.try_recv() {
                Ok(()) => {
                    self.update_window_size()?;
                    self.refresh_screen()?;
                }
                // No signal has been received, that's ok.
                Err(TryRecvError::Empty) => (),
                Err(err) => return Err(Error::MPSCTryRecv(err)),
            }
            let mut bytes = io::stdin().bytes();
            // Match on the next byte received or, if the first byte is <ESC> ('\x1b'), on the next
            // few bytes.
            match bytes.next().transpose()? {
                Some(b'\x1b') => {
                    return Ok(match bytes.next().transpose()? {
                        Some(b @ b'[') | Some(b @ b'O') => match (b, bytes.next().transpose()?) {
                            (b'[', Some(b'A')) => Key::Arrow(AKey::Up),
                            (b'[', Some(b'B')) => Key::Arrow(AKey::Down),
                            (b'[', Some(b'C')) => Key::Arrow(AKey::Right),
                            (b'[', Some(b'D')) => Key::Arrow(AKey::Left),
                            (b'[', Some(b'H')) | (b'O', Some(b'H')) => Key::Home,
                            (b'[', Some(b'F')) | (b'O', Some(b'F')) => Key::End,
                            (b'[', mut c @ Some(b'0'..=b'8')) => {
                                let mut d = bytes.next().transpose()?;
                                if let (Some(b'1'), Some(b';')) = (c, d) {
                                    // 1 is the default modifier value. Therefore, <ESC>[1;5C is
                                    // equivalent to <ESC>[5C, etc.
                                    c = bytes.next().transpose()?;
                                    d = bytes.next().transpose()?;
                                }
                                match (c, d) {
                                    (Some(c), Some(b'~')) if c == b'1' || c == b'7' => Key::Home,
                                    (Some(c), Some(b'~')) if c == b'4' || c == b'8' => Key::End,
                                    (Some(b'3'), Some(b'~')) => Key::Delete,
                                    (Some(b'5'), Some(b'~')) => Key::Page(PageKey::Up),
                                    (Some(b'6'), Some(b'~')) => Key::Page(PageKey::Down),
                                    (Some(b'5'), Some(b'A')) => Key::CtrlArrow(AKey::Up),
                                    (Some(b'5'), Some(b'B')) => Key::CtrlArrow(AKey::Down),
                                    (Some(b'5'), Some(b'C')) => Key::CtrlArrow(AKey::Right),
                                    (Some(b'5'), Some(b'D')) => Key::CtrlArrow(AKey::Left),
                                    _ => Key::Escape,
                                }
                            }
                            (b'O', Some(b'a')) => Key::CtrlArrow(AKey::Up),
                            (b'O', Some(b'b')) => Key::CtrlArrow(AKey::Down),
                            (b'O', Some(b'c')) => Key::CtrlArrow(AKey::Right),
                            (b'O', Some(b'd')) => Key::CtrlArrow(AKey::Left),
                            _ => Key::Escape,
                        },
                        _ => Key::Escape,
                    });
                }
                Some(a) => return Ok(Key::Char(a)),
                None => continue,
            }
        }
    }

    /// Update the `screen_rows`, `window_width`, `screen_cols` and `ln_padding` attributes.
    fn update_window_size(&mut self) -> Result<(), Error> {
        let (window_height, window_width) = terminal::get_window_size()?;
        self.screen_rows = window_height.saturating_sub(2); // Make room for the status bar and status message
        self.window_width = window_width;
        self.update_screen_cols();
        Ok(())
    }

    /// Update the `screen_cols` and `ln_padding` attributes based on the maximum number of digits
    /// for line numbers (since the left padding depends on this number of digits).
    fn update_screen_cols(&mut self) {
        // The maximum number of digits to use for the line number is the number of digits of the
        // last line number. This is equal to the number of times we can divide this number by ten,
        // computed below using `successors`.
        let n_digits =
            successors(Some(self.rows.len()), |u| Some(u / 10).filter(|u| *u > 0)).count();
        let show_line_num = self.config.show_line_num && n_digits + 2 < self.window_width / 4;
        self.ln_pad = if show_line_num { n_digits + 2 } else { 0 };
        self.screen_cols = self.window_width.saturating_sub(self.ln_pad);
    }

    /// Given a file path, try to find a syntax highlighting configuration that matches the path
    /// extension in one of the config directories (`/etc/kibi/syntax.d`, etc.). If such a
    /// configuration is found, set the `syntax` attribute of the editor.
    fn select_syntax_highlight(&mut self, path: &Path) -> Result<(), Error> {
        let conf_dirs = &self.config.conf_dirs;
        let extension = path.extension().and_then(std::ffi::OsStr::to_str);
        if let Some(s) = extension.and_then(|e| SyntaxConf::get(e, conf_dirs).transpose()) {
            self.syntax = s?
        }
        Ok(())
    }

    /// Update a row, given its index. If `ignore_following_rows` is `false` and the highlight state
    /// has changed during the update (for instance, it is now in "multi-line comment" state, keep
    /// updating the next rows
    fn update_row(&mut self, y: usize, ignore_following_rows: bool) {
        let mut hl_state = if y > 0 { self.rows[y - 1].hl_state } else { HLState::Normal };
        for row in self.rows.iter_mut().skip(y) {
            let previous_hl_state = row.hl_state;
            hl_state = row.update(&self.syntax, hl_state, self.config.tab_stop);
            if ignore_following_rows || hl_state == previous_hl_state {
                return;
            }
            // If the state has changed (for instance, a multi-line comment started in this row),
            // continue updating the following rows
        }
    }

    /// Update all the rows.
    fn update_all_rows(&mut self) {
        let mut hl_state = HLState::Normal;
        for row in &mut self.rows {
            hl_state = row.update(&self.syntax, hl_state, self.config.tab_stop);
        }
    }

    /// Insert a byte at the current cursor position. If there is no row at the current cursor
    /// position, add a new row and insert the byte.
    fn insert_byte(&mut self, c: u8) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            row.chars.insert(self.cursor.x, c)
        } else {
            self.rows.push(Row::new(vec![c]));
            // The number of rows has changed. The left padding may need to be updated.
            self.update_screen_cols();
        }
        self.update_row(self.cursor.y, false);
        self.cursor.x += 1;
        self.n_bytes += 1;
        self.dirty = true
    }

    /// Insert a new line at the current cursor position and move the cursor to the start of the new
    /// line. If the cursor is in the middle of a row, split off that row.
    fn insert_new_line(&mut self) {
        let (position, new_row_chars) = if self.cursor.x == 0 {
            (self.cursor.y, Vec::new())
        } else {
            // self.rows[self.cursor.y] must exist, since cursor.x = 0 for any cursor.y ≥ row.len()
            let new_chars = self.rows[self.cursor.y].chars.split_off(self.cursor.x);
            self.update_row(self.cursor.y, true);
            (self.cursor.y + 1, new_chars)
        };
        self.rows.insert(position, Row::new(new_row_chars));
        self.update_row(position, false);
        self.update_screen_cols();
        self.cursor.y += 1;
        self.cursor.x = 0;
        self.dirty = true;
    }

    /// Delete a character at the current cursor position. If the cursor is located at the beginning
    /// of a row that is not the first or last row, merge the current row and the previous row. If
    /// the cursor is located after the last row, move up to the last character of the previous row.
    fn delete_char(&mut self) {
        if self.cursor.x > 0 {
            let row = &mut self.rows[self.cursor.y];
            // Obtain the number of bytes to be removed: could be 1-4 (UTF-8 character size).
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
            // The number of rows has changed. The left padding may need to be updated.
            self.update_screen_cols();
            self.dirty = true;
            self.cursor.y -= 1;
        } else if self.cursor.y == self.rows.len() {
            // If the cursor is located after the last row, pressing backspace is equivalent to
            // pressing the left arrow key.
            self.move_cursor(&AKey::Left);
        }
    }

    fn duplicate_current_row(&mut self) {
        if let Some(row) = self.current_row() {
            let new_row = Row::new(row.chars.clone());
            self.n_bytes += new_row.chars.len() as u64;
            self.rows.insert(self.cursor.y + 1, new_row);
            self.update_row(self.cursor.y + 1, false);
            self.cursor.y += 1;
            self.dirty = true;
            // The line number has changed
            self.update_screen_cols();
        }
    }

    /// Try to load a file. If found, load the rows and update the render and syntax highlighting.
    /// If not found, do not return an error.
    fn load(&mut self, path: &Path) -> Result<(), Error> {
        match File::open(path) {
            Ok(file) => {
                for line in BufReader::new(file).split(b'\n') {
                    self.rows.push(Row::new(line?));
                }
                // If the file ends with an empty line or is empty, we need to append an empty row
                // to `self.rows`. Unfortunately, BufReader::split doesn't yield an empty Vec in
                // this case, so we need to check the last byte directly.
                let mut file = File::open(path)?;
                file.seek(io::SeekFrom::End(0))?;
                if file.bytes().next().transpose()?.map_or(true, |b| b == b'\n') {
                    self.rows.push(Row::new(Vec::new()));
                }
                self.update_all_rows();
                // The number of rows has changed. The left padding may need to be updated.
                self.update_screen_cols();
                self.n_bytes = self.rows.iter().map(|row| row.chars.len() as u64).sum();
            }
            Err(e) if e.kind() == NotFound => self.rows.push(Row::new(Vec::new())),
            Err(e) => return Err(e.into()),
        }
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
                file.write_all(&[b'\n'])?;
                written += 1
            }
        }
        file.sync_all()?;
        Ok(written)
    }

    /// Save the text to a file and handle all errors. Errors and success messages will be printed
    /// to the status bar. Return whether the file was successfully saved.
    fn save_and_handle_io_errors(&mut self, file_name: &str) -> bool {
        match self.save(file_name) {
            Ok(written) => {
                self.dirty = false;
                set_status!(self, "{} written to {}", format_size(written as u64), file_name);
                true
            }
            Err(err) => {
                set_status!(self, "Can't save! I/O error: {}", err);
                false
            }
        }
    }

    /// Save to a file after obtaining the file path from the prompt. If successful, the `file_name`
    /// attribute of the editor will be set and syntax highlighting will be updated.
    fn save_as(&mut self, file_name: String) -> Result<(), Error> {
        // TODO: What if file_name already exists?
        if self.save_and_handle_io_errors(&file_name) {
            // If save was successful
            self.select_syntax_highlight(Path::new(&file_name))?;
            self.file_name = Some(file_name);
            self.update_all_rows();
        }
        Ok(())
    }

    /// Scroll the terminal window vertically and horizontally (i.e. adjusting the row offset and
    /// the column offset) so that the cursor can be shown.
    fn scroll(&mut self) {
        let rx = self.rx();
        if self.cursor.y < self.cursor.roff {
            self.cursor.roff = self.cursor.y;
        } else if self.cursor.y >= self.cursor.roff + self.screen_rows {
            self.cursor.roff = self.cursor.y - self.screen_rows + 1;
        }

        if rx < self.cursor.coff {
            self.cursor.coff = rx;
        } else if rx >= self.cursor.coff + self.screen_cols {
            self.cursor.coff = rx - self.screen_cols + 1;
        }
    }

    /// Draw the left part of the screen: line numbers and vertical bar.
    fn draw_left_padding<T: Display>(&self, buffer: &mut String, val: T) {
        if self.ln_pad >= 2 {
            // \x1b[38;5;240m: Dark grey color; \u{2502}: pipe "│"
            buffer.push_str(&format!("\x1b[38;5;240m{:>1$} \u{2502}", val, self.ln_pad - 2));
            buffer.push_str(RESET_FMT)
        }
    }

    /// Return whether the file being edited is empty or not. If there is more than one row, even if
    /// all the rows are empty, `is_empty` returns `false`, since the text contains new lines.
    fn is_empty(&self) -> bool { self.rows.len() <= 1 && self.n_bytes == 0 }

    /// Draw rows of text and empty rows on the terminal, by adding characters to the buffer.
    fn draw_rows(&self, buffer: &mut String) {
        let row_it = self.rows.iter().map(Some).chain(repeat(None)).enumerate();
        for (i, row) in row_it.skip(self.cursor.roff).take(self.screen_rows) {
            buffer.push_str(CLEAR_LINE_RIGHT_OF_CURSOR);
            if let Some(row) = row {
                // Draw a row of text
                self.draw_left_padding(buffer, i + 1);
                row.draw(self.cursor.coff, self.screen_cols, buffer);
            } else {
                // Draw an empty row
                self.draw_left_padding(buffer, '~');
                if self.is_empty() && i == self.screen_rows / 3 {
                    let welcome_message = format!("Kibi - version {}", env!("CARGO_PKG_VERSION"));
                    buffer.push_str(&format!("{:^1$.1$}", welcome_message, self.screen_cols));
                }
            }
            buffer.push_str("\r\n");
        }
    }

    /// Draw the status bar on the terminal, by adding characters to the buffer.
    fn draw_status_bar(&self, buffer: &mut String) {
        // Left part of the status bar
        let modified = if self.dirty { " (modified)" } else { "" };
        let mut left =
            format!("{:.30}{}", self.file_name.as_deref().unwrap_or("[No Name]"), modified);
        left.truncate(self.window_width);

        // Right part of the status bar
        let size = format_size(self.n_bytes + self.rows.len().saturating_sub(1) as u64);
        let right =
            format!("{} | {} | {}:{}", self.syntax.name, size, self.cursor.y + 1, self.rx() + 1);

        // Draw
        let rw = self.window_width.saturating_sub(left.len());
        buffer.push_str(&format!("{}{}{:>4$.4$}{}\r\n", REVERSE_VIDEO, left, right, RESET_FMT, rw));
    }

    /// Draw the message bar on the terminal, by adding characters to the buffer.
    fn draw_message_bar(&self, buffer: &mut String) {
        buffer.push_str(CLEAR_LINE_RIGHT_OF_CURSOR);
        let msg_duration = self.config.message_duration;
        if let Some(sm) = self.status_msg.as_ref().filter(|sm| sm.time.elapsed() < msg_duration) {
            buffer.push_str(&sm.msg[..sm.msg.len().min(self.window_width)]);
        }
    }

    /// Refresh the screen: update the offsets, draw the rows, the status bar, the message bar, and
    /// move the cursor to the correct position.
    fn refresh_screen(&mut self) -> Result<(), Error> {
        self.scroll();
        let mut buffer = format!("{}{}", HIDE_CURSOR, MOVE_CURSOR_TO_START);
        self.draw_rows(&mut buffer);
        self.draw_status_bar(&mut buffer);
        self.draw_message_bar(&mut buffer);
        let (cursor_x, cursor_y) = if self.prompt_mode.is_none() {
            // If not in prompt mode, position the cursor according to the `cursor` attributes.
            (self.rx() - self.cursor.coff + 1 + self.ln_pad, self.cursor.y - self.cursor.roff + 1)
        } else {
            // If in prompt mode, position the cursor on the prompt line at the end of the line.
            (self.status_msg.as_ref().map_or(0, |sm| sm.msg.len() + 1), self.screen_rows + 2)
        };
        // Move the cursor
        buffer.push_str(&format!("\x1b[{};{}H{}", cursor_y, cursor_x, SHOW_CURSOR));
        terminal::print_and_flush(&buffer)
    }

    /// Process a key that has been pressed, when not in prompt mode. Returns whether the program
    /// should exit, and optionally the prompt mode to switch to.
    fn process_keypress(&mut self, key: &Key) -> Result<(bool, Option<PromptMode>), Error> {
        // This won't be mutated, unless key is Key::Character(EXIT)
        let mut quit_times = self.config.quit_times;
        let mut prompt_mode = None;

        match key {
            // TODO: CtrlArrow should move to next word
            Key::Arrow(arrow) | Key::CtrlArrow(arrow) => self.move_cursor(arrow),
            Key::Page(PageKey::Up) => {
                self.cursor.y = self.cursor.roff.saturating_sub(self.screen_rows);
                self.update_cursor_x_position();
            }
            Key::Page(PageKey::Down) => {
                self.cursor.y = (self.cursor.roff + 2 * self.screen_rows - 1).min(self.rows.len());
                self.update_cursor_x_position();
            }
            Key::Home => self.cursor.x = 0,
            Key::End => self.cursor.x = self.current_row().map_or(0, |row| row.chars.len()),
            Key::Char(b'\r') => self.insert_new_line(), // Enter
            Key::Char(BACKSPACE) | Key::Char(DELETE_BIS) => self.delete_char(), // Backspace or Ctrl + H
            Key::Delete => {
                self.move_cursor(&AKey::Right);
                self.delete_char()
            }
            Key::Escape | Key::Char(REFRESH_SCREEN) => (),
            Key::Char(EXIT) => {
                quit_times = self.quit_times - 1;
                if !self.dirty || quit_times == 0 {
                    return Ok((true, None));
                }
                let times = if quit_times > 1 { "times" } else { "time" };
                set_status!(self, "Press Ctrl+Q {} more {} to quit.", quit_times, times);
            }
            Key::Char(SAVE) => match self.file_name.take() {
                // TODO: Can we avoid using take() then reassigning the value to file_name?
                Some(file_name) => {
                    self.save_and_handle_io_errors(&file_name);
                    self.file_name = Some(file_name)
                }
                None => prompt_mode = Some(PromptMode::Save(String::new())),
            },
            Key::Char(FIND) => {
                prompt_mode = Some(PromptMode::Find(String::new(), self.cursor.clone(), None))
            }
            Key::Char(GOTO) => prompt_mode = Some(PromptMode::GoTo(String::new())),
            Key::Char(DUPLICATE) => self.duplicate_current_row(),
            Key::Char(c) => self.insert_byte(*c),
        }
        self.quit_times = quit_times;
        Ok((false, prompt_mode))
    }

    /// Try to find a query, this is called after pressing Ctrl-F and for each key that is pressed.
    /// `last_match` is the last row that was matched, `forward` indicates whether to search forward
    /// or backward. Returns the row of a new match, or `None` if the search was unsuccessful.
    fn find(&mut self, query: &str, last_match: &Option<usize>, forward: bool) -> Option<usize> {
        let num_rows = self.rows.len();
        let mut current = last_match.unwrap_or_else(|| num_rows.saturating_sub(1));
        // TODO: Handle multiple matches per line
        for _ in 0..num_rows {
            current = (current + if forward { 1 } else { num_rows - 1 }) % num_rows;
            let row = &mut self.rows[current];
            if let Some(rx) = row.render.find(&query) {
                self.cursor.y = current as usize;
                self.cursor.x = row.rx2cx[rx];
                // Try to reset the column offset; if the match is after the offset, this
                // will be updated in self.scroll() so that the result is visible
                self.cursor.coff = 0;
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
    pub fn run(&mut self, file_name: Option<String>) -> Result<(), Error> {
        if let Some(path) = file_name.as_ref().map(Path::new) {
            self.select_syntax_highlight(path)?;
            self.load(path)?;
        } else {
            self.rows.push(Row::new(Vec::new()));
        }
        self.file_name = file_name;
        loop {
            if let Some(mode) = self.prompt_mode.as_ref() {
                set_status!(self, "{}", mode.status_msg());
            }
            self.refresh_screen()?;
            let key = self.loop_until_keypress()?;
            // TODO: Can we avoid using take()?
            self.prompt_mode = match self.prompt_mode.take() {
                // process_keypress returns (should_quit, prompt_mode)
                None => match self.process_keypress(&key)? {
                    (true, _) => return Ok(()),
                    (false, prompt_mode) => prompt_mode,
                },
                Some(prompt_mode) => prompt_mode.process_keypress(self, &key)?,
            }
        }
    }
}

impl<'a> Drop for Editor<'a> {
    /// When the editor is droped, restore the original termios.
    fn drop(&mut self) {
        terminal::set_termios(&self.orig_termios).expect("Could not restore original termios.");
    }
}

/// The prompt mode.
enum PromptMode {
    /// Save(prompt buffer)
    Save(String),
    /// Find(prompt buffer, saved cursor state, last match)
    Find(String, CursorState, Option<usize>),
    /// GoTo(prompt buffer)
    GoTo(String),
}

// TODO: Use trait with mode_status_msg and process_keypress, implement the trait for separate
//  structs for Save and Find?
impl PromptMode {
    /// Return the status message to print for the selected `PromptMode`.
    fn status_msg(&self) -> String {
        match self {
            Self::Save(buffer) => format!("Save as: {}", buffer),
            Self::Find(buffer, ..) => format!("Search (Use ESC/Arrows/Enter): {}", buffer),
            Self::GoTo(buffer) => format!("Enter line number[:column number]: {}", buffer),
        }
    }

    /// Process a keypress event for the selected `PromptMode`.
    fn process_keypress(self, ed: &mut Editor, key: &Key) -> Result<Option<Self>, Error> {
        ed.status_msg = None;
        match self {
            Self::Save(b) => match process_prompt_keypress(b, key) {
                PromptState::Active(b) => return Ok(Some(Self::Save(b))),
                PromptState::Cancelled => set_status!(ed, "Save aborted"),
                PromptState::Completed(file_name) => ed.save_as(file_name)?,
            },
            Self::Find(b, saved_cursor, last_match) => {
                if let Some(row_idx) = last_match {
                    ed.rows[row_idx].match_segment = None
                }
                match process_prompt_keypress(b, key) {
                    PromptState::Active(query) => {
                        let (last_match, forward) = match key {
                            Key::Arrow(AKey::Right) | Key::Arrow(AKey::Down) | Key::Char(FIND) => {
                                (last_match, true)
                            }
                            Key::Arrow(AKey::Left) | Key::Arrow(AKey::Up) => (last_match, false),
                            _ => (None, true),
                        };
                        let curr_match = ed.find(&query, &last_match, forward);
                        return Ok(Some(Self::Find(query, saved_cursor, curr_match)));
                    }
                    // The prompt was cancelled. Restore the previous position.
                    PromptState::Cancelled => ed.cursor = saved_cursor,
                    // Cursor has already been moved, do nothing
                    PromptState::Completed(_) => (),
                }
            }
            Self::GoTo(b) => match process_prompt_keypress(b, key) {
                PromptState::Active(b) => return Ok(Some(Self::GoTo(b))),
                PromptState::Cancelled => (),
                PromptState::Completed(b) => {
                    let mut split = b
                        .splitn(2, ':')
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
                        (Err(e), _) | (_, Err(e)) => set_status!(ed, "Parsing error: {}", e),
                        (Ok(None), _) => (),
                    }
                }
            },
        }
        Ok(None)
    }
}

/// The state of the prompt after processing a keypress event.
enum PromptState {
    // Active contains the current buffer
    Active(String),
    // Completed contains the final string
    Completed(String),
    Cancelled,
}

/// Process a prompt keypress event and return the new state for the prompt.
fn process_prompt_keypress(mut buffer: String, key: &Key) -> PromptState {
    match key {
        Key::Char(b'\r') => return PromptState::Completed(buffer),
        Key::Escape | Key::Char(EXIT) => return PromptState::Cancelled,
        Key::Char(BACKSPACE) | Key::Char(DELETE_BIS) => {
            buffer.pop();
        }
        Key::Char(c @ 0..=126) if !c.is_ascii_control() => buffer.push(*c as char),
        // No-op
        _ => (),
    }
    PromptState::Active(buffer)
}
