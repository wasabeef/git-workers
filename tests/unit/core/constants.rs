//! Unit tests for constants module
//!
//! Tests to ensure constants are properly defined and have expected values.

use git_workers::constants::*;

#[test]
fn test_emoji_constants() {
    assert!(!EMOJI_HOME.is_empty());
    assert!(!EMOJI_LOCKED.is_empty());
    assert!(!EMOJI_BRANCH.is_empty());
    assert!(!EMOJI_DETACHED.is_empty());
    assert!(!EMOJI_FOLDER.is_empty());
}

#[test]
fn test_icon_constants() {
    assert!(!ICON_CREATE.is_empty());
    assert!(!ICON_DELETE.is_empty());
    assert!(!ICON_LIST.is_empty());
    assert!(!ICON_RENAME.is_empty());
    assert!(!ICON_SEARCH.is_empty());
    assert!(!ICON_SWITCH.is_empty());
    assert!(!ICON_LOCAL_BRANCH.is_empty());
    assert!(!ICON_REMOTE_BRANCH.is_empty());
}

#[test]
fn test_header_constants() {
    assert!(!HEADER_CREATE_WORKTREE.is_empty());
    // Note: Only HEADER_CREATE_WORKTREE is currently defined in constants
    // assert!(!HEADER_DELETE_WORKTREE.is_empty());
    // assert!(!HEADER_SWITCH_WORKTREE.is_empty());
}

#[test]
fn test_prompt_constants() {
    assert!(!PROMPT_WORKTREE_NAME.is_empty());
    assert!(!PROMPT_SELECT_BRANCH.is_empty());
    assert!(!PROMPT_SELECT_TAG.is_empty());
    assert!(!PROMPT_SELECT_WORKTREE_LOCATION.is_empty());
    assert!(!PROMPT_CUSTOM_PATH.is_empty());
}

#[test]
fn test_message_constants() {
    assert!(!MSG_CREATING_FIRST_WORKTREE.is_empty());
    assert!(!MSG_ALREADY_IN_WORKTREE.is_empty());
    assert!(!INFO_OPERATION_CANCELLED.is_empty());
    assert!(!WARNING_NO_WORKTREES.is_empty());
}

#[test]
fn test_error_constants() {
    assert!(!ERROR_WORKTREE_NAME_EMPTY.is_empty());
    assert!(!ERROR_CUSTOM_PATH_EMPTY.is_empty());
    assert!(!ERROR_NO_REPO_WORKING_DIR.is_empty());
    assert!(!ERROR_NO_WORKING_DIR.is_empty());
}

#[test]
fn test_git_constants() {
    assert_eq!(GIT_CMD, "git");
    assert_eq!(GIT_WORKTREE, "worktree");
    assert_eq!(GIT_ADD, "add");
    assert_eq!(GIT_BRANCH, "branch");
    assert_eq!(GIT_REPAIR, "repair");
    assert_eq!(GIT_DIR, ".git");
}

#[test]
fn test_hook_constants() {
    assert_eq!(HOOK_POST_CREATE, "post-create");
    assert_eq!(HOOK_PRE_REMOVE, "pre-remove");
    assert_eq!(HOOK_POST_SWITCH, "post-switch");
}

#[test]
fn test_numeric_constants() {
    assert_eq!(DEFAULT_MENU_SELECTION, 0);
    assert_eq!(WORKTREE_LOCATION_SAME_LEVEL, 0);
    assert_eq!(WORKTREE_LOCATION_SUBDIRECTORY, 1);
    assert_eq!(WORKTREE_LOCATION_CUSTOM_PATH, 2);
    assert_eq!(COMMIT_ID_SHORT_LENGTH, 8);
    assert_eq!(TAG_MESSAGE_TRUNCATE_LENGTH, 50);
}

#[test]
fn test_path_constants() {
    assert_eq!(WORKTREES_SUBDIR, "worktrees");
    assert_eq!(DEFAULT_REPO_NAME, "repo");
    assert_eq!(LOCK_FILE_NAME, "git-workers-worktree.lock");
}

#[test]
fn test_git_reserved_names() {
    assert!(GIT_RESERVED_NAMES.contains(&"HEAD"));
    assert!(GIT_RESERVED_NAMES.contains(&"refs"));
    assert!(GIT_RESERVED_NAMES.contains(&"objects"));
    assert!(GIT_RESERVED_NAMES.contains(&"HEAD"));
    // Test a few key git reserved names
    assert!(GIT_RESERVED_NAMES.contains(&"hooks"));
    assert!(GIT_RESERVED_NAMES.contains(&"info"));
    assert!(GIT_RESERVED_NAMES.contains(&"logs"));
}

#[test]
fn test_time_format() {
    assert_eq!(TIME_FORMAT, "%Y-%m-%d %H:%M");
}

#[test]
fn test_section_header_function() {
    let header = section_header("Test");
    assert!(header.contains("Test"));
    assert!(header.len() > "Test".len()); // Should include decoration
}
