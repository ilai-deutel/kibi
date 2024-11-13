//! # ANSI Escape sequences

/// Switches to the main buffer.
pub(crate) const USE_MAIN_SCREEN: &str = "\x1b[?1049l";

/// Switches to a new alternate screen buffer.
pub(crate) const USE_ALTERNATE_SCREEN: &str = "\x1b[?1049h";

/// Reset the formatting
pub(crate) const RESET_FMT: &str = "\x1b[m";

/// Invert foreground and background color
pub(crate) const REVERSE_VIDEO: &str = "\x1b[7m";

/// Move the cursor to 1:1
pub(crate) const MOVE_CURSOR_TO_START: &str = "\x1b[H";

/// DECTCTEM: Make the cursor invisible
pub(crate) const HIDE_CURSOR: &str = "\x1b[?25l";
/// DECTCTEM: Make the cursor visible
pub(crate) const SHOW_CURSOR: &str = "\x1b[?25h";

/// Clear screen from cursor down
pub(crate) const CLEAR_SCREEN_FROM_CURSOR_DOWN: &str = "\x1b[J";

/// Report the cursor position to the application.
pub(crate) const DEVICE_STATUS_REPORT: &str = "\x1b[6n";

/// Reposition the cursor to the end of the window
pub(crate) const REPOSITION_CURSOR_END: &str = "\x1b[999C\x1b[999B";
