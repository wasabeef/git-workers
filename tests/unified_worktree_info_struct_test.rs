use git_workers::{
    constants::{TEST_AUTHOR_NAME, TEST_COMMIT_MESSAGE},
    git::{CommitInfo, WorktreeInfo},
};
use std::path::{Path, PathBuf};

#[test]
fn test_worktree_info_struct_basic() {
    let info = WorktreeInfo {
        name: "test".to_string(),
        path: PathBuf::from("/test/path"),
        branch: "main".to_string(),
        is_locked: false,
        is_current: true,
        has_changes: false,
        last_commit: None,
        ahead_behind: Some((1, 2)),
    };

    assert_eq!(info.name, "test");
    assert_eq!(info.path, PathBuf::from("/test/path"));
    assert_eq!(info.branch, "main");
    assert!(!info.is_locked);
    assert!(info.is_current);
    assert!(!info.has_changes);
    assert!(info.last_commit.is_none());
    assert_eq!(info.ahead_behind, Some((1, 2)));
}

#[test]
fn test_worktree_info_struct_with_different_values() {
    let info = WorktreeInfo {
        name: "test-worktree".to_string(),
        path: Path::new("/path/to/worktree").to_path_buf(),
        branch: "main".to_string(),
        is_locked: false,
        is_current: true,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    };

    assert_eq!(info.name, "test-worktree");
    assert_eq!(info.path, Path::new("/path/to/worktree"));
    assert_eq!(info.branch, "main");
    assert!(info.is_current);
    assert!(!info.has_changes);
    assert!(info.ahead_behind.is_none());
}

#[test]
fn test_worktree_info_struct_all_fields() {
    // Test with all fields having non-default values
    let last_commit = CommitInfo {
        id: "abc123".to_string(),
        message: TEST_COMMIT_MESSAGE.to_string(),
        author: TEST_AUTHOR_NAME.to_string(),
        time: "2024-01-01 10:00".to_string(),
    };

    let info = WorktreeInfo {
        name: "feature-worktree".to_string(),
        path: PathBuf::from("/workspace/project/worktrees/feature"),
        branch: "feature/new-feature".to_string(),
        is_locked: true,
        is_current: false,
        has_changes: true,
        last_commit: Some(last_commit.clone()),
        ahead_behind: Some((5, 3)),
    };

    assert_eq!(info.name, "feature-worktree");
    assert_eq!(
        info.path.to_str().unwrap(),
        "/workspace/project/worktrees/feature"
    );
    assert_eq!(info.branch, "feature/new-feature");
    assert!(info.is_locked);
    assert!(!info.is_current);
    assert!(info.has_changes);

    let commit = info.last_commit.unwrap();
    assert_eq!(commit.id, "abc123");
    assert_eq!(commit.message, TEST_COMMIT_MESSAGE);
    assert_eq!(commit.author, TEST_AUTHOR_NAME);
    assert_eq!(commit.time, "2024-01-01 10:00");

    assert_eq!(info.ahead_behind, Some((5, 3)));
}

#[test]
fn test_commit_info_struct() {
    let commit = CommitInfo {
        id: "def456".to_string(),
        message: "Initial commit".to_string(),
        author: "John Doe".to_string(),
        time: "2024-01-02 15:30".to_string(),
    };

    assert_eq!(commit.id, "def456");
    assert_eq!(commit.message, "Initial commit");
    assert_eq!(commit.author, "John Doe");
    assert_eq!(commit.time, "2024-01-02 15:30");
}

#[test]
fn test_worktree_info_edge_cases() {
    // Test with empty strings and edge case values
    let info = WorktreeInfo {
        name: String::new(),
        path: PathBuf::new(),
        branch: String::new(),
        is_locked: false,
        is_current: false,
        has_changes: false,
        last_commit: None,
        ahead_behind: Some((0, 0)),
    };

    assert!(info.name.is_empty());
    assert_eq!(info.path, PathBuf::new());
    assert!(info.branch.is_empty());
    assert_eq!(info.ahead_behind, Some((0, 0)));
}
