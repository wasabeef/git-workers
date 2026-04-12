pub use crate::adapters::config::loader::{find_config_file_path, find_config_file_path_internal};
pub use crate::app::presenter::get_worktree_icon;
pub use crate::usecases::cleanup_worktrees::cleanup_old_worktrees;
pub use crate::usecases::delete_worktree::{
    batch_delete_worktrees, prepare_batch_delete_items, BatchDeleteConfig,
};
pub use crate::usecases::edit_hooks::edit_hooks;
pub use crate::usecases::search_worktrees::{
    create_search_items, search_worktrees, validate_search_selection, SearchAnalysis, SearchConfig,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::SEARCH_CURRENT_INDICATOR;
    use crate::git::WorktreeInfo;
    use crate::infrastructure::git::GitWorktreeManager;
    use anyhow::Result;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper to create a test repository
    #[allow(dead_code)]
    fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
        let temp_dir = TempDir::new()?;

        // Initialize repository
        Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .output()?;

        // Configure git
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()?;

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()?;

        // Create initial commit
        fs::write(temp_dir.path().join("README.md"), "# Test")?;
        Command::new("git")
            .arg("add")
            .arg("README.md")
            .current_dir(temp_dir.path())
            .output()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(temp_dir.path())
            .output()?;

        let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
        Ok((temp_dir, manager))
    }

    #[test]
    fn test_create_search_items() -> Result<()> {
        let worktree_info = WorktreeInfo {
            name: "feature-branch".to_string(),
            git_name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: true,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        };
        let worktrees = vec![worktree_info];

        let analysis = create_search_items(&worktrees);

        assert_eq!(analysis.total_count, 1);
        assert!(analysis.has_current);
        assert_eq!(analysis.items.len(), 1);
        assert!(analysis.items[0].contains("feature-branch"));
        assert!(analysis.items[0].contains("feature/test"));
        assert!(analysis.items[0].contains(SEARCH_CURRENT_INDICATOR));

        Ok(())
    }

    #[test]
    fn test_validate_search_selection() -> Result<()> {
        let worktree_info = WorktreeInfo {
            name: "feature-branch".to_string(),
            git_name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: false,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        };
        let worktrees = vec![worktree_info];

        // Valid selection
        let selected = validate_search_selection(&worktrees, 0)?;
        assert_eq!(selected.name, "feature-branch");

        // Invalid selection
        let result = validate_search_selection(&worktrees, 1);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_prepare_batch_delete_items() -> Result<()> {
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
                git_name: "main".to_string(),
                path: std::path::PathBuf::from("/test/main"),
                branch: "main".to_string(),
                is_current: true,
                is_locked: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
            WorktreeInfo {
                name: "feature-branch".to_string(),
                git_name: "feature-branch".to_string(),
                path: std::path::PathBuf::from("/test/feature-branch"),
                branch: "feature/test".to_string(),
                is_current: false,
                is_locked: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
        ];

        let items = prepare_batch_delete_items(&worktrees);

        // Should only include non-current worktrees
        assert_eq!(items.len(), 1);
        assert!(items[0].contains("feature-branch"));
        assert!(items[0].contains("feature/test"));
        assert!(!items[0].contains("main"));

        Ok(())
    }
}
