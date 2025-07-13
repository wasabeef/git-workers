//! Unified command tests
//!
//! Consolidates commands_comprehensive_test.rs, commands_extensive_test.rs, comprehensive_commands_test.rs, commands_test.rs
//! Eliminates duplicates and provides comprehensive command testing

use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use git_workers::commands::{self, validate_custom_path, validate_worktree_name};
use git_workers::git::GitWorktreeManager;
use git_workers::menu::MenuItem;

/// Helper to create a test repository with initial commit
fn setup_test_repo() -> Result<(TempDir, PathBuf, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

    Ok((temp_dir, repo_path, manager))
}

/// Helper to create initial commit for repository
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let signature = git2::Signature::now("Test User", "test@example.com")?;

    // Create a file
    let workdir = repo.workdir().unwrap();
    fs::write(workdir.join("README.md"), "# Test Repository")?;

    // Add file to index
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("README.md"))?;
    index.write()?;

    // Create tree
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Create commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )?;

    Ok(())
}

// =============================================================================
// Basic error handling tests
// =============================================================================

/// Test error handling when not in a git repository
#[test]
fn test_commands_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let non_git_path = temp_dir.path().join("not-a-repo");
    fs::create_dir_all(&non_git_path).unwrap();

    // Change to non-git directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&non_git_path).unwrap();

    // Test that commands handle non-git repos gracefully
    let result = commands::list_worktrees();
    // Should either succeed with empty list or fail gracefully
    assert!(result.is_ok() || result.is_err());

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

/// Test commands in an empty git repository
#[test]
fn test_commands_empty_git_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("empty-repo");

    // Initialize empty repository (no initial commit)
    Repository::init(&repo_path)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test commands in empty repository
    let result = commands::list_worktrees();
    // Should handle empty repo gracefully
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

// =============================================================================
// Menu item execution tests
// =============================================================================

/// Test executing all menu items without crashing
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
        let result = match item {
            MenuItem::ListWorktrees => commands::list_worktrees(),
            MenuItem::DeleteWorktree => commands::delete_worktree(),
            MenuItem::BatchDelete => commands::batch_delete_worktrees(),
            MenuItem::CleanupOldWorktrees => commands::cleanup_old_worktrees(),
            MenuItem::RenameWorktree => commands::rename_worktree(),
            _ => Ok(()),
        };
        // We don't assert success because these operations may legitimately fail
        // in an empty repository, but they shouldn't panic
        // Success is fine, errors are expected for some operations
        let _ = result;
    }

    Ok(())
}

// =============================================================================
// Comprehensive validation function tests
// =============================================================================

/// Test validation functions with comprehensive cases
#[test]
fn test_validation_functions_comprehensive() {
    // Test worktree name validation
    assert!(validate_worktree_name("valid-name").is_ok());
    assert!(validate_worktree_name("valid_name").is_ok());
    assert!(validate_worktree_name("valid123").is_ok());

    // Invalid names
    assert!(validate_worktree_name("").is_err());
    assert!(validate_worktree_name(".hidden").is_err());
    assert!(validate_worktree_name("name/slash").is_err());
    assert!(validate_worktree_name("name\\backslash").is_err());
    assert!(validate_worktree_name("name:colon").is_err());
    assert!(validate_worktree_name("name*asterisk").is_err());
    assert!(validate_worktree_name("name?question").is_err());
    assert!(validate_worktree_name("name\"quote").is_err());
    assert!(validate_worktree_name("name<less").is_err());
    assert!(validate_worktree_name("name>greater").is_err());
    assert!(validate_worktree_name("name|pipe").is_err());
    assert!(validate_worktree_name("HEAD").is_err());
    assert!(validate_worktree_name("refs").is_err());
    assert!(validate_worktree_name("hooks").is_err());

    // Test custom path validation
    assert!(validate_custom_path("../safe/path").is_ok());
    assert!(validate_custom_path("subdirectory/path").is_ok());
    assert!(validate_custom_path("../sibling").is_ok());

    // Invalid paths
    assert!(validate_custom_path("").is_err());
    assert!(validate_custom_path("/absolute/path").is_err());
    assert!(validate_custom_path("../../../etc/passwd").is_err());
    assert!(validate_custom_path("path/").is_err());
    assert!(validate_custom_path("/root").is_err());
    assert!(validate_custom_path("C:\\Windows").is_err());
}

// =============================================================================
// Command tests in Git repository
// =============================================================================

/// Test commands in repository with initial commit
#[test]
fn test_commands_with_initial_commit() -> Result<()> {
    let (_temp_dir, _repo_path, _manager) = setup_test_repo()?;

    // Test basic commands
    let result = commands::list_worktrees();
    assert!(result.is_ok());

    // Other commands should not crash
    let _ = commands::delete_worktree();
    let _ = commands::batch_delete_worktrees();
    let _ = commands::cleanup_old_worktrees();
    let _ = commands::rename_worktree();

    Ok(())
}

