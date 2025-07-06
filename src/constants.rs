//! Constants used throughout the application
//!
//! This module centralizes hardcoded values that were previously scattered
//! throughout the codebase. It provides a single source of truth for
//! commonly used strings, numbers, and formatting constants.
//!
//! # Organization
//!
//! Constants are organized into the following categories:
//! - **UI Messages**: User-facing messages and prompts
//! - **UI Formatting**: Layout and display constants
//! - **Default Values**: Default strings for various states
//! - **Git Operations**: Git command and reference constants
//! - **File System**: Path and file-related constants
//! - **Error Messages**: Error and warning message templates
//! - **Icons and Labels**: UI icons and labels
//! - **Validation**: Character sets and validation rules
//!
//! # Usage
//!
//! ```rust
//! use git_workers::constants::{ICON_CREATE, ERROR_WORKTREE_CREATE};
//!
//! // Use icons in UI
//! println!("{} Create new worktree", ICON_CREATE);
//!
//! // Use error messages
//! eprintln!("{}", ERROR_WORKTREE_CREATE.replace("{}", "feature-branch"));
//! ```

#![allow(dead_code)] // Allow unused constants for future use

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
pub const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // seconds (5 minutes)

// Git constants
pub const COMMIT_ID_SHORT_LENGTH: usize = 8;
pub const LOCK_FILE_NAME: &str = "git-workers-worktree.lock";

// Directory depth limits
pub const MAX_DIRECTORY_DEPTH: usize = 50;

// Timeout and interval values
pub const PROGRESS_BAR_TICK_MILLIS: u64 = 100;
pub const DEFAULT_WORKTREE_CLEANUP_DAYS: &str = "30";

// UI display-related values
pub const UI_HEADER_LINES: usize = 7;
pub const UI_FOOTER_LINES: usize = 4;
pub const UI_MIN_ITEMS_PER_PAGE: usize = 5;
pub const UI_NAME_COL_MIN_WIDTH: usize = 8;
pub const UI_BRANCH_COL_EXTRA_WIDTH: usize = 10;
pub const UI_MODIFIED_COL_WIDTH: usize = 8;
pub const UI_PATH_COL_WIDTH: usize = 40;

// File size calculations
pub const BYTES_PER_KB: u64 = 1024;
pub const BYTES_PER_MB: u64 = 1024 * 1024;

// Exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = 1;

// Git commands
pub const GIT_CMD: &str = "git";
pub const GIT_WORKTREE: &str = "worktree";
pub const GIT_BRANCH: &str = "branch";
pub const GIT_TAG: &str = "tag";

// Git subcommands
pub const GIT_ADD: &str = "add";
pub const GIT_LIST: &str = "list";
pub const GIT_REMOVE: &str = "remove";
pub const GIT_PRUNE: &str = "prune";
pub const GIT_REPAIR: &str = "repair";
pub const GIT_MOVE: &str = "move";
pub const GIT_REV_PARSE: &str = "rev-parse";

// Git options
pub const GIT_OPT_BRANCH: &str = "-b";
pub const GIT_OPT_FORCE: &str = "--force";
pub const GIT_OPT_PORCELAIN: &str = "--porcelain";
pub const GIT_OPT_RENAME: &str = "-m";
pub const GIT_OPT_LIST: &str = "--list";
pub const GIT_OPT_GIT_COMMON_DIR: &str = "--git-common-dir";
pub const GIT_OPT_GIT_DIR: &str = "--git-dir";
pub const GIT_OPT_DETACH: &str = "--detach";

// Git reference paths
pub const GIT_REFS_TAGS: &str = "refs/tags/";
pub const GIT_REFS_REMOTES: &str = "refs/remotes/";
pub const GIT_GITDIR_PREFIX: &str = "gitdir: ";
pub const GIT_GITDIR_SUFFIX: &str = "/.git\n";
pub const GIT_ORIGIN: &str = "origin/";

// Git reserved paths
pub const GIT_DIR: &str = ".git";
pub const GIT_WORKTREE_DIR: &str = "worktrees";
pub const GIT_COMMONDIR_FILE: &str = "commondir";

// Shell commands
pub const SHELL_CMD: &str = "sh";
pub const SHELL_OPT_COMMAND: &str = "-c";

// Error messages
pub const ERROR_LOCK_EXISTS: &str =
    "Another git-workers process is currently creating a worktree. Please wait and try again.";
