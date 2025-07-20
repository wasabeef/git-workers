//! Repository information display functionality
//!
//! This module provides functions to determine and format repository
//! information for display in the application header. It intelligently
//! detects the repository type and current context to provide meaningful
//! information to the user.
//!
//! # Repository Types
//!
//! The module handles several repository configurations:
//! - **Bare repositories**: Repositories without working directories
//! - **Main worktrees**: The primary working directory with worktrees
//! - **Worktrees**: Secondary working directories linked to a main repository
//! - **Standard repositories**: Regular Git repositories without worktrees
//! - **Non-Git directories**: Fallback for directories outside Git control

use crate::constants::{GIT_COMMONDIR_FILE, MAIN_SUFFIX, UNKNOWN_VALUE};
use crate::git::GitWorktreeManager;
use std::env;

/// Gets a formatted string representing the current repository context
///
/// This function determines whether we're in a bare repository, a worktree,
/// or the main working directory and formats the information accordingly.
///
/// # Return Format
///
/// - For bare repositories: `"repo-name.bare"`
/// - For worktrees: `"parent-repo (worktree-name)"`
/// - For main worktree: `"repo-name (main)"`
/// - For non-git directories: `"directory-name"`
///
/// # Example Output
///
/// ```text
/// my-project.bare              // Bare repository
/// my-project (feature-branch)  // Worktree
/// my-project (main)           // Main worktree with other worktrees
/// my-project                  // Regular repository without worktrees
/// ```
pub fn get_repository_info() -> String {
    // Try to get Git repository information
    if let Ok(manager) = GitWorktreeManager::new() {
        let repo = manager.repo();
        let current_dir = env::current_dir().unwrap_or_else(|_| UNKNOWN_VALUE.into());

        if repo.is_bare() {
            // For bare repository, show the bare repo name
            // Extract just the directory name from the full path
            let bare_name = repo
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(UNKNOWN_VALUE);
            bare_name.to_string()
        } else {
            // For worktree, get the main repository path from git config
            let worktree_name = current_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(UNKNOWN_VALUE);

            // Check if we're in a worktree by looking for .git file (not directory)
            let dot_git_path = current_dir.join(".git");
            if dot_git_path.is_file() {
                // This is a worktree - read the .git file to get the actual git directory
                if let Ok(content) = std::fs::read_to_string(&dot_git_path) {
                    // The .git file contains "gitdir: /path/to/main/.git/worktrees/name"
                    if let Some(gitdir_path) = content.strip_prefix("gitdir: ") {
                        let gitdir = std::path::PathBuf::from(gitdir_path.trim());
                        // Check for commondir in the actual git directory
                        let commondir_path = gitdir.join(GIT_COMMONDIR_FILE);
                        if commondir_path.exists() {
                            if let Ok(commondir_content) = std::fs::read_to_string(&commondir_path)
                            {
                                // commondir contains a relative path like "../.."
                                let common_dir = gitdir.join(commondir_content.trim());

                                // Normalize the path to resolve .. components
                                if let Ok(normalized_common_dir) = common_dir.canonicalize() {
                                    // Extract the parent repository name
                                    if let Some(parent_name) = normalized_common_dir
                                        .parent()
                                        .and_then(|p| p.file_name())
                                        .and_then(|name| name.to_str())
                                    {
                                        return format!("{parent_name} ({worktree_name})");
                                    }
                                }
                            }
                        }
                    }
                }
            } else if repo
                .path()
                .join(crate::constants::WORKTREES_SUBDIR)
                .exists()
            {
                // This is likely the main worktree of a repository with worktrees
                // The presence of a "worktrees" directory indicates this is a main repo
                if let Some(repo_path) = repo.workdir() {
                    if let Some(repo_name) = repo_path.file_name().and_then(|name| name.to_str()) {
                        return format!("{repo_name}{MAIN_SUFFIX}");
                    }
                }
            }

            // Default: just show worktree name
            // This handles regular repositories without worktrees
            worktree_name.to_string()
        }
    } else {
        // Not in a git repository
        // Fall back to showing the current directory name
        let current_dir = env::current_dir().unwrap_or_else(|_| UNKNOWN_VALUE.into());
        current_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(UNKNOWN_VALUE)
            .to_string()
    }
}
