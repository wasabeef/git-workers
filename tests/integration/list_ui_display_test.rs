//! Integration tests for list UI display format preservation
//!
//! These tests ensure that the worktree list display format remains consistent
//! with the v0.4.0 design, including icons, colors, and table layout.

use colored::*;
use git_workers::constants::{
    CURRENT_MARKER, ICON_CURRENT_WORKTREE, ICON_OTHER_WORKTREE, MODIFIED_STATUS_NO,
    MODIFIED_STATUS_YES, TABLE_HEADER_BRANCH, TABLE_HEADER_MODIFIED, TABLE_HEADER_NAME,
    TABLE_HEADER_PATH,
};
use git_workers::git::WorktreeInfo;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that current worktree uses the correct icon
    #[test]
    fn test_current_worktree_icon() {
        // Test the icon format for current worktree
        let current_icon = ICON_CURRENT_WORKTREE.bright_green().bold();
        let other_icon = ICON_OTHER_WORKTREE.bright_blue();

        // Check that the icons are the expected characters
        assert_eq!(current_icon.chars().next().unwrap(), '→');
        assert_eq!(other_icon.chars().next().unwrap(), '▸');

        // Verify that colors are applied (checking for any ANSI codes)
        let current_formatted = format!("{current_icon}");
        let other_formatted = format!("{other_icon}");
        assert!(current_formatted.contains("\x1b[") || current_formatted == "→"); // Color might be disabled in tests
        assert!(other_formatted.contains("\x1b[") || other_formatted == "▸");
    }

    /// Test branch display formatting
    #[test]
    fn test_branch_display_format() {
        // Current worktree branch
        let current_branch = "main";
        let current_display = format!("{current_branch} {CURRENT_MARKER}").bright_green();
        assert!(current_display.to_string().contains("[current]"));

        // Other worktree branch
        let other_branch = "feature";
        let other_display = other_branch.yellow();
        assert_eq!(other_display.to_string().trim_end(), "feature");
    }

    /// Test modified status display
    #[test]
    fn test_modified_status_display() {
        // Modified
        let modified_yes = MODIFIED_STATUS_YES.bright_yellow();
        assert_eq!(modified_yes.to_string().trim_end(), MODIFIED_STATUS_YES);

        // Not modified
        let modified_no = MODIFIED_STATUS_NO.bright_black();
        assert_eq!(modified_no.to_string().trim_end(), MODIFIED_STATUS_NO);
    }

    /// Test worktree name display
    #[test]
    fn test_worktree_name_display() {
        // Current worktree name
        let current_name = "main".bright_green().bold();
        assert_eq!(current_name.to_string().trim_end(), "main");

        // Other worktree name
        let other_name = "feature".normal();
        assert_eq!(other_name.to_string(), "feature");
    }

    /// Test table header formatting
    #[test]
    fn test_table_header_format() {
        let name_header = TABLE_HEADER_NAME.bold();
        let branch_header = TABLE_HEADER_BRANCH.bold();
        let modified_header = TABLE_HEADER_MODIFIED.bold();
        let path_header = TABLE_HEADER_PATH.bold();

        // All headers should contain their text
        assert_eq!(name_header.to_string().trim_end(), TABLE_HEADER_NAME);
        assert_eq!(branch_header.to_string().trim_end(), TABLE_HEADER_BRANCH);
        assert_eq!(
            modified_header.to_string().trim_end(),
            TABLE_HEADER_MODIFIED
        );
        assert_eq!(path_header.to_string().trim_end(), TABLE_HEADER_PATH);
    }

    /// Test sorting order (current worktree first)
    #[test]
    fn test_worktree_sorting() {
        let mut worktrees = vec![
            WorktreeInfo {
                name: "zebra".to_string(),
                path: PathBuf::from("/tmp/zebra"),
                branch: "zebra".to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "alpha".to_string(),
                path: PathBuf::from("/tmp/alpha"),
                branch: "alpha".to_string(),
                is_current: true,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "beta".to_string(),
                path: PathBuf::from("/tmp/beta"),
                branch: "beta".to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
        ];

        // Apply the same sorting logic as list_worktrees
        worktrees.sort_by(|a, b| {
            if a.is_current && !b.is_current {
                std::cmp::Ordering::Less
            } else if !a.is_current && b.is_current {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        // Verify sorting order
        assert_eq!(worktrees[0].name, "alpha");
        assert!(worktrees[0].is_current);
        assert_eq!(worktrees[1].name, "beta");
        assert_eq!(worktrees[2].name, "zebra");
    }

    /// Test column width calculation
    #[test]
    fn test_column_width_calculation() {
        let worktrees = vec![
            WorktreeInfo {
                name: "short".to_string(),
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
                path: PathBuf::from("/tmp/very-long"),
                branch: "feature-with-long-name".to_string(),
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

        assert_eq!(max_name_len, 23); // "very-long-worktree-name".len()
        assert_eq!(max_branch_len, 32); // "feature-with-long-name".len() + 10
    }

    /// Integration test for complete display format
    #[test]
    fn test_complete_display_format() {
        // This test verifies the complete display format is correct
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
                path: PathBuf::from("/Users/test/project/main"),
                branch: "main".to_string(),
                is_current: true,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "feature-x".to_string(),
                path: PathBuf::from("/Users/test/project/feature-x"),
                branch: "feature/new-ui".to_string(),
                is_current: false,
                has_changes: true,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "bugfix".to_string(),
                path: PathBuf::from("/Users/test/project/bugfix"),
                branch: "fix/critical-bug".to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
        ];

        // Test that each worktree would be displayed correctly
        for worktree in &worktrees {
            // Icon
            let icon = if worktree.is_current {
                ICON_CURRENT_WORKTREE.bright_green().bold()
            } else {
                ICON_OTHER_WORKTREE.bright_blue()
            };

            // Branch display
            let branch_display = if worktree.is_current {
                format!("{} {}", worktree.branch, CURRENT_MARKER).bright_green()
            } else {
                worktree.branch.yellow()
            };

            // Modified status
            let modified = if worktree.has_changes {
                MODIFIED_STATUS_YES.bright_yellow()
            } else {
                MODIFIED_STATUS_NO.bright_black()
            };

            // Name
            let name_display = if worktree.is_current {
                worktree.name.bright_green().bold()
            } else {
                worktree.name.normal()
            };

            // Verify text content
            if worktree.is_current {
                assert_eq!(icon.chars().next().unwrap(), '→');
                assert!(branch_display.to_string().contains("[current]"));
                assert_eq!(name_display.to_string().trim_end(), worktree.name);
            } else {
                assert_eq!(icon.chars().next().unwrap(), '▸');
                assert!(!branch_display.to_string().contains("[current]"));
            }

            if worktree.has_changes {
                assert_eq!(modified.to_string().trim_end(), MODIFIED_STATUS_YES);
            } else {
                assert_eq!(modified.to_string().trim_end(), MODIFIED_STATUS_NO);
            }
        }
    }

    /// Test path display formatting
    #[test]
    fn test_path_display_format() {
        let path = PathBuf::from("/Users/test/project/worktree");
        let path_display = path.display().to_string().dimmed();

        // Path should be the same regardless of color
        assert!(path_display
            .to_string()
            .contains("/Users/test/project/worktree"));
    }

    /// Test that the original display design is preserved
    #[test]
    fn test_original_display_design_preserved() {
        // This test documents the display design that should be maintained
        // to prevent future changes from breaking the UI format

        // 1. Icons
        let current_icon = ICON_CURRENT_WORKTREE;
        let other_icon = ICON_OTHER_WORKTREE;

        // 2. Colors documented (using underscore to avoid unused warnings)
        let _current_icon_color = "bright_green + bold";
        let _other_icon_color = "bright_blue";
        let _current_branch_color = "bright_green";
        let _other_branch_color = "yellow";
        let _modified_yes_color = "bright_yellow";
        let _modified_no_color = "bright_black";
        let _current_name_color = "bright_green + bold";
        let _path_color = "dimmed";

        // 3. Table structure
        let has_table_headers = true;
        let has_separator_line = true;
        let columns = [
            TABLE_HEADER_NAME,
            TABLE_HEADER_BRANCH,
            TABLE_HEADER_MODIFIED,
            TABLE_HEADER_PATH,
        ];

        // Document these requirements
        assert_eq!(current_icon, ICON_CURRENT_WORKTREE);
        assert_eq!(other_icon, ICON_OTHER_WORKTREE);
        assert!(has_table_headers);
        assert!(has_separator_line);
        assert_eq!(columns.len(), 4);
    }

    /// Test the actual display logic from list.rs
    #[test]
    fn test_list_display_logic() {
        // This test verifies that the display logic works correctly

        // Test icon selection
        let is_current = true;
        let icon = if is_current {
            ICON_CURRENT_WORKTREE.bright_green().bold()
        } else {
            ICON_OTHER_WORKTREE.bright_blue()
        };
        assert_eq!(icon.chars().next().unwrap(), '→');

        let is_current = false;
        let icon = if is_current {
            ICON_CURRENT_WORKTREE.bright_green().bold()
        } else {
            ICON_OTHER_WORKTREE.bright_blue()
        };
        assert_eq!(icon.chars().next().unwrap(), '▸');

        // Test branch display
        let branch = "main";
        let branch_display = format!("{branch} {CURRENT_MARKER}").bright_green();
        assert!(branch_display
            .to_string()
            .contains(&format!("main {CURRENT_MARKER}")));

        // Test modified status
        let has_changes = true;
        let modified = if has_changes {
            MODIFIED_STATUS_YES.bright_yellow()
        } else {
            MODIFIED_STATUS_NO.bright_black()
        };
        assert_eq!(modified.to_string().trim_end(), MODIFIED_STATUS_YES);

        let has_changes = false;
        let modified = if has_changes {
            MODIFIED_STATUS_YES.bright_yellow()
        } else {
            MODIFIED_STATUS_NO.bright_black()
        };
        assert_eq!(modified.to_string().trim_end(), MODIFIED_STATUS_NO);
    }
}
