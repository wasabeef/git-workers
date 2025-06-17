//! Menu item definitions and display formatting
//!
//! This module defines the menu items available in the interactive interface
//! and their display representations.

use std::fmt;

/// Available menu items in the interactive interface
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
    /// Exit the application
    Exit,
}

impl fmt::Display for MenuItem {
    /// Formats menu items with consistent icons and spacing
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::ListWorktrees => write!(f, "•  List worktrees"),
            MenuItem::SearchWorktrees => write!(f, "?  Search worktrees"),
            MenuItem::CreateWorktree => write!(f, "+  Create worktree"),
            MenuItem::DeleteWorktree => write!(f, "-  Delete worktree"),
            MenuItem::BatchDelete => write!(f, "=  Batch delete worktrees"),
            MenuItem::CleanupOldWorktrees => write!(f, "~  Cleanup old worktrees"),
            MenuItem::SwitchWorktree => write!(f, "→  Switch worktree"),
            MenuItem::RenameWorktree => write!(f, "*  Rename worktree"),
            MenuItem::Exit => write!(f, "x  Exit"),
        }
    }
}
