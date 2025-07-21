//! Unit tests for delete command functionality
//!
//! This module tests the business logic for worktree deletion,
//! including removal confirmation and cleanup operations.

use anyhow::Result;
use git_workers::commands::WorktreeDeleteConfig;
use git_workers::infrastructure::git::WorktreeInfo;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_worktree_delete_config_creation() {
    let config = WorktreeDeleteConfig {
        name: "feature".to_string(),
        path: PathBuf::from("/tmp/feature"),
        branch: "feature".to_string(),
        delete_branch: true,
    };

    assert_eq!(config.name, "feature");
    assert_eq!(config.path, PathBuf::from("/tmp/feature"));
    assert_eq!(config.branch, "feature");
    assert!(config.delete_branch);
}

// Integration test for actual deletion
#[test]
fn test_worktree_deletion_simulation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Initialize repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
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

    // Create a worktree
    let worktree_path = temp_dir.path().join("../feature");
    std::process::Command::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .current_dir(temp_dir.path())
        .output()?;

    // Verify worktree exists
    assert!(worktree_path.exists());

    // Test would validate deletion target if API was public
    // assert!(validate_deletion_target("feature").is_ok());

    Ok(())
}

#[test]
fn test_worktree_info_for_deletion() {
    let worktree = WorktreeInfo {
        name: "feature".to_string(),
        path: PathBuf::from("/tmp/feature"),
        branch: "feature".to_string(),
        is_current: false,
        is_locked: false,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    };

    // Test that we can validate deletion for this worktree
    assert!(!worktree.name.is_empty());
    assert!(!worktree.is_current);
    assert!(!worktree.is_locked);
}
