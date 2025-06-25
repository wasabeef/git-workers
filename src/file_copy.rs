//! File copy functionality for worktree creation
//!
//! This module provides functionality to copy files from the main worktree
//! to newly created worktrees. This is particularly useful for files that
//! are gitignored but necessary for the project to function.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::FilesConfig;
use crate::git::GitWorktreeManager;

/// Copies configured files from source to destination worktree
///
/// This function handles the file copying logic with proper error handling
/// and security checks to prevent directory traversal attacks.
///
/// # Arguments
///
/// * `config` - The files configuration specifying what to copy
/// * `destination_path` - The path to the newly created worktree
/// * `manager` - Git worktree manager for accessing repository information
///
/// # Returns
///
/// * `Ok(Vec<String>)` - List of successfully copied files
/// * `Err(...)` - Error if critical failure occurs
///
/// # Security
///
/// This function validates all paths to ensure they don't escape the
/// repository boundaries using directory traversal techniques.
pub fn copy_configured_files(
    config: &FilesConfig,
    destination_path: &Path,
    manager: &GitWorktreeManager,
) -> Result<Vec<String>> {
    if config.copy.is_empty() {
        return Ok(Vec::new());
    }

    // Determine source directory
    let source_dir = determine_source_directory(config, manager)?;

    // Check for circular reference only if source contains destination
    // (destination containing source is OK, as it's common for worktrees)
    let source_canonical = source_dir
        .canonicalize()
        .unwrap_or_else(|_| source_dir.clone());
    let dest_canonical = destination_path
        .canonicalize()
        .unwrap_or_else(|_| destination_path.to_path_buf());

    if source_canonical == dest_canonical {
        return Err(anyhow!("Source and destination are the same directory"));
    }

    let mut copied_files = Vec::new();
    let mut warnings = Vec::new();

    for file_pattern in &config.copy {
        // Validate path to prevent directory traversal
        if !is_safe_path(file_pattern) {
            warnings.push(format!("Skipping unsafe path: {}", file_pattern));
            continue;
        }

        let source_path = source_dir.join(file_pattern);
        let dest_path = destination_path.join(file_pattern);

        match copy_file_or_directory(&source_path, &dest_path) {
            Ok(()) => {
                copied_files.push(file_pattern.clone());
            }
            Err(e) => {
                // Log warning but continue with other files
                warnings.push(format!("Failed to copy {}: {}", file_pattern, e));
            }
        }
    }

    // Print warnings if any
    for warning in warnings {
        eprintln!("Warning: {}", warning);
    }

    Ok(copied_files)
}

/// Determines the source directory for file copying
///
/// If a source is specified in the config, it uses that.
/// Otherwise, it attempts to find the main worktree directory.
fn determine_source_directory(
    config: &FilesConfig,
    manager: &GitWorktreeManager,
) -> Result<PathBuf> {
    if let Some(source) = &config.source {
        // Use configured source
        let path = PathBuf::from(source);
        if path.is_absolute() {
            Ok(path)
        } else {
            // Make relative to repository
            manager
                .repo()
                .workdir()
                .map(|w| w.join(source))
                .ok_or_else(|| anyhow!("Cannot determine repository working directory"))
        }
    } else {
        // Find main worktree
        find_main_worktree(manager)
    }
}

/// Finds the source directory for file copying
///
/// Uses the same priority as .git-workers.toml discovery:
/// - For bare repositories: follows the same search pattern
/// - For non-bare repositories: follows the same search pattern
fn find_main_worktree(manager: &GitWorktreeManager) -> Result<PathBuf> {
    let repo = manager.repo();

    if repo.is_bare() {
        // For bare repositories - same logic as config file search
        find_source_in_bare_repo(repo)
    } else {
        // For non-bare repositories - same logic as config file search
        find_source_in_regular_repo(repo)
    }
}