pub const ERROR_LOCK_CREATE: &str = "Failed to create lock file: {}";
pub const ERROR_NO_WORKING_DIR: &str = "No working directory";
pub const ERROR_NO_PARENT_BARE_REPO: &str = "Cannot find parent directory of bare repository";
pub const ERROR_NO_REPO_WORKING_DIR: &str = "Cannot find repository working directory";
pub const ERROR_NO_PARENT_DIR: &str = "Cannot find parent directory";
pub const ERROR_NO_REPO_DIR: &str = "Cannot determine repository directory";
pub const ERROR_WORKTREE_PATH_EXISTS: &str = "Worktree path already exists: {}";
pub const ERROR_WORKTREE_CREATE: &str = "Failed to create worktree: {}";
pub const ERROR_CONFIG_LOAD: &str = "Failed to load config";
pub const ERROR_CONFIG_READ: &str = "Failed to read {}: {}";
pub const ERROR_CONFIG_PARSE: &str = "Failed to parse {}: {}";
pub const ERROR_REPO_URL_MISMATCH: &str = "Repository URL mismatch!";
pub const ERROR_EXPECTED_URL_PREFIX: &str = "Expected: ";
pub const ERROR_ACTUAL_URL_PREFIX: &str = "Actual: ";
pub const ERROR_HOOKS_NOT_EXECUTED: &str = "Hooks will not be executed.";
pub const ERROR_HOOK_EXIT_CODE: &str = "Hook command failed with exit code: {:?}";
pub const ERROR_HOOK_WAIT_PREFIX: &str = "Failed to wait for hook command: ";
pub const ERROR_HOOK_EXECUTE_PREFIX: &str = "Failed to execute hook command: ";
pub const ERROR_TERMINAL_REQUIRED: &str = "Error: git-workers requires a terminal environment.";
pub const ERROR_NON_INTERACTIVE: &str = "Non-interactive environments are not supported.";
pub const ERROR_PERMISSION_DENIED: &str = "Failed to create worktree: permission denied";
pub const ERROR_COMMAND_NOT_FOUND: &str = "Hook execution failed: command not found";

// Prompt messages
pub const PROMPT_ACTION: &str = "What would you like to do?";
pub const PROMPT_WORKTREE_NAME: &str = "Enter worktree name";
pub const PROMPT_SELECT_WORKTREE: &str = "Select a worktree to switch to";
pub const PROMPT_SELECT_BRANCH: &str = "Select a branch";
pub const PROMPT_SELECT_TAG: &str = "Select a tag";
pub const PROMPT_DAYS_TO_KEEP: &str = "Days to keep";
pub const PROMPT_SWITCH_WORKTREE: &str = "Switch to the new worktree?";
pub const PROMPT_CUSTOM_PATH: &str = "Custom path (relative to repository root)";
pub const PROMPT_NEW_BRANCH_NAME: &str = "New branch name";
pub const PROMPT_BASE_BRANCH: &str = "Base branch for new branch";
pub const PROMPT_FIRST_WORKTREE: &str = "First worktree - choose location:";
pub const PROMPT_WORKTREE_LOCATION: &str = "Select worktree location pattern";
pub const PROMPT_BRANCH_OPTION: &str = "Select branch option";
pub const PROMPT_DELETE_BRANCH: &str = "Delete branch '{}' as well?";
pub const PROMPT_DELETE_WORKTREE: &str = "Are you sure you want to delete worktree '{}'?";
pub const PROMPT_RENAME_BRANCH: &str = "Rename branch '{}' to '{}' as well?";
pub const PROMPT_NEW_WORKTREE_NAME: &str = "New worktree name";
pub const PROMPT_CLEANUP_CONFIRM: &str = "Delete {} worktrees?";

// Success messages
pub const SUCCESS_WORKTREE_CREATED: &str = "Worktree created successfully!";
pub const SUCCESS_OPERATION_COMPLETED: &str = "Operation completed!";

// Warning messages
pub const WARNING_NO_WORKTREES: &str = "• No worktrees found.";
pub const WARNING_NO_WORKTREES_SEARCH: &str = "• No worktrees to search.";
pub const WARNING_ALREADY_IN_WORKTREE: &str = "• Already in this worktree.";
pub const WARNING_WORKTREE_NAME_EMPTY: &str = "Worktree name cannot be empty";
pub const WARNING_INVALID_WORKTREE_NAME: &str = "Invalid worktree name: {}";
pub const WARNING_CUSTOM_PATH_EMPTY: &str = "Custom path cannot be empty";
pub const WARNING_INVALID_CUSTOM_PATH: &str = "Invalid custom path: {}";
pub const WARNING_NO_BRANCHES: &str = "No branches found, creating from HEAD";
pub const WARNING_NO_TAGS: &str = "No tags found, creating from HEAD";
pub const WARNING_BRANCH_NAME_EMPTY: &str = "Branch name cannot be empty";

