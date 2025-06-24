//! Constants used throughout the application
//!
//! This module centralizes hardcoded values that were previously scattered
//! throughout the codebase. It provides a single source of truth for
//! commonly used strings, numbers, and formatting constants.

use colored::*;

// UI Messages
pub const MSG_PRESS_ANY_KEY: &str = "Press any key to continue...";
pub const MSG_SWITCH_FILE_WARNING: &str = "Warning: Failed to write switch file: {}";

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

// Common Messages - Not used currently but available for future refactoring
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_FOUND: &str = "• No worktrees found.";
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_TO_SEARCH: &str = "• No worktrees to search.";
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_TO_DELETE: &str = "• No worktrees to delete.";
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_AVAILABLE: &str = "• No worktrees available for deletion.";
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_SELECTED: &str = "• No worktrees selected.";
#[allow(dead_code)]
pub const MSG_NO_WORKTREES_TO_RENAME: &str = "• No worktrees to rename.";
#[allow(dead_code)]
pub const MSG_CURRENT_WORKTREE: &str = "• Already in this worktree.";
#[allow(dead_code)]
pub const MSG_CANNOT_DELETE_CURRENT: &str = "  (Cannot delete the current worktree)";
#[allow(dead_code)]
pub const MSG_CANNOT_RENAME_CURRENT: &str = "  (Cannot rename the current worktree)";

/// Creates a section header with title and separator
pub fn section_header(title: &str) -> String {
    format!(
        "{}\n{}",
        title.bright_cyan().bold(),
        "=".repeat(SEPARATOR_WIDTH).bright_blue()
    )
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

// Configuration
pub const CONFIG_FILE_NAME: &str = ".git-workers.toml";
