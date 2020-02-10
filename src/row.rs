//! # Row
//!
//! Utilities for rows. A `Row` owns the underlying characters, the rendered string and the syntax
//! highlighting information.

use crate::ansi_escape::*;
use crate::syntax::{HighlightType, SyntaxConfig};

/// The "Highlight State" of the row
#[derive(Clone, Copy, PartialEq)]
pub(super) enum HLState {
    /// Normal state.
    Normal,
    /// A multi-line comment has been open, but not yet closed.
    MultiLineComment,
    /// A string has been open with the given quote character (b'\'' or b'"'), but not yet closed.
    String(u8),
    /// A multi-line string has been open, but not yet closed.
    MultiLineString,
}

impl Default for HLState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Represents a row of characters and how it is rendered.
#[derive(Default)]
pub(super) struct Row {
    /// The characters of the row.
    pub(super) chars: Vec<u8>,
    /// How the characters are rendered. In particular, tabs are converted into several spaces.
    pub(super) render: String,
    /// The vector of `HighlightType` for each rendered character.
    pub(super) hl: Vec<HighlightType>,
    /// The final state of the row.
    pub(super) hl_state: HLState,
    /// If not `None`, the range that is currently matched during a FIND operation.
    pub(super) match_segment: Option<std::ops::Range<usize>>,
}

impl Row {
    /// Create a new row, containing characters `chars`.
    pub(super) fn new(chars: Vec<u8>) -> Self {
        Self { chars, ..Self::default() }
    }