// Info messages
pub const INFO_EXITING: &str = "Exiting Git Workers...";
pub const INFO_TIP: &str = "Tip:";
pub const INFO_USE_CREATE: &str = "Use '{}' to create your first worktree";
pub const INFO_CREATING_WORKTREE: &str = "Creating worktree: {}";
pub const INFO_OPERATION_CANCELLED: &str = "Operation cancelled";
pub const INFO_WILL_KEEP_DAYS: &str = "Will keep worktrees for {} days";
pub const INFO_CREATING_WORKTREE_PROGRESS: &str = "Creating worktree...";
pub const INFO_RUNNING_HOOKS: &str = "Running {} hooks...";
pub const INFO_HOOK_COMMAND_PREFIX: &str = "  > ";

// UI Icons
pub const ICON_LIST: &str = "•";
pub const ICON_SEARCH: &str = "?";
pub const ICON_CREATE: &str = "+";
pub const ICON_DELETE: &str = "-";
pub const ICON_BATCH_DELETE: &str = "=";
pub const ICON_CLEANUP: &str = "~";
pub const ICON_SWITCH: &str = "→";
pub const ICON_RENAME: &str = "*";
pub const ICON_EDIT: &str = "⚙";
pub const ICON_EXIT: &str = "x";
pub const ICON_SPINNER: &str = "⏳";
pub const ICON_SUCCESS: &str = "✓";
pub const ICON_ERROR: &str = "✗";
pub const ICON_WARNING: &str = "⚠";
pub const ICON_ARROW: &str = "▸";
pub const ICON_COMPUTER: &str = "💻";
pub const ICON_CLOUD: &str = "⛅️";
pub const ICON_TAG: &str = "🏷️";
pub const ICON_FILE: &str = "📄";
pub const ICON_INFO: &str = "ℹ️";
pub const ICON_QUESTION: &str = "?";

// UI Labels
pub const LABEL_PATH: &str = "Path:";
pub const LABEL_BRANCH: &str = "Branch:";
pub const LABEL_NAME: &str = "Name:";
pub const LABEL_MODIFIED: &str = "Modified";
pub const LABEL_YES: &str = "Yes";
pub const LABEL_NO: &str = "No";
pub const LABEL_NEW_BRANCH: &str = "New Branch:";
pub const LABEL_FROM: &str = "From:";
pub const LABEL_PREVIEW: &str = "Preview:";
pub const LABEL_REPOSITORY: &str = "Repository:";
pub const LABEL_STATUS: &str = "Status:";
pub const LABEL_AUTHOR: &str = "Author:";
pub const LABEL_DATE: &str = "Date:";
pub const LABEL_MESSAGE: &str = "Message:";
pub const LABEL_CURRENT: &str = "Current:";
pub const LABEL_DEFAULT: &str = "Default:";

// Menu items
pub const MENU_LIST_WORKTREES: &str = "•  List worktrees";
pub const MENU_SEARCH_WORKTREES: &str = "?  Search worktrees";
pub const MENU_CREATE_WORKTREE: &str = "+  Create worktree";
pub const MENU_DELETE_WORKTREE: &str = "-  Delete worktree";
pub const MENU_BATCH_DELETE: &str = "=  Batch delete worktrees";
pub const MENU_CLEANUP_OLD: &str = "~  Cleanup old worktrees";
pub const MENU_SWITCH_WORKTREE: &str = "→  Switch worktree";
pub const MENU_RENAME_WORKTREE: &str = "*  Rename worktree";
pub const MENU_EDIT_HOOKS: &str = "⚙  Edit hooks";
pub const MENU_EXIT: &str = "x  Exit";

// Branch creation options
pub const OPTION_CREATE_FROM_HEAD: &str = "Create from current HEAD";
pub const OPTION_SELECT_BRANCH: &str = "Select existing branch";
pub const OPTION_SELECT_TAG: &str = "Select tag";

// Worktree location options
pub const OPTION_SAME_LEVEL: &str = "Same level as repository";
pub const OPTION_SUBDIRECTORY: &str = "In repository subdirectory (recommended)";
pub const OPTION_CUSTOM_PATH: &str = "Custom path (specify relative to project root)";

