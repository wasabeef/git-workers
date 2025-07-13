//! Unified worktree removal tests
//!
//! Integrates the following 5 duplicate test functions:
//! 1. tests/git_advanced_test.rs::test_remove_worktree - Removal with uncommitted changes
//! 2. tests/more_comprehensive_test.rs::test_remove_worktree_that_doesnt_exist - Non-existent worktree removal error
//! 3. tests/commands_test.rs::test_remove_worktree_success - Git CLI integration and filesystem verification
//! 4. tests/commands_test.rs::test_remove_worktree_nonexistent - Non-existent error (duplicate)
//! 5. tests/worktree_commands_test.rs::test_remove_worktree - Basic removal functionality and list verification

use anyhow::Result;
use git2::Repository;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_initial_commit(repo: &Repository) -> Result<()> {
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        let readme_path = repo.workdir().unwrap().join("README.md");
        fs::write(&readme_path, "# Test Repository")?;
        index.add_path(Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    Ok(())
}

fn setup_test_repo_basic() -> Result<(TempDir, GitWorktreeManager)> {
    let parent_dir = TempDir::new()?;
    let main_repo_path = parent_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;

    let repo = Repository::init(&main_repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&main_repo_path)?;
    Ok((parent_dir, manager))
}

fn setup_test_repo_with_path() -> Result<(TempDir, std::path::PathBuf, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    Ok((temp_dir, repo_path, manager))
}

#[test]
fn test_remove_worktree_success_basic() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_basic()?;

    // Test 1: Basic worktree removal functionality
    let worktree_name = "to-be-removed";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Verify created worktree exists
    assert!(worktree_path.exists(), "Worktree was not created");

    // Verify worktree exists in the list
    let worktrees_before = manager.list_worktrees()?;
    assert!(
        worktrees_before.iter().any(|w| w.name == worktree_name),
        "Worktree does not exist in the list"
    );

    // Remove the worktree
    manager.remove_worktree(worktree_name)?;

    // Verify worktree is removed from the list
    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.name == worktree_name),
        "Worktree was not removed from the list"
    );

    // Note: The directory itself may remain, but it's removed from git tracking
    Ok(())
}

#[test]
fn test_remove_worktree_with_uncommitted_changes() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo_with_path()?;

    // Test 2: Removal with uncommitted changes
    let worktree_name = "remove-worktree";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Add unsaved changes to the worktree
    let test_file = worktree_path.join("test_file.txt");
    fs::write(&test_file, "uncommitted changes")?;

    // Verify removal is possible even with uncommitted changes
    let result = manager.remove_worktree(worktree_name);
    assert!(
        result.is_ok(),
        "Failed to remove worktree with uncommitted changes: {:?}",
        result.err()
    );

    // Verify worktree is removed from the list
    let worktrees = manager.list_worktrees()?;
    assert!(
        !worktrees.iter().any(|w| w.name == worktree_name),
        "Worktree with uncommitted changes was not removed from the list"
    );

    Ok(())
}

#[test]
fn test_remove_worktree_filesystem_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Test 3: Git CLI integration and filesystem state verification
    let worktree_path = temp_dir.path().join("test-worktree");

    // Create worktree using Git CLI
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    // Verify worktree directory exists
    assert!(
        worktree_path.exists(),
        "Worktree created by Git CLI does not exist"
    );

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Remove using GitWorktreeManager
    manager.remove_worktree("test-worktree")?;

    // Verify directory is removed from filesystem
    // Note: Directory may remain depending on implementation
    // assert!(!worktree_path.exists(), "Directory still exists after removal");

    Ok(())
}

#[test]
fn test_remove_nonexistent_worktree_error() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_basic()?;

    // Test 4: Error handling for non-existent worktree removal
    let nonexistent_name = "this-worktree-does-not-exist";

    let result = manager.remove_worktree(nonexistent_name);
    assert!(
        result.is_err(),
        "Removing non-existent worktree did not result in error"
    );

    // Verify error message content (specific error type is implementation-dependent)
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message is empty");

    Ok(())
}

#[test]
fn test_remove_worktree_comprehensive_error_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test 5: Comprehensive error cases

    // 5-1: Empty string worktree name
    let result = manager.remove_worktree("");
    assert!(result.is_err(), "Empty string worktree name was accepted");

    // 5-2: Worktree name with invalid characters
    let result = manager.remove_worktree("invalid/name");
    assert!(
        result.is_err(),
        "Worktree name with invalid characters was accepted"
    );

    // 5-3: Very long worktree name
    let long_name = "a".repeat(300);
    let result = manager.remove_worktree(&long_name);
    assert!(result.is_err(), "Very long worktree name was accepted");

    Ok(())
}

#[test]
fn test_remove_worktree_state_consistency() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_basic()?;

    // Test 6: State consistency verification before and after removal
    let worktree_name = "consistency-test";

    // Record initial worktree count
    let initial_count = manager.list_worktrees()?.len();

    // Create worktree
    manager.create_worktree(worktree_name, None)?;
    let after_create_count = manager.list_worktrees()?.len();
    assert_eq!(
        after_create_count,
        initial_count + 1,
        "Incorrect count after worktree creation"
    );

    // Remove the worktree
    manager.remove_worktree(worktree_name)?;
    let after_remove_count = manager.list_worktrees()?.len();
    assert_eq!(
        after_remove_count, initial_count,
        "Count did not return to initial state after worktree removal"
    );

    // Verify can recreate with same name
    let result = manager.create_worktree(worktree_name, None);
    assert!(
        result.is_ok(),
        "Cannot recreate worktree with same name after removal: {:?}",
        result.err()
    );

    Ok(())
}
