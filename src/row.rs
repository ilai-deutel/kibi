//! # Row
//!
//! Utilities for rows. A `Row` owns the underlying characters, the rendered
//! string and the syntax highlighting information.

use std::{fmt::Write, iter::repeat};

use unicode_width::UnicodeWidthChar;

use crate::ansi_escape::{RESET_FMT, REVERSE_VIDEO};
use crate::error::Error;
use crate::syntax::{Conf as SyntaxConf, HlType};

/// The "Highlight State" of the row
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum HlState {
    /// Normal state.
    #[default]
    Normal,
    /// A multi-line comment has been open, but not yet closed.
    MultiLineComment,
    /// A string has been open with the given quote character (for instance
    /// b'\'' or b'"'), but not yet closed.
    String(u8),
    /// A multi-line string has been open, but not yet closed.
    MultiLineString,
}

/// Represents a row of characters and how it is rendered.
#[derive(Default)]
pub struct Row {
    /// The characters of the row.
    pub chars: Vec<u8>,
    /// How the characters are rendered. In particular, tabs are converted into
    /// several spaces, and bytes may be combined into single UTF-8
    /// characters.
    render: String,
    /// Mapping from indices in `self.chars` to the corresponding indices in
    /// `self.render`.
    pub cx2rx: Vec<usize>,
    /// Mapping from indices in `self.render` to the corresponding indices in
    /// `self.chars`.
    pub rx2cx: Vec<usize>,
    /// The vector of `HLType` for each rendered character.
    hl: Vec<HlType>,
    /// The final state of the row.
    pub hl_state: HlState,
    /// If not `None`, the range that is currently matched during a FIND
    /// operation.
    pub match_segment: Option<std::ops::Range<usize>>,
}

impl Row {
    /// Create a new row, containing characters `chars`.
    pub fn new(chars: Vec<u8>) -> Self { Self { chars, cx2rx: vec![0], ..Self::default() } }

    // TODO: Combine update and update_syntax
    /// Update the row: convert tabs into spaces and compute highlight symbols
    /// The `hl_state` argument is the `HLState` for the previous row.
    pub fn update(&mut self, syntax: &SyntaxConf, hl_state: HlState, tab: usize) -> HlState {
        let (..) = (self.render.clear(), self.cx2rx.clear(), self.rx2cx.clear());
        let (mut cx, mut rx) = (0, 0);
        for c in String::from_utf8_lossy(&self.chars).chars() {
            // The number of rendered characters
            let n_rend_chars = if c == '\t' { tab - (rx % tab) } else { c.width().unwrap_or(1) };
            self.render.push_str(&(if c == '\t' { " ".repeat(n_rend_chars) } else { c.into() }));
            self.cx2rx.extend(repeat(rx).take(c.len_utf8()));
            self.rx2cx.extend(repeat(cx).take(n_rend_chars));
            (rx, cx) = (rx + n_rend_chars, cx + c.len_utf8());
        }
        let (..) = (self.cx2rx.push(rx), self.rx2cx.push(cx));
        self.update_syntax(syntax, hl_state)
    }

    /// Obtain the character size, in bytes, given its position in
    /// `self.render`. This is done in constant time by using the difference
    /// between `self.rx2cx[rx]` and the cx for the next character.
    pub fn get_char_size(&self, rx: usize) -> usize {
        let cx0 = self.rx2cx[rx];
        self.rx2cx.iter().skip(rx + 1).map(|cx| cx - cx0).find(|d| *d > 0).unwrap_or(1)
    }

