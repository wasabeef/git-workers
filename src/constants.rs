//! Constants used throughout the application
//!
//! This module centralizes hardcoded values that were previously scattered
//! throughout the codebase. It provides a single source of truth for
//! commonly used strings, numbers, and formatting constants.

use colored::*;

// UI Messages
pub const MSG_PRESS_ANY_KEY: &str = "Press any key to continue...";
pub const MSG_SWITCH_FILE_WARNING_PREFIX: &str = "Warning: Failed to write switch file: ";

// UI Formatting
pub const SEPARATOR_WIDTH: usize = 40;
pub const HEADER_SEPARATOR_WIDTH: usize = 50;

// Default Values
pub const DEFAULT_BRANCH_UNKNOWN: &str = "unknown";
pub const DEFAULT_BRANCH_DETACHED: &str = "detached";
pub const DEFAULT_AUTHOR_UNKNOWN: &str = "Unknown";
pub const DEFAULT_MESSAGE_NONE: &str = "No message";

// Time Format
pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

// Switch Marker
pub const SWITCH_TO_PREFIX: &str = "SWITCH_TO:";

/// Creates a section header with title and separator
pub fn section_header(title: &str) -> String {
    let title_formatted = title.bright_cyan().bold();
    let separator = "=".repeat(SEPARATOR_WIDTH).bright_blue();
    format!("{title_formatted}\n{separator}")
}

/// Creates a main header separator
pub fn header_separator() -> String {
    "=".repeat(HEADER_SEPARATOR_WIDTH).bright_blue().to_string()
}

// Git references
pub const GIT_REMOTE_PREFIX: &str = "origin/";
pub const GIT_DEFAULT_MAIN_WORKTREE: &str = "main";

// Directory patterns
pub const WORKTREES_SUBDIR: &str = "worktrees";
pub const BRANCH_SUBDIR: &str = "branch";

// Git branches
pub const DEFAULT_BRANCH_MAIN: &str = "main";
pub const DEFAULT_BRANCH_MASTER: &str = "master";

// Configuration
pub const CONFIG_FILE_NAME: &str = ".git-workers.toml";

// Git internals
pub const GIT_RESERVED_NAMES: &[&str] = &["HEAD", "refs", "hooks", "info", "objects", "logs"];
pub const GIT_CRITICAL_DIRS: &[&str] = &["objects", "refs", "hooks", "info", "logs", "worktrees"];

// Filesystem limits
pub const MAX_WORKTREE_NAME_LENGTH: usize = 255;
pub const MAX_FILE_SIZE_MB: u64 = 100;

// Special characters
pub const INVALID_FILESYSTEM_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
pub const WINDOWS_RESERVED_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

// Timeouts
pub const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

// Git constants
pub const COMMIT_ID_SHORT_LENGTH: usize = 8;
pub const LOCK_FILE_NAME: &str = "git-workers-worktree.lock";

// Directory depth limits
pub const MAX_DIRECTORY_DEPTH: usize = 50;
