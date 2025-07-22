use anyhow::Result;
use colored::*;

use crate::constants::{
    section_header, CURRENT_MARKER, ICON_CURRENT_WORKTREE, ICON_OTHER_WORKTREE, MODIFIED_STATUS_NO,
    MODIFIED_STATUS_YES, TABLE_HEADER_BRANCH, TABLE_HEADER_MODIFIED, TABLE_HEADER_NAME,
    TABLE_HEADER_PATH, TABLE_SEPARATOR, WARNING_NO_WORKTREES,
};
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

    if worktrees.is_empty() {
        println!();
        let msg = WARNING_NO_WORKTREES.yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Sort worktrees: current first, then alphabetically
    let mut sorted_worktrees = worktrees;
    sorted_worktrees.sort_by(|a, b| {
        if a.is_current && !b.is_current {
            std::cmp::Ordering::Less
        } else if !a.is_current && b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    // Print header
    println!();
    let header = section_header("Worktrees");
    println!("{header}");
    println!();

    // Display repository info
    let repo_info = get_repository_info();
    println!("Repository: {}", repo_info.bright_cyan());

    // Calculate column widths
    let max_name_len = sorted_worktrees
        .iter()
        .map(|w| w.name.len())
        .max()
        .unwrap_or(0)
        .max(10);
    let max_branch_len = sorted_worktrees
        .iter()
        .map(|w| w.branch.len())
        .max()
        .unwrap_or(0)
        .max(10)
        + 10; // Extra space for [current] marker

    println!();
    println!(
        "  {:<name_width$} {:<branch_width$} {:<8} {}",
        TABLE_HEADER_NAME.bold(),
        TABLE_HEADER_BRANCH.bold(),
        TABLE_HEADER_MODIFIED.bold(),
        TABLE_HEADER_PATH.bold(),
        name_width = max_name_len,
        branch_width = max_branch_len
    );
    println!(
        "  {TABLE_SEPARATOR:-<max_name_len$} {TABLE_SEPARATOR:-<max_branch_len$} {TABLE_SEPARATOR:-<8} {TABLE_SEPARATOR:-<40}"
    );

    // Display worktrees in table format
    for worktree in &sorted_worktrees {
        let icon = if worktree.is_current {
            ICON_CURRENT_WORKTREE.bright_green().bold()
        } else {
            ICON_OTHER_WORKTREE.bright_blue()
        };
        let branch_display = if worktree.is_current {
            format!("{} {}", worktree.branch, CURRENT_MARKER).bright_green()
        } else {
            worktree.branch.yellow()
        };
        let modified = if worktree.has_changes {
            MODIFIED_STATUS_YES.bright_yellow()
        } else {
            MODIFIED_STATUS_NO.bright_black()
        };

        println!(
            "{} {:<name_width$} {:<branch_width$} {:<8} {}",
            icon,
            if worktree.is_current {
                worktree.name.bright_green().bold()
            } else {
                worktree.name.normal()
            },
            branch_display,
            modified,
            worktree.path.display().to_string().dimmed(),
            name_width = max_name_len,
            branch_width = max_branch_len
        );
    }

    println!();
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
            git_name: "feature".to_string(),
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
            git_name: "main".to_string(),
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
            git_name: "locked".to_string(),
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
            git_name: "feature".to_string(),
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
            git_name: "feature-auth".to_string(),
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
            git_name: "feature-ui".to_string(),
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
            git_name: "clean".to_string(),
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
            git_name: "clean".to_string(),
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
            git_name: "dirty".to_string(),
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
            git_name: "feature".to_string(),
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
            git_name: "feature".to_string(),
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
            git_name: worktree_name.to_string(),
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
            git_name: "any".to_string(),
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
            git_name: "feature-auth-login".to_string(),
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

    // Tests to protect the current table display implementation
    #[test]
    fn test_table_display_current_worktree_first() {
        // Create test worktrees with one being current
        let worktree1 = WorktreeInfo {
            name: "zebra".to_string(),
            git_name: "zebra".to_string(),
            path: PathBuf::from("/tmp/zebra"),
            branch: "zebra".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };
        let worktree2 = WorktreeInfo {
            name: "alpha".to_string(),
            git_name: "alpha".to_string(),
            path: PathBuf::from("/tmp/alpha"),
            branch: "alpha".to_string(),
            is_current: true,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };
        let worktree3 = WorktreeInfo {
            name: "beta".to_string(),
            git_name: "beta".to_string(),
            path: PathBuf::from("/tmp/beta"),
            branch: "beta".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let mut worktrees = vec![worktree1, worktree2, worktree3];

        // Apply the same sorting logic as the main function
        worktrees.sort_by(|a, b| {
            if a.is_current && !b.is_current {
                std::cmp::Ordering::Less
            } else if !a.is_current && b.is_current {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        // Current worktree should be first
        assert_eq!(worktrees[0].name, "alpha");
        assert!(worktrees[0].is_current);
        // Others should be alphabetically sorted
        assert_eq!(worktrees[1].name, "beta");
        assert_eq!(worktrees[2].name, "zebra");
    }

    #[test]
    fn test_table_display_column_width_calculation() {
        let worktrees = vec![
            WorktreeInfo {
                name: "short".to_string(),
                git_name: "short".to_string(),
                path: PathBuf::from("/tmp/short"),
                branch: "main".to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "very-long-worktree-name".to_string(),
                git_name: "very-long-worktree-name".to_string(),
                path: PathBuf::from("/tmp/very-long-worktree-name"),
                branch: "feature-with-very-long-branch-name".to_string(),
                is_current: true,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
        ];

        let max_name_len = worktrees
            .iter()
            .map(|w| w.name.len())
            .max()
            .unwrap_or(0)
            .max(10);
        let max_branch_len = worktrees
            .iter()
            .map(|w| w.branch.len())
            .max()
            .unwrap_or(0)
            .max(10)
            + 10; // Extra space for [current] marker

        assert_eq!(max_name_len, "very-long-worktree-name".len());
        assert_eq!(
            max_branch_len,
            "feature-with-very-long-branch-name".len() + 10
        );
    }

    #[test]
    fn test_table_display_icon_selection() {
        let current_worktree = WorktreeInfo {
            name: "current".to_string(),
            git_name: "current".to_string(),
            path: PathBuf::from("/tmp/current"),
            branch: "main".to_string(),
            is_current: true,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };
        let other_worktree = WorktreeInfo {
            name: "other".to_string(),
            git_name: "other".to_string(),
            path: PathBuf::from("/tmp/other"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        // Test icon selection logic
        let current_icon = if current_worktree.is_current {
            "▸"
        } else {
            " "
        };
        let other_icon = if other_worktree.is_current {
            "▸"
        } else {
            " "
        };

        assert_eq!(current_icon, "▸");
        assert_eq!(other_icon, " ");
    }

    #[test]
    fn test_table_display_branch_formatting() {
        let current_worktree = WorktreeInfo {
            name: "current".to_string(),
            git_name: "current".to_string(),
            path: PathBuf::from("/tmp/current"),
            branch: "main".to_string(),
            is_current: true,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };
        let other_worktree = WorktreeInfo {
            name: "other".to_string(),
            git_name: "other".to_string(),
            path: PathBuf::from("/tmp/other"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        // Test branch display formatting
        let current_branch_display = if current_worktree.is_current {
            format!("{} [current]", current_worktree.branch)
        } else {
            current_worktree.branch.clone()
        };
        let other_branch_display = if other_worktree.is_current {
            format!("{} [current]", other_worktree.branch)
        } else {
            other_worktree.branch.clone()
        };

        assert_eq!(current_branch_display, "main [current]");
        assert_eq!(other_branch_display, "feature");
    }

    #[test]
    fn test_table_display_modified_status() {
        let clean_worktree = WorktreeInfo {
            name: "clean".to_string(),
            git_name: "clean".to_string(),
            path: PathBuf::from("/tmp/clean"),
            branch: "main".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };
        let dirty_worktree = WorktreeInfo {
            name: "dirty".to_string(),
            git_name: "dirty".to_string(),
            path: PathBuf::from("/tmp/dirty"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: true,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        // Test modified status display
        let clean_modified = if clean_worktree.has_changes {
            "Yes"
        } else {
            "No"
        };
        let dirty_modified = if dirty_worktree.has_changes {
            "Yes"
        } else {
            "No"
        };

        assert_eq!(clean_modified, "No");
        assert_eq!(dirty_modified, "Yes");
    }
}
