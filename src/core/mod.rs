//! Core business logic module
//!
//! This module contains the core business logic for git-workers,
//! independent of UI and infrastructure concerns.

pub mod validation;

// Re-export commonly used items
pub use validation::{validate_custom_path, validate_worktree_name};