// Environment variables
pub const ENV_NO_COLOR: &str = "NO_COLOR";
pub const ENV_FORCE_COLOR: &str = "FORCE_COLOR";
pub const ENV_CLICOLOR_FORCE: &str = "CLICOLOR_FORCE";
pub const ENV_GW_SWITCH_FILE: &str = "GW_SWITCH_FILE";
pub const ENV_CI: &str = "CI";
pub const ENV_CLICOLOR_FORCE_VALUE: &str = "1";

// Control characters
pub const CTRL_U: char = '\x15';
pub const CTRL_W: char = '\x17';
pub const ANSI_CLEAR_LINE: &str = "\r\x1b[K";

// Template variables
pub const TEMPLATE_WORKTREE_NAME: &str = "{{worktree_name}}";
pub const TEMPLATE_WORKTREE_PATH: &str = "{{worktree_path}}";

// Format strings
pub const FORMAT_DEFAULT_VALUE: &str = "[{}]";
pub const FORMAT_SEPARATOR: &str = " ";

// Special values
pub const UNKNOWN_VALUE: &str = "unknown";
pub const MAIN_SUFFIX: &str = " (main)";
pub const BARE_SUFFIX: &str = ".bare";

// Path separators and patterns
pub const PATH_SEPARATOR: char = '/';
pub const PATH_SEPARATOR_WINDOWS: char = '\\';
pub const PATH_PARENT: &str = "..";
pub const PATH_CURRENT: &str = ".";
pub const PATH_COLON: char = ':';

// Git porcelain prefixes
pub const PORCELAIN_WORKTREE: &str = "worktree ";

// File operations
pub const FILE_READ_MODE: &str = "r";
pub const FILE_WRITE_MODE: &str = "w";

// UI thresholds
pub const FUZZY_SEARCH_THRESHOLD: usize = 5;
pub const TAG_MESSAGE_TRUNCATE_LENGTH: usize = 50;

// File path validation
pub const WINDOWS_PATH_MIN_LENGTH: usize = 3;
pub const COLON_POSITION_WINDOWS: usize = 1;

// UI icons for branches and tags
pub const ICON_LOCAL_BRANCH: &str = "💻 ";
pub const ICON_REMOTE_BRANCH: &str = "⛅️ ";
pub const ICON_TAG_INDICATOR: &str = "🏷️  ";

// Display precision
pub const FILE_SIZE_DISPLAY_PRECISION: usize = 1;

// Action selection indices
pub const ACTION_USE_WORKTREE_NAME: usize = 0;
pub const ACTION_CHANGE_BRANCH_NAME: usize = 1;
pub const ACTION_CANCEL: usize = 2;

// Remote branch action indices
pub const ACTION_CREATE_NEW_BRANCH: usize = 0;
pub const ACTION_USE_LOCAL_BRANCH: usize = 1;
pub const ACTION_CANCEL_OPERATION: usize = 2;

// Time related
pub const TIMESTAMP_NANOSECONDS_DEFAULT: u32 = 0;

// Array operations
pub const WINDOW_SIZE_PAIRS: usize = 2;

// Worktree location pattern indices
pub const WORKTREE_LOCATION_SAME_LEVEL: usize = 0;
pub const WORKTREE_LOCATION_SUBDIRECTORY: usize = 1;
pub const WORKTREE_LOCATION_CUSTOM_PATH: usize = 2;

// Branch option indices
pub const BRANCH_OPTION_CREATE_FROM_HEAD: usize = 0;
pub const BRANCH_OPTION_SELECT_BRANCH: usize = 1;
pub const BRANCH_OPTION_SELECT_TAG: usize = 2;

// Hook types
pub const HOOK_POST_CREATE: &str = "post-create";
pub const HOOK_PRE_REMOVE: &str = "pre-remove";
pub const HOOK_POST_SWITCH: &str = "post-switch";

// Array indices
pub const WINDOW_FIRST_INDEX: usize = 0;
pub const WINDOW_SECOND_INDEX: usize = 1;
pub const GIT_HEAD_INDEX: usize = 0;

// Character literals
pub const CHAR_SPACE: char = ' ';
pub const CHAR_DOT: char = '.';

// Default values
pub const DEFAULT_EMPTY_STRING: &str = "";
pub const DEFAULT_REPO_NAME: &str = "repo";

// Git URL suffix
pub const GIT_URL_SUFFIX: &str = ".git";

// Path component indices and minimums
pub const PATH_COMPONENT_SECOND_INDEX: usize = 1;
pub const MIN_PATH_COMPONENTS_FOR_SUBDIR: usize = 1;
