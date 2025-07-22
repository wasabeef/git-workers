//! Unit tests for list command functionality
//!
//! This module tests the business logic for worktree listing
//! and display formatting.

use git_workers::infrastructure::git::{CommitInfo, WorktreeInfo};
use std::path::PathBuf;

#[test]
fn test_worktree_info_creation_basic() {
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

    assert_eq!(worktree.name, "feature");
    assert_eq!(worktree.branch, "feature");
    assert!(!worktree.is_current);
    assert!(!worktree.is_locked);
    assert!(!worktree.has_changes);
    assert!(worktree.last_commit.is_none());
}

#[test]
fn test_worktree_info_with_commit() {
    let commit = CommitInfo {
        id: "abc123".to_string(),
        message: "Test commit".to_string(),
        author: "Test Author".to_string(),
        time: "2024-01-01 12:00".to_string(),
    };

    let worktree = WorktreeInfo {
        name: "feature".to_string(),
        git_name: "feature".to_string(),
        path: PathBuf::from("/tmp/feature"),
        branch: "feature".to_string(),
        is_current: false,
        is_locked: false,
        has_changes: false,
        last_commit: Some(commit),
        ahead_behind: Some((2, 3)),
    };

    assert!(worktree.last_commit.is_some());
    assert_eq!(worktree.ahead_behind, Some((2, 3)));
    assert_eq!(worktree.last_commit.as_ref().unwrap().id, "abc123");
}

#[test]
fn test_commit_info_creation() {
    let commit = CommitInfo {
        id: "abc123".to_string(),
        message: "Test commit message".to_string(),
        author: "Test Author".to_string(),
        time: "2024-01-01 12:00".to_string(),
    };

    assert_eq!(commit.id, "abc123");
    assert_eq!(commit.message, "Test commit message");
    assert_eq!(commit.author, "Test Author");
    assert_eq!(commit.time, "2024-01-01 12:00");
}
