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

// Git commit info defaults (used in git.rs)
pub const GIT_COMMIT_AUTHOR_UNKNOWN: &str = "Unknown";
pub const GIT_COMMIT_MESSAGE_NONE: &str = "No message";

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
pub const INVALID_FILESYSTEM_CHARS: &[char] = &[
    '/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0', ';', '\'', '`',
];
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
pub const PROMPT_CUSTOM_PATH: &str = "Directory path";
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
pub const PROMPT_CONFLICT_ACTION: &str = "What would you like to do?";

// Success messages
pub const SUCCESS_WORKTREE_CREATED: &str = "Worktree created successfully!";
pub const SUCCESS_OPERATION_COMPLETED: &str = "Operation completed!";

// Additional UI messages
pub const MSG_CREATING_FIRST_WORKTREE: &str = "Creating first worktree...";
pub const MSG_PRESS_ESC_TO_CANCEL: &str = " (ESC to cancel)";

// Git error messages
pub const GIT_BRANCH_NOT_FOUND_MSG: &str = "Branch '{}' not found";
pub const GIT_CANNOT_RENAME_CURRENT: &str =
    "Cannot rename current worktree. Please switch to another worktree first.";
pub const GIT_WORKTREE_NOT_FOUND: &str = "Worktree not found: {}";
pub const GIT_INVALID_BRANCH_NAME: &str = "Invalid branch name: {}";

// Emoji icons
pub const EMOJI_HOME: &str = "🏠";
pub const EMOJI_LOCKED: &str = "🔒";
pub const EMOJI_BRANCH: &str = "🌿";
pub const EMOJI_DETACHED: &str = "🔗";
pub const EMOJI_FOLDER: &str = "📁";

// File operations
pub const FILE_COPY_COPYING_FILES: &str = "Copying configured files...";
pub const FILE_COPY_NO_FILES: &str = "No files were copied";
pub const FILE_COPY_SKIPPED_LARGE: &str = "Skipping large file";
pub const FILE_COPY_FAILED: &str = "Failed to copy";
pub const FILE_COPY_SKIPPING_UNSAFE: &str = "Skipping unsafe path";
pub const FILE_COPY_NOT_FOUND: &str = "Not found";
pub const FILE_COPY_COPIED_SUCCESS: &str = "Copied";
pub const SIZE_UNIT_MB: &str = "MB";

// Error detection patterns
pub const ERROR_NO_SUCH_FILE: &str = "No such file or directory";
pub const ERROR_NOT_FOUND: &str = "not found";

// Main worktree detection
pub const MAIN_WORKTREE_NAMES: &[&str] = &["main", "master"];

// File size formatting
pub const FILE_SIZE_MB_SUFFIX: &str = " MB";
pub const FILE_SIZE_DECIMAL_PLACES: usize = 1;

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

// Default selection indices
pub const DEFAULT_MENU_SELECTION: usize = 0;

// Section headers
pub const HEADER_WORKTREES: &str = "Worktrees";
pub const HEADER_SEARCH_WORKTREES: &str = "Search Worktrees";
pub const HEADER_CREATE_WORKTREE: &str = "Create New Worktree";

// Input prompts (additional)
pub const PROMPT_SELECT_WORKTREE_SWITCH: &str = "Select a worktree to switch to";
pub const PROMPT_SELECT_WORKTREE_LOCATION: &str = "Select worktree location pattern";
pub const PROMPT_SELECT_BRANCH_OPTION: &str = "Select branch option";

// Environment variables
pub const ENV_EDITOR: &str = "EDITOR";
pub const ENV_VISUAL: &str = "VISUAL";

// Default editors
pub const DEFAULT_EDITOR_WINDOWS: &str = "notepad";
pub const DEFAULT_EDITOR_UNIX: &str = "vi";

// Error messages (additional)
pub const ERROR_WORKTREE_NAME_EMPTY: &str = "Worktree name cannot be empty";
pub const ERROR_CUSTOM_PATH_EMPTY: &str = "Custom path cannot be empty";

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

