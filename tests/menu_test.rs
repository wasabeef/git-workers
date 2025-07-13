use git_workers::menu::MenuItem;

#[test]
fn test_menu_item_creation() {
    // Test that all menu items can be created
    let items = [
        MenuItem::ListWorktrees,
        MenuItem::SearchWorktrees,
        MenuItem::CreateWorktree,
        MenuItem::DeleteWorktree,
        MenuItem::BatchDelete,
        MenuItem::CleanupOldWorktrees,
        MenuItem::SwitchWorktree,
        MenuItem::RenameWorktree,
        MenuItem::Exit,
    ];

    assert_eq!(items.len(), 9);
}

#[test]
fn test_menu_item_clone() {
    let item = MenuItem::CreateWorktree;
    let cloned = item;

    assert_eq!(format!("{item}"), format!("{cloned}"));
}

#[test]
fn test_menu_item_debug() {
    // Test Debug implementation
    let item = MenuItem::ListWorktrees;
    let debug_str = format!("{item:?}");
    assert_eq!(debug_str, "ListWorktrees");
}

#[test]
fn test_menu_item_partial_eq() {
    // Test PartialEq implementation
    assert_eq!(MenuItem::ListWorktrees, MenuItem::ListWorktrees);
    assert_ne!(MenuItem::ListWorktrees, MenuItem::CreateWorktree);

    let item1 = MenuItem::SwitchWorktree;
    let item2 = MenuItem::SwitchWorktree;
    let item3 = MenuItem::DeleteWorktree;

    assert_eq!(item1, item2);
    assert_ne!(item1, item3);
}

#[test]
fn test_menu_item_comprehensive_equality() {
    let all_items = vec![
        MenuItem::ListWorktrees,
        MenuItem::SearchWorktrees,
        MenuItem::CreateWorktree,
        MenuItem::DeleteWorktree,
        MenuItem::BatchDelete,
        MenuItem::CleanupOldWorktrees,
        MenuItem::SwitchWorktree,
        MenuItem::RenameWorktree,
        MenuItem::Exit,
    ];

    // Each item should equal itself
    for item in &all_items {
        assert_eq!(item, item);
    }

    // Each item should not equal any other item
    for (i, item1) in all_items.iter().enumerate() {
        for (j, item2) in all_items.iter().enumerate() {
            if i != j {
                assert_ne!(item1, item2);
            }
        }
    }
}

#[test]
fn test_menu_item_icons() {
    // Test that icons are correctly included in display strings
    let list_display = format!("{}", MenuItem::ListWorktrees);
    assert!(list_display.contains("•"));

    let search_display = format!("{}", MenuItem::SearchWorktrees);
    assert!(search_display.contains("?"));

    let create_display = format!("{}", MenuItem::CreateWorktree);
    assert!(create_display.contains("+"));

    let delete_display = format!("{}", MenuItem::DeleteWorktree);
    assert!(delete_display.contains("-"));

    let batch_delete_display = format!("{}", MenuItem::BatchDelete);
    assert!(batch_delete_display.contains("="));

    let cleanup_display = format!("{}", MenuItem::CleanupOldWorktrees);
    assert!(cleanup_display.contains("~"));

    let switch_display = format!("{}", MenuItem::SwitchWorktree);
    assert!(switch_display.contains("→"));

    let rename_display = format!("{}", MenuItem::RenameWorktree);
    assert!(rename_display.contains("*"));

    let exit_display = format!("{}", MenuItem::Exit);
    assert!(exit_display.contains("x"));
}

#[test]
fn test_menu_item_text_content() {
    // Test that text content is correctly included
    assert!(format!("{}", MenuItem::ListWorktrees).contains("List worktrees"));
    assert!(format!("{}", MenuItem::SearchWorktrees).contains("Search worktrees"));
    assert!(format!("{}", MenuItem::CreateWorktree).contains("Create worktree"));
    assert!(format!("{}", MenuItem::DeleteWorktree).contains("Delete worktree"));
    assert!(format!("{}", MenuItem::BatchDelete).contains("Batch delete"));
    assert!(format!("{}", MenuItem::CleanupOldWorktrees).contains("Cleanup old"));
    assert!(format!("{}", MenuItem::SwitchWorktree).contains("Switch worktree"));
    assert!(format!("{}", MenuItem::RenameWorktree).contains("Rename worktree"));
    assert!(format!("{}", MenuItem::Exit).contains("Exit"));
}

#[test]
fn test_menu_item_match_pattern() {
    // Test that menu items can be used in match statements
    let item = MenuItem::CreateWorktree;

    let result = match item {
        MenuItem::ListWorktrees => "list",
        MenuItem::SearchWorktrees => "search",
        MenuItem::CreateWorktree => "create",
        MenuItem::DeleteWorktree => "delete",
        MenuItem::BatchDelete => "batch_delete",
        MenuItem::CleanupOldWorktrees => "cleanup",
        MenuItem::SwitchWorktree => "switch",
        MenuItem::RenameWorktree => "rename",
        MenuItem::EditHooks => "edit_hooks",
        MenuItem::Exit => "exit",
    };

    assert_eq!(result, "create");
}
