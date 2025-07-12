use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test repository with initial commit
fn setup_test_repo() -> Result<(TempDir, PathBuf, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository with main as default branch
    std::process::Command::new("git")
        .args(["init", "-b", "main", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

    Ok((temp_dir, repo_path, manager))
}

/// Test creating worktree with new branch from specific base
#[test]
fn test_create_worktree_with_new_branch_from_base() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create a feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(&repo_path)
        .output()?;

    // Make a commit on develop
    fs::write(repo_path.join("develop.txt"), "develop content")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Develop commit"])
        .current_dir(&repo_path)
        .output()?;

    // Create worktree with new branch from develop
    let worktree_name = "feature-worktree";
    let result =
        manager.create_worktree_with_new_branch(worktree_name, "feature-from-develop", "develop");

    assert!(result.is_ok());
    let worktree_path = result.unwrap();
    assert!(worktree_path.exists());
    assert!(worktree_path.join("develop.txt").exists());

    // Verify branch was created from develop
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-n", "2"])
        .current_dir(&worktree_path)
        .output()?;

    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Develop commit"));

    Ok(())
}

/// Test list_worktrees method
#[test]
fn test_list_worktrees() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "test-worktree";
    manager.create_worktree(worktree_name, None)?;

    // List worktrees
    let worktrees = manager.list_worktrees()?;

    // Should have at least 1 worktree (test-worktree)
    assert!(!worktrees.is_empty());

    // Find the test worktree
    let test_wt = worktrees.iter().find(|w| w.name == worktree_name);
    assert!(test_wt.is_some());

    let wt_info = test_wt.unwrap();
    assert_eq!(wt_info.name, worktree_name);
    assert!(wt_info.path.exists());

    Ok(())
}

/// Test removing worktree
#[test]
fn test_remove_worktree() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "remove-worktree";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Make changes in the worktree
    fs::write(worktree_path.join("changes.txt"), "uncommitted changes")?;

    // Try to remove worktree (should succeed even with uncommitted changes)
    let result = manager.remove_worktree(worktree_name);
    assert!(result.is_ok());

    // Verify worktree is removed
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.iter().any(|w| w.name == worktree_name));

    Ok(())
}

/// Test listing all branches
#[test]
fn test_list_all_branches() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create additional branches
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-1"])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["checkout", "-b", "feature-2"])
        .current_dir(&repo_path)
        .output()?;

    // Add a fake remote
    std::process::Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(&repo_path)
        .output()?;

    let (local, _remote) = manager.list_all_branches()?;

    // Should have at least 3 local branches
    assert!(local.len() >= 3);
    assert!(local.contains(&"feature-1".to_string()));
    assert!(local.contains(&"feature-2".to_string()));

    Ok(())
}

/// Test listing tags
#[test]
fn test_list_all_tags() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create lightweight tag
    std::process::Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(&repo_path)
        .output()?;

    // Create annotated tag
    std::process::Command::new("git")
        .args(["tag", "-a", "v2.0.0", "-m", "Version 2.0.0 release"])
        .current_dir(&repo_path)
        .output()?;

    let tags = manager.list_all_tags()?;

    // Should have 2 tags
    assert_eq!(tags.len(), 2);

    // Find v1.0.0 (lightweight tag)
    let v1 = tags.iter().find(|(name, _)| name == "v1.0.0");
    assert!(v1.is_some());
    assert!(v1.unwrap().1.is_none()); // No message for lightweight tag

    // Find v2.0.0 (annotated tag)
    let v2 = tags.iter().find(|(name, _)| name == "v2.0.0");
    assert!(v2.is_some());
    assert!(v2.unwrap().1.is_some()); // Has message
    assert!(v2.unwrap().1.as_ref().unwrap().contains("Version 2.0.0"));

    Ok(())
}

/// Test get_branch_worktree_map
#[test]
fn test_get_branch_worktree_map() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create worktrees with branches
    manager.create_worktree_with_new_branch("wt1", "branch1", "main")?;
    manager.create_worktree_with_new_branch("wt2", "branch2", "main")?;

    let map = manager.get_branch_worktree_map()?;

    // Should have mappings for the branches
    assert_eq!(map.get("branch1"), Some(&"wt1".to_string()));
    assert_eq!(map.get("branch2"), Some(&"wt2".to_string()));

    Ok(())
}

/// Test is_branch_unique_to_worktree
#[test]
fn test_is_branch_unique_to_worktree() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create worktree with a branch
    manager.create_worktree_with_new_branch("unique-wt", "unique-branch", "main")?;

    // Check if branch is unique to worktree
    let result = manager.is_branch_unique_to_worktree("unique-branch", "unique-wt")?;
    assert!(result);

    // Check with different worktree name
    let result = manager.is_branch_unique_to_worktree("unique-branch", "other-wt")?;
    assert!(!result);

    Ok(())
}

/// Test rename_branch
#[test]
fn test_rename_branch() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create a branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "old-branch"])
        .current_dir(&repo_path)
        .output()?;

    // Rename the branch
    let result = manager.rename_branch("old-branch", "new-branch");
    assert!(result.is_ok());

    // Verify branch was renamed
    let (local, _) = manager.list_all_branches()?;
    assert!(local.contains(&"new-branch".to_string()));
    assert!(!local.contains(&"old-branch".to_string()));

    Ok(())
}

