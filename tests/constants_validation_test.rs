//! Regression prevention tests for constants and format functions
//!
//! This test file detects changes to important constants and format functions
//! added in Phase 2-3, improving refactoring resistance.

use git_workers::constants::{
    ICON_ERROR, ICON_INFO, ICON_SUCCESS, ICON_WARNING, MAX_FILE_SIZE_MB, MENU_CREATE_WORKTREE,
    MENU_LIST_WORKTREES, MENU_SEARCH_WORKTREES, SEPARATOR_WIDTH, TEMPLATE_WORKTREE_NAME,
    TEMPLATE_WORKTREE_PATH,
};

#[test]
fn test_section_header_format_stability() {
    // Regression prevention for format changes
    let result = git_workers::constants::section_header("Test");

    // Verify basic structure
    assert!(result.contains("Test"), "Title is not included");
    assert!(result.contains("="), "Separator character is not included");
    assert_eq!(
        result.lines().count(),
        2,
        "Number of lines differs from expected"
    );

    // Verify specific format (accounting for ANSI color codes)
    let lines: Vec<&str> = result.lines().collect();
    assert!(
        lines[0].contains("Test"),
        "Title line should contain 'Test'"
    );
    assert!(
        lines[1].starts_with("\u{1b}[") || lines[1].starts_with("="),
        "Separator line should start with ANSI code or '='"
    );
}

#[test]
fn test_critical_constants_unchanged() {
    // Prevent unintentional changes to critical constants
    assert_eq!(MAX_FILE_SIZE_MB, 100, "File size limit has been changed");
    assert_eq!(SEPARATOR_WIDTH, 40, "Separator line width has been changed");
}

#[test]
fn test_menu_constants_stability() {
    // Prevent unintentional changes to menu display strings
    assert_eq!(
        MENU_LIST_WORKTREES, "•  List worktrees",
        "List menu display has been changed"
    );
    assert_eq!(
        MENU_SEARCH_WORKTREES, "?  Search worktrees",
        "Search menu display has been changed"
    );
    assert_eq!(
        MENU_CREATE_WORKTREE, "+  Create worktree",
        "Create menu display has been changed"
    );
}

#[test]
fn test_format_consistency() {
    // Verify consistency of multiple format functions
    let header1 = git_workers::constants::section_header("Header1");
    let header2 = git_workers::constants::section_header("Header2");

    // Both have the same structure
    assert_eq!(
        header1.lines().count(),
        header2.lines().count(),
        "Header structure consistency has been lost"
    );

    // Separator line lengths match
    let separator1 = header1.lines().nth(1).unwrap();
    let separator2 = header2.lines().nth(1).unwrap();

    // Compare actual character count excluding ANSI escape sequences
    let clean_sep1 = separator1
        .chars()
        .filter(|c| !c.is_control() && *c != '\u{1b}')
        .count();
    let clean_sep2 = separator2
        .chars()
        .filter(|c| !c.is_control() && *c != '\u{1b}')
        .count();

    assert_eq!(
        clean_sep1, clean_sep2,
        "Separator line lengths do not match"
    );
}

#[test]
fn test_separator_width_usage() {
    // Verify that SEPARATOR_WIDTH constant is actually being used
    let header = git_workers::constants::section_header("Test");
    let separator_line = header.lines().nth(1).unwrap();

    // Verify that the actual number of separator characters matches SEPARATOR_WIDTH
    // Count the number of "=" excluding ANSI escape sequences
    let equals_count = separator_line.chars().filter(|c| *c == '=').count();
    assert_eq!(
        equals_count, SEPARATOR_WIDTH,
        "Separator line length does not match SEPARATOR_WIDTH constant"
    );
}

#[test]
fn test_header_format_structure() {
    // Verify header format structure
    let header = git_workers::constants::section_header("Test");

    // Verify basic structure
    assert!(header.contains("Test"), "Title is not included");
    assert!(header.contains("="), "Separator character is not included");
    assert_eq!(
        header.lines().count(),
        2,
        "Number of lines differs from expected"
    );
}

#[test]
fn test_menu_item_format_consistency() {
    // Verify uniform format of menu items
    let list_menu = MENU_LIST_WORKTREES;
    let search_menu = MENU_SEARCH_WORKTREES;
    let create_menu = MENU_CREATE_WORKTREE;

    // Verify that all follow the same format "symbol  description"
    assert!(
        list_menu.starts_with('•'),
        "List menu leading symbol has been changed"
    );
    assert!(
        search_menu.starts_with('?'),
        "Search menu leading symbol has been changed"
    );
    assert!(
        create_menu.starts_with('+'),
        "Create menu leading symbol has been changed"
    );

    // Verify that all follow the same format with two spaces
    assert!(
        list_menu.starts_with("•  "),
        "List menu format has been changed"
    );
    assert!(
        search_menu.starts_with("?  "),
        "Search menu format has been changed"
    );
    assert!(
        create_menu.starts_with("+  "),
        "Create menu format has been changed"
    );
}

#[test]
fn test_icon_constants_stability() {
    // Verify stability of icon constants
    assert_eq!(ICON_SUCCESS, "✓", "Success icon has been changed");
    assert_eq!(ICON_ERROR, "✗", "Error icon has been changed");
    assert_eq!(ICON_INFO, "ℹ️", "Info icon has been changed");
    assert_eq!(ICON_WARNING, "⚠", "Warning icon has been changed");
}

