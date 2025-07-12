use std::env;

/// Test application constants and metadata
#[test]
fn test_application_metadata() {
    // Test crate information from Cargo.toml
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let description = env!("CARGO_PKG_DESCRIPTION");

    assert_eq!(name, "git-workers");
    assert!(!version.is_empty());
    assert!(!description.is_empty());

    // Test version format
    let version_parts: Vec<&str> = version.split('.').collect();
    assert!(version_parts.len() >= 2);

    // Test description contains relevant keywords
    let desc_lower = description.to_lowercase();
    assert!(desc_lower.contains("git") || desc_lower.contains("worktree"));
}

/// Test menu item display and functionality
#[test]
fn test_menu_items() {
    use git_workers::menu::MenuItem;

    let items = vec![
        MenuItem::ListWorktrees,
        MenuItem::SearchWorktrees,
        MenuItem::CreateWorktree,
        MenuItem::DeleteWorktree,
        MenuItem::BatchDelete,
        MenuItem::CleanupOldWorktrees,
        MenuItem::SwitchWorktree,
        MenuItem::RenameWorktree,
        MenuItem::EditHooks,
        MenuItem::Exit,
    ];

    // Test that all items can be displayed
    for item in &items {
        let display = format!("{item}");
        let debug = format!("{item:?}");

        assert!(!display.is_empty());
        assert!(!debug.is_empty());
    }

    // Test specific menu item content
    assert!(format!("{}", MenuItem::ListWorktrees).contains("List"));
    assert!(format!("{}", MenuItem::CreateWorktree).contains("Create"));
    assert!(format!("{}", MenuItem::Exit).contains("Exit"));
}

/// Test environment variable handling
#[test]
fn test_environment_handling() {
    // Test CI environment detection
    let original_ci = env::var("CI").ok();

    // Test setting CI
    env::set_var("CI", "true");
    assert_eq!(env::var("CI").unwrap(), "true");

    // Test unsetting CI
    env::remove_var("CI");
    assert!(env::var("CI").is_err());

    // Test shell integration environment
    let original_switch_file = env::var("GW_SWITCH_FILE").ok();

    env::set_var("GW_SWITCH_FILE", "/tmp/test-switch");
    assert_eq!(env::var("GW_SWITCH_FILE").unwrap(), "/tmp/test-switch");

    // Restore original values
    if let Some(ci) = original_ci {
        env::set_var("CI", ci);
    }
    if let Some(switch_file) = original_switch_file {
        env::set_var("GW_SWITCH_FILE", switch_file);
    } else {
        env::remove_var("GW_SWITCH_FILE");
    }
}

/// Test constants used in main
#[test]
fn test_main_constants() {
    use git_workers::constants::*;

    // Test exit codes
    assert_eq!(EXIT_SUCCESS, 0);
    assert_eq!(EXIT_FAILURE, 1);

    // Test UI constants
    assert!(!MSG_PRESS_ANY_KEY.is_empty());
    assert!(!SWITCH_TO_PREFIX.is_empty());

    // Test icons
    assert!(!ICON_LIST.is_empty());
    assert!(!ICON_CREATE.is_empty());
    assert!(!ICON_EXIT.is_empty());
    assert!(!ICON_ERROR.is_empty());
}

/// Test error handling constants
#[test]
fn test_error_constants() {
    use git_workers::constants::*;

    // Test error messages
    assert!(!ERROR_NO_WORKING_DIR.is_empty());
    assert!(!ERROR_TERMINAL_REQUIRED.is_empty());
    assert!(!ERROR_NON_INTERACTIVE.is_empty());
    assert!(!ERROR_PERMISSION_DENIED.is_empty());

    // Test that error messages are descriptive
    assert!(ERROR_TERMINAL_REQUIRED.contains("terminal"));
    assert!(ERROR_NON_INTERACTIVE.contains("interactive"));
    assert!(ERROR_PERMISSION_DENIED.contains("permission"));
}

