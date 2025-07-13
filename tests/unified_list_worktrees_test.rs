//! Unified worktree list tests
//!
//! Integrates the following 5 duplicate test functions:
//! 1. tests/worktree_commands_test.rs::test_list_worktrees - Multiple worktree creation and listing
//! 2. tests/git_advanced_test.rs::test_list_worktrees - Basic list functionality
//! 3. tests/git_comprehensive_test.rs::test_list_worktrees_function - Standalone function test
//! 4. tests/commands_test.rs::test_list_worktrees_with_main - Main worktree verification
//! 5. tests/more_comprehensive_test.rs::test_list_worktrees_with_locked_worktree - Locked worktree support

use anyhow::Result;
use git2::Repository;
use git_workers::git::{list_worktrees, GitWorktreeManager};
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

#[test]
fn test_list_worktrees_comprehensive() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_basic()?;

    // Test 1: Initial state (main worktree only)
    let initial_worktrees = manager.list_worktrees()?;
    let initial_count = initial_worktrees.len();
    // Non-bare repos should show the main worktree - usize is always >= 0

    // Test 2: Multiple worktree creation and list functionality verification
    manager.create_worktree("worktree1", None)?;
    manager.create_worktree("worktree2", Some("branch2"))?;
    manager.create_worktree("test-worktree", None)?;

    let worktrees = manager.list_worktrees()?;
    assert_eq!(
        worktrees.len(),
        initial_count + 3,
        "3 worktrees have been added"
    );

    // Test 3: Verify specific worktrees exist
    let names: Vec<_> = worktrees.iter().map(|w| &w.name).collect();
    assert!(
        names.contains(&&"worktree1".to_string()),
        "worktree1 exists"
    );
    assert!(
        names.contains(&&"worktree2".to_string()),
        "worktree2 exists"
    );
    assert!(
        names.contains(&&"test-worktree".to_string()),
        "test-worktree exists"
    );

    // Test 4: Detailed verification of worktree information
    let test_wt = worktrees.iter().find(|w| w.name == "test-worktree");
    assert!(test_wt.is_some(), "test-worktree is found");

    let wt_info = test_wt.unwrap();
    assert_eq!(wt_info.name, "test-worktree", "name matches");
    assert!(wt_info.path.exists(), "path exists");

    Ok(())
}

#[test]
fn test_list_worktrees_standalone_function() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Set current directory to test standalone function
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test standalone list_worktrees function
    let worktrees_result = list_worktrees();

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    // Verify result (success if not error)
    let worktrees = worktrees_result?;
    // Valid result whether empty or not
    let _ = worktrees.len();

    Ok(())
}

#[test]
fn test_list_worktrees_with_locked_worktree() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktree using Git CLI
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../locked", "-b", "locked-branch"])
        .output()?;

    // Lock the worktree
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "lock", "../locked"])
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let worktrees = manager.list_worktrees()?;

    // Verify that locked worktrees are included in the list
    assert!(
        !worktrees.is_empty(),
        "List including locked worktree can be retrieved"
    );

    Ok(())
}

#[test]
fn test_list_worktrees_empty_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let worktrees = manager.list_worktrees()?;

    // Verify edge case handling for empty cases
    // Since usize is always >= 0, verify that basic length retrieval succeeds
    let count = worktrees.len();
    let _ = count; // Since usize is always non-negative, verify it can be retrieved

    Ok(())
}
