//! # Row
//!
//! Utilities for rows. A `Row` owns the underlying characters, the rendered string and the syntax
//! highlighting information.

use std::{fmt::Write, iter::repeat};

use unicode_width::UnicodeWidthChar;

use crate::ansi_escape::{RESET_FMT, REVERSE_VIDEO};
use crate::error::Error;
use crate::syntax::{Conf as SyntaxConf, HlType};

/// The "Highlight State" of the row
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum HlState {
    /// Normal state.
    #[default]
    Normal,
    /// A multi-line comment has been open, but not yet closed.
    MultiLineComment,
    /// A string has been open with the given quote character (for instance b'\'' or b'"'), but not yet closed.
    String(u8),
    /// A multi-line string has been open, but not yet closed.
    MultiLineString,
}

/// Represents a row of characters and how it is rendered.
#[derive(Default)]
pub struct Row {
    /// The characters of the row.
    pub chars: Vec<u8>,
    /// How the characters are rendered. In particular, tabs are converted into several spaces, and
    /// bytes may be combined into single UTF-8 characters.
    render: String,
    /// Mapping from indices in `self.chars` to the corresponding indices in `self.render`.
    pub cx2rx: Vec<usize>,
    /// Mapping from indices in `self.render` to the corresponding indices in `self.chars`.
    pub rx2cx: Vec<usize>,
    /// The vector of `HLType` for each rendered character.
    hl: Vec<HlType>,
    /// The final state of the row.
    pub hl_state: HlState,
    /// If not `None`, the range that is currently matched during a FIND operation.
    pub match_segment: Option<std::ops::Range<usize>>,
}

impl Row {
    /// Create a new row, containing characters `chars`.
    pub fn new(chars: Vec<u8>) -> Self { Self { chars, cx2rx: vec![0], ..Self::default() } }

    // TODO: Combine update and update_syntax
    /// Update the row: convert tabs into spaces and compute highlight symbols
    /// The `hl_state` argument is the `HLState` for the previous row.
    pub fn update(&mut self, syntax: &SyntaxConf, hl_state: HlState, tab: usize) -> HlState {
        self.render.clear();
        self.cx2rx.clear();
        self.rx2cx.clear();
        let (mut cx, mut rx) = (0, 0);
        for c in String::from_utf8_lossy(&self.chars).chars() {
            // The number of bytes used to store the character
            let n_bytes = c.len_utf8();
            // The number of rendered characters
            let n_rend_chars = if c == '\t' { tab - (rx % tab) } else { c.width().unwrap_or(1) };
            self.render.push_str(&(if c == '\t' { " ".repeat(n_rend_chars) } else { c.into() }));
            self.cx2rx.extend(std::iter::repeat(rx).take(n_bytes));
            self.rx2cx.extend(std::iter::repeat(cx).take(n_rend_chars));
            (rx, cx) = (rx + n_rend_chars, cx + n_bytes);
        }
        self.cx2rx.push(rx);
        self.rx2cx.push(cx);
        self.update_syntax(syntax, hl_state)
    }

    /// Obtain the character size, in bytes, given its position in `self.render`. This is done in
    /// constant time by using the difference between `self.rx2cx[rx]` and the cx for the next
    /// character.
    pub fn get_char_size(&self, rx: usize) -> usize {
        let cx0 = self.rx2cx[rx];
        self.rx2cx.iter().skip(rx + 1).map(|cx| cx - cx0).find(|d| *d > 0).unwrap_or(1)
    }

