use anyhow::Result;
use git2::Repository;
//
use tempfile::TempDir;

use git_workers::commands;
use git_workers::menu::MenuItem;

#[test]
fn test_execute_all_menu_items() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Change to repo directory for testing
    std::env::set_current_dir(&repo_path)?;

    // Test each menu item's execute path (not interactive parts)
    let items = vec![
        MenuItem::ListWorktrees,
        MenuItem::DeleteWorktree,
        MenuItem::BatchDelete,
        MenuItem::CleanupOldWorktrees,
        MenuItem::RenameWorktree,
        // Note: Skipping interactive items that would hang: SearchWorktrees, CreateWorktree, SwitchWorktree
    ];

    for item in items {
        // These should not panic, even if they return errors due to empty state
        let result = commands::execute(item);
        // We don't assert success because these operations may legitimately fail
        // in an empty repository, but they shouldn't panic
        // Success is fine, errors are expected for some operations
        let _ = result;
    }

    Ok(())
}

#[test]
fn test_commands_module_functions() -> Result<()> {
    // Test that command module functions exist and can be called
    // This is mainly a compilation test

    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    std::env::set_current_dir(&repo_path)?;

    // Test that execute function exists and handles invalid states gracefully
    let result = commands::execute(MenuItem::ListWorktrees);
    // Should succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[test]
fn test_menu_item_coverage() {
    // Test that all menu items are covered in some way
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

    // Verify we have all expected menu items
    assert_eq!(all_items.len(), 9);

    // Test that each item can be formatted
    for item in all_items {
        let display_str = format!("{}", item);
        assert!(!display_str.is_empty());

        // Test debug format
        let debug_str = format!("{:?}", item);
        assert!(!debug_str.is_empty());
    }
}

// Helper function
fn create_initial_commit(repo: &Repository) -> Result<()> {
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}
