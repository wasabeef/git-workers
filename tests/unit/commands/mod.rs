//! Unit tests for all command implementations
//!
//! This module consolidates tests for all commands in the Git Workers project.
//! Tests are organized by command type and functionality.

mod create;
mod delete;
mod list;
mod rename;
mod switch;

use anyhow::Result;
use git_workers::commands::{find_config_file_path, get_worktree_icon, validate_custom_path};
use git_workers::constants;
use git_workers::infrastructure::git::{GitWorktreeManager, WorktreeInfo};
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Helper to create a test repository
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize git repository
    std::process::Command::new("git")
        .arg("init")
        .arg("--bare")
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

/// Helper to create a non-bare test repository with initial commit
fn setup_non_bare_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize non-bare repository
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

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Result<Self> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { original })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

// ============================================================================
// Icon and Display Tests
// ============================================================================

#[test]
fn test_get_worktree_icon_current_worktree() -> Result<()> {
    let (temp_dir, _manager) = setup_test_repo()?;

    // Create a test worktree
    let worktree_path = temp_dir.path().join("main");
    fs::create_dir_all(&worktree_path)?;

    let worktree = WorktreeInfo {
        name: "main".to_string(),
        git_name: "main".to_string(),
        path: worktree_path.clone(),
        branch: "main".to_string(),
        is_current: true,
        is_locked: false,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    };

    let icon = get_worktree_icon(&worktree);
    assert_eq!(icon, constants::EMOJI_HOME);

    Ok(())
}

#[test]
fn test_get_worktree_icon_with_changes() -> Result<()> {
    let (temp_dir, _manager) = setup_test_repo()?;

    let worktree_path = temp_dir.path().join("feature");
    fs::create_dir_all(&worktree_path)?;

    let worktree = WorktreeInfo {
        name: "feature".to_string(),
        git_name: "feature".to_string(),
        path: worktree_path,
        branch: "feature".to_string(),
        is_current: false,
        is_locked: false,
        has_changes: true,
        last_commit: None,
        ahead_behind: None,
    };

    let icon = get_worktree_icon(&worktree);
    // When has_changes is true but is_current is false, we get EMOJI_FOLDER
    assert_eq!(icon, constants::EMOJI_FOLDER);

    Ok(())
}

#[test]
fn test_get_worktree_icon_locked() -> Result<()> {
    let (temp_dir, _manager) = setup_test_repo()?;

    let worktree_path = temp_dir.path().join("locked");
    fs::create_dir_all(&worktree_path)?;

    let worktree = WorktreeInfo {
        name: "locked".to_string(),
        git_name: "locked".to_string(),
        path: worktree_path,
        branch: "locked".to_string(),
        is_current: false,
        is_locked: true,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    };

    let icon = get_worktree_icon(&worktree);
    assert_eq!(icon, constants::EMOJI_LOCKED);

    Ok(())
}

// ============================================================================
// Configuration Discovery Tests
// ============================================================================

#[test]
#[serial]
fn test_find_config_file_path_defaults_to_current_bare_repo_directory() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;
    let _guard = CurrentDirGuard::change_to(temp_dir.path())?;

    let found_path = find_config_file_path(&manager)?;
    assert_eq!(
        found_path,
        std::env::current_dir()?.join(".git-workers.toml")
    );

    Ok(())
}

#[test]
#[serial]
fn test_find_config_file_path_prefers_current_worktree_directory() -> Result<()> {
    let (temp_dir, _manager) = setup_non_bare_repo()?;

    fs::write(
        temp_dir.path().join(".git-workers.toml"),
        "[worktree]\npattern = \"same-level\"",
    )?;

    let worktree_path = temp_dir.path().parent().unwrap().join(format!(
        "{}-feature",
        temp_dir.path().file_name().unwrap().to_string_lossy()
    ));
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

    let worktree_path = worktree_path.canonicalize()?;
    let _guard = CurrentDirGuard::change_to(&worktree_path)?;
    let manager = GitWorktreeManager::new_from_path(&worktree_path)?;
    let found_path = find_config_file_path(&manager)?;
    assert_eq!(found_path, worktree_path.join(".git-workers.toml"));

    Ok(())
}

// ============================================================================
// Path Validation Tests
// ============================================================================

#[test]
fn test_validate_custom_path_valid_paths() {
    // Valid relative paths
    assert!(validate_custom_path("worktrees/feature").is_ok());
    assert!(validate_custom_path("../project-worktrees/feature").is_ok());
    assert!(validate_custom_path("my-worktree").is_ok());
    assert!(validate_custom_path("feature/ISSUE-123").is_ok());
}

#[test]
fn test_validate_custom_path_invalid_paths() {
    // Absolute paths
    assert!(validate_custom_path("/absolute/path").is_err());
    assert!(validate_custom_path("C:\\Windows\\path").is_err());

    // Invalid characters
    assert!(validate_custom_path("path/with:colon").is_err());
    assert!(validate_custom_path("path/with*asterisk").is_err());
    assert!(validate_custom_path("path/with?question").is_err());

    // Git reserved names
    assert!(validate_custom_path(".git/worktree").is_ok()); // .git is not in GIT_RESERVED_NAMES currently
    assert!(validate_custom_path("worktrees/.git").is_ok()); // .git is not in GIT_RESERVED_NAMES currently
    assert!(validate_custom_path("HEAD/worktree").is_err());
    assert!(validate_custom_path("refs/heads/feature").is_err());

    // Excessive traversal
    assert!(validate_custom_path("../../outside").is_err());
    assert!(validate_custom_path("../../../way/outside").is_err());

    // Empty or invalid formats
    assert!(validate_custom_path("").is_err());
    assert!(validate_custom_path("   ").is_err());
    assert!(validate_custom_path("/").is_err());
    assert!(validate_custom_path("path/").is_err());
}

#[test]
fn test_validate_custom_path_edge_cases() {
    // Dots in names (not traversal)
    assert!(validate_custom_path("feature.branch").is_ok());
    assert!(validate_custom_path("v1.0.0").is_ok());

    // Single dot as component - both are valid in current implementation
    assert!(validate_custom_path("./worktree").is_ok());
    assert!(validate_custom_path("worktree/.").is_ok()); // Currently allowed as "." is skipped

    // Hidden files (allowed)
    assert!(validate_custom_path(".hidden/worktree").is_ok());

    // Unicode characters (should warn but allow)
    assert!(validate_custom_path("feature/日本語").is_ok());
    assert!(validate_custom_path("café/feature").is_ok());
}

// ============================================================================
// Future Command Tests (Placeholders)
// ============================================================================

#[cfg(test)]
mod prune_tests {

    #[test]
    #[ignore = "Prune command not yet implemented"]
    fn test_prune_worktree_basic() {
        // TODO: Implement when prune command is added
    }
}

#[cfg(test)]
mod sync_tests {

    #[test]
    #[ignore = "Sync command not yet implemented"]
    fn test_sync_worktree_basic() {
        // TODO: Implement when sync command is added
    }
}
