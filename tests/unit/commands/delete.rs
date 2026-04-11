//! Unit tests for delete command functionality
//!
//! This module tests the business logic for worktree deletion,
//! including removal confirmation and cleanup operations.

use anyhow::Result;
use git_workers::commands::{delete_worktree_with_ui, execute_deletion, WorktreeDeleteConfig};
use git_workers::git::GitWorktreeManager;
use git_workers::infrastructure::git::WorktreeInfo;
use git_workers::ui::MockUI;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .arg("add")
        .arg("README.md")
        .current_dir(temp_dir.path())
        .output()?;
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()?;
    std::process::Command::new("git")
        .args(["branch", "-m", "main"])
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

fn unique_name(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

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
fn test_execute_deletion_removes_worktree_and_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let name = unique_name("delete-me");
    let worktree_path = manager.create_worktree_with_new_branch(&name, &name, "main")?;

    let config = WorktreeDeleteConfig {
        name: name.clone(),
        path: worktree_path.clone(),
        branch: name.clone(),
        delete_branch: true,
    };

    execute_deletion(&config, &manager)?;

    assert!(!worktree_path.exists());

    let (local_branches, _) = manager.list_all_branches()?;
    assert!(!local_branches.contains(&name));

    Ok(())
}

#[test]
fn test_delete_worktree_with_ui_cancel_keeps_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let name = unique_name("delete-cancel");
    manager.create_worktree_with_new_branch(&name, &name, "main")?;

    let ui = MockUI::new().with_selection(0);
    delete_worktree_with_ui(&manager, &ui)?;

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == name));

    Ok(())
}

#[test]
fn test_worktree_info_for_deletion() {
    let worktree = WorktreeInfo {
        name: "feature".to_string(),
        git_name: "feature".to_string(),
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