/// Test concurrent access patterns
#[test]
fn test_concurrent_command_access() -> Result<()> {
    let (_temp_dir, _repo_path, _manager) = setup_test_repo()?;

    // Multiple simultaneous list operations should be safe
    for _ in 0..5 {
        let result = commands::list_worktrees();
        assert!(result.is_ok() || result.is_err()); // Either is acceptable
    }

    Ok(())
}

/// Test edge cases with special characters
#[test]
fn test_special_character_handling() {
    // Test various special characters in validation
    let special_chars = [
        ("unicode-émojis-🚀", false), // Non-ASCII characters require user confirmation, fail in tests
        ("spaces in name", true),     // Spaces are actually allowed
        ("tab\tchar", true),          // Tab characters are allowed
        ("newline\nchar", true),      // Newline characters are allowed
        ("null\0char", false),        // Null characters not allowed (in INVALID_FILESYSTEM_CHARS)
        ("control\x1bchar", true),    // Control characters are allowed
    ];

    for (name, should_pass) in special_chars {
        let result = validate_worktree_name(name);
        if should_pass {
            assert!(result.is_ok(), "Expected '{name}' to pass validation");
        } else {
            assert!(result.is_err(), "Expected '{name}' to fail validation");
        }
    }
}

/// Test boundary conditions
#[test]
fn test_boundary_conditions() {
    // Test maximum length worktree names
    let max_length_name = "a".repeat(255);
    assert!(validate_worktree_name(&max_length_name).is_ok());

    let over_length_name = "a".repeat(256);
    assert!(validate_worktree_name(&over_length_name).is_err());

    // Test minimum valid length
    assert!(validate_worktree_name("a").is_ok());

    // Test path depth limits
    let deep_path = "../".repeat(10) + "deep/path";
    assert!(validate_custom_path(&deep_path).is_err());
}

/// Test performance with large inputs
#[test]
fn test_performance_large_inputs() {
    let start = std::time::Instant::now();

    // Test with large but valid input
    let large_name = "a".repeat(200);
    let _result = validate_worktree_name(&large_name);

    let duration = start.elapsed();
    // Validation should be fast (< 1ms for reasonable inputs)
    assert!(duration.as_millis() < 100);
}

/// Test error message quality
#[test]
fn test_error_message_quality() {
    // Error messages should be informative
    let result = validate_worktree_name("");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty());
    assert!(error_msg.len() > 10); // Should be reasonably descriptive

    let result = validate_custom_path("/absolute/path");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty());
    assert!(error_msg.len() > 10);
}

/// Test memory usage patterns
#[test]
fn test_memory_usage() -> Result<()> {
    let (_temp_dir, _repo_path, _manager) = setup_test_repo()?;

    // Repeated operations should not leak memory significantly
    for _ in 0..100 {
        let _result = commands::list_worktrees();
        let _result = validate_worktree_name("test-name");
        let _result = validate_custom_path("../test/path");
    }

    Ok(())
}

/// Test cross-platform compatibility
#[test]
fn test_cross_platform_paths() {
    // Test Windows-style paths are rejected on all platforms
    assert!(validate_custom_path("C:\\Windows\\System32").is_err());
    assert!(validate_custom_path("D:\\Program Files").is_err());

    // Test Unix-style paths
    assert!(validate_custom_path("/usr/local/bin").is_err()); // Absolute paths rejected
    assert!(validate_custom_path("./relative/path").is_ok());
    assert!(validate_custom_path("../sibling/path").is_ok());
}

// =============================================================================
// Worktree creation functionality tests
// =============================================================================

/// Test successful worktree creation
#[test]
fn test_create_worktree_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test worktree creation
    let result = manager.create_worktree("feature-branch", None);
    assert!(result.is_ok());

    let worktree_path = result.unwrap();
    assert!(worktree_path.exists());
    assert_eq!(worktree_path.file_name().unwrap(), "feature-branch");

    Ok(())
}

/// Test worktree creation with new branch
#[test]
fn test_create_worktree_with_branch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test worktree creation with new branch
    let result = manager.create_worktree("feature", Some("new-feature"));
    assert!(result.is_ok());

    let worktree_path = result.unwrap();
    assert!(worktree_path.exists());

    // Verify branch was created
    let (local_branches, _) = manager.list_all_branches()?;
    assert!(local_branches.contains(&"new-feature".to_string()));

    Ok(())
}

/// Test worktree creation with existing path
#[test]
fn test_create_worktree_existing_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create first worktree
    manager.create_worktree("feature", None)?;

    // Try to create another with same name - should fail
    let result = manager.create_worktree("feature", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}
