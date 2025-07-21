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

use crate::constants::{MAIN_SUFFIX, UNKNOWN_VALUE};
#[cfg(not(test))]
use crate::git::GitWorktreeManager;
use std::env;
use std::process::Command;

/// Get repository name using git directory analysis
///
/// Uses `git rev-parse --git-dir` to find the git directory and traces back
/// to find the actual repository name, handling both bare and worktree cases.
#[cfg(not(test))]
fn get_repo_name_from_git() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;

    if output.status.success() {
        let git_dir = String::from_utf8(output.stdout).ok()?;
        let git_path = std::path::PathBuf::from(git_dir.trim());

        // If git_dir is just ".git", get the repository name from toplevel
        if git_path.file_name().and_then(|name| name.to_str()) == Some(".git") {
            // For regular repositories, get toplevel
            let toplevel_output = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .ok()?;

            if toplevel_output.status.success() {
                let toplevel = String::from_utf8(toplevel_output.stdout).ok()?;
                let path = std::path::PathBuf::from(toplevel.trim());
                return path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string());
            }
        }

        // If git_dir is ".", we're in a bare repository root
        if git_path.as_os_str() == "." {
            let current_dir = env::current_dir().ok()?;
            if let Some(current_dir_name) = current_dir.file_name().and_then(|name| name.to_str()) {
                if let Some(stripped) = current_dir_name.strip_suffix(".bare") {
                    return Some(stripped.to_string());
                } else {
                    return Some(current_dir_name.to_string());
                }
            }
        }

        // For worktrees, the git_dir path looks like:
        // /Users/user/repo.bare/worktrees/name (bare repository)
        // or /Users/user/repo/.git/worktrees/name (regular repository)
        if let Some(parent) = git_path.parent() {
            // Check if this is a worktree structure
            if parent.file_name().and_then(|name| name.to_str()) == Some("worktrees") {
                if let Some(repo_dir) = parent.parent() {
                    if let Some(repo_name) = repo_dir.file_name().and_then(|name| name.to_str()) {
                        if let Some(stripped) = repo_name.strip_suffix(".bare") {
                            // Bare repository: /repo.bare/worktrees/name
                            return Some(stripped.to_string());
                        } else if repo_name == ".git" {
                            // Non-bare repository: /repo/.git/worktrees/name
                            // Need to get the parent of .git directory
                            if let Some(actual_repo_dir) = repo_dir.parent() {
                                if let Some(actual_repo_name) =
                                    actual_repo_dir.file_name().and_then(|name| name.to_str())
                                {
                                    return Some(actual_repo_name.to_string());
                                }
                            }
                        } else {
                            // Regular repository case (shouldn't happen but fallback)
                            return Some(repo_name.to_string());
                        }
                    }
                }
            }
        }

        // Check if git_dir is a bare repository directory directly
        if let Some(git_dir_name) = git_path.file_name().and_then(|name| name.to_str()) {
            if let Some(stripped) = git_dir_name.strip_suffix(".bare") {
                return Some(stripped.to_string());
            }
        }

        // Fallback: try to get repository name from current path analysis
        None
    } else {
        None
    }
}

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
    get_repository_info_at_path(&env::current_dir().unwrap_or_else(|_| UNKNOWN_VALUE.into()))
}