// Additional constants that were identified during hardcode audit
pub const MSG_NO_WORKTREES_TO_SEARCH: &str = "• No worktrees to search.";
pub const MSG_SEARCH_FUZZY_ENABLED: &str = "Type to search worktrees (fuzzy search enabled):";
pub const MSG_ALREADY_IN_WORKTREE: &str = "• Already in this worktree.";
pub const SEARCH_CURRENT_INDICATOR: &str = " (current)";

// File copy operation constants
pub const FILE_COPY_SAME_DIRECTORY: &str = "Source and destination are the same directory";
pub const FILE_COPY_SKIPPING_LARGE: &str = "Skipping large file";

// Pluralization helpers
pub const PLURAL_EMPTY: &str = "";
pub const PLURAL_S: &str = "s";

// Git operation error messages
pub const GIT_CANNOT_FIND_PARENT: &str = "Cannot find parent directory";
pub const GIT_CANNOT_RENAME_DETACHED: &str = "Cannot rename worktree with detached HEAD";
pub const GIT_NEW_NAME_NO_SPACES: &str = "New name cannot contain spaces";

// Additional hardcoded values found in fourth audit
// Error context messages
pub const ERROR_FAILED_TO_CREATE_PARENT_DIR: &str = "Failed to create parent directory: ";
pub const ERROR_FAILED_TO_COPY_FILE: &str = "Failed to copy file from {} to {}";
pub const ERROR_FAILED_TO_CREATE_DIR: &str = "Failed to create directory: ";
pub const ERROR_SOURCE_PATH_NOT_FOUND: &str = "Source path not found: ";
pub const ERROR_SOURCE_NOT_FILE_OR_DIR: &str = "Source is neither a file nor a directory: ";
pub const ERROR_MAX_DEPTH_EXCEEDED: &str =
    "Maximum directory depth ({}) exceeded. Possible circular reference.";
pub const ERROR_GIT_DIR_NO_PARENT: &str = "Git directory has no parent";
pub const ERROR_NO_MAIN_WORKTREE_FOUND: &str = "No main worktree found with {} file";
pub const ERROR_REPO_NO_WORKING_DIR: &str = "Repository has no working directory";

// Info messages
pub const INFO_SKIPPING_SYMLINK: &str = "Skipping symlink: ";
pub const INFO_SKIPPING_CIRCULAR_REF: &str = "Skipping circular reference: ";
pub const INFO_FAILED_TO_COPY: &str = "Failed to copy";

// Git references
pub const GIT_REFS_HEADS: &str = "refs/heads/";

// Git command arguments
pub const GIT_ARG_TRACK: &str = "--track";
pub const GIT_ARG_NO_TRACK: &str = "--no-track";
pub const GIT_ARG_NO_GUESS_REMOTE: &str = "--no-guess-remote";

// File permissions
pub const FILE_PERMISSION_READONLY: u32 = 0o444;
pub const FILE_PERMISSION_READWRITE: u32 = 0o644;

// Directory names
pub const MOCK_GIT_DIR: &str = ".mockgit";

// Test-specific constants
pub const TEST_AUTHOR_NAME: &str = "Test Author";
pub const TEST_AUTHOR_EMAIL: &str = "test@example.com";
pub const TEST_COMMIT_MESSAGE: &str = "Test commit";
pub const TEST_README_FILE: &str = "README.md";
pub const TEST_README_CONTENT: &str = "# Test";
pub const TEST_CONFIG_CONTENT: &str = "[files]\ncopy = [\".env\"]";
pub const TEST_ENV_CONTENT: &str = "TEST=1";
pub const TEST_GITIGNORE_CONTENT: &str = ".env";

// Hook execution messages
pub const HOOK_EXECUTING: &str = "Executing hook: ";
pub const HOOK_OUTPUT: &str = "Hook output: ";

// Symlink warning icon
pub const ICON_SYMLINK_WARNING: &str = "⚠️";

// Git file names
pub const GIT_FILE_GITDIR: &str = "gitdir";
pub const GIT_FILE_COMMONDIR: &str = "commondir";

// Special file names
pub const FILE_DOT_ENV: &str = ".env";
pub const FILE_DOT_ENV_EXAMPLE: &str = ".env.example";