/// Test formatting utilities
#[test]
fn test_formatting_utilities() {
    use git_workers::constants::{header_separator, section_header};

    // Test section header formatting
    let header = section_header("Test Section");
    assert!(header.contains("Test Section"));
    assert!(header.contains("="));

    // Test main header separator
    let separator = header_separator();
    assert!(!separator.is_empty());
    assert!(separator.contains("="));
}

/// Test application state constants
#[test]
fn test_application_state() {
    use git_workers::constants::*;

    // Test time formatting
    assert!(!TIME_FORMAT.is_empty());
    assert!(TIME_FORMAT.contains("%"));

    // Test default values
    assert!(!DEFAULT_BRANCH_UNKNOWN.is_empty());
    assert!(!DEFAULT_BRANCH_DETACHED.is_empty());
    assert!(!DEFAULT_AUTHOR_UNKNOWN.is_empty());
    assert!(!DEFAULT_MESSAGE_NONE.is_empty());
}

/// Test separator and formatting constants
#[test]
fn test_separator_constants() {
    use git_workers::constants::*;

    // Test that constants are defined and can be used
    let _sep_width = SEPARATOR_WIDTH;
    let _header_sep_width = HEADER_SEPARATOR_WIDTH;

    // Test that separators can be created
    let sep1 = "=".repeat(SEPARATOR_WIDTH);
    let sep2 = "=".repeat(HEADER_SEPARATOR_WIDTH);

    assert_eq!(sep1.len(), SEPARATOR_WIDTH);
    assert_eq!(sep2.len(), HEADER_SEPARATOR_WIDTH);
}

/// Test git-related constants
#[test]
fn test_git_constants() {
    use git_workers::constants::*;

    // Test git references
    assert!(!GIT_REMOTE_PREFIX.is_empty());
    assert!(!GIT_DEFAULT_MAIN_WORKTREE.is_empty());

    // Test directory patterns
    assert!(!WORKTREES_SUBDIR.is_empty());
    assert!(!BRANCH_SUBDIR.is_empty());

    // Test git command constants
    assert!(!GIT_CMD.is_empty());
    assert!(!GIT_WORKTREE.is_empty());
    assert!(!GIT_BRANCH.is_empty());
}

/// Test numeric constants
#[test]
fn test_numeric_constants() {
    use git_workers::constants::*;

    // Test array indices
    assert_eq!(WINDOW_FIRST_INDEX, 0);
    assert_eq!(WINDOW_SECOND_INDEX, 1);
    assert_eq!(GIT_HEAD_INDEX, 0);

    // Test size constants can be used
    let _commit_length = COMMIT_ID_SHORT_LENGTH;
    let _lock_timeout = STALE_LOCK_TIMEOUT_SECS;
    let _window_pairs = WINDOW_SIZE_PAIRS;
}

/// Test path and file constants
#[test]
fn test_path_constants() {
    use git_workers::constants::*;

    // Test path components
    assert_eq!(PATH_COMPONENT_SECOND_INDEX, 1);
    let _min_components = MIN_PATH_COMPONENTS_FOR_SUBDIR;

    // Test file-related constants
    assert!(!CONFIG_FILE_NAME.is_empty());
    assert!(!LOCK_FILE_NAME.is_empty());

    // Test git directory constants
    assert!(!GIT_DIR.is_empty());
    assert!(!GIT_GITDIR_PREFIX.is_empty());
    assert!(!GIT_GITDIR_SUFFIX.is_empty());
}

/// Test reserved names validation
#[test]
fn test_reserved_names() {
    use git_workers::constants::GIT_RESERVED_NAMES;

    // Test that reserved names list is not empty
    assert!(!GIT_RESERVED_NAMES.is_empty());

    // Test that common git names are included
    assert!(GIT_RESERVED_NAMES.contains(&"HEAD"));
    assert!(GIT_RESERVED_NAMES.contains(&"refs"));
    assert!(GIT_RESERVED_NAMES.contains(&"objects"));

    // Test that all names are non-empty
    for name in GIT_RESERVED_NAMES {
        assert!(!name.is_empty());
    }
}