    // TODO: Combine update and update_syntax
    /// Update the row: convert tabs into spaces and compute highlight symbols
    pub(super) fn update(
        &mut self,
        syntax: &SyntaxConfig,
        previous_hl_state: HLState,
        tab: usize,
    ) -> HLState {
        self.render.clear();
        for c in &self.chars {
            if *c == b'\t' {
                self.render.push_str(&" ".repeat(tab - (self.render.len() % tab)));
            } else {
                self.render.push(*c as char);
            }
        }
        self.update_syntax(syntax, previous_hl_state)
    }
    /// Convert an index in `self.chars` to the corresponding index in `self.render`.
    pub(super) fn cx2rx(&self, cx: usize, tab: usize) -> usize {
        self.chars[..cx].iter().fold(0, |r, c| r + if *c == b'\t' { tab - (r % tab) } else { 1 })
    }
    /// Convert an index in `self.render` to the corresponding index in `self.chars`.
    /// If the index is too large, return the size of `self.chars`.
    pub(super) fn rx2cx(&self, rx: usize, tab: usize) -> usize {
        let mut current_rx = 0;
        for (cx, c) in self.chars.iter().enumerate() {
            current_rx += if *c == b'\t' { tab - (current_rx % tab) } else { 1 };
            if current_rx > rx {
                return cx;
            }
        }
        self.chars.len()
    }
    /// Update the syntax highlighting types of the row.
    fn update_syntax(&mut self, syntax: &SyntaxConfig, mut hl_state: HLState) -> HLState {
        self.hl.clear();
        let line = self.render.as_bytes();

        let find_substring =
            |i: usize, s: &str| line.get(i..(i + s.len())).map_or(false, |r| r.eq(s.as_bytes()));

        let push_repeat =
            |hl_vec: &mut Vec<HighlightType>, hl_type, n| (0..n).for_each(|_| hl_vec.push(hl_type));

        while self.hl.len() < line.len() {
            let i = self.hl.len();

            for comment_start in &syntax.sl_comment_start {
                if hl_state == HLState::Normal && find_substring(i, comment_start) {
                    push_repeat(&mut self.hl, HighlightType::Comment, line.len() - i);
                    continue;
                }
            }
            if let Some((mc_start, mc_end)) = &syntax.ml_comment_delim {
                if hl_state == HLState::MultiLineComment {
                    if find_substring(i, mc_end) {
                        // Highlight the remaining symbols of the multi line comment end
                        push_repeat(&mut self.hl, HighlightType::MLComment, mc_end.len());
                        hl_state = HLState::Normal;
                    } else {
                        self.hl.push(HighlightType::MLComment);
                    }
                    continue;
                } else if hl_state == HLState::Normal && find_substring(i, mc_start) {
                    // Highlight the remaining symbols of the multi line comment start
                    push_repeat(&mut self.hl, HighlightType::MLComment, mc_start.len());
                    hl_state = HLState::MultiLineComment;
                    continue;
                }
            }

            // TODO: Reuse some code from the multiline comment part above?
            if let Some(ml_string_delim) = &syntax.ml_string_delim {
                if hl_state == HLState::MultiLineString {
                    if find_substring(i, ml_string_delim) {
                        // Highlight the remaining symbol of the delimiter
                        push_repeat(&mut self.hl, HighlightType::MLString, ml_string_delim.len());
                        hl_state = HLState::Normal;
                    } else {
                        self.hl.push(HighlightType::MLString);
                    }
                    continue;
                } else if find_substring(i, ml_string_delim) {
                    push_repeat(&mut self.hl, HighlightType::MLString, ml_string_delim.len());
                    hl_state = HLState::MultiLineString;
                    continue;
                }
            }

            let c = line[i];

            // At this point, hl_state is Normal or String

            if syntax.hightlight_sl_strings {
                if let HLState::String(quote) = hl_state {
                    self.hl.push(HighlightType::String);
                    if c == quote {
                        hl_state = HLState::Normal;
                    } else if c == b'\\' && i != line.len() - 1 {
                        self.hl.push(HighlightType::String);
                    }
                    continue;
                } else if c == b'"' || c == b'\'' {
                    hl_state = HLState::String(c);
                    self.hl.push(HighlightType::String);
                    continue;
                }
            }

            let prev_sep = (i == 0) || is_separator(line[i - 1]);
            let prev_hl_type = if i == 0 { HighlightType::Normal } else { self.hl[i - 1] };

            if syntax.highlight_numbers
                && ((c.is_ascii_digit() && prev_sep)
                    || (prev_hl_type == HighlightType::Number && !prev_sep && !is_separator(c)))
            {
                self.hl.push(HighlightType::Number);
                continue;
            }

            if prev_sep {
                for (keyword_highlight_type, keyword_list) in &syntax.keywords {
                    for keyword in keyword_list.iter() {
                        if find_substring(i, keyword)
                            // Make sure that names such as in_comment are not partially 
                            // highlighted (even though "in" is a keyword in rust)
                            && line.get(i + keyword.len()).map_or(true, |c| is_separator(*c))
                        {
                            push_repeat(&mut self.hl, *keyword_highlight_type, keyword.len())
                        }
                    }
                }
            }

            self.hl.push(HighlightType::Normal);
        }
        self.hl_state = match hl_state {
            // String state doesn't propagate to the next row
            HLState::String(_) => HLState::Normal,
            _ => hl_state,
        };
        self.hl_state
    }
    /// Draw the row and write the result to a buffer. An `offset` can be given, as well as a limit
    /// on the length of the row (`max_len`). After writing the characters, clear the rest of the
    /// line and move the cursor to the start of the next line.
    pub(super) fn draw(&self, offset: usize, max_len: usize, buffer: &mut String) {
        let mut current_hl_type = HighlightType::Normal;
        let bytes = self.render.bytes().skip(offset).take(max_len);
        for (c, (i, mut hl_type)) in bytes.zip(self.hl.iter().enumerate().skip(offset)) {
            if c.is_ascii_control() {
                let rendered_char = if c <= 26 { (b'@' + c) as char } else { '?' };
                buffer.push_str(&format!("{}{}{}", REVERSE_VIDEO, rendered_char, RESET_FMT,));
                // Restore previous color
                if current_hl_type != HighlightType::Normal {
                    buffer.push_str(&current_hl_type.to_string());
                }
            } else {
                if let Some(match_segment) = &self.match_segment {
                    if match_segment.contains(&i) {
                        // Set the highlight type to Match, i.e. set the background to cyan
                        hl_type = &HighlightType::Match
                    } else if i == match_segment.end {
                        // Reset the formatting, in particular the background
                        buffer.push_str(RESET_FMT)
                    }
                }
                if current_hl_type != *hl_type {
                    buffer.push_str(&hl_type.to_string());
                    current_hl_type = *hl_type;
                }
                buffer.push(c as char);
            }
        }
        buffer.push_str(RESET_FMT);
    }
}

/// Return whether `c` is an ASCII separator.
fn is_separator(c: u8) -> bool {
    c.is_ascii_whitespace() || c == b'\0' || (c.is_ascii_punctuation() && c != b'_')
}
