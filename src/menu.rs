//! Menu item definitions and display formatting
//!
//! This module defines the menu items available in the interactive interface
//! and their display representations. Each menu item corresponds to a specific
//! worktree management operation.
//!
//! # Design
//!
//! Menu items are represented as an enum to ensure type safety and make it
//! easy to add new operations. Each item has a consistent display format with
//! an icon prefix for visual clarity.

use crate::constants::*;
use std::fmt;

/// Available menu items in the interactive interface
///
/// Each variant represents a distinct operation that can be performed
/// on Git worktrees. The order of variants matches the typical workflow
/// and frequency of use.
///
/// # Display Format
///
/// Each menu item is displayed with:
/// - A unique icon/symbol prefix
/// - A descriptive label
/// - Consistent spacing for alignment
///
/// # Example
///
/// ```
/// use git_workers::menu::MenuItem;
///
/// let item = MenuItem::CreateWorktree;
/// println!("{}", item); // Output: "+  Create worktree"
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    /// List all worktrees with status information
    ListWorktrees,
    /// Search through worktrees using fuzzy matching
    SearchWorktrees,
    /// Create a new worktree
    CreateWorktree,
    /// Delete a single worktree
    DeleteWorktree,
    /// Delete multiple worktrees at once
    BatchDelete,
    /// Remove worktrees older than a specified age
    CleanupOldWorktrees,
    /// Switch to a different worktree (changes directory)
    SwitchWorktree,
    /// Rename an existing worktree
    RenameWorktree,
    /// Edit hooks configuration
    EditHooks,
    /// Exit the application
    Exit,
}

impl fmt::Display for MenuItem {
    /// Formats menu items with consistent icons and spacing
    ///
    /// Each menu item is formatted with a distinctive icon followed by
    /// two spaces and a descriptive label. This creates a visually
    /// appealing and easy-to-scan menu.
    ///
    /// # Icon Meanings
    ///
    /// - `•` List - Bullet point for viewing items
    /// - `?` Search - Question mark for queries
    /// - `+` Create - Plus sign for adding
    /// - `-` Delete - Minus sign for removing
    /// - `=` Batch - Equals sign for multiple items
    /// - `~` Cleanup - Tilde for maintenance tasks
    /// - `→` Switch - Arrow for navigation
    /// - `*` Rename - Asterisk for modification
    /// - `⚙` Settings - Gear for configuration
    /// - `x` Exit - X for closing
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::ListWorktrees => write!(f, "{MENU_LIST_WORKTREES}"),
            MenuItem::SearchWorktrees => write!(f, "{MENU_SEARCH_WORKTREES}"),
            MenuItem::CreateWorktree => write!(f, "{MENU_CREATE_WORKTREE}"),
            MenuItem::DeleteWorktree => write!(f, "{MENU_DELETE_WORKTREE}"),
            MenuItem::BatchDelete => write!(f, "{MENU_BATCH_DELETE}"),
            MenuItem::CleanupOldWorktrees => write!(f, "{MENU_CLEANUP_OLD}"),
            MenuItem::SwitchWorktree => write!(f, "{MENU_SWITCH_WORKTREE}"),
            MenuItem::RenameWorktree => write!(f, "{MENU_RENAME_WORKTREE}"),
            MenuItem::EditHooks => write!(f, "{MENU_EDIT_HOOKS}"),
            MenuItem::Exit => write!(f, "{MENU_EXIT}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_list_worktrees() {
        let item = MenuItem::ListWorktrees;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_LIST_WORKTREES));
    }

    #[test]
    fn test_fmt_search_worktrees() {
        let item = MenuItem::SearchWorktrees;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_SEARCH_WORKTREES));
    }

    #[test]
    fn test_fmt_create_worktree() {
        let item = MenuItem::CreateWorktree;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_CREATE_WORKTREE));
    }

    #[test]
    fn test_fmt_delete_worktree() {
        let item = MenuItem::DeleteWorktree;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_DELETE_WORKTREE));
    }

    #[test]
    fn test_fmt_batch_delete() {
        let item = MenuItem::BatchDelete;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_BATCH_DELETE));
    }

    #[test]
    fn test_fmt_cleanup_old() {
        let item = MenuItem::CleanupOldWorktrees;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_CLEANUP_OLD));
    }

    #[test]
    fn test_fmt_switch_worktree() {
        let item = MenuItem::SwitchWorktree;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_SWITCH_WORKTREE));
    }

    #[test]
    fn test_fmt_rename_worktree() {
        let item = MenuItem::RenameWorktree;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_RENAME_WORKTREE));
    }

    #[test]
    fn test_fmt_edit_hooks() {
        let item = MenuItem::EditHooks;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_EDIT_HOOKS));
    }

    #[test]
    fn test_fmt_exit() {
        let item = MenuItem::Exit;
        let formatted = format!("{item}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains(MENU_EXIT));
    }

    #[test]
    fn test_menu_item_enum_properties() {
        // Test that the enum variants are properly defined
        let items = [
            MenuItem::ListWorktrees,
            MenuItem::SearchWorktrees,
            MenuItem::CreateWorktree,
            MenuItem::DeleteWorktree,
            MenuItem::BatchDelete,
            MenuItem::CleanupOldWorktrees,
            MenuItem::SwitchWorktree,
            MenuItem::RenameWorktree,
            MenuItem::EditHooks,
            MenuItem::Exit,
        ];

        // Test that all items can be formatted without panic
        for item in items {
            let formatted = format!("{item}");
            assert!(
                !formatted.is_empty(),
                "Menu item should format to non-empty string"
            );
        }
    }

    #[test]
    fn test_menu_item_equality() {
        assert_eq!(MenuItem::CreateWorktree, MenuItem::CreateWorktree);
        assert_ne!(MenuItem::CreateWorktree, MenuItem::DeleteWorktree);
    }

    #[test]
    fn test_menu_item_clone() {
        let item = MenuItem::CreateWorktree;
        let cloned = item;
        assert_eq!(item, cloned);
    }
}