#[test]
fn test_template_variable_constants() {
    // Verify stability of template variable constants
    assert_eq!(
        TEMPLATE_WORKTREE_NAME, "{{worktree_name}}",
        "Worktree name template has been changed"
    );
    assert_eq!(
        TEMPLATE_WORKTREE_PATH, "{{worktree_path}}",
        "Worktree path template has been changed"
    );

    // Verify that template variables are in a replaceable format
    assert!(
        TEMPLATE_WORKTREE_NAME.starts_with("{{") && TEMPLATE_WORKTREE_NAME.ends_with("}}"),
        "Worktree name template is not in the correct format"
    );
    assert!(
        TEMPLATE_WORKTREE_PATH.starts_with("{{") && TEMPLATE_WORKTREE_PATH.ends_with("}}"),
        "Worktree path template is not in the correct format"
    );
}

#[test]
fn test_constants_value_types() {
    // Verify type and value validity of constants
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(
            MAX_FILE_SIZE_MB > 0,
            "File size limit must be a positive value"
        );
        assert!(MAX_FILE_SIZE_MB <= 1000, "File size limit is too large");

        assert!(SEPARATOR_WIDTH > 10, "Separator line width is too short");
        assert!(SEPARATOR_WIDTH <= 100, "Separator line width is too long");
    }
}

#[test]
fn test_unicode_icon_consistency() {
    // Verify consistency of Unicode icons
    let icons = vec![ICON_SUCCESS, ICON_ERROR, ICON_INFO, ICON_WARNING];

    for icon in icons {
        assert!(!icon.is_empty(), "Icon is an empty string");
        assert!(icon.chars().count() <= 3, "Icon is too long");

        // Verify it's a basic Unicode character
        for ch in icon.chars() {
            assert!(
                ch.is_alphanumeric() || ch.is_ascii_punctuation() || ch as u32 > 127,
                "Invalid character in icon: {ch}"
            );
        }
    }
}

#[test]
fn test_template_variable_format() {
    // Verify naming convention of template variables
    let templates = vec![TEMPLATE_WORKTREE_NAME, TEMPLATE_WORKTREE_PATH];

    for template in templates {
        // Template variables are in {{variable_name}} format
        assert!(
            template.starts_with("{{"),
            "Template variable does not have correct opening format: {template}"
        );
        assert!(
            template.ends_with("}}"),
            "Template variable does not have correct closing format: {template}"
        );

        // Verify that the inner variable name is valid
        let inner = &template[2..template.len() - 2];
        assert!(!inner.is_empty(), "Template variable name is empty");
        assert!(
            inner.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "Template variable name contains invalid characters: {inner}"
        );
    }
}

#[test]
fn test_constant_immutability() {
    // Verify immutability of constants (guaranteed at compile time, but explicitly tested)
    let original_max_size = MAX_FILE_SIZE_MB;
    let original_separator_width = SEPARATOR_WIDTH;

    // Reconfirm that values are as expected
    assert_eq!(
        MAX_FILE_SIZE_MB, original_max_size,
        "Constant value has been changed at runtime"
    );
    assert_eq!(
        SEPARATOR_WIDTH, original_separator_width,
        "Constant value has been changed at runtime"
    );
}

#[test]
fn test_icon_message_format_consistency() {
    // Verify format consistency of icon messages with test strings
    let test_messages = vec![
        format!("{ICON_ERROR} Test error"),
        format!("{ICON_SUCCESS} Test success"),
        format!("{ICON_INFO} Test info"),
        format!("{ICON_WARNING} Test warning"),
    ];

    for message in test_messages {
        // Verify that the message is not empty
        assert!(!message.trim().is_empty(), "Message is empty");

        // Verify that the icon is at the beginning
        let has_valid_icon = message.starts_with(ICON_SUCCESS)
            || message.starts_with(ICON_ERROR)
            || message.starts_with(ICON_INFO)
            || message.starts_with(ICON_WARNING);
        assert!(has_valid_icon, "No valid icon found: {message}");
    }
}

#[test]
fn test_menu_accessibility() {
    // Verify accessibility of menu items
    let menu_items = vec![
        MENU_LIST_WORKTREES,
        MENU_SEARCH_WORKTREES,
        MENU_CREATE_WORKTREE,
    ];

    for item in menu_items {
        // Verify that it contains an icon or ASCII symbol
        let has_icon_or_symbol =
            item.chars().any(|c| c as u32 > 127) || item.chars().any(|c| c.is_ascii_punctuation());
        assert!(
            has_icon_or_symbol,
            "Menu item does not contain an icon or symbol: {item}"
        );

        // Verify appropriate length
        assert!(item.len() >= 5, "Menu item is too short: {item}");
        assert!(item.len() <= 50, "Menu item is too long: {item}");

        // Verify that whitespace is properly placed
        assert!(
            item.contains("  "),
            "Menu item does not have appropriate whitespace: {item}"
        );
    }
}

#[test]
fn test_constants_documentation_compliance() {
    // Verify that constants are part of properly documented public API
    // These constants must be accessible from outside

    // Verify that constants can actually be accessed (guaranteed at compile time)
    let _max_size = MAX_FILE_SIZE_MB;
    let _separator = SEPARATOR_WIDTH;
    let _success = ICON_SUCCESS;
    let _error = ICON_ERROR;
    let _info = ICON_INFO;
    let _warning = ICON_WARNING;
    let _template_name = TEMPLATE_WORKTREE_NAME;
    let _template_path = TEMPLATE_WORKTREE_PATH;
    let _menu_list = MENU_LIST_WORKTREES;
    let _menu_search = MENU_SEARCH_WORKTREES;
    let _menu_create = MENU_CREATE_WORKTREE;

    // Test succeeds if all constants are accessible
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(true, "All constants are accessible");
    }
}