// Array slice ranges
pub const SLICE_START: usize = 0;
pub const SLICE_END_ONE: usize = 1;

// Time units
pub const SECONDS_IN_DAY: u64 = 86400;

// Default branch names array (for iteration)
pub const DEFAULT_BRANCHES: &[&str] = &["main", "master"];

// Worktree list header format
pub const WORKTREE_LIST_HEADER: &str = "worktree ";

// Git command error patterns
pub const GIT_ERROR_INVALID_REF: &str = "is not a valid ref";
pub const GIT_ERROR_NOT_VALID_REF: &str = "not a valid ref";

// Progress messages
pub const PROGRESS_CREATING_WORKTREE: &str = "Creating worktree";
pub const PROGRESS_COPYING_FILES: &str = "Copying files";
pub const PROGRESS_RUNNING_HOOKS: &str = "Running hooks";

// Validation error prefixes
pub const VALIDATION_ERROR_PREFIX: &str = "Validation failed: ";

// File operation prefixes
pub const FILE_OP_COPYING: &str = "Copying: ";
pub const FILE_OP_SKIPPING: &str = "Skipping: ";
pub const FILE_OP_CREATED: &str = "Created: ";

// Git status prefixes
pub const GIT_STATUS_BRANCH: &str = "branch ";
pub const GIT_STATUS_HEAD: &str = "HEAD ";

// Menu display formats
pub const MENU_FORMAT_WITH_ICON: &str = "{}  {}";
pub const MENU_FORMAT_SIMPLE: &str = "{}";

// Error detail separators
pub const ERROR_DETAIL_SEPARATOR: &str = ": ";
pub const ERROR_CONTEXT_SEPARATOR: &str = " - ";

// Path join separators
pub const PATH_JOIN_SLASH: &str = "/";

// Git output parsing
pub const GIT_OUTPUT_SPLIT_CHAR: char = '\n';
pub const GIT_OUTPUT_TAB_CHAR: char = '\t';

// Number formatting
pub const DECIMAL_FORMAT_ONE: &str = ".1";
pub const DECIMAL_FORMAT_TWO: &str = ".2";

// Boolean string values
pub const BOOL_TRUE_STR: &str = "true";
pub const BOOL_FALSE_STR: &str = "false";

// Exit messages
pub const EXIT_MSG_GOODBYE: &str = "Goodbye!";
pub const EXIT_MSG_CANCELLED: &str = "Cancelled.";

// List UI display constants
pub const ICON_CURRENT_WORKTREE: &str = "→";
pub const ICON_OTHER_WORKTREE: &str = "▸";
pub const MODIFIED_STATUS_YES: &str = "Yes";
pub const MODIFIED_STATUS_NO: &str = "No";
pub const TABLE_HEADER_NAME: &str = "Name";
pub const TABLE_HEADER_BRANCH: &str = "Branch";
pub const TABLE_HEADER_MODIFIED: &str = "Modified";
pub const TABLE_HEADER_PATH: &str = "Path";
pub const TABLE_SEPARATOR: &str = "-";
pub const CURRENT_MARKER: &str = "[current]";

// Prompt suffixes
pub const PROMPT_SUFFIX_COLON: &str = ": ";
pub const PROMPT_SUFFIX_QUESTION: &str = "? ";

// Display list bullet
pub const DISPLAY_BULLET: &str = "• ";

// Git worktree states
pub const WORKTREE_STATE_BARE: &str = "bare";
pub const WORKTREE_STATE_DETACHED: &str = "detached";
pub const WORKTREE_STATE_BRANCH: &str = "branch";

// Error recovery suggestions
pub const SUGGEST_CHECK_PATH: &str = "Please check the path and try again.";
pub const SUGGEST_CHECK_PERMISSIONS: &str = "Please check file permissions.";

// Confirmation prompts
pub const CONFIRM_CONTINUE: &str = "Continue?";
pub const CONFIRM_PROCEED: &str = "Proceed?";

// Status indicators
pub const STATUS_OK: &str = "[OK]";
pub const STATUS_FAILED: &str = "[FAILED]";
pub const STATUS_SKIPPED: &str = "[SKIPPED]";

