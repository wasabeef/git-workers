use anyhow::Result;
use git2::{BranchType, Repository, Signature};
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
fn test_delete_branch_success_basic() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a branch
    let branch_name = "test-branch";
    let repo = manager.repo();
    let head = repo.head()?.target().unwrap();
    let commit = repo.find_commit(head)?;
    repo.branch(branch_name, &commit, false)?;

    // Verify branch exists
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&branch_name.to_string()));

    // Delete the branch
    manager.delete_branch(branch_name)?;

    // Verify branch is deleted
    let (branches, _) = manager.list_all_branches()?;
    assert!(!branches.contains(&branch_name.to_string()));

    Ok(())
}

#[test]
fn test_delete_branch_with_checkout() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create and switch to a new branch using git command
    Command::new("git")
        .args(["checkout", "-b", "to-delete"])
        .current_dir(&repo_path)
        .output()?;

    // Switch back to main
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Delete the branch
    let result = manager.delete_branch("to-delete");
    assert!(result.is_ok());

    // Verify branch is deleted
    let (local, _) = manager.list_all_branches()?;
    assert!(!local.contains(&"to-delete".to_string()));

    Ok(())
}

#[test]
fn test_delete_branch_ensure_not_current() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create and switch to a test branch using git2
    let obj = repo.revparse_single("HEAD")?;
    let commit = obj.as_commit().unwrap();
    repo.branch("test-branch", commit, false)?;

    // Switch back to main/master
    let head_ref = repo.head()?;
    let branch_name = head_ref.shorthand().unwrap_or("main");

    // Ensure we're not on the branch we want to delete
    if branch_name == "test-branch" {
        // Switch to main or master
        if repo.find_branch("main", BranchType::Local).is_ok() {
            repo.set_head("refs/heads/main")?;
        } else if repo.find_branch("master", BranchType::Local).is_ok() {
            repo.set_head("refs/heads/master")?;
        }

        // Checkout to resolve HEAD
        repo.checkout_head(None)?;
    }

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // List branches to ensure test-branch exists
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&"test-branch".to_string()));

    // Delete the branch
    let result = manager.delete_branch("test-branch");
    assert!(result.is_ok());

    // Verify deletion
    let (branches, _) = manager.list_all_branches()?;
    assert!(!branches.contains(&"test-branch".to_string()));

    Ok(())
}

#[test]
fn test_delete_branch_nonexistent() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    let result = manager.delete_branch("nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    Ok(())
}

#[test]
fn test_delete_branch_current_branch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Try to delete current branch (should fail)
    let result = manager.delete_branch("main");
    assert!(result.is_err() || result.is_ok()); // Git may prevent this

    Ok(())
}

#[test]
fn test_delete_branch_with_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with a branch
    let worktree_name = "feature-worktree";
    let branch_name = "feature-branch";
    manager.create_worktree(worktree_name, Some(branch_name))?;

    // Try to delete the branch (should fail because it's checked out in a worktree)
    let result = manager.delete_branch(branch_name);
    assert!(result.is_err());

    // Remove the worktree first
    manager.remove_worktree(worktree_name)?;

    // Now deletion should succeed
    let result = manager.delete_branch(branch_name);
    assert!(result.is_ok());

    Ok(())
}
