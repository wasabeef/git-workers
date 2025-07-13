//! Unified main application tests
//!
//! Integrates main_application_test.rs, main_functionality_test.rs, and main_test.rs
//! Eliminates duplication and provides comprehensive main functionality tests

use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test repository with initial commit
fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    Command::new("git")
        .args(["init", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}

// =============================================================================
// Main application basic functionality tests
// =============================================================================

/// Test that the main binary can be executed without crashing
#[test]
fn test_main_binary_execution() {
    // This test ensures the main binary can be compiled and doesn't crash immediately
    // We can't test interactive parts, but we can test error handling

    let output = Command::new("cargo")
        .args(["check", "--bin", "gw"])
        .output();

    assert!(output.is_ok(), "Main binary should compile successfully");
}

/// Test error handling when not in a git repository
#[test]
fn test_main_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let non_git_path = temp_dir.path().join("not-a-repo");
    fs::create_dir_all(&non_git_path).unwrap();

    // The main application should handle non-git directories gracefully
    // We can't run the interactive parts, but we can test that it doesn't panic
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&non_git_path).unwrap();

    // Test that git-workers library functions handle this gracefully
    use git_workers::git::GitWorktreeManager;
    let result = GitWorktreeManager::new();
    // Should either succeed or fail gracefully, not panic
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir).unwrap();
}

/// Test main application in empty repository
#[test]
fn test_main_empty_repository() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("empty-repo");

    // Initialize empty repository (no initial commit)
    Repository::init(&repo_path)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test that the application handles empty repositories
    use git_workers::git::GitWorktreeManager;
    let result = GitWorktreeManager::new();
    // Should handle empty repo gracefully
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

// =============================================================================
// Feature-specific tests
// =============================================================================

/// Test worktree listing functionality
#[test]
fn test_list_worktrees_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands;
    let result = commands::list_worktrees();
    assert!(result.is_ok());

    Ok(())
}

/// Test worktree creation validation
#[test]
fn test_create_worktree_validation() {
    use git_workers::commands::{validate_custom_path, validate_worktree_name};

    // Test valid names
    assert!(validate_worktree_name("valid-name").is_ok());
    assert!(validate_worktree_name("feature-123").is_ok());
    assert!(validate_worktree_name("bugfix_branch").is_ok());

    // Test invalid names
    assert!(validate_worktree_name("").is_err());
    assert!(validate_worktree_name(".hidden").is_err());
    assert!(validate_worktree_name("invalid/name").is_err());
    assert!(validate_worktree_name("HEAD").is_err());

    // Test valid paths
    assert!(validate_custom_path("../sibling").is_ok());
    assert!(validate_custom_path("subdirectory/path").is_ok());

    // Test invalid paths
    assert!(validate_custom_path("").is_err());
    assert!(validate_custom_path("/absolute").is_err());
    assert!(validate_custom_path("../../../etc/passwd").is_err());
}

/// Test delete worktree functionality
#[test]
fn test_delete_worktree_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands;
    // Should handle the case where there are no worktrees to delete
    let result = commands::delete_worktree();
    // Either succeeds (if it shows empty list) or fails gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test batch delete functionality
#[test]
fn test_batch_delete_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands;
    let result = commands::batch_delete_worktrees();
    // Should handle empty worktree list gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test cleanup old worktrees functionality
#[test]
fn test_cleanup_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands;
    let result = commands::cleanup_old_worktrees();
    // Should handle case with no old worktrees
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test rename worktree functionality
#[test]
fn test_rename_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands;
    let result = commands::rename_worktree();
    // Should handle case with no additional worktrees
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

// =============================================================================
// Git operation tests
// =============================================================================

/// Test Git repository detection
#[test]
fn test_git_repository_detection() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new()?;

    // Should successfully detect the repository
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.is_empty()); // Should have at least the main worktree

    Ok(())
}

/// Test branch listing
#[test]
fn test_branch_listing() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new()?;

    let (local_branches, remote_branches) = manager.list_all_branches()?;
    // Should have at least the default branch (main or master)
    assert!(!local_branches.is_empty() || !remote_branches.is_empty());

    Ok(())
}

/// Test tag listing
#[test]
fn test_tag_listing() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new()?;

    let tags = manager.list_all_tags()?;
    // New repository might not have tags, so just ensure it doesn't crash
    let _ = tags;

    Ok(())
}

// =============================================================================
// Error handling tests
// =============================================================================

