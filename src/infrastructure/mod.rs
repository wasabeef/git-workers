//! Infrastructure layer module
//!
//! This module contains all infrastructure concerns including:
//! - Git operations and repository management
//! - File system operations
//! - External process execution
//! - Hook system for lifecycle events

pub mod file_copy;
pub mod filesystem;
pub mod git;
pub mod hooks;

// Re-export commonly used items
pub use file_copy::copy_configured_files;
pub use filesystem::{FileSystem, RealFileSystem};
pub use git::{GitWorktreeManager, WorktreeInfo};
pub use hooks::{execute_hooks, HookContext};

// Re-export FilesConfig from config module
pub use super::config::FilesConfig;
