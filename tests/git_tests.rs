use git_workers::git::WorktreeInfo;
use git_workers::menu::MenuItem;
use std::path::PathBuf;

#[test]
fn test_menu_item_display() {
    assert_eq!(MenuItem::ListWorktrees.to_string(), "•  List worktrees");
    assert_eq!(MenuItem::SearchWorktrees.to_string(), "?  Search worktrees");
    assert_eq!(MenuItem::CreateWorktree.to_string(), "+  Create worktree");
    assert_eq!(MenuItem::DeleteWorktree.to_string(), "-  Delete worktree");
    assert_eq!(
        MenuItem::BatchDelete.to_string(),
        "=  Batch delete worktrees"
    );
    assert_eq!(
        MenuItem::CleanupOldWorktrees.to_string(),
        "~  Cleanup old worktrees"
    );
    assert_eq!(MenuItem::SwitchWorktree.to_string(), "→  Switch worktree");
    assert_eq!(MenuItem::RenameWorktree.to_string(), "*  Rename worktree");
    assert_eq!(MenuItem::Exit.to_string(), "x  Exit");
}

#[test]
fn test_worktree_info_struct() {
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
    assert_eq!(info.branch, "main");
    assert!(info.is_current);
    assert!(!info.has_changes);
    assert_eq!(info.ahead_behind, Some((1, 2)));
}