// Git refspec patterns
pub const REFSPEC_HEADS: &str = "+refs/heads/*:refs/remotes/origin/*";

// Common file extensions
pub const EXT_TOML: &str = ".toml";
pub const EXT_JSON: &str = ".json";
pub const EXT_YAML: &str = ".yaml";
pub const EXT_YML: &str = ".yml";

// Test constants
pub const TEST_TITLE: &str = "Test Title";
pub const TEST_EQUALS_SIGN: &str = "=";
pub const TEST_PERCENT_SIGN: &str = "%";
pub const TEST_GIT_HEAD: &str = "HEAD";
pub const TEST_GIT_REFS: &str = "refs";

// Hardcoded string values from create.rs
pub const STRING_SAME_LEVEL: &str = "same-level";
pub const STRING_SUBDIRECTORY: &str = "subdirectory";
pub const STRING_CUSTOM: &str = "custom";
pub const ERROR_INVALID_WORKTREE_LOCATION: &str = "Invalid worktree location type: {}";
pub const ERROR_CUSTOM_PATH_REQUIRED: &str = "Custom path required when location is 'custom'";
pub const ERROR_INVALID_LOCATION_TYPE: &str = "Invalid location type: {}";
pub const ERROR_CANNOT_DETERMINE_PARENT_DIR: &str = "Cannot determine parent directory";

// Git command format strings
pub const FORMAT_REFS_TAGS: &str = "refs/tags/{}";
pub const FORMAT_WORKTREE_PATH_SAME_LEVEL: &str = "../{}";
pub const FORMAT_WORKTREE_PATH_SUBDIRECTORY: &str = "{}/{}";
pub const FORMAT_CUSTOM_PATH_WITH_NAME: &str = "{}/{}";
pub const FORMAT_DOT_WITH_NAME: &str = "./{}";

// UI text strings from create.rs
pub const MSG_FIRST_WORKTREE_CHOOSE: &str = "First worktree - choose location:";
pub const MSG_SPECIFY_DIRECTORY_PATH: &str = "Specify directory path (relative to project root):";
pub const MSG_EXAMPLES_WORKTREE_NAME: &str = "Examples (worktree name: '{}'):";
pub const MSG_EXAMPLE_BRANCH: &str = "branch/";
pub const MSG_EXAMPLE_HOTFIX: &str = "hotfix/";
pub const MSG_EXAMPLE_PARENT: &str = "../";
pub const MSG_EXAMPLE_DOT: &str = "./";
pub const MSG_CREATES_AT_BRANCH: &str = "→ creates at ./branch/{}";
pub const MSG_CREATES_AT_HOTFIX: &str = "→ creates at ./hotfix/{}";
pub const MSG_CREATES_AT_PARENT: &str = "→ creates at ../{} (outside project)";
pub const MSG_CREATES_AT_DOT: &str = "→ creates at ./{} (project root)";

// Branch options text
pub const OPTION_CREATE_FROM_HEAD_FULL: &str = "Create from current HEAD";
pub const OPTION_SELECT_BRANCH_FULL: &str = "Select branch";
pub const OPTION_SELECT_TAG_FULL: &str = "Select tag";

// Worktree location options text
pub const FORMAT_SAME_LEVEL_OPTION: &str = "Same level as repository (../{})";
pub const FORMAT_SUBDIRECTORY_OPTION: &str = "In subdirectory ({}/{}/{})";
pub const OPTION_CUSTOM_PATH_FULL: &str = "Custom path (specify relative to project root)";

// Branch list formatting
pub const FORMAT_BRANCH_WITH_WORKTREE: &str = "{}{}";
pub const FORMAT_BRANCH_WITHOUT_WORKTREE: &str = "{}{}";
pub const MSG_BRANCH_IN_USE_BY: &str = " (in use by '{}')";

