//! # ANSI Escape sequences

/// Switches to the main buffer.
pub(crate) const USE_MAIN_SCREEN: &str = "\x1b[?1049l";

/// Switches to a new alternate screen buffer.
pub(crate) const USE_ALTERNATE_SCREEN: &str = "\x1b[?1049h";

/// Reset the formatting
pub(crate) const RESET: &str = "\x1b[m";

/// White background: invert foreground and background color
pub(crate) const WBG: &str = "\x1b[7m";

/// Move the cursor to 1:1
pub(crate) const MOVE_CURSOR_TO_START: &str = "\x1b[H";

/// DECTCTEM: Make the cursor invisible
pub(crate) const HIDE_CURSOR: &str = "\x1b[?25l";
/// DECTCTEM: Make the cursor visible
pub(crate) const SHOW_CURSOR: &str = "\x1b[?25h";

/// Clear line right of the current position of the cursor
pub(crate) const CLEAR_LINE_RIGHT_OF_CURSOR: &str = "\x1b[K";

/// Report the cursor position to the application.
pub(crate) const DEVICE_STATUS_REPORT: &str = "\x1b[6n";

/// Reposition the cursor to the end of the window
pub(crate) const REPOSITION_CURSOR_END: &str = "\x1b[999C\x1b[999B";

pub(crate) fn push_colored(buffer: &mut String, color: &str, message: &str, use_color: bool) {
    for s in &[if use_color { color } else { "" }, message, if use_color { RESET } else { "" }] {
        buffer.push_str(s);
    }
}
