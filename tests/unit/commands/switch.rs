//! Unit tests for switch command functionality
//!
//! This module tests the business logic for worktree switching,
//! including validation and state management.

use anyhow::Result;
use git_workers::commands::{switch_worktree_with_ui, WorktreeSwitchConfig};
use git_workers::git::GitWorktreeManager;
use git_workers::ui::MockUI;
use serial_test::serial;
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

#[test]
fn test_worktree_switch_config() {
    let config = WorktreeSwitchConfig {
        target_name: "feature".to_string(),
        target_path: PathBuf::from("/tmp/feature"),
        target_branch: "feature".to_string(),
    };

    assert_eq!(config.target_name, "feature");
    assert_eq!(config.target_path, PathBuf::from("/tmp/feature"));
    assert_eq!(config.target_branch, "feature");
}

#[test]
#[serial]
fn test_switch_worktree_with_ui_writes_selected_path() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;
    let unique = format!(
        "feature-switch-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let target_path = manager.create_worktree_with_new_branch(&unique, &unique, "main")?;
    let switch_file = temp_dir.path().join("switch-target.txt");
    let worktrees = manager.list_worktrees()?;
    let selection = if worktrees.iter().any(|w| w.is_current) {
        1
    } else {
        0
    };

    std::env::set_var("GW_SWITCH_FILE", &switch_file);

    let ui = MockUI::new().with_selection(selection);
    let switched = switch_worktree_with_ui(&manager, &ui)?;

    std::env::remove_var("GW_SWITCH_FILE");

    assert!(switched);
    let written_path = fs::read_to_string(&switch_file)?;
    assert_eq!(written_path.trim(), target_path.to_string_lossy());

    Ok(())
}

#[test]
fn test_switch_worktree_with_ui_cancelled_selection_returns_false() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let unique = format!(
        "feature-cancel-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    manager.create_worktree_with_new_branch(&unique, &unique, "main")?;

    let ui = MockUI::new();
    let switched = switch_worktree_with_ui(&manager, &ui)?;

    assert!(!switched);

    Ok(())
}
