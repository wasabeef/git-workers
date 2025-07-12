use std::env;

/// Test application metadata and versioning
#[test]
fn test_application_metadata_comprehensive() {
    // Test all available metadata from Cargo.toml
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let repository = env!("CARGO_PKG_REPOSITORY");

    // Basic validation
    assert_eq!(name, "git-workers");
    assert!(!version.is_empty());
    assert!(!description.is_empty());
    assert!(!authors.is_empty());
    assert!(!repository.is_empty());

    // Version format validation (semantic versioning)
    let version_parts: Vec<&str> = version.split('.').collect();
    assert!(version_parts.len() >= 2);
    assert!(version_parts.len() <= 4); // major.minor.patch[-pre]

    // Each version component should be numeric (before any pre-release suffix)
    for (i, part) in version_parts.iter().enumerate().take(3) {
        let numeric_part = part.split('-').next().unwrap();
        assert!(
            numeric_part.chars().all(|c| c.is_ascii_digit()),
            "Version part {i} '{numeric_part}' should be numeric"
        );
    }

    // Description should contain relevant keywords
    let desc_lower = description.to_lowercase();
    assert!(
        desc_lower.contains("git")
            || desc_lower.contains("worktree")
            || desc_lower.contains("interactive")
    );

    // Repository should be a valid URL
    assert!(repository.starts_with("https://"));
    assert!(repository.contains("github.com") || repository.contains("gitlab.com"));
}

/// Test environment variable detection and handling
#[test]
fn test_environment_detection() {
    let original_ci = env::var("CI").ok();
    let original_term = env::var("TERM").ok();
    let original_switch_file = env::var("GW_SWITCH_FILE").ok();

    // Test CI environment detection
    env::set_var("CI", "true");
    assert_eq!(env::var("CI").unwrap(), "true");

    env::set_var("CI", "false");
    assert_eq!(env::var("CI").unwrap(), "false");

    env::remove_var("CI");
    assert!(env::var("CI").is_err());

    // Test terminal detection
    env::set_var("TERM", "xterm-256color");
    assert_eq!(env::var("TERM").unwrap(), "xterm-256color");

    env::set_var("TERM", "dumb");
    assert_eq!(env::var("TERM").unwrap(), "dumb");

    // Test shell integration environment
    let test_switch_file = "/tmp/test-gw-switch";
    env::set_var("GW_SWITCH_FILE", test_switch_file);
    assert_eq!(env::var("GW_SWITCH_FILE").unwrap(), test_switch_file);

    // Test environment cleanup
    env::remove_var("GW_SWITCH_FILE");
    assert!(env::var("GW_SWITCH_FILE").is_err());

    // Restore original environment
    if let Some(ci) = original_ci {
        env::set_var("CI", ci);
    }
    if let Some(term) = original_term {
        env::set_var("TERM", term);
    } else {
        env::remove_var("TERM");
    }
    if let Some(switch_file) = original_switch_file {
        env::set_var("GW_SWITCH_FILE", switch_file);
    }
}

/// Test application constants used in main
#[test]
fn test_main_application_constants() {
    use git_workers::constants::*;

    // Test exit codes
    assert_eq!(EXIT_SUCCESS, 0);
    assert_eq!(EXIT_FAILURE, 1);

    // Test user interface messages
    assert!(!MSG_PRESS_ANY_KEY.is_empty());
    assert!(MSG_PRESS_ANY_KEY.contains("Press"));
    assert!(MSG_PRESS_ANY_KEY.contains("key"));

    // Test switch file markers
    assert!(!SWITCH_TO_PREFIX.is_empty());
    assert_eq!(SWITCH_TO_PREFIX, "SWITCH_TO:");

    // Test warning messages
    assert!(!WARNING_NO_WORKTREES.is_empty());
    assert!(WARNING_NO_WORKTREES.to_lowercase().contains("worktree"));
}

