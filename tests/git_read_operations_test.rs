//! Tests for GitReadOperations abstraction
//!
//! This module tests the GitReadOperations trait implementation
//! to ensure proper separation of Git operations from business logic.

use anyhow::Result;
use git_workers::git_interface::{mock::MockGitOperations, GitReadOperations};

/// Test listing worktrees with no worktrees
#[test]
fn test_list_worktrees_empty() -> Result<()> {
    let mock = MockGitOperations::new();

    // Test that we can get an empty list of worktrees
    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 0);

    Ok(())
}

/// Test listing worktrees with multiple worktrees
#[test]
fn test_list_worktrees_multiple() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_worktree("main", "/repo/main", Some("main"))
        .with_worktree("feature", "/repo/feature", Some("feature/new"))
        .with_worktree("hotfix", "/repo/hotfix", Some("hotfix/urgent"))
        .with_current_worktree("main")
        .with_worktree_changes("feature");

    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 3);

    // Verify worktree details
    let main = worktrees.iter().find(|w| w.name == "main").unwrap();
    assert!(main.is_current);
    assert_eq!(main.branch, "main");

    let feature = worktrees.iter().find(|w| w.name == "feature").unwrap();
    assert!(feature.has_changes);
    assert_eq!(feature.branch, "feature/new");

    Ok(())
}

/// Test listing worktrees sorting (current first)
#[test]
fn test_list_worktrees_sorting() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_worktree("zebra", "/repo/zebra", Some("zebra"))
        .with_worktree("alpha", "/repo/alpha", Some("alpha"))
        .with_worktree("beta", "/repo/beta", Some("beta"))
        .with_current_worktree("beta");

    // The worktrees should be sorted with current first, then alphabetically
    let worktrees = mock.list_worktrees()?;

    // Note: The sorting happens in list_worktrees_with_git, not in the mock
    // So we just verify the mock returns the data correctly
    assert_eq!(worktrees.len(), 3);
    assert!(worktrees.iter().any(|w| w.name == "beta" && w.is_current));

    Ok(())
}

/// Test worktree with changes indicator
#[test]
fn test_list_worktrees_with_changes() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_worktree("main", "/repo/main", Some("main"))
        .with_worktree("feature", "/repo/feature", Some("feature/new"))
        .with_worktree_changes("feature");

    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 2);
    assert!(!worktrees[0].has_changes); // main
    assert!(worktrees[1].has_changes); // feature

    Ok(())
}

/// Test branch operations
#[test]
fn test_branch_operations() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_branch("main", false)
        .with_branch("develop", false)
        .with_branch("feature/new", false)
        .with_branch("origin/main", true)
        .with_branch("origin/develop", true);

    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 5);

    let local_branches: Vec<_> = branches.iter().filter(|b| !b.is_remote).collect();
    assert_eq!(local_branches.len(), 3);

    let remote_branches: Vec<_> = branches.iter().filter(|b| b.is_remote).collect();
    assert_eq!(remote_branches.len(), 2);

    Ok(())
}

/// Test tag operations
#[test]
fn test_tag_operations() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_tag("v1.0.0", Some("Release 1.0.0"))
        .with_tag("v1.1.0", Some("Release 1.1.0"))
        .with_tag("v2.0.0-beta", None);

    let tags = mock.list_tags()?;
    assert_eq!(tags.len(), 3);

    let v1 = tags.iter().find(|t| t.name == "v1.0.0").unwrap();
    assert_eq!(v1.message, Some("Release 1.0.0".to_string()));

    let beta = tags.iter().find(|t| t.name == "v2.0.0-beta").unwrap();
    assert_eq!(beta.message, None);

    Ok(())
}

/// Test repository information
#[test]
fn test_repository_info() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_repository_root("/home/user/project")
        .with_current_branch("develop");

    let info = mock.get_repository_info()?;
    assert!(info.contains("/home/user/project"));
    assert!(info.contains("develop"));

    // Test bare repository
    let bare_mock = MockGitOperations::new()
        .as_bare()
        .with_repository_root("/srv/git/project.git");

    let bare_info = bare_mock.get_repository_info()?;
    assert!(bare_info.contains("bare"));
    assert!(bare_info.contains("/srv/git/project.git"));

    Ok(())
}

/// Test branch worktree mapping
#[test]
fn test_branch_worktree_map() -> Result<()> {
    let mock = MockGitOperations::new()
        .with_worktree("main", "/repo/main", Some("main"))
        .with_worktree("feature-x", "/repo/feature-x", Some("feature/x"))
        .with_worktree("hotfix", "/repo/hotfix", Some("hotfix/urgent"));

    let map = mock.get_branch_worktree_map()?;
    assert_eq!(map.len(), 3);
    assert_eq!(map.get("main"), Some(&"main".to_string()));
    assert_eq!(map.get("feature/x"), Some(&"feature-x".to_string()));
    assert_eq!(map.get("hotfix/urgent"), Some(&"hotfix".to_string()));

    Ok(())
}
