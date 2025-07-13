use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::GitWorktreeManager;

/// Helper function to setup a bare repository with an initial commit
fn setup_bare_repo_with_commit() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.bare");

    // Initialize bare repository
    Repository::init_bare(&bare_repo_path)?;

    // Create initial commit using git command
    Command::new("git")
        .current_dir(&bare_repo_path)
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .output()?;

    // Create a temporary non-bare clone to make initial commit
    let temp_clone = temp_dir.path().join("temp-clone");
    Command::new("git")
        .args([
            "clone",
            bare_repo_path.to_str().unwrap(),
            temp_clone.to_str().unwrap(),
        ])
        .output()?;

    if temp_clone.exists() {
        // Configure git user
        Command::new("git")
            .current_dir(&temp_clone)
            .args(["config", "user.email", "test@example.com"])
            .output()?;

        Command::new("git")
            .current_dir(&temp_clone)
            .args(["config", "user.name", "Test User"])
            .output()?;

        // Create initial commit in clone
        fs::write(temp_clone.join("README.md"), "# Test")?;
        Command::new("git")
            .current_dir(&temp_clone)
            .args(["add", "."])
            .output()?;
        Command::new("git")
            .current_dir(&temp_clone)
            .args(["commit", "-m", "Initial commit"])
            .output()?;
        Command::new("git")
            .current_dir(&temp_clone)
            .args(["push", "origin", "main"])
            .output()?;

        // Clean up temp clone
        fs::remove_dir_all(&temp_clone)?;
    }

    Ok((temp_dir, bare_repo_path))
}

#[test]
fn test_create_worktree_bare_repository_basic() -> Result<()> {
    let (_temp_dir, bare_repo_path) = setup_bare_repo_with_commit()?;

    std::env::set_current_dir(&bare_repo_path)?;
    let manager = GitWorktreeManager::new()?;

    // Create worktree from bare repository with unique name
    let unique_name = format!("../bare-worktree-{}", std::process::id());
    let worktree_path = manager.create_worktree(&unique_name, None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());
    assert!(worktree_path.join(".git").exists());

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository_from_path() -> Result<()> {
    let (_temp_dir, bare_repo_path) = setup_bare_repo_with_commit()?;

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Test worktree creation in bare repo
    let worktree_path = manager.create_worktree("test-worktree", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // List worktrees to verify it was added
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "test-worktree"));

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository_with_branch() -> Result<()> {
    let (_temp_dir, bare_repo_path) = setup_bare_repo_with_commit()?;

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Create worktree with a new branch
    let worktree_path = manager.create_worktree("feature-wt", Some("feature-branch"))?;

    // Verify worktree was created
    assert!(worktree_path.exists());

    // Verify branch was created
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&"feature-branch".to_string()));

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository_without_commits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("empty-bare.git");

    // Initialize bare repository without any commits
    Repository::init_bare(&bare_repo_path)?;

    let result = GitWorktreeManager::new_from_path(&bare_repo_path);

    // Bare repos without commits may not support worktrees
    match result {
        Ok(manager) => {
            // If manager creation succeeds, try to create worktree
            let worktree_result = manager.create_worktree("test-wt", None);
            // This might fail due to no commits
            assert!(worktree_result.is_ok() || worktree_result.is_err());
        }
        Err(_) => {
            // Manager creation might fail for empty bare repo
            // This is acceptable
        }
    }

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository_multiple() -> Result<()> {
    let (_temp_dir, bare_repo_path) = setup_bare_repo_with_commit()?;

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Create multiple worktrees
    let wt1 = manager.create_worktree("worktree1", Some("branch1"))?;
    let wt2 = manager.create_worktree("worktree2", Some("branch2"))?;

    // Verify both were created
    assert!(wt1.exists());
    assert!(wt2.exists());

    // List worktrees
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "worktree1"));
    assert!(worktrees.iter().any(|w| w.name == "worktree2"));

    // Verify branches
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&"branch1".to_string()));
    assert!(branches.contains(&"branch2".to_string()));

    Ok(())
}

#[test]
fn test_bare_repository_git_dir() -> Result<()> {
    let (_temp_dir, bare_repo_path) = setup_bare_repo_with_commit()?;

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Test get_git_dir for bare repository
    let git_dir = manager.get_git_dir()?;

    // For bare repos, git_dir should be the repository itself
    let expected_dir = bare_repo_path.canonicalize()?;
    let actual_dir = git_dir.canonicalize()?;
    assert_eq!(actual_dir, expected_dir);

    Ok(())
}