/// Test menu system components
#[test]
fn test_menu_system_comprehensive() {
    use git_workers::menu::MenuItem;

    // Test all menu items
    let all_items = vec![
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

    // Verify all items have unique display strings
    let mut display_strings = Vec::new();
    for item in &all_items {
        let display = format!("{item}");
        assert!(!display.is_empty());
        assert!(!display_strings.contains(&display));
        display_strings.push(display);
    }

    // Test debug representation
    for item in &all_items {
        let debug = format!("{item:?}");
        assert!(!debug.is_empty());
        assert!(debug.is_ascii());
    }

    // Test specific menu item content
    let list_display = MenuItem::ListWorktrees.to_string();
    assert!(list_display.to_lowercase().contains("list"));

    let create_display = MenuItem::CreateWorktree.to_string();
    assert!(create_display.to_lowercase().contains("create"));

    let exit_display = MenuItem::Exit.to_string();
    assert!(exit_display.to_lowercase().contains("exit"));
}

/// Test error handling constants and patterns
#[test]
fn test_error_handling_system() {
    use git_workers::constants::*;

    // Test terminal-related errors
    assert!(!ERROR_TERMINAL_REQUIRED.is_empty());
    assert!(ERROR_TERMINAL_REQUIRED.to_lowercase().contains("terminal"));

    assert!(!ERROR_NON_INTERACTIVE.is_empty());
    assert!(ERROR_NON_INTERACTIVE.to_lowercase().contains("interactive"));

    // Test permission errors
    assert!(!ERROR_PERMISSION_DENIED.is_empty());
    assert!(ERROR_PERMISSION_DENIED
        .to_lowercase()
        .contains("permission"));

    // Test working directory errors
    assert!(!ERROR_NO_WORKING_DIR.is_empty());
    assert!(ERROR_NO_WORKING_DIR.to_lowercase().contains("working"));

    // Test all error messages are descriptive
    let error_constants = [
        ERROR_TERMINAL_REQUIRED,
        ERROR_NON_INTERACTIVE,
        ERROR_PERMISSION_DENIED,
        ERROR_NO_WORKING_DIR,
        ERROR_WORKTREE_CREATE,
        ERROR_CONFIG_LOAD,
    ];

    for error_msg in error_constants {
        assert!(!error_msg.is_empty());
        assert!(error_msg.len() > 10); // Should be descriptive
        assert!(error_msg.chars().any(|c| c.is_ascii_alphabetic()));
    }
}

/// Test formatting and display utilities
#[test]
fn test_formatting_utilities_comprehensive() {
    use git_workers::constants::{
        header_separator, section_header, HEADER_SEPARATOR_WIDTH, SEPARATOR_WIDTH,
    };

    // Test section header creation
    let header = section_header("Test Section");
    assert!(header.contains("Test Section"));
    assert!(header.contains("="));
    assert!(header.chars().any(|c| c == '\n')); // Should have newline

    // Test with different section names
    let headers = ["Worktrees", "Configuration", "Git Status"];
    for section_name in headers {
        let formatted = section_header(section_name);
        assert!(formatted.contains(section_name));
        assert!(formatted.contains("="));
    }

    // Test main header separator
    let separator = header_separator();
    assert!(!separator.is_empty());
    assert!(separator.chars().all(|c| c == '=' || c.is_whitespace()));
    assert_eq!(separator.trim(), "=".repeat(HEADER_SEPARATOR_WIDTH));

    // Test separator constants can be used
    let _sep_width = SEPARATOR_WIDTH;
    let _header_sep_width = HEADER_SEPARATOR_WIDTH;

    // Test manual separator creation
    let manual_sep = "=".repeat(SEPARATOR_WIDTH);
    assert_eq!(manual_sep.len(), SEPARATOR_WIDTH);
}

/// Test application state constants
#[test]
fn test_application_state_constants() {
    use git_workers::constants::*;

    // Test time formatting
    assert!(!TIME_FORMAT.is_empty());
    assert!(TIME_FORMAT.contains("%"));

    // Test default values
    assert!(!DEFAULT_BRANCH_UNKNOWN.is_empty());
    assert!(!DEFAULT_BRANCH_DETACHED.is_empty());
    assert!(!DEFAULT_AUTHOR_UNKNOWN.is_empty());
    assert!(!DEFAULT_MESSAGE_NONE.is_empty());

    // Test that defaults are reasonable
    assert_eq!(DEFAULT_BRANCH_UNKNOWN, "unknown");
    assert_eq!(DEFAULT_BRANCH_DETACHED, "detached");
    assert!(DEFAULT_AUTHOR_UNKNOWN.to_lowercase().contains("unknown"));
    assert!(DEFAULT_MESSAGE_NONE.to_lowercase().contains("no"));

    // Test configuration defaults
    assert!(!CONFIG_FILE_NAME.is_empty());
    assert!(CONFIG_FILE_NAME.ends_with(".toml"));
    assert_eq!(CONFIG_FILE_NAME, ".git-workers.toml");
}

/// Test icon and label constants
#[test]
fn test_icon_and_label_constants() {
    use git_workers::constants::*;

    // Test icons are non-empty and reasonable length
    let icons = [
        ICON_LIST,
        ICON_SEARCH,
        ICON_CREATE,
        ICON_DELETE,
        ICON_CLEANUP,
        ICON_SWITCH,
        ICON_RENAME,
        ICON_EDIT,
        ICON_EXIT,
        ICON_ERROR,
        ICON_QUESTION,
        ICON_ARROW,
        ICON_LOCAL_BRANCH,
        ICON_REMOTE_BRANCH,
        ICON_TAG_INDICATOR,
    ];

    for icon in icons {
        assert!(!icon.is_empty());
        assert!(icon.len() <= 10); // Icons should be reasonably short
    }

    // Test labels
    let labels = [
        LABEL_BRANCH,
        LABEL_MODIFIED,
        LABEL_NAME,
        LABEL_PATH,
        LABEL_YES,
        LABEL_NO,
    ];

    for label in labels {
        assert!(!label.is_empty());
        // Labels may contain non-alphabetic characters (e.g., spaces, punctuation)
        assert!(label.chars().any(|c| c.is_ascii_alphabetic()));
    }

    // Test specific label values
    assert_eq!(LABEL_YES, "Yes");
    assert_eq!(LABEL_NO, "No");
}

/// Test numeric constants and limits
#[test]
fn test_numeric_constants_comprehensive() {
    use git_workers::constants::*;

    // Test UI layout constants can be used
    let _items_per_page = UI_MIN_ITEMS_PER_PAGE;
    let _header_lines = UI_HEADER_LINES;
    let _footer_lines = UI_FOOTER_LINES;
    let _name_col_width = UI_NAME_COL_MIN_WIDTH;
    let _path_col_width = UI_PATH_COL_WIDTH;
    let _modified_col_width = UI_MODIFIED_COL_WIDTH;
    let _branch_col_extra = UI_BRANCH_COL_EXTRA_WIDTH;

    // Test array indices
    assert_eq!(WINDOW_FIRST_INDEX, 0);
    assert_eq!(WINDOW_SECOND_INDEX, 1);
    assert_eq!(GIT_HEAD_INDEX, 0);
    assert_eq!(PATH_COMPONENT_SECOND_INDEX, 1);

    // Test size constants can be used
    let _commit_length = COMMIT_ID_SHORT_LENGTH;
    let _max_name_length = MAX_WORKTREE_NAME_LENGTH;

    // Test timing constants can be used
    let _tick_millis = PROGRESS_BAR_TICK_MILLIS;
    let _lock_timeout = STALE_LOCK_TIMEOUT_SECS;
}

/// Test git-related constants
#[test]
fn test_git_constants_comprehensive() {
    use git_workers::constants::*;

    // Test git command constants
    assert!(!GIT_CMD.is_empty());
    assert_eq!(GIT_CMD, "git");

    assert!(!GIT_WORKTREE.is_empty());
    assert_eq!(GIT_WORKTREE, "worktree");

    assert!(!GIT_BRANCH.is_empty());
    assert_eq!(GIT_BRANCH, "branch");

    // Test git directory constants
    assert!(!GIT_DIR.is_empty());
    assert_eq!(GIT_DIR, ".git");

    // Test git references
    assert!(!GIT_REMOTE_PREFIX.is_empty());
    assert_eq!(GIT_REMOTE_PREFIX, "origin/");

    assert!(!GIT_DEFAULT_MAIN_WORKTREE.is_empty());
    assert_eq!(GIT_DEFAULT_MAIN_WORKTREE, "main");

    // Test directory patterns
    assert!(!WORKTREES_SUBDIR.is_empty());
    assert_eq!(WORKTREES_SUBDIR, "worktrees");

    assert!(!BRANCH_SUBDIR.is_empty());
    assert_eq!(BRANCH_SUBDIR, "branch");

    // Test reserved names array
    assert!(!GIT_RESERVED_NAMES.is_empty());
    assert!(GIT_RESERVED_NAMES.contains(&"HEAD"));
    assert!(GIT_RESERVED_NAMES.contains(&"refs"));
    assert!(GIT_RESERVED_NAMES.contains(&"objects"));

    // All reserved names should be non-empty
    for name in GIT_RESERVED_NAMES {
        assert!(!name.is_empty());
        // Note: Some git names like "HEAD" are uppercase by convention
    }
}

/// Test character and string validation constants
#[test]
fn test_validation_constants() {
    use git_workers::constants::*;

    // Test invalid character arrays
    assert!(!INVALID_FILESYSTEM_CHARS.is_empty());
    assert!(INVALID_FILESYSTEM_CHARS.contains(&'/'));
    assert!(INVALID_FILESYSTEM_CHARS.contains(&'\\'));

    assert!(!WINDOWS_RESERVED_CHARS.is_empty());
    assert!(WINDOWS_RESERVED_CHARS.contains(&'<'));
    assert!(WINDOWS_RESERVED_CHARS.contains(&'>'));
    assert!(WINDOWS_RESERVED_CHARS.contains(&':'));

    // Test git critical directories
    assert!(!GIT_CRITICAL_DIRS.is_empty());
    assert!(GIT_CRITICAL_DIRS.contains(&"objects"));
    assert!(GIT_CRITICAL_DIRS.contains(&"refs"));
    assert!(GIT_CRITICAL_DIRS.contains(&"hooks"));

    // All critical dirs should be valid directory names
    for dir in GIT_CRITICAL_DIRS {
        assert!(!dir.is_empty());
        assert!(!dir.contains('/'));
        assert!(!dir.contains('\\'));
    }
}
