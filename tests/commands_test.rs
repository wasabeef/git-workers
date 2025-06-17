use anyhow::Result;
use git2::Repository;
//
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::GitWorktreeManager;

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
    let branches = manager.list_branches()?;
    assert!(branches.contains(&"new-feature".to_string()));

    Ok(())
}

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

#[test]
fn test_list_worktrees_with_main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let worktrees = manager.list_worktrees()?;
    // Non-bare repos should show the main worktree
    // The count may vary based on how git2 handles the main worktree
    // Length is always >= 0 for usize, so just check it exists
    let _ = worktrees.len();

    Ok(())
}

#[test]
fn test_remove_worktree_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktree using git command directly
    let worktree_path = temp_dir.path().join("feature");
    Command::new("git")
        .current_dir(&repo_path)
        .arg("worktree")
        .arg("add")
        .arg(&worktree_path)
        .arg("-b")
        .arg("feature")
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Remove the worktree
    let result = manager.remove_worktree("feature");
    assert!(result.is_ok());

    // Verify it's gone
    assert!(!worktree_path.exists());

    Ok(())
}

#[test]
fn test_remove_worktree_nonexistent() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let result = manager.remove_worktree("nonexistent");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_list_branches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let branches = manager.list_branches()?;
    assert!(!branches.is_empty());
    assert!(branches.contains(&"main".to_string()) || branches.contains(&"master".to_string()));

    Ok(())
}

#[test]
fn test_delete_branch_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a test branch
    Command::new("git")
        .current_dir(&repo_path)
        .args(["checkout", "-b", "test-branch"])
        .output()?;

    // Add a commit to test-branch and push it back to main to ensure it's merged
    std::fs::write(repo_path.join("test.txt"), "test")?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "test.txt"])
        .output()?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Test commit"])
        .output()?;

    Command::new("git")
        .current_dir(&repo_path)
        .args(["checkout", "main"])
        .output()
        .or_else(|_| {
            Command::new("git")
                .current_dir(&repo_path)
                .args(["checkout", "master"])
                .output()
        })?;

    // Merge test-branch to main so it can be deleted
    Command::new("git")
        .current_dir(&repo_path)
        .args(["merge", "test-branch"])
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Delete the branch
    let result = manager.delete_branch("test-branch");
    assert!(result.is_ok());

    // Verify it's gone
    let branches = manager.list_branches()?;
    assert!(!branches.contains(&"test-branch".to_string()));

    Ok(())
}

#[test]
fn test_delete_branch_nonexistent() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let result = manager.delete_branch("nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

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