    /// Update the syntax highlighting types of the row.
    fn update_syntax(&mut self, syntax: &SyntaxConf, mut hl_state: HlState) -> HlState {
        self.hl.clear();
        let line = self.render.as_bytes();

        // Delimiters for multi-line comments and multi-line strings, as Option<&String, &String>
        let ml_comment_delims = syntax.ml_comment_delims.as_ref().map(|(start, end)| (start, end));
        let ml_string_delims = syntax.ml_string_delim.as_ref().map(|x| (x, x));

        'syntax_loop: while self.hl.len() < line.len() {
            let i = self.hl.len();
            let find_str =
                |s: &str| line.get(i..(i + s.len())).map_or(false, |r| r.eq(s.as_bytes()));

            if hl_state == HlState::Normal && syntax.sl_comment_start.iter().any(|s| find_str(s)) {
                self.hl.extend(repeat(HlType::Comment).take(line.len() - i));
                continue;
            }

            // Multi-line strings and multi-line comments have the same behavior; the only
            // differences are: the start/end delimiters, the `HLState`, the `HLType`.
            for (delims, mstate, mtype) in &[
                (ml_comment_delims, HlState::MultiLineComment, HlType::MlComment),
                (ml_string_delims, HlState::MultiLineString, HlType::MlString),
            ] {
                if let Some((start, end)) = delims {
                    if hl_state == *mstate {
                        if find_str(end) {
                            // Highlight the remaining symbols of the multi line comment end
                            self.hl.extend(repeat(mtype).take(end.len()));
                            hl_state = HlState::Normal;
                        } else {
                            self.hl.push(*mtype);
                        }
                        continue 'syntax_loop;
                    } else if hl_state == HlState::Normal && find_str(start) {
                        // Highlight the remaining symbols of the multi line comment start
                        self.hl.extend(repeat(mtype).take(start.len()));
                        hl_state = *mstate;
                        continue 'syntax_loop;
                    }
                }
            }

            let c = line[i];

            // At this point, hl_state is Normal or String
            if let HlState::String(quote) = hl_state {
                self.hl.push(HlType::String);
                if c == quote {
                    hl_state = HlState::Normal;
                } else if c == b'\\' && i != line.len() - 1 {
                    self.hl.push(HlType::String);
                }
                continue;
            } else if syntax.sl_string_quotes.contains(&(c as char)) {
                hl_state = HlState::String(c);
                self.hl.push(HlType::String);
                continue;
            }

            let prev_sep = (i == 0) || is_sep(line[i - 1]);

            if syntax.highlight_numbers
                && ((c.is_ascii_digit() && prev_sep)
                    || (i != 0 && self.hl[i - 1] == HlType::Number && !prev_sep && !is_sep(c)))
            {
                self.hl.push(HlType::Number);
                continue;
            }

            if prev_sep {
                // This filters makes sure that names such as "in_comment" are not partially
                // highlighted (even though "in" is a keyword in rust)
                // The argument is the keyword that is matched at `i`.
                let s_filter = |kw: &str| line.get(i + kw.len()).map_or(true, |c| is_sep(*c));
                for (keyword_highlight_type, kws) in &syntax.keywords {
                    for keyword in kws.iter().filter(|kw| find_str(kw) && s_filter(kw)) {
                        self.hl.extend(repeat(*keyword_highlight_type).take(keyword.len()));
                    }
                }
            }

            self.hl.push(HlType::Normal);
        }

        // String state doesn't propagate to the next row
        self.hl_state =
            if matches!(hl_state, HlState::String(_)) { HlState::Normal } else { hl_state };
        self.hl_state
    }

    /// Draw the row and write the result to a buffer. An `offset` can be given, as well as a limit
    /// on the length of the row (`max_len`). After writing the characters, clear the rest of the
    /// line and move the cursor to the start of the next line.
    pub fn draw(&self, offset: usize, max_len: usize, buffer: &mut String) -> Result<(), Error> {
        let mut current_hl_type = HlType::Normal;
        let chars = self.render.chars().skip(offset).take(max_len);
        let mut rx = self.render.chars().take(offset).map(|c| c.width().unwrap_or(1)).sum();
        for (c, mut hl_type) in chars.zip(self.hl.iter().skip(offset)) {
            if c.is_ascii_control() {
                let rendered_char = if (c as u8) <= 26 { (b'@' + c as u8) as char } else { '?' };
                write!(buffer, "{REVERSE_VIDEO}{rendered_char}{RESET_FMT}")?;
                // Restore previous color
                if current_hl_type != HlType::Normal {
                    buffer.push_str(&current_hl_type.to_string());
                }
            } else {
                if let Some(match_segment) = &self.match_segment {
                    if match_segment.contains(&rx) {
                        // Set the highlight type to Match, i.e. set the background to cyan
                        hl_type = &HlType::Match;
                    } else if rx == match_segment.end {
                        // Reset the formatting, in particular the background
                        buffer.push_str(RESET_FMT);
                    }
                }
                if current_hl_type != *hl_type {
                    buffer.push_str(&hl_type.to_string());
                    current_hl_type = *hl_type;
                }
                buffer.push(c);
            }
            rx += c.width().unwrap_or(1);
        }
        buffer.push_str(RESET_FMT);
        Ok(())
    }
}

/// Return whether `c` is an ASCII separator.
fn is_sep(c: u8) -> bool {
    c.is_ascii_whitespace() || c == b'\0' || (c.is_ascii_punctuation() && c != b'_')
}
