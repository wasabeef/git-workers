use anyhow::Result;
use colored::*;

use crate::constants::{section_header, WARNING_NO_WORKTREES};
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::repository_info::get_repository_info;
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::press_any_key_to_continue;

/// Format worktree display string
#[allow(dead_code)]
pub fn format_worktree_display(worktree: &WorktreeInfo, verbose: bool) -> String {
    let mut parts = vec![worktree.name.clone()];

    if worktree.is_current {
        parts.push("(current)".to_string());
    }

    if worktree.is_locked {
        parts.push("(locked)".to_string());
    }

    if worktree.has_changes {
        parts.push("(changes)".to_string());
    }

    if verbose {
        parts.push(format!("- {}", worktree.path.display()));

        if let Some(ref commit) = worktree.last_commit {
            parts.push(format!("[{}]", commit.id));
        }

        if let Some((ahead, behind)) = worktree.ahead_behind {
            parts.push(format!("↑{ahead} ↓{behind}"));
        }
    }

    parts.join(" ")
}

/// Check if worktree should be shown based on filters
#[allow(dead_code)]
pub fn should_show_worktree(worktree: &WorktreeInfo, show_all: bool, filter: Option<&str>) -> bool {
    // If filter is provided, check if it matches
    if let Some(f) = filter {
        return worktree.name.contains(f);
    }

    // If show_all is true, show everything
    if show_all {
        return true;
    }

    // Otherwise, only show worktrees with changes
    worktree.has_changes
}

/// Lists all worktrees with detailed information
///
/// Displays a comprehensive list of all worktrees in the repository,
/// including the current worktree and their paths, branches, and status.
///
/// # Display Format
///
/// The list shows:
/// - Repository information (URL, default branch)
/// - Formatted table of worktrees with:
///   - Name (highlighted if current)
///   - Branch name (colored by type)
///   - Path (absolute path to worktree)
///   - Modified status indicator
///
/// # Returns
///
/// Returns `Ok(())` on successful completion.
///
/// # Errors
///
/// Returns an error if Git repository operations fail.
pub fn list_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    let ui = DialoguerUI;
    list_worktrees_with_ui(&manager, &ui)
}