/// Finds source directory in bare repository
/// Following the same priority as config file search
fn find_source_in_bare_repo(repo: &git2::Repository) -> Result<PathBuf> {
    // Get the default branch name from HEAD
    let default_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "main".to_string());

    if let Ok(cwd) = std::env::current_dir() {
        // 1. First check current directory
        if cwd.join(".git-workers.toml").exists() {
            return Ok(cwd);
        }

        // 2. Check default branch directory in current directory
        let default_in_current = cwd.join(&default_branch);
        if default_in_current.exists() && default_in_current.is_dir() {
            return Ok(default_in_current);
        }

        // Also check main/master if different from default
        if default_branch != "main" {
            let main_dir = cwd.join("main");
            if main_dir.exists() && main_dir.is_dir() {
                return Ok(main_dir);
            }
        }
        if default_branch != "master" {
            let master_dir = cwd.join("master");
            if master_dir.exists() && master_dir.is_dir() {
                return Ok(master_dir);
            }
        }

        // 3. Try to detect worktree pattern
        if let Ok(output) = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&cwd)
            .output()
        {
            let worktree_paths = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| {
                    if line.starts_with("worktree ") {
                        Some(line.trim_start_matches("worktree ").to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !worktree_paths.is_empty() {
                let parent_dirs: Vec<_> = worktree_paths
                    .iter()
                    .filter_map(|p| Path::new(p).parent())
                    .collect();

                if let Some(first_parent) = parent_dirs.first() {
                    if parent_dirs.iter().all(|p| p == first_parent) {
                        let default_dir = first_parent.join(&default_branch);
                        if default_dir.exists() && default_dir.is_dir() {
                            return Ok(default_dir);
                        }

                        // Fallback to main/master
                        if default_branch != "main" {
                            let main_dir = first_parent.join("main");
                            if main_dir.exists() && main_dir.is_dir() {
                                return Ok(main_dir);
                            }
                        }
                        if default_branch != "master" {
                            let master_dir = first_parent.join("master");
                            if master_dir.exists() && master_dir.is_dir() {
                                return Ok(master_dir);
                            }
                        }
                    }
                }
            }
        }

        // 4. Fallback: Check common subdirectories
        for subdir in &["branch", "worktrees"] {
            let branch_dir = cwd.join(subdir).join(&default_branch);
            if branch_dir.exists() && branch_dir.is_dir() {
                return Ok(branch_dir);
            }
        }

        // 5. Check sibling directories
        if let Some(parent) = cwd.parent() {
            let default_dir = parent.join(&default_branch);
            if default_dir.exists() && default_dir.is_dir() {
                return Ok(default_dir);
            }
        }
    }

    Err(anyhow!(
        "Could not find source directory for file copying in bare repository"
    ))
}

/// Finds source directory in non-bare repository
/// Following the same priority as config file search
fn find_source_in_regular_repo(repo: &git2::Repository) -> Result<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        // 1. First check current directory
        if cwd.join(".git-workers.toml").exists() {
            return Ok(cwd);
        }

        // 2. Check if this is the main worktree
        if let Some(workdir) = repo.workdir() {
            let workdir_path = workdir.to_path_buf();

            // Check if current directory is the main worktree
            if cwd == workdir_path {
                return Ok(workdir_path);
            }

            // If not, check if the main worktree exists
            let git_path = workdir_path.join(".git");
            if git_path.is_dir() && workdir_path.exists() {
                return Ok(workdir_path);
            }
        }

        // 3. Look for main/master in parent directories
        if let Some(parent) = cwd.parent() {
            if parent.file_name().is_some_and(|n| n == "worktrees") {
                // We're in worktrees subdirectory
                if let Some(repo_root) = parent.parent() {
                    if repo_root.join(".git").is_dir() {
                        return Ok(repo_root.to_path_buf());
                    }

                    // Check main/master subdirectories
                    let main_dir = repo_root.join("main");
                    if main_dir.exists() && main_dir.is_dir() {
                        return Ok(main_dir);
                    }

                    let master_dir = repo_root.join("master");
                    if master_dir.exists() && master_dir.is_dir() {
                        return Ok(master_dir);
                    }
                }
            } else {
                // Check parent for main/master
                let main_dir = parent.join("main");
                if main_dir.exists() && main_dir.is_dir() {
                    return Ok(main_dir);
                }

                let master_dir = parent.join("master");
                if master_dir.exists() && master_dir.is_dir() {
                    return Ok(master_dir);
                }
            }
        }
    }

    // Final fallback: use repository working directory
    repo.workdir()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("No working directory found"))
}

/// Validates that a path is safe and doesn't use directory traversal
///
/// Rejects paths containing:
/// - `..` (parent directory)
/// - Absolute paths
/// - Paths starting with `/` or `~`
fn is_safe_path(path: &str) -> bool {
    !path.contains("..")
        && !path.starts_with('/')
        && !path.starts_with('~')
        && !Path::new(path).is_absolute()
}

/// Copies a file or directory from source to destination
///
/// Creates parent directories as needed.
/// For directories, copies recursively.
fn copy_file_or_directory(source: &Path, dest: &Path) -> Result<()> {
    if !source.exists() {
        return Err(anyhow!("Source does not exist: {}", source.display()));
    }

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    if source.is_file() {
        // Check if source is a symlink
        if source.symlink_metadata()?.is_symlink() {
            eprintln!("Warning: Skipping symbolic link: {}", source.display());
            return Ok(());
        }
        fs::copy(source, dest).with_context(|| {
            format!(
                "Failed to copy file from {} to {}",
                source.display(),
                dest.display()
            )
        })?;
    } else if source.is_dir() {
        copy_directory_recursive(source, dest)?;
    }

    Ok(())
}

/// Maximum directory depth to prevent infinite recursion
const MAX_DIRECTORY_DEPTH: usize = 50;

/// Recursively copies a directory
fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
    copy_directory_recursive_impl(source, dest, 0)
}

/// Internal implementation with depth tracking
fn copy_directory_recursive_impl(source: &Path, dest: &Path, depth: usize) -> Result<()> {
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(anyhow!(
            "Directory nesting too deep (> {} levels): {}",
            MAX_DIRECTORY_DEPTH,
            source.display()
        ));
    }
    fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create directory: {}", dest.display()))?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if file_type.is_file() {
            fs::copy(&source_path, &dest_path)
                .with_context(|| format!("Failed to copy file: {}", source_path.display()))?;
        } else if file_type.is_dir() {
            copy_directory_recursive_impl(&source_path, &dest_path, depth + 1)?;
        } else if file_type.is_symlink() {
            // Warn about skipping symlinks
            eprintln!("Warning: Skipping symbolic link: {}", source_path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_path() {
        // Safe paths
        assert!(is_safe_path(".env"));
        assert!(is_safe_path("config/local.json"));
        assert!(is_safe_path("deeply/nested/file.txt"));

        // Unsafe paths
        assert!(!is_safe_path("../../../etc/passwd"));
        assert!(!is_safe_path("/etc/passwd"));
        assert!(!is_safe_path("~/sensitive"));
        assert!(!is_safe_path("some/../../../path"));
    }
}
