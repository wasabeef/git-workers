use git_workers::menu::MenuItem;

#[test]
fn test_menu_item_display_comprehensive() {
    // Test Display implementation for all menu items with exact string matching
    assert_eq!(format!("{}", MenuItem::ListWorktrees), "•  List worktrees");
    assert_eq!(
        format!("{}", MenuItem::SearchWorktrees),
        "?  Search worktrees"
    );
    assert_eq!(
        format!("{}", MenuItem::CreateWorktree),
        "+  Create worktree"
    );
    assert_eq!(
        format!("{}", MenuItem::DeleteWorktree),
        "-  Delete worktree"
    );
    assert_eq!(
        format!("{}", MenuItem::BatchDelete),
        "=  Batch delete worktrees"
    );
    assert_eq!(
        format!("{}", MenuItem::CleanupOldWorktrees),
        "~  Cleanup old worktrees"
    );
    assert_eq!(
        format!("{}", MenuItem::SwitchWorktree),
        "→  Switch worktree"
    );
    assert_eq!(
        format!("{}", MenuItem::RenameWorktree),
        "*  Rename worktree"
    );
    assert_eq!(format!("{}", MenuItem::EditHooks), "⚙  Edit hooks");
    assert_eq!(format!("{}", MenuItem::Exit), "x  Exit");
}

#[test]
fn test_menu_item_to_string_method() {
    // Test to_string() method specifically
    assert_eq!(MenuItem::ListWorktrees.to_string(), "•  List worktrees");
    assert_eq!(MenuItem::SearchWorktrees.to_string(), "?  Search worktrees");
    assert_eq!(MenuItem::CreateWorktree.to_string(), "+  Create worktree");
    assert_eq!(MenuItem::DeleteWorktree.to_string(), "-  Delete worktree");
    assert_eq!(
        MenuItem::BatchDelete.to_string(),
        "=  Batch delete worktrees"
    );
    assert_eq!(
        MenuItem::CleanupOldWorktrees.to_string(),
        "~  Cleanup old worktrees"
    );
    assert_eq!(MenuItem::SwitchWorktree.to_string(), "→  Switch worktree");
    assert_eq!(MenuItem::RenameWorktree.to_string(), "*  Rename worktree");
    assert_eq!(MenuItem::Exit.to_string(), "x  Exit");
}

#[test]
fn test_all_menu_items_have_display() {
    // Test that all menu items can be converted to strings and aren't empty
    let items = [
        MenuItem::ListWorktrees,
        MenuItem::SwitchWorktree,
        MenuItem::SearchWorktrees,
        MenuItem::CreateWorktree,
        MenuItem::DeleteWorktree,
        MenuItem::BatchDelete,
        MenuItem::CleanupOldWorktrees,
        MenuItem::RenameWorktree,
        MenuItem::EditHooks,
        MenuItem::Exit,
    ];

    for item in &items {
        let display = item.to_string();
        assert!(
            !display.is_empty(),
            "Menu item {item:?} should have non-empty display"
        );

        // Check that all have proper formatting with icon and description
        assert!(
            display.len() > 3,
            "Menu item display should have icon and description"
        );

        // Check for consistent spacing (two spaces between icon and text)
        let parts: Vec<&str> = display.splitn(2, "  ").collect();
        assert_eq!(
            parts.len(),
            2,
            "Menu item should have icon separated by two spaces from text"
        );
    }
}

#[test]
fn test_menu_item_icon_consistency() {
    // Verify all menu items have consistent icon formatting
    let items_and_icons = [
        (MenuItem::ListWorktrees, "•"),
        (MenuItem::SearchWorktrees, "?"),
        (MenuItem::CreateWorktree, "+"),
        (MenuItem::DeleteWorktree, "-"),
        (MenuItem::BatchDelete, "="),
        (MenuItem::CleanupOldWorktrees, "~"),
        (MenuItem::SwitchWorktree, "→"),
        (MenuItem::RenameWorktree, "*"),
        (MenuItem::EditHooks, "⚙"),
        (MenuItem::Exit, "x"),
    ];

    for (item, expected_icon) in &items_and_icons {
        let display = item.to_string();
        assert!(
            display.starts_with(expected_icon),
            "Menu item {item:?} should start with icon '{expected_icon}'"
        );
    }
}

#[test]
fn test_menu_item_text_consistency() {
    // Verify all menu items have proper text descriptions
    let items_and_texts = [
        (MenuItem::ListWorktrees, "List worktrees"),
        (MenuItem::SearchWorktrees, "Search worktrees"),
        (MenuItem::CreateWorktree, "Create worktree"),
        (MenuItem::DeleteWorktree, "Delete worktree"),
        (MenuItem::BatchDelete, "Batch delete worktrees"),
        (MenuItem::CleanupOldWorktrees, "Cleanup old worktrees"),
        (MenuItem::SwitchWorktree, "Switch worktree"),
        (MenuItem::RenameWorktree, "Rename worktree"),
        (MenuItem::EditHooks, "Edit hooks"),
        (MenuItem::Exit, "Exit"),
    ];

    for (item, expected_text) in &items_and_texts {
        let display = item.to_string();
        assert!(
            display.ends_with(expected_text),
            "Menu item {item:?} should end with text '{expected_text}'"
        );
    }
}
