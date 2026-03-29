// ANSI color codes — ported from .commons/scripts/shell/colors/constants
#![allow(dead_code)]

pub const RESET: &str = "\x1b[0m";

pub const BLACK: &str = "\x1b[0;30m";
pub const RED: &str = "\x1b[0;31m";
pub const GREEN: &str = "\x1b[0;32m";
pub const YELLOW: &str = "\x1b[0;33m";
pub const BLUE: &str = "\x1b[0;34m";
pub const MAGENTA: &str = "\x1b[0;35m";
pub const CYAN: &str = "\x1b[0;36m";
pub const WHITE: &str = "\x1b[0;37m";

pub const BOLD_BLACK: &str = "\x1b[1;30m";
pub const BOLD_RED: &str = "\x1b[1;31m";
pub const BOLD_GREEN: &str = "\x1b[1;32m";
pub const BOLD_YELLOW: &str = "\x1b[1;33m";
pub const BOLD_BLUE: &str = "\x1b[1;34m";
pub const BOLD_MAGENTA: &str = "\x1b[1;35m";
pub const BOLD_CYAN: &str = "\x1b[1;36m";
pub const BOLD_WHITE: &str = "\x1b[1;37m";

pub const BRIGHT_BLACK: &str = "\x1b[0;90m";
pub const BRIGHT_GREEN: &str = "\x1b[0;92m";
pub const BRIGHT_YELLOW: &str = "\x1b[0;93m";
pub const BRIGHT_BLUE: &str = "\x1b[0;94m";
pub const BRIGHT_WHITE: &str = "\x1b[0;97m";
pub const ORANGE: &str = "\x1b[38;5;208m";

// Semantic presets
pub const INFO: &str = BOLD_BLUE;
pub const SUCCESS: &str = BOLD_GREEN;
pub const WARNING: &str = BOLD_YELLOW;
pub const ERROR: &str = BOLD_RED;
pub const DIM: &str = BRIGHT_BLACK;