    /// Update the syntax highlighting types of the row.
    fn update_syntax(&mut self, syntax: &SyntaxConf, mut hl_state: HlState) -> HlState {
        self.hl.clear();
        let line = self.render.as_bytes();

        // Delimiters for multi-line comments and multi-line strings, as Option<&String,
        // &String>
        let ml_comment_delims = syntax.ml_comment_delims.as_ref().map(|(start, end)| (start, end));
        let ml_string_delims = syntax.ml_string_delim.as_ref().map(|x| (x, x));

        'syntax_loop: while self.hl.len() < line.len() {
            let i = self.hl.len();
            let find_str = |s: &str| line.get(i..(i + s.len())).is_some_and(|r| r.eq(s.as_bytes()));

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

    /// Draw the row and write the result to a buffer. An `offset` can be given,
    /// as well as a limit on the length of the row (`max_len`). After
    /// writing the characters, clear the rest of the line and move the
    /// cursor to the start of the next line.
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
const fn is_sep(c: u8) -> bool {
    c.is_ascii_whitespace() || c == b'\0' || (c.is_ascii_punctuation() && c != b'_')
}

#[cfg(test)]
mod tests {
    use super::*; // Imports Row, HlState, etc.
    use crate::syntax::Conf as SyntaxConf; 
    // Removed unused import: use crate::syntax::HlType; 
    // super::* already imports public items from the same crate like HlType

    // Helper function to create a default SyntaxConf for testing
    fn default_syntax_conf() -> SyntaxConf {
        SyntaxConf {
            name: "test_lang".to_string(),
            highlight_numbers: true, // Default to true for some tests
            sl_string_quotes: Vec::new(),
            sl_comment_start: Vec::new(),
            ml_comment_delims: None,
            ml_string_delim: None,
            keywords: Vec::new(),
        }
    }

    // --- Tests for Row::update ---

    #[test]
    fn test_row_update_simple_ascii() {
        let mut row = Row::new("hello".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        // Call update, which also calls update_syntax. We are primarily testing render/cx2rx/rx2cx here.
        row.update(&syntax_conf, HlState::Normal, 4); 
        assert_eq!(row.render, "hello");
        assert_eq!(row.chars, "hello".as_bytes());
        assert_eq!(row.cx2rx, vec![0, 1, 2, 3, 4, 5]); // cx 0-4 map to rx 0-4, cx 5 (end) maps to rx 5 (end)
        assert_eq!(row.rx2cx, vec![0, 1, 2, 3, 4, 5]); // rx 0-4 map to cx 0-4, rx 5 (end) maps to cx 5 (end)
    }

    #[test]
    fn test_row_update_empty_line() {
        let mut row = Row::new("".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row.render, "");
        assert_eq!(row.chars, "".as_bytes());
        // Based on code: clear() then push(0)
        assert_eq!(row.cx2rx, vec![0]); 
        assert_eq!(row.rx2cx, vec![0]);
    }

    #[test]
    fn test_row_update_tab_expansion_simple() {
        let mut row = Row::new("a\tb".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); // tab_stop = 4
        assert_eq!(row.render, "a   b"); // 'a' (rx 0), tab is 3 spaces (rx 1,2,3), 'b' (rx 4)
        assert_eq!(row.chars, "a\tb".as_bytes());
        // cx: 0('a') -> rx 0
        // cx: 1('\t') -> rx 1 (start of tab)
        // cx: 2('b') -> rx 4 (after tab)
        // cx: 3(end) -> rx 5 (end of render)
        assert_eq!(row.cx2rx, vec![0, 1, 4, 5]);
        // rx: 0('a') -> cx 0
        // rx: 1,2,3('   ') -> cx 1 (all part of tab)
        // rx: 4('b') -> cx 2
        // rx: 5(end) -> cx 3 (end of chars)
        assert_eq!(row.rx2cx, vec![0, 1, 1, 1, 2, 3]);
    }

    #[test]
    fn test_row_update_tab_at_start() {
        let mut row = Row::new("\ta".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 8); // tab_stop = 8
        assert_eq!(row.render, "        a"); // tab is 8 spaces, 'a' (rx 8)
        assert_eq!(row.chars, "\ta".as_bytes());
        // cx: 0('\t') -> rx 0
        // cx: 1('a') -> rx 8
        // cx: 2(end) -> rx 9
        assert_eq!(row.cx2rx, vec![0, 8, 9]);
        // rx: 0-7('        ') -> cx 0
        // rx: 8('a') -> cx 1
        // rx: 9(end) -> cx 2
        assert_eq!(row.rx2cx, vec![0,0,0,0,0,0,0,0, 1, 2]);
    }

    #[test]
    fn test_row_update_multiple_tabs() {
        let mut row = Row::new("a\tb\tc".as_bytes().to_vec()); // tab_stop = 2
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 2);
        assert_eq!(row.render, "a b c"); // a (rx0), tab(1 sp, rx1), b (rx2), tab(1 sp, rx3), c (rx4)
        // cx: 0('a') -> rx 0
        // cx: 1('\t') -> rx 1
        // cx: 2('b') -> rx 2
        // cx: 3('\t') -> rx 3
        // cx: 4('c') -> rx 4
        // cx: 5(end) -> rx 5
        assert_eq!(row.cx2rx, vec![0, 1, 2, 3, 4, 5]);
        // rx: 0('a') -> cx 0
        // rx: 1(' ') -> cx 1 (tab1)
        // rx: 2('b') -> cx 2
        // rx: 3(' ') -> cx 3 (tab2)
        // rx: 4('c') -> cx 4
        // rx: 5(end) -> cx 5
        assert_eq!(row.rx2cx, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_row_update_tabs_and_spaces() {
        // Input: " \t " (space, tab, space), tab_stop = 4
        let mut row = Row::new(" \t ".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        // ' ' (rx 0)
        // '\t' at rx 1: n_rend = 4 - (1%4) = 3 spaces. (rx 1,2,3)
        // ' ' at rx 4: (rx 4)
        // Render: "    " (1 space, 3 for tab, 1 space) = "     " (5 spaces)
        assert_eq!(row.render, "     ");
        // cx: 0(' ') -> rx 0
        // cx: 1('\t') -> rx 1
        // cx: 2(' ') -> rx 4 (after space and 3-space tab)
        // cx: 3(end) -> rx 5
        assert_eq!(row.cx2rx, vec![0, 1, 4, 5]);
        // rx: 0(' ') -> cx 0
        // rx: 1,2,3('   ') -> cx 1 (tab)
        // rx: 4(' ') -> cx 2
        // rx: 5(end) -> cx 3
        assert_eq!(row.rx2cx, vec![0, 1, 1, 1, 2, 3]);
    }

    #[test]
    fn test_row_update_unicode_simple() {
        let mut row = Row::new("こんにちは".as_bytes().to_vec()); // 5 chars, each 3 bytes, each 2 cells wide
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row.render, "こんにちは"); // Should be 10 cells wide in render
        // Total bytes = 5 * 3 = 15. Total render cells = 5 * 2 = 10.
        // cx2rx should have 15+1 = 16 elements. rx2cx should have 10+1 = 11 elements.
        let mut expected_cx2rx = vec![];
        let mut current_rx = 0;
        for _ in 0..5 { // 5 characters
            for _ in 0..3 { // each 3 bytes long
                expected_cx2rx.push(current_rx);
            }
            current_rx += 2; // each 2 cells wide
        }
        expected_cx2rx.push(10); // final render position
        assert_eq!(row.cx2rx, expected_cx2rx);

        let mut expected_rx2cx = vec![];
        let mut current_cx = 0;
        for _ in 0..5 { // 5 characters
            for _ in 0..2 { // each 2 cells wide
                expected_rx2cx.push(current_cx);
            }
            current_cx += 3; // each 3 bytes long
        }
        expected_rx2cx.push(15); // final char position
        assert_eq!(row.rx2cx, expected_rx2cx);
    }

    // --- Tests for Row::update_syntax ---

    #[test]
    fn test_update_syntax_no_rules() {
        let mut row = Row::new("hello world".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        // update calls update_syntax internally. We inspect .hl and .hl_state after.
        row.update(&syntax_conf, HlState::Normal, 4); 
        assert_eq!(row.hl, vec![HlType::Normal; row.render.len()]);
        assert_eq!(row.hl_state, HlState::Normal);
    }

    #[test]
    fn test_update_syntax_sl_comment() {
        let mut row = Row::new("code // comment".as_bytes().to_vec());
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.sl_comment_start = vec!["//".to_string()];
        row.update(&syntax_conf, HlState::Normal, 4);
        
        let mut expected_hl = vec![HlType::Normal; "code ".len()];
        expected_hl.extend(vec![HlType::Comment; "// comment".len()]);
        assert_eq!(row.hl, expected_hl);
        assert_eq!(row.hl_state, HlState::Normal);

        // Comment at start of line
        let mut row_comment_start = Row::new("// comment".as_bytes().to_vec());
        row_comment_start.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_comment_start.hl, vec![HlType::Comment; "// comment".len()]);
    }

    #[test]
    fn test_update_syntax_ml_comment() {
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.ml_comment_delims = Some(("/*".to_string(), "*/".to_string()));

        // Terminated ML comment
        let mut row1 = Row::new("code /* comment */ code".as_bytes().to_vec());
        row1.update(&syntax_conf, HlState::Normal, 4);
        let mut expected1 = vec![HlType::Normal; "code ".len()];
        expected1.extend(vec![HlType::MlComment; "/* comment */".len()]);
        expected1.extend(vec![HlType::Normal; " code".len()]);
        assert_eq!(row1.hl, expected1);
        assert_eq!(row1.hl_state, HlState::Normal);

        // Unterminated ML comment
        let mut row2 = Row::new("code /* comment".as_bytes().to_vec());
        let final_state2 = row2.update(&syntax_conf, HlState::Normal, 4);
        let mut expected2 = vec![HlType::Normal; "code ".len()];
        expected2.extend(vec![HlType::MlComment; "/* comment".len()]);
        assert_eq!(row2.hl, expected2);
        assert_eq!(final_state2, HlState::MultiLineComment); // Check returned state
        assert_eq!(row2.hl_state, HlState::MultiLineComment); // Check stored state

        // Continued ML comment
        let mut row3 = Row::new(" still comment */ code".as_bytes().to_vec());
        let final_state3 = row3.update(&syntax_conf, HlState::MultiLineComment, 4); // Pass previous state
        let mut expected3 = vec![HlType::MlComment; " still comment */".len()];
        expected3.extend(vec![HlType::Normal; " code".len()]);
        assert_eq!(row3.hl, expected3);
        assert_eq!(final_state3, HlState::Normal);
        assert_eq!(row3.hl_state, HlState::Normal);
        
        // ML comment not closed (ends line)
        let mut row4 = Row::new("/* open ".as_bytes().to_vec());
        row4.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row4.hl, vec![HlType::MlComment; "/* open ".len()]);
        assert_eq!(row4.hl_state, HlState::MultiLineComment);
    }
    
    #[test]
    fn test_update_syntax_ml_comment_nested_like() {
        // Current implementation is linear, no true nesting.
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.ml_comment_delims = Some(("/*".to_string(), "*/".to_string()));
        let mut row = Row::new("/* outer /* inner */ also_outer ".as_bytes().to_vec());
        row.update(&syntax_conf, HlState::Normal, 4);
        // Expected: "/* outer /* inner " is MlComment. "*/" closes the *first* "/*".
        // " also_outer " becomes Normal.
        let mut expected_hl = vec![HlType::MlComment; "/* outer /* inner */".len()];
        expected_hl.extend(vec![HlType::Normal; " also_outer ".len()]);
        assert_eq!(row.hl, expected_hl);
        assert_eq!(row.hl_state, HlState::Normal);
    }


    #[test]
    fn test_update_syntax_sl_string() {
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.sl_string_quotes = vec!['"', '\''];

        // Double quotes
        let mut row_dq = Row::new("text \"string\" end".as_bytes().to_vec());
        row_dq.update(&syntax_conf, HlState::Normal, 4);
        let mut expected_dq = vec![HlType::Normal; "text ".len()];
        expected_dq.extend(vec![HlType::String; "\"string\"".len()]);
        expected_dq.extend(vec![HlType::Normal; " end".len()]);
        assert_eq!(row_dq.hl, expected_dq);
        assert_eq!(row_dq.hl_state, HlState::Normal); // String state resets

        // Single quotes
        let mut row_sq = Row::new("text 'string' end".as_bytes().to_vec());
        row_sq.update(&syntax_conf, HlState::Normal, 4);
        let mut expected_sq = vec![HlType::Normal; "text ".len()];
        expected_sq.extend(vec![HlType::String; "'string'".len()]);
        expected_sq.extend(vec![HlType::Normal; " end".len()]);
        assert_eq!(row_sq.hl, expected_sq);

        // Unterminated string
        let mut row_unterm = Row::new("text \"open string".as_bytes().to_vec());
        row_unterm.update(&syntax_conf, HlState::Normal, 4);
        let mut expected_unterm = vec![HlType::Normal; "text ".len()];
        expected_unterm.extend(vec![HlType::String; "\"open string".len()]);
        assert_eq!(row_unterm.hl, expected_unterm);
        assert_eq!(row_unterm.hl_state, HlState::Normal); // String state resets even if unterminated

        // String with escape
        let mut row_esc = Row::new(r#""esc \" quote""#.as_bytes().to_vec());
        row_esc.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_esc.hl, vec![HlType::String; r#""esc \" quote""#.len()]);
    }

    #[test]
    fn test_update_syntax_ml_string() {
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.ml_string_delim = Some("\"\"\"".to_string());

        // Terminated ML string
        let mut row1 = Row::new("code \"\"\"string\"\"\" code".as_bytes().to_vec());
        row1.update(&syntax_conf, HlState::Normal, 4);
        let mut expected1 = vec![HlType::Normal; "code ".len()];
        expected1.extend(vec![HlType::MlString; "\"\"\"string\"\"\"".len()]);
        expected1.extend(vec![HlType::Normal; " code".len()]);
        assert_eq!(row1.hl, expected1);
        assert_eq!(row1.hl_state, HlState::Normal);

        // Unterminated ML string
        let mut row2 = Row::new("code \"\"\"string".as_bytes().to_vec());
        let final_state2 = row2.update(&syntax_conf, HlState::Normal, 4);
        let mut expected2 = vec![HlType::Normal; "code ".len()];
        expected2.extend(vec![HlType::MlString; "\"\"\"string".len()]);
        assert_eq!(row2.hl, expected2);
        assert_eq!(final_state2, HlState::MultiLineString);
        assert_eq!(row2.hl_state, HlState::MultiLineString);

        // Continued ML string
        let mut row3 = Row::new(" still string\"\"\" code".as_bytes().to_vec());
        let final_state3 = row3.update(&syntax_conf, HlState::MultiLineString, 4);
        let mut expected3 = vec![HlType::MlString; " still string\"\"\"".len()];
        expected3.extend(vec![HlType::Normal; " code".len()]);
        assert_eq!(row3.hl, expected3);
        assert_eq!(final_state3, HlState::Normal);
        assert_eq!(row3.hl_state, HlState::Normal);
    }
    
    #[test]
    fn test_update_syntax_keywords() {
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.keywords = vec![
            (HlType::Keyword1, vec!["let".to_string(), "fn".to_string()]),
            (HlType::Keyword2, vec!["true".to_string(), "Result".to_string()]),
        ];

        let mut row = Row::new("let x = true; fn get_Result()".as_bytes().to_vec());
        row.update(&syntax_conf, HlState::Normal, 4);
        
        let mut expected_hl = Vec::new();
        expected_hl.extend(vec![HlType::Keyword1; "let".len()]);
        expected_hl.extend(vec![HlType::Normal; " x = ".len()]);
        expected_hl.extend(vec![HlType::Keyword2; "true".len()]);
        expected_hl.extend(vec![HlType::Normal; "; ".len()]);
        expected_hl.extend(vec![HlType::Keyword1; "fn".len()]);
        expected_hl.extend(vec![HlType::Normal; " get_".len()]);
        // "Result" is not preceded by a separator (it's preceded by '_'), so it's Normal.
        expected_hl.extend(vec![HlType::Normal; "Result".len()]); 
        expected_hl.extend(vec![HlType::Normal; "()".len()]);
        assert_eq!(row.hl, expected_hl);

        // Test non-keyword not highlighted
        let mut row_non_kw = Row::new("letter".as_bytes().to_vec());
        row_non_kw.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_non_kw.hl, vec![HlType::Normal; "letter".len()]);
        
        // Test keyword at end of line
        let mut row_kw_end = Row::new("is fn".as_bytes().to_vec());
        row_kw_end.update(&syntax_conf, HlState::Normal, 4);
        let mut expected_kw_end = vec![HlType::Normal; "is ".len()];
        expected_kw_end.extend(vec![HlType::Keyword1; "fn".len()]);
        assert_eq!(row_kw_end.hl, expected_kw_end);
    }

    #[test]
    fn test_update_syntax_numbers() {
        let mut row = Row::new("123 val 4.56 (789)".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf(); // highlight_numbers is true by default in helper
        row.update(&syntax_conf, HlState::Normal, 4);

        let mut expected_hl = Vec::new();
        expected_hl.extend(vec![HlType::Number; "123".len()]);
        expected_hl.extend(vec![HlType::Normal; " val ".len()]);
        expected_hl.extend(vec![HlType::Number; "4".len()]); // Number
        expected_hl.extend(vec![HlType::Normal; ".".len()]); // Separator, so Normal
        expected_hl.extend(vec![HlType::Number; "56".len()]);// Number
        expected_hl.extend(vec![HlType::Normal; " (".len()]); // Separator, so Normal
        expected_hl.extend(vec![HlType::Number; "789".len()]);// Number
        expected_hl.extend(vec![HlType::Normal; ")".len()]); // Separator, so Normal
        assert_eq!(row.hl, expected_hl);
        
        // Number not preceded by separator
        let mut row2 = Row::new("val123".as_bytes().to_vec());
        row2.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row2.hl, vec![HlType::Normal; "val123".len()]);
    }
    
    #[test]
    fn test_update_syntax_empty_whitespace_lines() {
        let mut row_empty = Row::new("".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row_empty.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_empty.hl, Vec::<HlType>::new());
        assert_eq!(row_empty.hl_state, HlState::Normal);

        let mut row_whitespace = Row::new("   \t  ".as_bytes().to_vec());
        row_whitespace.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_whitespace.hl, vec![HlType::Normal; row_whitespace.render.len()]);
        assert_eq!(row_whitespace.hl_state, HlState::Normal);
    }
    
    #[test]
    fn test_update_syntax_string_state_reset_behavior() {
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.sl_string_quotes = vec!['"'];

        let mut row = Row::new("\"unterminated string".as_bytes().to_vec());
        // HlState returned by update() is the state *after* considering the current line for propagation.
        // row.hl_state is the state *stored on the row*, which for strings, is always reset.
        let final_propagated_state = row.update(&syntax_conf, HlState::Normal, 4);
        
        assert_eq!(row.hl, vec![HlType::String; row.render.len()]);
        // Per current code: `self.hl_state = if matches!(hl_state, HlState::String(_)) { HlState::Normal } else { hl_state };`
        // This means the *stored* state `row.hl_state` is reset if the *final internal* state was String.
        // And `update_syntax` returns this *reset* state for strings.
        assert_eq!(row.hl_state, HlState::Normal); 
        assert_eq!(final_propagated_state, HlState::Normal); 

        // Compare with MultiLineComment which does propagate
        syntax_conf.ml_comment_delims = Some(("/*".to_string(), "*/".to_string()));
        let mut row_ml_comment = Row::new("/* unterminated comment".as_bytes().to_vec());
        let final_propagated_ml_state = row_ml_comment.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row_ml_comment.hl_state, HlState::MultiLineComment); // Stored state
        assert_eq!(final_propagated_ml_state, HlState::MultiLineComment); // Returned state
    }

    // --- Tests for Row::get_char_size ---
    #[test]
    fn test_get_char_size_ascii() {
        let mut row = Row::new("hello".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); // Populates rx2cx

        assert_eq!(row.get_char_size(0), 1); // 'h'
        assert_eq!(row.get_char_size(1), 1); // 'e'
        assert_eq!(row.get_char_size(4), 1); // 'o' (last char)
    }

    #[test]
    fn test_get_char_size_unicode() {
        // "a世b" -> chars: a (1 byte, rx 0, width 1), 世 (3 bytes, rx 1, width 2), b (1 byte, rx 3, width 1)
        // render: "a世b" (rx length 4)
        // rx2cx: [0 (a), 1 (世), 1 (世), 4 (b), 5 (end)]
        let mut row = Row::new("a世b".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row.render, "a世b");
        assert_eq!(row.rx2cx, vec![0, 1, 1, 4, 5]);

        assert_eq!(row.get_char_size(0), 1); // 'a'
        assert_eq!(row.get_char_size(1), 3); // '世' (starts at rx 1)
        // rx=2 is the second cell of '世'. get_char_size is usually called with rx for the start of a char.
        // If called for rx=2 (middle of '世'), cx0 = rx2cx[2]=1.
        // rx2cx.iter().skip(2+1=3) is [4,5]. map gives [4-1=3, 5-1=4]. find gives 3. Correct.
        assert_eq!(row.get_char_size(2), 3); 
        assert_eq!(row.get_char_size(3), 1); // 'b' (starts at rx 3)
    }

    #[test]
    fn test_get_char_size_with_tabs() {
        // "a\tb", tab_stop = 4. render: "a   b" (rx length 5)
        // rx2cx: [0('a'), 1('\t'), 1('\t'), 1('\t'), 2('b'), 3(end)]
        let mut row = Row::new("a\tb".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        assert_eq!(row.render, "a   b");
        assert_eq!(row.rx2cx, vec![0, 1, 1, 1, 2, 3]);

        assert_eq!(row.get_char_size(0), 1); // 'a'
        assert_eq!(row.get_char_size(1), 1); // '\t' (space at rx 1 is part of tab)
        assert_eq!(row.get_char_size(2), 1); // '\t' (space at rx 2 is part of tab)
        assert_eq!(row.get_char_size(3), 1); // '\t' (space at rx 3 is part of tab)
        assert_eq!(row.get_char_size(4), 1); // 'b'
    }
    
    // --- Tests for Row::draw ---
    #[test]
    fn test_draw_simple_ascii() {
        let mut row = Row::new("hello".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); 

        let mut buffer = String::new();
        row.draw(0, 5, &mut buffer).unwrap();
        // Assuming first char is Normal, and current_hl_type is Normal, no initial Normal.to_string()
        assert_eq!(buffer, format!("hello{}", RESET_FMT));

        buffer.clear();
        row.draw(0, 3, &mut buffer).unwrap(); // max_len < render_len
        assert_eq!(buffer, format!("hel{}", RESET_FMT));

        buffer.clear();
        row.draw(2, 3, &mut buffer).unwrap(); // offset, max_len fits
        assert_eq!(buffer, format!("llo{}", RESET_FMT));
        
        buffer.clear();
        row.draw(2, 10, &mut buffer).unwrap(); // offset, max_len > available
        assert_eq!(buffer, format!("llo{}", RESET_FMT));
    }

    #[test]
    fn test_draw_empty_row() {
        let mut row = Row::new("".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4);
        let mut buffer = String::new();
        row.draw(0, 10, &mut buffer).unwrap();
        assert_eq!(buffer, RESET_FMT.to_string());
    }
    
    #[test]
    fn test_draw_with_highlighting() {
        let mut row = Row::new("keyword 123".as_bytes().to_vec());
        let mut syntax_conf = default_syntax_conf();
        syntax_conf.keywords = vec![(HlType::Keyword1, vec!["keyword".to_string()])];
        // highlight_numbers is true in default_syntax_conf helper
        row.update(&syntax_conf, HlState::Normal, 4);
        // Expected hl: [K1;7], [Normal;1], [Num;3] for "keyword 123"

        let mut buffer = String::new();
        row.draw(0, row.render.len(), &mut buffer).unwrap();
        
        // "keyword" is K1. " " is Normal. "123" is Number.
        // Draw logic: K1 "keyword" -> Normal " " -> Num "123" -> Reset
        let expected = format!(
            "{}{}{}{}{}{}{}",
            HlType::Keyword1.to_string(), "keyword",
            HlType::Normal.to_string(), " ",
            HlType::Number.to_string(), "123",
            RESET_FMT
        );
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_draw_match_segment() {
        let mut row = Row::new("find this text".as_bytes().to_vec());
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); // All Normal hl
        row.match_segment = Some(5..9); // "this" (rx indices)

        let mut buffer = String::new();
        row.draw(0, row.render.len(), &mut buffer).unwrap();
        // Expected: "find " (Normal) Match("this") ResetNormal(" text") Reset
        // No initial Normal.to_string() because "f" is Normal.
        let expected_str = format!(
            "find {}{}{}{}{}{}", 
            HlType::Match.to_string(), "this",
            RESET_FMT, // After match segment processing within the char loop
            HlType::Normal.to_string(), // To restore color for " text"
            " text",
            RESET_FMT // Final reset
        );
        assert_eq!(buffer, expected_str);
    }

    #[test]
    fn test_draw_control_characters() {
        let mut row = Row::new("a\0b\x01c\x1fd".as_bytes().to_vec()); // a<NUL>b<SOH>c<US>d
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); // All Normal hl

        let mut buffer = String::new();
        row.draw(0, row.render.len(), &mut buffer).unwrap();
        // NUL->'@', SOH->'A', US(31)->'?'
        // Expected: "a"<REV@RESET>"b"<REVARESET>"c"<REV?RESET>"d"<RESET_FINAL>
        // No initial Normal.to_string() as 'a' is Normal.
        // No color restoration after RESET_FMT if current_hl_type was Normal.
        let expected_control_chars = format!(
            "a{}{}{}{}{}{}{}{}{}{}{}{}{}", 
            REVERSE_VIDEO, "@", RESET_FMT, 
            "b",                        
            REVERSE_VIDEO, "A", RESET_FMT, 
            "c",                        
            REVERSE_VIDEO, "?", RESET_FMT, 
            "d",                        
            RESET_FMT                   
        );
        assert_eq!(buffer, expected_control_chars);
    }
    
    #[test]
    fn test_draw_unicode_chars() {
        let mut row = Row::new("こんにちは".as_bytes().to_vec()); 
        let syntax_conf = default_syntax_conf();
        row.update(&syntax_conf, HlState::Normal, 4); // All Normal hl
        
        let mut buffer = String::new();
        row.draw(0, row.render.len(), &mut buffer).unwrap();
        // No initial Normal.to_string() if "こ" is Normal.
        let expected_unicode = format!("こんにちは{}", RESET_FMT);
        assert_eq!(buffer, expected_unicode);

        // With offset and max_len
        buffer.clear();
        row.draw(1, 1, &mut buffer).unwrap(); // Draw "ん" (offset 1, take 1 char)
        // 'ん' is Normal. current_hl_type is Normal.
        let expected_offset_unicode_str = format!("ん{}", RESET_FMT);
        assert_eq!(buffer, expected_offset_unicode_str);
    }
}

// HlState needs to be Debug for assert_eq in later tests
// Add #[derive(Debug)] to HlState enum definition
// (This comment is a placeholder for where the change would be if not done via replace)
// Original HlState definition:
// pub enum HlState { ... }
// Needs to become:
// #[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
// pub enum HlState { ... }
// This change will be done in a separate step if this one succeeds,
// or incorporated if the tool allows modifying different parts of the file.
// For now, the `replace_with_git_merge_diff` will just add the tests module.
// The change to HlState is actually in the original file code, not the test block.

// Let's try to include the HlState change in this diff.
// The diff tool should handle changes to multiple parts of the file.