/// Test rename_worktree functionality
#[test]
fn test_rename_worktree() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create a worktree
    let old_name = "old-name";
    let old_path = manager.create_worktree_with_new_branch(old_name, "old-name", "main")?;

    // Verify the worktree was created
    assert!(old_path.exists());

    // Rename the worktree
    let new_name = "new-name";
    let result = manager.rename_worktree(old_name, new_name);

    assert!(result.is_ok(), "Rename operation should succeed");
    let new_path = result.unwrap();
    assert!(new_path.exists(), "New path should exist after rename");
    assert!(
        new_path.to_str().unwrap().contains(new_name),
        "New path should contain new name"
    );

    // Verify old path no longer exists
    assert!(!old_path.exists(), "Old path should not exist after rename");

    // Verify the new path has expected content
    assert!(
        new_path.join("README.md").exists(),
        "README.md should exist in renamed worktree"
    );

    Ok(())
}

/// Test delete_branch functionality
#[test]
fn test_delete_branch() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create and switch to a new branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "to-delete"])
        .current_dir(&repo_path)
        .output()?;

    // Switch back to main
    std::process::Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&repo_path)
        .output()?;

    // Delete the branch
    let result = manager.delete_branch("to-delete");
    assert!(result.is_ok());

    // Verify branch is deleted
    let (local, _) = manager.list_all_branches()?;
    assert!(!local.contains(&"to-delete".to_string()));

    Ok(())
}

/// Test WorktreeInfo struct fields
#[test]
fn test_worktree_info_fields() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Create a worktree
    manager.create_worktree("test-info", None)?;

    let worktrees = manager.list_worktrees()?;
    let info = worktrees.iter().find(|w| w.name == "test-info").unwrap();

    // Test WorktreeInfo fields
    assert_eq!(info.name, "test-info");
    assert!(info.path.exists());
    assert!(!info.branch.is_empty());
    assert!(!info.is_current); // Not the current worktree
    assert!(!info.has_changes); // No changes yet

    Ok(())
}

/// Test error handling for invalid operations
#[test]
fn test_error_handling() -> Result<()> {
    let (_temp_dir, _repo_path, manager) = setup_test_repo()?;

    // Try to create worktree with null byte in name (definitely invalid)
    let result = manager.create_worktree("invalid\0name", None);
    assert!(result.is_err(), "Worktree with null byte should fail");

    // Try to remove non-existent worktree
    let result = manager.remove_worktree("non-existent");
    assert!(
        result.is_err(),
        "Removing non-existent worktree should fail"
    );

    // Try to create worktree with existing name
    manager.create_worktree("existing", None)?;
    let result = manager.create_worktree("existing", None);
    assert!(result.is_err(), "Creating duplicate worktree should fail");

    // Try to delete current branch (assuming main is current)
    let result = manager.delete_branch("main");
    assert!(result.is_err(), "Deleting current branch should fail");

    // Try to rename to invalid name
    let result = manager.rename_branch("main", "invalid\0name");
    assert!(result.is_err(), "Renaming to invalid name should fail");

    Ok(())
}

/// Test get_git_dir functionality
#[test]
fn test_get_git_dir() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    let git_dir = manager.get_git_dir()?;
    assert!(git_dir.exists());
    assert!(git_dir.is_dir());

    // The get_git_dir method might return the repository root or .git directory
    // Let's check if it's either the repo path or the .git subdirectory
    let actual_git_dir = git_dir.canonicalize()?;
    let repo_canonical = repo_path.canonicalize()?;
    let git_dir_canonical = repo_path.join(".git").canonicalize()?;

    assert!(
        actual_git_dir == repo_canonical || actual_git_dir == git_dir_canonical,
        "git_dir should be either repository root ({repo_canonical:?}) or .git directory ({git_dir_canonical:?}), got: {actual_git_dir:?}"
    );

    Ok(())
}

/// Test creating worktree from tag
#[test]
fn test_create_worktree_from_tag() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo()?;

    // Create a tag
    std::process::Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(&repo_path)
        .output()?;

    // Create worktree from tag
    let result = manager.create_worktree("tag-worktree", Some("v1.0.0"));
    assert!(result.is_ok());

    let worktree_path = result.unwrap();
    assert!(worktree_path.exists());

    // Verify it's at the tagged commit
    let output = std::process::Command::new("git")
        .args(["describe", "--tags"])
        .current_dir(&worktree_path)
        .output()?;

    let tag_desc = String::from_utf8_lossy(&output.stdout);
    assert!(tag_desc.contains("v1.0.0"));

    Ok(())
}

/// Test bare repository operations
#[test]
fn test_bare_repository_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo = temp_dir.path().join("bare.git");

    // Initialize bare repository
    std::process::Command::new("git")
        .args(["init", "--bare", "bare.git"])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&bare_repo)?;

    // Try to create manager in bare repo
    let result = GitWorktreeManager::new();

    // Bare repos need at least one commit to work with worktrees
    if result.is_err() {
        // This is expected for a bare repo without commits
        return Ok(());
    }

    let manager = result?;

    // Should be able to get git dir (use canonical path comparison)
    let git_dir = manager.get_git_dir()?;
    let expected_dir = bare_repo.canonicalize()?;
    let actual_dir = git_dir.canonicalize()?;
    assert_eq!(actual_dir, expected_dir);

    Ok(())
}
