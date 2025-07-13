use anyhow::Result;
use git2::{Repository, Signature};
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::GitWorktreeManager;

/// Helper function to create initial commit
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    Ok(())
}

/// Helper function to setup test repository
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    Ok((temp_dir, manager))
}

#[test]
fn test_is_branch_unique_to_worktree_basic() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create worktree with a unique branch
    manager.create_worktree_with_new_branch("unique-wt", "unique-branch", "main")?;

    // Check if branch is unique to worktree
    let result = manager.is_branch_unique_to_worktree("unique-branch", "unique-wt")?;
    assert!(result, "Branch should be unique to its worktree");

    // Check with different worktree name
    let result = manager.is_branch_unique_to_worktree("unique-branch", "other-wt")?;
    assert!(
        !result,
        "Branch should not be unique to a different worktree"
    );

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree_with_git_command() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktrees with git CLI
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../feature", "-b", "feature-branch"])
        .output()?;

    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../another", "main"])
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // feature-branch should be unique to feature worktree
    let unique = manager.is_branch_unique_to_worktree("feature-branch", "feature")?;
    assert!(
        unique,
        "feature-branch should be unique to feature worktree"
    );

    // main branch is not unique (used by another worktree)
    let unique = manager.is_branch_unique_to_worktree("main", "test-repo")?;
    assert!(
        !unique,
        "main branch is not unique as it's used by another worktree"
    );

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree_shared_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create two worktrees using the same branch (main)
    manager.create_worktree("worktree1", None)?;
    manager.create_worktree("worktree2", None)?;

    // main branch is not unique to any single worktree
    let unique = manager.is_branch_unique_to_worktree("main", "worktree1")?;
    assert!(
        !unique,
        "main branch should not be unique when used by multiple worktrees"
    );

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree_nonexistent_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Check a branch that doesn't exist
    let result = manager.is_branch_unique_to_worktree("nonexistent-branch", "some-worktree");

    // Should handle gracefully (either false or error)
    // Error is also acceptable
    if let Ok(unique) = result {
        assert!(!unique, "Nonexistent branch should not be unique");
    }

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree_detached_head() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a worktree in detached HEAD state
    let head_oid = repo.head()?.target().unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../detached", &head_oid.to_string()])
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Detached HEAD worktree doesn't have a branch
    let result = manager.is_branch_unique_to_worktree("main", "detached");

    // Should handle gracefully
    // Error is also acceptable
    if let Ok(unique) = result {
        assert!(
            !unique,
            "Branch should not be unique to detached HEAD worktree"
        );
    }

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree_edge_cases() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with a branch
    manager.create_worktree_with_new_branch("test-wt", "test-branch", "main")?;

    // Test with empty strings
    let result = manager.is_branch_unique_to_worktree("", "test-wt");
    // Error is acceptable
    if let Ok(unique) = result {
        assert!(!unique, "Empty branch name should not be unique");
    }

    let result = manager.is_branch_unique_to_worktree("test-branch", "");
    // Error is acceptable
    if let Ok(unique) = result {
        assert!(!unique, "Empty worktree name should not match");
    }

    Ok(())
}
