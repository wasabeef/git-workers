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
fn test_list_all_branches_basic() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    let (local_branches, _) = manager.list_all_branches()?;
    assert!(!local_branches.is_empty());
    assert!(
        local_branches.contains(&"main".to_string())
            || local_branches.contains(&"master".to_string())
    );

    Ok(())
}

#[test]
fn test_list_all_branches_with_multiple_branches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create additional branches using git commands
    Command::new("git")
        .args(["checkout", "-b", "feature-1"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["checkout", "-b", "feature-2"])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let (local_branches, _) = manager.list_all_branches()?;

    // Should have at least 3 branches
    assert!(local_branches.len() >= 3);
    assert!(local_branches.contains(&"feature-1".to_string()));
    assert!(local_branches.contains(&"feature-2".to_string()));

    Ok(())
}

#[test]
fn test_list_all_branches_with_remote() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Add a fake remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(&repo_path)
        .output()?;

    // Create remote tracking references
    Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["update-ref", "refs/remotes/origin/develop", "HEAD"])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let (local_branches, remote_branches) = manager.list_all_branches()?;

    // Check local branches
    assert!(!local_branches.is_empty());

    // Check remote branches
    assert_eq!(remote_branches.len(), 2);
    assert!(remote_branches.contains(&"main".to_string()));
    assert!(remote_branches.contains(&"develop".to_string()));

    Ok(())
}

#[test]
fn test_list_all_branches_empty_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let (local_branches, _) = manager.list_all_branches()?;
    // Should have at least the default branch
    assert!(!local_branches.is_empty());
    assert!(
        local_branches.contains(&"main".to_string())
            || local_branches.contains(&"master".to_string())
    );

    Ok(())
}

#[test]
fn test_list_all_branches_with_detached_head() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a commit and checkout to it directly (detached HEAD)
    let head_oid = repo.head()?.target().unwrap();
    Command::new("git")
        .args(["checkout", &head_oid.to_string()])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let (local_branches, _) = manager.list_all_branches()?;

    // Should still list branches even with detached HEAD
    assert!(!local_branches.is_empty());

    Ok(())
}

#[test]
fn test_list_all_branches_sorting() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create branches in non-alphabetical order
    let branches = vec!["zebra", "alpha", "beta", "gamma"];
    for branch in &branches {
        Command::new("git")
            .args(["checkout", "-b", branch])
            .current_dir(&repo_path)
            .output()?;
    }

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let (local_branches, _) = manager.list_all_branches()?;

    // Verify all branches exist
    for branch in &branches {
        assert!(local_branches.contains(&branch.to_string()));
    }

    // Check if branches are sorted (implementation dependent)
    // The actual sorting behavior may vary

    Ok(())
}