/// Internal implementation of list_worktrees with dependency injection
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
/// * `ui` - User interface implementation for testability
pub fn list_worktrees_with_ui(manager: &GitWorktreeManager, _ui: &dyn UserInterface) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    println!();
    let header = section_header("Git Worktrees");
    println!("{header}");
    println!();

    if worktrees.is_empty() {
        let msg = WARNING_NO_WORKTREES.yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Display repository info
    let repo_info = get_repository_info();
    println!("Repository: {}", repo_info.bright_cyan());
    println!();

    // Display worktrees
    for worktree in &worktrees {
        let name = if worktree.is_current {
            format!("{} [current]", worktree.name.bright_white().bold())
        } else {
            worktree.name.to_string()
        };

        let branch = worktree.branch.yellow();
        let path = worktree.path.display();

        println!("• {name}");
        println!("  Branch: {branch}");
        println!("  Path:   {path}");
        println!();
    }

    press_any_key_to_continue()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_worktree_display_basic() {
        let worktree = WorktreeInfo {
            name: "feature".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let display = format_worktree_display(&worktree, false);
        assert_eq!(display, "feature");
    }

    #[test]
    fn test_format_worktree_display_current() {
        let worktree = WorktreeInfo {
            name: "main".to_string(),
            path: PathBuf::from("/tmp/main"),
            branch: "main".to_string(),
            is_current: true,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let display = format_worktree_display(&worktree, false);
        assert_eq!(display, "main (current)");
    }

    #[test]
    fn test_format_worktree_display_locked_changes() {
        let worktree = WorktreeInfo {
            name: "locked".to_string(),
            path: PathBuf::from("/tmp/locked"),
            branch: "locked".to_string(),
            is_current: false,
            has_changes: true,
            last_commit: None,
            ahead_behind: None,
            is_locked: true,
        };

        let display = format_worktree_display(&worktree, false);
        assert_eq!(display, "locked (locked) (changes)");
    }

    #[test]
    fn test_format_worktree_display_verbose() {
        let worktree = WorktreeInfo {
            name: "feature".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let display = format_worktree_display(&worktree, true);
        assert!(display.contains("- /tmp/feature"));
    }

    #[test]
    fn test_should_show_worktree_with_filter_match() {
        let worktree = WorktreeInfo {
            name: "feature-auth".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        assert!(should_show_worktree(&worktree, false, Some("auth")));
    }

    #[test]
    fn test_should_show_worktree_with_filter_no_match() {
        let worktree = WorktreeInfo {
            name: "feature-ui".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        assert!(!should_show_worktree(&worktree, false, Some("auth")));
    }

    #[test]
    fn test_should_show_worktree_show_all() {
        let worktree = WorktreeInfo {
            name: "clean".to_string(),
            path: PathBuf::from("/tmp/clean"),
            branch: "clean".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        assert!(should_show_worktree(&worktree, true, None));
    }

    #[test]
    fn test_should_show_worktree_only_changes() {
        let clean_worktree = WorktreeInfo {
            name: "clean".to_string(),
            path: PathBuf::from("/tmp/clean"),
            branch: "clean".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let dirty_worktree = WorktreeInfo {
            name: "dirty".to_string(),
            path: PathBuf::from("/tmp/dirty"),
            branch: "dirty".to_string(),
            is_current: false,
            has_changes: true,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        assert!(!should_show_worktree(&clean_worktree, false, None));
        assert!(should_show_worktree(&dirty_worktree, false, None));
    }

    // Add 5 new tests for better coverage
    #[test]
    fn test_format_worktree_display_verbose_with_commit() {
        let test_commit_id = "abc123def";
        let test_path = "/tmp/feature";
        let worktree = WorktreeInfo {
            name: "feature".to_string(),
            path: PathBuf::from(test_path),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: Some(crate::infrastructure::git::CommitInfo {
                id: test_commit_id.to_string(),
                message: "Add feature".to_string(),
                author: "test@example.com".to_string(),
                time: "2023-01-01".to_string(),
            }),
            ahead_behind: None,
            is_locked: false,
        };

        let display = format_worktree_display(&worktree, true);
        assert!(display.contains(&format!("[{test_commit_id}]")));
        assert!(display.contains(&format!("- {test_path}")));
    }

    #[test]
    fn test_format_worktree_display_verbose_with_ahead_behind() {
        let ahead_count = 2;
        let behind_count = 3;
        let worktree = WorktreeInfo {
            name: "feature".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: Some((ahead_count, behind_count)),
            is_locked: false,
        };

        let display = format_worktree_display(&worktree, true);
        assert!(display.contains(&format!("↑{ahead_count} ↓{behind_count}")));
    }

    #[test]
    fn test_format_worktree_display_all_flags() {
        let worktree_name = "complex";
        let worktree = WorktreeInfo {
            name: worktree_name.to_string(),
            path: PathBuf::from("/tmp/complex"),
            branch: "complex".to_string(),
            is_current: true,
            has_changes: true,
            last_commit: None,
            ahead_behind: None,
            is_locked: true,
        };

        let display = format_worktree_display(&worktree, false);
        assert_eq!(
            display,
            format!("{worktree_name} (current) (locked) (changes)")
        );
    }

    #[test]
    fn test_should_show_worktree_empty_filter() {
        let worktree = WorktreeInfo {
            name: "any".to_string(),
            path: PathBuf::from("/tmp/any"),
            branch: "any".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        // Empty string filter should match anything
        assert!(should_show_worktree(&worktree, false, Some("")));
    }

    #[test]
    fn test_should_show_worktree_partial_filter() {
        let test_filters = vec!["auth", "feature", "login"];
        let no_match_filter = "ui";
        let worktree = WorktreeInfo {
            name: "feature-auth-login".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        // Partial matches should work
        for filter in test_filters {
            assert!(should_show_worktree(&worktree, false, Some(filter)));
        }
        assert!(!should_show_worktree(
            &worktree,
            false,
            Some(no_match_filter)
        ));
    }
}