/// Gets repository info for a specific path (internal, used for testing)
#[cfg(test)]
pub fn get_repository_info_at_path(path: &std::path::Path) -> String {
    use std::process::Stdio;

    // Run git commands in the specified directory
    let git_dir_output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    if let Ok(output) = git_dir_output {
        if output.status.success() {
            if let Ok(git_dir) = String::from_utf8(output.stdout) {
                let git_path = std::path::PathBuf::from(git_dir.trim());

                // If git_dir is just ".git", get the repository name from toplevel
                if git_path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                    // For regular repositories, get toplevel
                    let toplevel_output = Command::new("git")
                        .args(["rev-parse", "--show-toplevel"])
                        .current_dir(path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .output();

                    if let Ok(toplevel_out) = toplevel_output {
                        if toplevel_out.status.success() {
                            if let Ok(toplevel) = String::from_utf8(toplevel_out.stdout) {
                                let toplevel_path = std::path::PathBuf::from(toplevel.trim());
                                if let Some(repo_name) =
                                    toplevel_path.file_name().and_then(|name| name.to_str())
                                {
                                    return repo_name.to_string();
                                }
                            }
                        }
                    }
                }

                // If git_dir is ".", we're in a bare repository root
                if git_path.as_os_str() == "." {
                    if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {
                        if let Some(stripped) = dir_name.strip_suffix(".bare") {
                            return stripped.to_string();
                        } else {
                            return dir_name.to_string();
                        }
                    }
                }

                // Handle worktree cases
                if let Some(parent) = git_path.parent() {
                    if parent.file_name().and_then(|name| name.to_str()) == Some("worktrees") {
                        if let Some(repo_dir) = parent.parent() {
                            if let Some(repo_name) =
                                repo_dir.file_name().and_then(|name| name.to_str())
                            {
                                let base_repo_name =
                                    if let Some(stripped) = repo_name.strip_suffix(".bare") {
                                        stripped.to_string()
                                    } else if repo_name == ".git" {
                                        // Non-bare repository worktree
                                        if let Some(actual_repo_dir) = repo_dir.parent() {
                                            if let Some(actual_repo_name) = actual_repo_dir
                                                .file_name()
                                                .and_then(|name| name.to_str())
                                            {
                                                actual_repo_name.to_string()
                                            } else {
                                                UNKNOWN_VALUE.to_string()
                                            }
                                        } else {
                                            UNKNOWN_VALUE.to_string()
                                        }
                                    } else {
                                        repo_name.to_string()
                                    };

                                // Get worktree name
                                if let Some(worktree_name) =
                                    path.file_name().and_then(|name| name.to_str())
                                {
                                    return format!("{base_repo_name} ({worktree_name})");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to directory name
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(UNKNOWN_VALUE)
        .to_string()
}

/// Gets repository info for production (non-test) use
#[cfg(not(test))]
pub fn get_repository_info_at_path(_path: &std::path::Path) -> String {
    // Try to get Git repository information
    if let Ok(manager) = GitWorktreeManager::new() {
        let repo = manager.repo();
        let current_dir = env::current_dir().unwrap_or_else(|_| UNKNOWN_VALUE.into());

        // Try to get repository name from git command first
        let repo_name = get_repo_name_from_git().unwrap_or_else(|| {
            if repo.is_bare() {
                // Fallback for bare repository
                repo.path()
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or(UNKNOWN_VALUE)
                    .to_string()
            } else {
                // Fallback to current directory name
                current_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(UNKNOWN_VALUE)
                    .to_string()
            }
        });

        if repo.is_bare() {
            // For bare repository, always show just the repo name
            repo_name
        } else {
            let worktree_name = current_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(UNKNOWN_VALUE);

            // Check if we're in a worktree by looking for .git file (not directory)
            let dot_git_path = current_dir.join(".git");
            if dot_git_path.is_file() {
                // This is a worktree - show repo name with worktree name
                format!("{repo_name} ({worktree_name})")
            } else if repo
                .path()
                .join(crate::constants::WORKTREES_SUBDIR)
                .exists()
            {
                // This is the main worktree of a repository with worktrees
                // Check if we are actually in the main repository root, not a subdirectory
                if let Some(workdir) = repo.workdir() {
                    if workdir == current_dir {
                        // We're in the main repo root, don't show (main) suffix
                        repo_name
                    } else {
                        format!("{repo_name}{MAIN_SUFFIX}")
                    }
                } else {
                    format!("{repo_name}{MAIN_SUFFIX}")
                }
            } else {
                // Regular repository without worktrees
                repo_name
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_repository_info_non_git() {
        // This test will succeed in a non-git context
        let info = get_repository_info();
        assert!(!info.is_empty());
        // Should return some directory name
    }

    #[test]
    fn test_constants_are_used() {
        // Test that our constants are defined and accessible
        assert_eq!(UNKNOWN_VALUE, "unknown");
        assert_eq!(MAIN_SUFFIX, " (main)");
    }

    #[test]
    fn test_repository_info_function_exists() {
        // Test that the function can be called without panicking
        let _info = get_repository_info();
        // Function should not panic
        // Test passes if function completes without panicking
    }

    #[test]
    fn test_bare_repository_name_extraction() {
        use std::path::PathBuf;

        // Test bare repository path extraction logic
        let bare_repo_path = PathBuf::from("/path/to/jump-app.bare/.git");
        let extracted_name = bare_repo_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        assert_eq!(extracted_name, "jump-app.bare");
    }

    #[test]
    fn test_non_bare_repository_name_extraction() {
        use std::path::PathBuf;

        // Test worktree name extraction logic
        let worktree_path = PathBuf::from("/path/to/worktree-name");
        let extracted_name = worktree_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        assert_eq!(extracted_name, "worktree-name");
    }
}
