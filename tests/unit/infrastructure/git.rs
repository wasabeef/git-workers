//! Unit tests for Git operations and GitWorktreeManager
//!
//! This module tests all Git-related functionality including worktree operations,
//! branch management, and repository interactions.

use anyhow::Result;
use git_workers::infrastructure::git::GitWorktreeManager;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a bare test repository
fn setup_bare_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    std::process::Command::new("git")
        .arg("init")
        .arg("--bare")
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

/// Create a non-bare test repository with initial commit
fn setup_repo_with_commit() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    // Set git config
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test Repository")?;
    std::process::Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(temp_dir.path())
        .output()?;
    std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

// ============================================================================
// Basic Operations Tests
// ============================================================================

#[test]
fn test_new_from_path_bare_repo() -> Result<()> {
    let (temp_dir, manager) = setup_bare_repo()?;

    assert!(manager.repo().is_bare());
    // On macOS, paths might be symlinked (e.g., /var -> /private/var)
    // So we canonicalize both paths before comparison
    assert_eq!(
        manager.get_git_dir()?.canonicalize()?,
        temp_dir.path().canonicalize()?
    );

    Ok(())
}

#[test]
fn test_new_from_path_non_bare_repo() -> Result<()> {
    let (temp_dir, manager) = setup_repo_with_commit()?;

    assert!(!manager.repo().is_bare());
    // On macOS, paths might be symlinked (e.g., /var -> /private/var)
    // So we canonicalize both paths before comparison
    assert_eq!(
        manager.get_git_dir()?.canonicalize()?,
        temp_dir.path().canonicalize()?
    );

    Ok(())
}

#[test]
fn test_new_from_path_not_a_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = GitWorktreeManager::new_from_path(temp_dir.path());

    assert!(result.is_err());
}

// ============================================================================
// Worktree Listing Tests
// ============================================================================

#[test]
fn test_list_worktrees_bare_repo() -> Result<()> {
    let (_temp_dir, manager) = setup_bare_repo()?;

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.is_empty());

    Ok(())
}

#[test]
fn test_list_worktrees_with_main() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    let worktrees = manager.list_worktrees()?;
    // In the current implementation, list_worktrees doesn't include the main worktree
    // for non-bare repositories. It only lists linked worktrees.
    assert_eq!(worktrees.len(), 0);

    Ok(())
}

#[test]
fn test_list_worktrees_multiple() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create additional worktrees with unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let feature_name = format!("feature-{timestamp}");
    let bugfix_name = format!("bugfix-{timestamp}");

    let _feature_path = manager.get_git_dir()?.parent().unwrap().join(&feature_name);
    manager.create_worktree_with_new_branch(&feature_name, &feature_name, "main")?;

    let _bugfix_path = manager.get_git_dir()?.parent().unwrap().join(&bugfix_name);
    manager.create_worktree_with_new_branch(&bugfix_name, &bugfix_name, "main")?;

    let worktrees = manager.list_worktrees()?;
    // The main worktree is not included in list_worktrees for non-bare repos
    assert_eq!(worktrees.len(), 2);

    let names: Vec<&str> = worktrees.iter().map(|w| w.name.as_str()).collect();
    assert!(names.contains(&feature_name.as_str()));
    assert!(names.contains(&bugfix_name.as_str()));

    Ok(())
}

// ============================================================================
// Branch Management Tests
// ============================================================================

#[test]
fn test_list_all_branches_empty() -> Result<()> {
    let (_temp_dir, manager) = setup_bare_repo()?;

    let (local_branches, remote_branches) = manager.list_all_branches()?;
    assert!(local_branches.is_empty());
    assert!(remote_branches.is_empty());

    Ok(())
}

#[test]
fn test_list_all_branches_with_local() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create some branches
    std::process::Command::new("git")
        .args(["branch", "feature"])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    std::process::Command::new("git")
        .args(["branch", "bugfix"])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    let (local_branches, _remote_branches) = manager.list_all_branches()?;
    assert!(local_branches.len() >= 3); // main/master + feature + bugfix
    assert!(local_branches.contains(&"feature".to_string()));
    assert!(local_branches.contains(&"bugfix".to_string()));

    Ok(())
}

#[test]
fn test_get_branch_worktree_map() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a worktree with a branch using unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let feature_name = format!("feature-{timestamp}");
    let worktree_name = format!("{feature_name}-wt");

    let _feature_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_with_new_branch(&worktree_name, &feature_name, "main")?;

    let map = manager.get_branch_worktree_map()?;
    assert!(map.contains_key(&feature_name));
    assert_eq!(map.get(&feature_name), Some(&worktree_name));

    Ok(())
}

#[test]
fn test_is_branch_unique_to_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a worktree with unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let feature_name = format!("feature-{timestamp}");
    let worktree_name = format!("{feature_name}-wt");

    let _feature_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_with_new_branch(&worktree_name, &feature_name, "main")?;

    // Create an unrelated branch
    let unrelated_name = format!("unrelated-{timestamp}");
    std::process::Command::new("git")
        .args(["branch", &unrelated_name])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    assert!(manager.is_branch_unique_to_worktree(&feature_name, &worktree_name)?);
    assert!(!manager.is_branch_unique_to_worktree(&unrelated_name, &worktree_name)?);

    Ok(())
}

// ============================================================================
// Tag Management Tests
// ============================================================================

#[test]
fn test_list_all_tags_empty() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    let tags = manager.list_all_tags()?;
    assert!(tags.is_empty());

    Ok(())
}

