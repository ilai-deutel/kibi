//! # ANSI Escape sequences

/// Clear from cursor to beginning of the screen
pub const CLEAR_SCREEN: &str = "\x1b[2J";

/// Reset the formatting
pub(super) const RESET_FMT: &str = "\x1b[m";

/// Invert foreground and background color
pub(super) const REVERSE_VIDEO: &str = "\x1b[7m";

/// Move the cursor to 1:1
pub const MOVE_CURSOR_TO_START: &str = "\x1b[H";

/// DECTCTEM: Make the cursor invisible
pub(super) const HIDE_CURSOR: &str = "\x1b[?25l";
/// DECTCTEM: Make the cursor visible
pub(super) const SHOW_CURSOR: &str = "\x1b[?25h";

/// Clear line right of the current position of the cursor
pub(super) const CLEAR_LINE_RIGHT_OF_CURSOR: &str = "\x1b[K";

/// Report the cursor position to the application.
pub(super) const DEVICE_STATUS_REPORT: &str = "\x1b[6n";

/// Reposition the cursor to the end of the window
pub(super) const REPOSITION_CURSOR_END: &str = "\x1b[999C\x1b[999B";