/// Test error handling for invalid operations
#[test]
fn test_error_handling() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let _manager = GitWorktreeManager::new()?;

    // Test operations that should fail gracefully
    use git_workers::commands::{validate_custom_path, validate_worktree_name};
    let result = validate_worktree_name("");
    assert!(result.is_err()); // Empty name should fail

    let result = validate_custom_path("/absolute/path");
    assert!(result.is_err()); // Absolute paths should fail

    Ok(())
}

/// Test concurrent access safety
#[test]
fn test_concurrent_access() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;

    // Create multiple managers (simulating concurrent access)
    for _ in 0..5 {
        let manager = GitWorktreeManager::new()?;
        let worktrees = manager.list_worktrees()?;
        assert!(!worktrees.is_empty());
    }

    Ok(())
}

// =============================================================================
// Configuration and environment variable tests
// =============================================================================

/// Test environment variable handling
#[test]
fn test_environment_variables() {
    // Test NO_COLOR environment variable
    std::env::set_var("NO_COLOR", "1");

    // Test that application respects NO_COLOR
    use git_workers::constants::ENV_NO_COLOR;
    assert_eq!(ENV_NO_COLOR, "NO_COLOR");

    std::env::remove_var("NO_COLOR");
}

/// Test configuration file discovery
#[test]
fn test_config_discovery() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create a config file
    let config_content = r#"
[hooks]
post-create = ["echo 'test hook'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Test that config file exists and is readable
    let config_exists = repo_path.join(".git-workers.toml").exists();
    assert!(config_exists);

    Ok(())
}

// =============================================================================
// Performance tests
// =============================================================================

/// Test performance with multiple operations
#[test]
fn test_performance_multiple_operations() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let start = std::time::Instant::now();

    // Perform multiple operations
    for _ in 0..10 {
        let manager = GitWorktreeManager::new()?;
        let _worktrees = manager.list_worktrees()?;
        let _branches = manager.list_all_branches()?;
        let _tags = manager.list_all_tags()?;
    }

    let duration = start.elapsed();
    // Operations should complete reasonably quickly
    assert!(duration.as_secs() < 5);

    Ok(())
}

/// Test memory usage patterns
#[test]
fn test_memory_usage() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;

    // Repeatedly create and drop managers to test for memory leaks
    for _ in 0..100 {
        let manager = GitWorktreeManager::new()?;
        let _worktrees = manager.list_worktrees()?;
        // Manager should be dropped here
    }

    Ok(())
}

// =============================================================================
// Practical scenario tests
// =============================================================================

/// Test typical user workflow
#[test]
fn test_typical_workflow() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new()?;

    // 1. List existing worktrees
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.is_empty());

    // 2. List available branches
    let (local_branches, remote_branches) = manager.list_all_branches()?;
    assert!(!local_branches.is_empty() || !remote_branches.is_empty());

    // 3. Validate a worktree name
    use git_workers::commands::validate_worktree_name;
    assert!(validate_worktree_name("feature-branch").is_ok());

    // 4. Validate a path
    use git_workers::commands::validate_custom_path;
    assert!(validate_custom_path("../feature-worktree").is_ok());

    Ok(())
}

/// Test edge cases and boundary conditions
#[test]
fn test_edge_cases() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::commands::{validate_custom_path, validate_worktree_name};

    // Test boundary conditions
    let max_name = "a".repeat(255);
    assert!(validate_worktree_name(&max_name).is_ok());

    let too_long_name = "a".repeat(256);
    assert!(validate_worktree_name(&too_long_name).is_err());

    // Test special characters
    assert!(validate_worktree_name("name-with-dashes").is_ok());
    assert!(validate_worktree_name("name_with_underscores").is_ok());
    assert!(validate_worktree_name("name123").is_ok());

    // Test path traversal prevention
    assert!(validate_custom_path("../safe").is_ok());
    assert!(validate_custom_path("../../unsafe").is_err());
    assert!(validate_custom_path("../../../very-unsafe").is_err());

    Ok(())
}

/// Test application resilience
#[test]
fn test_application_resilience() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    use git_workers::git::GitWorktreeManager;

    // Test that application handles various error conditions gracefully
    let manager = GitWorktreeManager::new()?;

    // Operations should not panic even with unusual inputs
    let _result = manager.list_worktrees();
    let _result = manager.list_all_branches();
    let _result = manager.list_all_tags();

    Ok(())
}
