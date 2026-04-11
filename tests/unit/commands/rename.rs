//! Unit tests for rename command functionality
//!
//! This module tests the business logic for worktree renaming,
//! including validation and path handling.

use anyhow::Result;
use git_workers::commands::{
    execute_rename, rename_worktree_with_ui, validate_rename_operation, WorktreeRenameConfig,
};
use git_workers::git::GitWorktreeManager;
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
fn test_worktree_rename_config() {
    let config = WorktreeRenameConfig {
        old_name: "old".to_string(),
        new_name: "new".to_string(),
        old_path: PathBuf::from("/tmp/old"),
        new_path: PathBuf::from("/tmp/new"),
        old_branch: "old".to_string(),
        new_branch: Some("new".to_string()),
        rename_branch: true,
    };

    assert_eq!(config.old_name, "old");
    assert_eq!(config.new_name, "new");
    assert_eq!(config.old_path, PathBuf::from("/tmp/old"));
    assert_eq!(config.new_path, PathBuf::from("/tmp/new"));
    assert_eq!(config.old_branch, "old");
    assert_eq!(config.new_branch, Some("new".to_string()));
    assert!(config.rename_branch);
}

#[test]
fn test_validate_rename_operation_rejects_reserved_and_equal_names() {
    assert!(validate_rename_operation("feature", "feature").is_err());
    assert!(validate_rename_operation("main", "other").is_err());
    assert!(validate_rename_operation("feature", "master").is_err());
}

#[test]
fn test_execute_rename_updates_worktree_path_without_branch_rename() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let old_name = unique_name("rename-me");
    let new_name = unique_name("renamed-worktree");
    let old_path = manager.create_worktree_with_new_branch(
        &old_name,
        &format!("feature/{old_name}"),
        "main",
    )?;
    let new_path = old_path.parent().unwrap().join(&new_name);

    let config = WorktreeRenameConfig {
        old_name: old_name.clone(),
        new_name: new_name.clone(),
        old_path: old_path.clone(),
        new_path: new_path.clone(),
        old_branch: format!("feature/{old_name}"),
        new_branch: None,
        rename_branch: false,
    };

    execute_rename(&config, &manager)?;

    let worktrees = manager.list_worktrees()?;
    let renamed = worktrees
        .iter()
        .find(|w| w.name == new_name)
        .expect("renamed worktree should exist");

    assert_eq!(renamed.git_name, old_name);
    assert_eq!(renamed.branch, format!("feature/{}", renamed.git_name));
    assert_eq!(renamed.path, new_path.canonicalize()?);

    Ok(())
}

#[test]
fn test_execute_rename_can_rename_branch_too() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let old_name = unique_name("branch-sync");
    let new_name = unique_name("branch-sync-renamed");
    let old_path = manager.create_worktree_with_new_branch(&old_name, &old_name, "main")?;
    let new_path = old_path.parent().unwrap().join(&new_name);

    let config = WorktreeRenameConfig {
        old_name: old_name.clone(),
        new_name: new_name.clone(),
        old_path: old_path.clone(),
        new_path,
        old_branch: old_name.clone(),
        new_branch: Some(new_name.clone()),
        rename_branch: true,
    };

    execute_rename(&config, &manager)?;

    let (local_branches, _) = manager.list_all_branches()?;
    assert!(local_branches.contains(&new_name));
    assert!(!local_branches.contains(&old_name));

    Ok(())
}

#[test]
fn test_rename_worktree_with_ui_cancel_keeps_existing_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let old_name = unique_name("preview-rename");
    let new_name = unique_name("renamed-preview");
    manager.create_worktree_with_new_branch(&old_name, &old_name, "main")?;

    let ui = MockUI::new()
        .with_selection(0)
        .with_input(&new_name)
        .with_confirm(true)
        .with_confirm(false);

    rename_worktree_with_ui(&manager, &ui)?;

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == old_name));
    assert!(!worktrees.iter().any(|w| w.name == new_name));

    Ok(())
}