// Conflict action messages
pub const MSG_CREATE_NEW_BRANCH_FROM: &str = "Create new branch '{}' from '{}'";
pub const MSG_CHANGE_BRANCH_NAME: &str = "Change the branch name";
pub const MSG_CANCEL: &str = "Cancel";
pub const MSG_USE_EXISTING_LOCAL: &str = "Use the existing local branch instead";
pub const MSG_USE_EXISTING_LOCAL_IN_USE: &str =
    "Use the existing local branch instead (in use by '{}')";
pub const MSG_ENTER_NEW_BRANCH_NAME: &str = "Enter new branch name (base: {})";
pub const MSG_BRANCH_ALREADY_EXISTS: &str = "Branch '{}' already exists";
pub const MSG_BRANCH_NAME_CANNOT_BE_EMPTY: &str = "Branch name cannot be empty";
pub const MSG_BRANCH_ALREADY_CHECKED_OUT: &str =
    "Branch '{}' is already checked out in worktree '{}'";
pub const MSG_LOCAL_BRANCH_EXISTS: &str = "A local branch '{}' already exists for remote '{}'";
pub const MSG_CREATE_NEW_BRANCH_FROM_REMOTE: &str = "Create new branch '{}' from '{}{}' ";
pub const MSG_PLEASE_SELECT_DIFFERENT: &str = "Please select a different option.";

// Tag formatting
pub const FORMAT_TAG_WITH_MESSAGE: &str = "{}{} - {}";
pub const FORMAT_TAG_WITHOUT_MESSAGE: &str = "{}{}";
pub const MSG_SEARCH_TAGS_FUZZY: &str = "Type to search tags (fuzzy search enabled):";
pub const MSG_SEARCH_BRANCHES_FUZZY: &str = "Type to search branches (fuzzy search enabled):";

// Preview labels and messages
pub const LABEL_PREVIEW_HEADER: &str = "Preview:";
pub const LABEL_NAME_PREVIEW: &str = "Name:";
pub const LABEL_NEW_BRANCH_PREVIEW: &str = "New Branch:";
pub const LABEL_BRANCH_PREVIEW: &str = "Branch:";
pub const LABEL_FROM_PREVIEW: &str = "From:";
pub const MSG_FROM_CURRENT_HEAD: &str = "Current HEAD";
pub const MSG_FROM_TAG_PREFIX: &str = "tag: ";

// Progress messages
pub const MSG_CREATING_WORKTREE: &str = "Creating worktree...";

// Success messages
pub const FORMAT_WORKTREE_CREATED: &str = "Created worktree '{}' at {}";
pub const MSG_COPYING_CONFIGURED_FILES: &str = "Copying configured files...";
pub const FORMAT_COPIED_FILES_COUNT: &str = "Copied {} files";
pub const MSG_COPIED_FILE_PREFIX: &str = "  ✓ ";

// Switch messages
pub const MSG_SWITCH_TO_NEW_WORKTREE: &str = "Switch to the new worktree?";
pub const MSG_SWITCHING_TO_WORKTREE: &str = "+ Switching to worktree '{}'";

// Error messages
pub const FORMAT_FAILED_CREATE_WORKTREE: &str = "Failed to create worktree: {}";
pub const FORMAT_FAILED_COPY_FILES: &str = "Failed to copy files: {}";
pub const FORMAT_HOOK_EXECUTION_WARNING: &str = "Hook execution warning: {}";
pub const FORMAT_INVALID_WORKTREE_NAME: &str = "Invalid worktree name: {}";
pub const FORMAT_INVALID_CUSTOM_PATH: &str = "Invalid custom path: {}";

// Default values
pub const REPO_NAME_FALLBACK: &str = "repo";
pub const SWITCH_CONFIRM_DEFAULT: bool = true;

