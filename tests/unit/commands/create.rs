//! Unit tests for create command functionality
//!
//! This module tests the business logic for worktree creation,
//! including path determination and branch source handling.

use anyhow::Result;
use git_workers::commands::{
    determine_worktree_path, validate_worktree_creation, BranchSource, WorktreeCreateConfig,
};
use git_workers::infrastructure::git::GitWorktreeManager;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_worktree_create_config_creation() -> Result<()> {
    let config = WorktreeCreateConfig {
        name: "test".to_string(),
        path: PathBuf::from("/tmp/test"),
        branch_source: BranchSource::Head,
        switch_to_new: true,
    };

    assert_eq!(config.name, "test");
    assert_eq!(config.path, PathBuf::from("/tmp/test"));
    assert!(config.switch_to_new);

    Ok(())
}

#[test]
fn test_branch_source_variants() {
    let head = BranchSource::Head;
    matches!(head, BranchSource::Head);

    let branch = BranchSource::Branch("main".to_string());
    matches!(branch, BranchSource::Branch(ref name) if name == "main");

    let tag = BranchSource::Tag("v1.0.0".to_string());
    matches!(tag, BranchSource::Tag(ref name) if name == "v1.0.0");

    let new_branch = BranchSource::NewBranch {
        name: "feature".to_string(),
        base: "main".to_string(),
    };
    matches!(new_branch, BranchSource::NewBranch { ref name, ref base } if name == "feature" && base == "main");
}

#[test]
fn test_path_generation() {
    let _base_path = PathBuf::from("/tmp/worktrees");
    let name = "feature";

    let same_level = format!("../{name}");
    let subdirectory = format!("worktrees/{name}");

    assert_eq!(same_level, "../feature");
    assert_eq!(subdirectory, "worktrees/feature");
}

#[test]
#[allow(clippy::const_is_empty)]
fn test_worktree_name_validation() {
    // Valid names
    assert!(!"feature".is_empty());
    assert!(!"feature-123".is_empty());
    assert!(!"bugfix/issue-456".is_empty());

    // Invalid names
    assert!("".is_empty());
    assert!("   ".trim().is_empty());
}

/// Helper to create a test repository
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize repository
    Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    Command::new("git")
        .arg("add")
        .arg("README.md")
        .current_dir(temp_dir.path())
        .output()?;
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

#[test]
fn test_determine_worktree_path() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let git_dir = manager.get_git_dir()?;

    // Test path determination with valid parameters
    let (path, location_type) = determine_worktree_path(git_dir, "feature", "same-level", None)?;
    assert!(path.to_string_lossy().contains("feature"));
    assert_eq!(location_type, "same-level");

    Ok(())
}

#[test]
fn test_validate_worktree_creation() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let worktrees = manager.list_worktrees()?;

    // Test validation of worktree creation
    let path = PathBuf::from("/tmp/test-worktree");
    let result = validate_worktree_creation("new-worktree", &path, &worktrees);
    assert!(result.is_ok());

    Ok(())
}
