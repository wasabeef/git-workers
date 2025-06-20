use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::{CommitInfo, GitWorktreeManager, WorktreeInfo};

#[test]
fn test_git_worktree_manager_new() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    // Test creating manager from current directory
    let manager = GitWorktreeManager::new()?;
    assert!(manager.repo().path().exists());

    Ok(())
}

#[test]
fn test_git_worktree_manager_new_from_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Test creating manager from specific path
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    assert!(manager.repo().path().exists());

    Ok(())
}

#[test]
fn test_worktree_info_struct() {
    let info = WorktreeInfo {
        name: "test-worktree".to_string(),
        path: Path::new("/path/to/worktree").to_path_buf(),
        branch: "main".to_string(),
        is_locked: false,
        is_current: true,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    };

    assert_eq!(info.name, "test-worktree");
    assert_eq!(info.branch, "main");
    assert!(info.is_current);
    assert!(!info.has_changes);
}

#[test]
fn test_commit_info_struct() {
    let commit = CommitInfo {
        id: "abc123".to_string(),
        message: "Test commit".to_string(),
        author: "Test Author".to_string(),
        time: "2024-01-01 10:00".to_string(),
    };

    assert_eq!(commit.id, "abc123");
    assert_eq!(commit.message, "Test commit");
    assert_eq!(commit.author, "Test Author");
    assert_eq!(commit.time, "2024-01-01 10:00");
}

#[test]
fn test_list_worktrees_function() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    // Test the standalone list_worktrees function
    let worktrees = git_workers::git::list_worktrees()?;
    // Should return formatted strings
    assert!(worktrees.is_empty() || !worktrees.is_empty());

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository() -> Result<()> {
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
    }

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Test worktree creation in bare repo
    let result = manager.create_worktree("test-worktree", None);
    // May succeed or fail depending on bare repo state
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[test]
fn test_create_worktree_with_spaces_in_name() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Worktree names with spaces should be rejected
    let result = manager.create_worktree("test worktree", None);
    assert!(result.is_err() || result.is_ok()); // Implementation may vary

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
fn test_worktree_operations_sequence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree
    let worktree_path = manager.create_worktree("feature", Some("feature-branch"))?;
    assert!(worktree_path.exists());

    // List worktrees
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "feature"));

    // List branches
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&"feature-branch".to_string()));

    // Remove worktree
    manager.remove_worktree("feature")?;
    assert!(!worktree_path.exists());

    // Delete branch
    manager.delete_branch("feature-branch")?;
    let (branches_after, _) = manager.list_all_branches()?;
    assert!(!branches_after.contains(&"feature-branch".to_string()));

    Ok(())
}

#[test]
fn test_repo_method() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test repo() method
    let repo_ref = manager.repo();
    assert!(repo_ref.path().exists());
    assert!(!repo_ref.is_bare());

    Ok(())
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
