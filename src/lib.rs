//! Git Workers - Interactive Git Worktree Manager
//!
//! Git Workers is a command-line tool that provides an interactive interface
//! for managing Git worktrees. It simplifies common worktree operations like
//! creation, deletion, switching, and searching.
//!
//! # Features
//!
//! - **Interactive Menu**: Navigate operations with an intuitive menu system
//! - **Shell Integration**: Automatically change directory when switching worktrees
//! - **Hook System**: Execute custom commands on worktree lifecycle events
//! - **Batch Operations**: Perform operations on multiple worktrees at once
//! - **Search Functionality**: Fuzzy search through worktrees
//! - **Rename Support**: Complete worktree renaming including Git metadata
//! - **Custom Paths**: Flexible worktree placement with validated custom paths
//! - **File Copying**: Automatically copy configured files to new worktrees
//!
//! # Architecture
//!
//! The library is organized into several modules:
//!
//! - [`commands`] - Command implementations for menu items
//! - [`config`] - Configuration file management
//! - [`git`] - Core Git operations and worktree management
//! - [`hooks`] - Hook system for custom commands
//! - [`menu`] - Menu item definitions
//! - [`repository_info`] - Repository context detection
//! - [`utils`] - Utility functions for terminal output
//! - [`input_esc_raw`] - Custom input handling with ESC key support
//!
//! # Usage Example
//!
//! ```no_run
//! use git_workers::git::GitWorktreeManager;
//! use git_workers::commands;
//!
//! // Create a worktree manager
//! let manager = GitWorktreeManager::new().expect("Failed to open repository");
//!
//! // List worktrees
//! let result = commands::list_worktrees();
//! ```

pub mod commands;
pub mod config;
pub mod constants;
pub mod file_copy;
pub mod git;
pub mod hooks;
pub mod input_esc_raw;
pub mod menu;
pub mod repository_info;
pub mod utils;
