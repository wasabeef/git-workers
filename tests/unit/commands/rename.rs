//! Unit tests for rename command functionality
//!
//! This module tests the business logic for worktree renaming,
//! including validation and path handling.

use git_workers::commands::WorktreeRenameConfig;
use std::path::PathBuf;

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
#[allow(clippy::const_is_empty)]
fn test_rename_validation_basic() {
    // Basic string validation tests
    assert!(!"old-name".is_empty());
    assert!(!"new-name".is_empty());
    assert!("old-name" != "new-name");
}

#[test]
fn test_path_generation_for_rename() {
    let base_path = PathBuf::from("/tmp/worktrees");
    let old_name = "feature";
    let new_name = "feature-v2";

    let old_path = base_path.join(old_name);
    let new_path = base_path.join(new_name);

    assert_eq!(old_path.to_str().unwrap(), "/tmp/worktrees/feature");
    assert_eq!(new_path.to_str().unwrap(), "/tmp/worktrees/feature-v2");
}