#[test]
fn test_list_all_tags_with_tags() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create lightweight tag
    std::process::Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    // Create annotated tag
    std::process::Command::new("git")
        .args(["tag", "-a", "v2.0.0", "-m", "Version 2.0.0"])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    let tags = manager.list_all_tags()?;
    assert_eq!(tags.len(), 2);

    // Check tag info
    let v1 = tags.iter().find(|t| t.0 == "v1.0.0").unwrap();
    assert!(v1.1.is_none()); // Lightweight tag has no message

    let v2 = tags.iter().find(|t| t.0 == "v2.0.0").unwrap();
    // Tag messages might have trailing newlines, so we trim them
    assert_eq!(v2.1.as_ref().unwrap().trim(), "Version 2.0.0");

    Ok(())
}

// ============================================================================
// Worktree Creation Tests
// ============================================================================

#[test]
fn test_create_worktree_from_head() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Use unique name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("new-feature-{timestamp}");

    let worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_from_head(&worktree_path, &worktree_name)?;

    assert!(worktree_path.exists());

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == worktree_name));

    Ok(())
}

#[test]
fn test_create_worktree_with_new_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Use unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let feature_name = format!("feature-new-{timestamp}");
    let worktree_name = format!("{feature_name}-wt");

    let worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_with_new_branch(&worktree_name, &feature_name, "main")?;

    assert!(worktree_path.exists());

    // Verify branch was created
    let (local_branches, _remote_branches) = manager.list_all_branches()?;
    assert!(local_branches.contains(&feature_name));

    Ok(())
}

#[test]
fn test_create_worktree_with_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Use unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let branch_name = format!("existing-{timestamp}");
    let worktree_name = format!("{branch_name}-wt");

    // Create a branch first
    std::process::Command::new("git")
        .args(["branch", &branch_name])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    let worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_with_branch(&worktree_path, &branch_name)?;

    assert!(worktree_path.exists());

    Ok(())
}

#[test]
fn test_create_worktree_from_tag() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Use unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let tag_name = format!("v1.0.0-{timestamp}");
    let worktree_name = format!("{tag_name}-wt");

    // Create a tag
    std::process::Command::new("git")
        .args(["tag", &tag_name])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    let worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    manager.create_worktree_with_branch(&worktree_path, &tag_name)?;

    assert!(worktree_path.exists());

    Ok(())
}

// ============================================================================
// Worktree Removal Tests
// ============================================================================

#[test]
fn test_remove_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a worktree
    let worktree_path = manager.get_git_dir()?.parent().unwrap().join("to-remove");
    manager.create_worktree_with_new_branch("to-remove", "to-remove", "main")?;

    // Remove it
    manager.remove_worktree("to-remove")?;

    assert!(!worktree_path.exists());

    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.iter().any(|w| w.name == "to-remove"));

    Ok(())
}

#[test]
fn test_remove_worktree_force() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a worktree
    let worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join("with-changes");
    manager.create_worktree_with_new_branch("with-changes", "with-changes", "main")?;

    // Make changes
    fs::write(worktree_path.join("new-file.txt"), "changes")?;

    // Remove with force
    manager.remove_worktree("with-changes")?;

    assert!(!worktree_path.exists());

    Ok(())
}

// ============================================================================
// Worktree Rename Tests
// ============================================================================

#[test]
fn test_rename_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Use unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let old_name = format!("old-name-{timestamp}");
    let new_name = format!("new-name-{timestamp}");

    // Create a worktree
    let old_path = manager.get_git_dir()?.parent().unwrap().join(&old_name);
    manager.create_worktree_with_new_branch(&old_name, &old_name, "main")?;

    // Rename it
    let new_path = manager.get_git_dir()?.parent().unwrap().join(&new_name);
    let returned_path = manager.rename_worktree(&old_name, &new_name)?;

    // Verify the rename succeeded
    assert!(!old_path.exists());
    assert!(new_path.exists());
    assert_eq!(returned_path, new_path);

    // The worktree should still be accessible via git
    let worktrees = manager.list_worktrees()?;
    // After renaming, the display name should change to the new name
    // but Git still tracks it by the original metadata name
    assert!(worktrees.iter().any(|w| w.name == new_name));

    Ok(())
}

#[test]
fn test_rename_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a branch
    std::process::Command::new("git")
        .args(["branch", "old-branch"])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    // Rename it
    manager.rename_branch("old-branch", "new-branch")?;

    let (local_branches, _remote_branches) = manager.list_all_branches()?;
    assert!(local_branches.contains(&"new-branch".to_string()));
    assert!(!local_branches.contains(&"old-branch".to_string()));

    Ok(())
}

// ============================================================================
// Repository Information Tests
// ============================================================================

// Note: get_current_branch, has_uncommitted_changes, and get_worktree_info methods
// are no longer part of the GitWorktreeManager API

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_create_worktree_existing_branch_error() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    // Create a branch with unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let feature_name = format!("feature-err-{timestamp}");

    std::process::Command::new("git")
        .args(["branch", &feature_name])
        .current_dir(manager.get_git_dir()?)
        .output()?;

    // Try to create worktree with new branch of same name
    let worktree_name = format!("{feature_name}-wt");
    let _worktree_path = manager
        .get_git_dir()?
        .parent()
        .unwrap()
        .join(&worktree_name);
    let result = manager.create_worktree_with_new_branch(&worktree_name, &feature_name, "main");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[test]
fn test_remove_nonexistent_worktree_error() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    let result = manager.remove_worktree("nonexistent");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_rename_nonexistent_worktree_error() -> Result<()> {
    let (_temp_dir, manager) = setup_repo_with_commit()?;

    let _new_path = manager.get_git_dir()?.parent().unwrap().join("new");
    let result = manager.rename_worktree("nonexistent", "new");

    assert!(result.is_err());

    Ok(())
}