// Path manipulation
pub const SLASH_CHAR: char = '/';
pub const ELLIPSIS: &str = "...";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_header() {
        let header = section_header(TEST_TITLE);
        assert!(header.contains(TEST_TITLE));
        assert!(header.contains(TEST_EQUALS_SIGN));
    }

    #[test]
    fn test_header_separator() {
        let separator = header_separator();
        // The separator should contain 50 equals signs
        assert!(separator.contains(&TEST_EQUALS_SIGN.repeat(HEADER_SEPARATOR_WIDTH)));
        // The actual length may vary due to ANSI color codes
        assert!(separator.contains(TEST_EQUALS_SIGN));
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_constants_non_empty() {
        // Test that important constants are not empty
        assert!(!MSG_PRESS_ANY_KEY.is_empty());
        assert!(!CONFIG_FILE_NAME.is_empty());
        assert!(!GIT_REMOTE_PREFIX.is_empty());
        assert!(!DEFAULT_BRANCH_MAIN.is_empty());
        assert!(!DEFAULT_BRANCH_MASTER.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_emoji_constants() {
        // Test that emoji constants are defined
        assert!(!EMOJI_HOME.is_empty());
        assert!(!EMOJI_LOCKED.is_empty());
        assert!(!EMOJI_BRANCH.is_empty());
        assert!(!EMOJI_DETACHED.is_empty());
        assert!(!EMOJI_FOLDER.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_icon_constants() {
        // Test that icon constants are defined
        assert!(!ICON_SUCCESS.is_empty());
        assert!(!ICON_ERROR.is_empty());
        assert!(!ICON_WARNING.is_empty());
        assert!(!ICON_SPINNER.is_empty());
    }

    #[test]
    fn test_git_constants() {
        // Test Git-related constants
        assert_eq!(GIT_CMD, "git");
        assert_eq!(GIT_WORKTREE, "worktree");
        assert_eq!(GIT_BRANCH, "branch");
        assert_eq!(GIT_TAG, "tag");
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_numeric_constants() {
        // Test that numeric constants have reasonable values
        assert!(SEPARATOR_WIDTH > 0);
        assert!(HEADER_SEPARATOR_WIDTH > 0);
        assert!(MAX_WORKTREE_NAME_LENGTH > 0);
        assert!(MAX_FILE_SIZE_MB > 0);
        assert!(MAX_DIRECTORY_DEPTH > 0);
    }

    #[test]
    fn test_path_constants() {
        // Test path-related constants
        assert_eq!(PATH_SEPARATOR, '/');
        assert_eq!(PATH_SEPARATOR_WINDOWS, '\\');
        assert_eq!(PATH_PARENT, "..");
        assert_eq!(PATH_CURRENT, ".");
        assert_eq!(GIT_DIR, ".git");
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_time_format() {
        // Test that time format is valid
        assert!(!TIME_FORMAT.is_empty());
        assert!(TIME_FORMAT.contains(TEST_PERCENT_SIGN));
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_git_reserved_names() {
        // Test that git reserved names array is not empty
        assert!(!GIT_RESERVED_NAMES.is_empty());
        assert!(GIT_RESERVED_NAMES.contains(&TEST_GIT_HEAD));
        assert!(GIT_RESERVED_NAMES.contains(&TEST_GIT_REFS));
    }

    #[test]
    fn test_hook_constants() {
        // Test hook type constants
        assert_eq!(HOOK_POST_CREATE, "post-create");
        assert_eq!(HOOK_PRE_REMOVE, "pre-remove");
        assert_eq!(HOOK_POST_SWITCH, "post-switch");
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_header_constants() {
        // Test header text constants
        assert!(!HEADER_WORKTREES.is_empty());
        assert!(!HEADER_SEARCH_WORKTREES.is_empty());
        assert!(!HEADER_CREATE_WORKTREE.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_message_constants() {
        // Test message constants
        assert!(!MSG_PRESS_ANY_KEY.is_empty());
        assert!(!SUCCESS_WORKTREE_CREATED.is_empty());
        assert!(!INFO_OPERATION_CANCELLED.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_prompt_constants() {
        // Test prompt constants
        assert!(!PROMPT_WORKTREE_NAME.is_empty());
        assert!(!PROMPT_SELECT_WORKTREE.is_empty());
        assert!(!PROMPT_SELECT_BRANCH.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_error_constants() {
        // Test error message constants
        assert!(!ERROR_LOCK_EXISTS.is_empty());
        assert!(!ERROR_WORKTREE_CREATE.is_empty());
        assert!(!ERROR_CONFIG_LOAD.is_empty());
    }
}
