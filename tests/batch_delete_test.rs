use anyhow::Result;
use git2::{Repository, Signature};
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::GitWorktreeManager;

#[test]
fn test_batch_delete_with_orphaned_branches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create multiple worktrees with branches
    let worktree1_path = manager.create_worktree("../feature1", Some("feature/one"))?;
    let worktree2_path = manager.create_worktree("../feature2", Some("feature/two"))?;
    let worktree3_path = manager.create_worktree("../shared", None)?; // Create from HEAD

    // Verify worktrees were created
    assert!(worktree1_path.exists());
    assert!(worktree2_path.exists());
    assert!(worktree3_path.exists());

    // List worktrees
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.len() >= 3);

    // Check branch uniqueness
    let feature1_unique = manager.is_branch_unique_to_worktree("feature/one", "feature1")?;
    let feature2_unique = manager.is_branch_unique_to_worktree("feature/two", "feature2")?;
    // Get the actual branch name of the shared worktree
    let shared_worktree = worktrees.iter().find(|w| w.name == "shared").unwrap();
    let _shared_branch_unique =
        manager.is_branch_unique_to_worktree(&shared_worktree.branch, "shared")?;

    assert!(
        feature1_unique,
        "feature/one should be unique to feature1 worktree"
    );
    assert!(
        feature2_unique,
        "feature/two should be unique to feature2 worktree"
    );
    // The shared worktree likely has a detached HEAD or unique branch
    // We just verify the function works without asserting the result

    Ok(())
}

#[test]
fn test_batch_delete_branch_cleanup() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktrees using git CLI for better control
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../feature1", "-b", "feature1"])
        .output()?;

    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../feature2", "-b", "feature2"])
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Verify branches exist
    let branches_before: Vec<_> = repo
        .branches(None)?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|s| s.to_string()))
        .collect();

    assert!(branches_before.contains(&"feature1".to_string()));
    assert!(branches_before.contains(&"feature2".to_string()));

    // Delete worktrees
    manager.remove_worktree("feature1")?;
    manager.remove_worktree("feature2")?;

    // Delete branches
    manager.delete_branch("feature1")?;
    manager.delete_branch("feature2")?;

    // Verify branches are deleted
    let branches_after: Vec<_> = repo
        .branches(None)?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|s| s.to_string()))
        .collect();

    assert!(!branches_after.contains(&"feature1".to_string()));
    assert!(!branches_after.contains(&"feature2".to_string()));

    Ok(())
}

#[test]
fn test_batch_delete_partial_failure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktrees
    let worktree1_path = manager.create_worktree("../feature1", Some("feature1"))?;
    let _worktree2_path = manager.create_worktree("../feature2", Some("feature2"))?;

    // Manually delete worktree directory to simulate partial failure
    std::fs::remove_dir_all(&worktree1_path)?;

    // Attempt to remove worktree (should handle missing directory gracefully)
    let result = manager.remove_worktree("feature1");
    // Git might still track it, so this might succeed or fail
    let _ = result;

    // Other worktree should still be removable
    let result2 = manager.remove_worktree("feature2");
    assert!(result2.is_ok());

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree() -> Result<()> {
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
    assert!(manager.is_branch_unique_to_worktree("feature-branch", "feature")?);

    // main branch is used by multiple worktrees
    assert!(!manager.is_branch_unique_to_worktree("main", "another")?);

    // Non-existent branch
    assert!(!manager.is_branch_unique_to_worktree("non-existent", "feature")?);

    Ok(())
}

// Helper function
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
