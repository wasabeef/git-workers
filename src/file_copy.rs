//! File copy functionality for worktree creation
//!
//! This module provides functionality to copy files from the main worktree
//! to newly created worktrees. This is particularly useful for files that
//! are gitignored but necessary for the project to function.

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::FilesConfig;
use crate::constants::{
    BYTES_PER_MB, COLON_POSITION_WINDOWS, GIT_DIR, ICON_ERROR, ICON_FILE, ICON_INFO, ICON_SUCCESS,
    ICON_WARNING, MAX_DIRECTORY_DEPTH, MAX_FILE_SIZE_MB, WINDOWS_PATH_MIN_LENGTH, WORKTREES_SUBDIR,
};
use crate::git::{GitWorktreeManager, WorktreeInfo};

/// Copies configured files from source to destination worktree
///
/// This function handles the file copying logic with proper error handling,
/// size limits, and security checks to prevent directory traversal attacks.
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
/// # File Size Limits
///
/// - Individual files larger than MAX_FILE_SIZE_MB are automatically skipped with a warning
/// - This prevents accidentally copying large binary files or build artifacts
///
/// # Security
///
/// This function validates all paths to ensure they don't escape the
/// repository boundaries using directory traversal techniques. Additionally:
/// - Symlinks are detected and skipped to prevent security issues
/// - Circular references are detected and prevented
/// - Maximum directory depth is enforced to prevent infinite recursion
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

    let msg = format!("{ICON_FILE} Copying configured files...").bright_cyan();
    println!("\n{msg}");

    for file_pattern in &config.copy {
        if !is_safe_path(file_pattern) {
            let warning = ICON_WARNING.yellow();
            let pattern = file_pattern.yellow();
            println!("  {warning} Skipping unsafe path: {pattern}");
            continue;
        }

        let source_path = source_dir.join(file_pattern);

        // Check file size before copying
        if source_path.exists() {
            if let Ok(size) = calculate_path_size(&source_path) {
                if size > MAX_FILE_SIZE && source_path.is_file() {
                    let warning = ICON_WARNING.yellow();
                    let pattern = file_pattern.yellow();
                    let size_mb = size as f64 / BYTES_PER_MB as f64;
                    println!("  {warning} Skipping large file: {pattern} ({size_mb:.1} MB)");
                    continue;
                }
            }
        }
        let dest_path = destination_path.join(file_pattern);

        match copy_file_or_directory(&source_path, &dest_path) {
            Ok(count) => {
                if count > 0 {
                    let checkmark = ICON_SUCCESS.green();
                    let pattern = file_pattern.green();
                    let plural = if count == 1 { "" } else { "s" };
                    println!("  {checkmark} Copied: {pattern} ({count} file{plural})");
                    copied_files.push(file_pattern.clone());
                }
            }
            Err(e) => {
                // Check if it's a "not found" error
                if e.to_string().contains("No such file or directory")
                    || e.to_string().contains("not found")
                {
                    let warning = ICON_WARNING.yellow();
                    let pattern = file_pattern.yellow();
                    println!("  {warning} Not found: {pattern} (skipping)");
                } else {
                    let cross = ICON_ERROR.red();
                    let pattern = file_pattern.red();
                    println!("  {cross} Failed to copy {pattern}: {e}");
                }
            }
        }
    }

    if copied_files.is_empty() {
        let info = ICON_INFO.blue();
        println!("  {info} No files were copied");
    }

    Ok(copied_files)
}

/// Determines the source directory for file copying
///
/// Priority:
/// 1. Explicitly configured source directory
/// 2. Main worktree directory (for bare repositories)
/// 3. Current working directory (for non-bare repositories)
fn determine_source_directory(
    config: &FilesConfig,
    manager: &GitWorktreeManager,
) -> Result<PathBuf> {
    if let Some(source) = &config.source {
        let path = PathBuf::from(source);
        if path.is_absolute() {
            return Ok(path);
        }
        // Make relative paths relative to repo root
        return Ok(manager.repo().path().join(source));
    }

    // For bare repositories, find the main worktree
    if manager.repo().is_bare() {
        find_source_in_bare_repo(manager)
    } else {
        // For non-bare repositories, use the main repository directory
        find_source_in_regular_repo(manager)
    }
}

/// Finds the main worktree in a bare repository setup
fn find_main_worktree(worktrees: &[WorktreeInfo]) -> Option<&WorktreeInfo> {
    // First try to find explicitly named main/master worktrees
    worktrees
        .iter()
        .find(|w| w.name == "main" || w.name == "master")
        .or_else(|| {
            // Otherwise, find the worktree that's a sibling of the git directory
            worktrees.iter().find(|w| {
                // Check if this worktree is at the same level as .git
                w.path
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .map(|name| name != WORKTREES_SUBDIR)
                    .unwrap_or(false)
            })
        })
}

/// Finds source directory in a bare repository
fn find_source_in_bare_repo(manager: &GitWorktreeManager) -> Result<PathBuf> {
    let worktrees = manager.list_worktrees()?;

    if let Some(main_worktree) = find_main_worktree(&worktrees) {
        // Look for config file in main worktree
        let config_path = main_worktree.path.join(crate::constants::CONFIG_FILE_NAME);
        if config_path.exists() {
            return Ok(main_worktree.path.clone());
        }
    }

    // If no main worktree or config found, check default locations
    let git_dir = manager.repo().path();
    let parent = git_dir
        .parent()
        .ok_or_else(|| anyhow!("Git directory has no parent"))?;

    // Check common worktree locations
    for name in &["main", "master"] {
        let worktree_path = parent.join(name);
        if worktree_path.exists()
            && worktree_path
                .join(crate::constants::CONFIG_FILE_NAME)
                .exists()
        {
            return Ok(worktree_path);
        }
    }

    Err(anyhow!(
        "No main worktree found with {} file",
        crate::constants::CONFIG_FILE_NAME
    ))
}

/// Finds source directory in a regular (non-bare) repository
fn find_source_in_regular_repo(manager: &GitWorktreeManager) -> Result<PathBuf> {
    // For non-bare repos, start with the working directory
    let cwd = std::env::current_dir()?;

    // If we're in the main repository directory, use it
    if cwd.join(GIT_DIR).exists() {
        return Ok(cwd);
    }

    // Otherwise, find the repository root
    let repo_workdir = manager
        .repo()
        .workdir()
        .ok_or_else(|| anyhow!("Repository has no working directory"))?;

    Ok(repo_workdir.to_path_buf())
}

/// Maximum file size for automatic copying in bytes
const MAX_FILE_SIZE: u64 = MAX_FILE_SIZE_MB * BYTES_PER_MB;

/// Calculates the total size of a file or directory
///
/// For directories, this recursively calculates the size of all files within.
///
/// # Returns
///
/// * `Ok(u64)` - Total size in bytes
/// * `Err` - If the path doesn't exist or can't be accessed
fn calculate_path_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let metadata = path.symlink_metadata()?;

    // Skip symlinks
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }

    if metadata.is_file() {
        Ok(metadata.len())
    } else if metadata.is_dir() {
        calculate_directory_size(path, 0)
    } else {
        Ok(0)
    }
}

/// Recursively calculates the size of a directory
fn calculate_directory_size(path: &Path, depth: usize) -> Result<u64> {
    if depth >= MAX_DIRECTORY_DEPTH {
        return Ok(0); // Stop at max depth
    }

    let mut total_size = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                if let Ok(dir_size) = calculate_directory_size(&path, depth + 1) {
                    total_size += dir_size;
                }
            }
        }
    }

    Ok(total_size)
}

/// Validates that a path is safe to use (no directory traversal)
///
/// # Security
///
/// This function ensures paths don't contain:
/// - Parent directory references (`..`)
/// - Absolute paths
/// - Other potentially dangerous patterns
fn is_safe_path(path: &str) -> bool {
    // Reject empty paths
    if path.is_empty() {
        return false;
    }

    // Reject absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }

    // Check for Windows absolute paths (C:\, D:\, etc.)
    if path.len() >= WINDOWS_PATH_MIN_LENGTH
        && path.chars().nth(COLON_POSITION_WINDOWS) == Some(':')
    {
        return false;
    }

    // Reject paths containing parent directory references
    let components: Vec<&str> = path.split(&['/', '\\'][..]).collect();
    for component in components {
        if component == ".." || component == "." {
            return false;
        }
    }

    true
}

/// Copies a file or directory from source to destination
///
/// # Returns
///
/// Returns the number of files copied
fn copy_file_or_directory(source: &Path, dest: &Path) -> Result<usize> {
    if !source.exists() {
        let source_path = source.display();
        return Err(anyhow!("Source path not found: {source_path}"));
    }

    // Symlink detection with warning
    if source.symlink_metadata()?.file_type().is_symlink() {
        let warning = "⚠️".yellow();
        let source_path = source.display();
        println!("  {warning} Skipping symlink: {source_path}");
        return Ok(0);
    }

    if source.is_file() {
        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory: {parent_display}",
                    parent_display = parent.display()
                )
            })?;
        }

        fs::copy(source, dest).with_context(|| {
            format!(
                "Failed to copy file from {} to {}",
                source.display(),
                dest.display()
            )
        })?;

        Ok(1)
    } else if source.is_dir() {
        copy_directory_recursive(source, dest, 0)
    } else {
        Err(anyhow!(
            "Source is neither a file nor a directory: {}",
            source.display()
        ))
    }
}

/// Recursively copies a directory and its contents
///
/// # Security
///
/// Includes depth limiting to prevent infinite recursion from circular symlinks
fn copy_directory_recursive(source: &Path, dest: &Path, depth: usize) -> Result<usize> {
    if depth >= MAX_DIRECTORY_DEPTH {
        return Err(anyhow!(
            "Maximum directory depth ({}) exceeded. Possible circular reference.",
            MAX_DIRECTORY_DEPTH
        ));
    }

    fs::create_dir_all(dest).with_context(|| {
        format!(
            "Failed to create directory: {dest_display}",
            dest_display = dest.display()
        )
    })?;

    let mut total_files = 0;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let source_path = entry.path();
        let dest_path = dest.join(&file_name);

        // Check for circular reference
        if source_path
            .canonicalize()
            .ok()
            .and_then(|canonical_source| {
                dest.canonicalize()
                    .ok()
                    .map(|canonical_dest| canonical_source.starts_with(&canonical_dest))
            })
            .unwrap_or(false)
        {
            println!(
                "  {} Skipping circular reference: {}",
                ICON_WARNING.yellow(),
                source_path.display()
            );
            continue;
        }

        match copy_directory_recursive_impl(&source_path, &dest_path, depth + 1) {
            Ok(count) => total_files += count,
            Err(e) => {
                println!(
                    "  {} Failed to copy {}: {}",
                    ICON_WARNING.yellow(),
                    source_path.display(),
                    e
                );
            }
        }
    }

    Ok(total_files)
}

/// Implementation helper for recursive directory copying
fn copy_directory_recursive_impl(source: &Path, dest: &Path, depth: usize) -> Result<usize> {
    // Symlink detection
    if source.symlink_metadata()?.file_type().is_symlink() {
        return Ok(0); // Skip symlinks silently in recursive copy
    }

    if source.is_file() {
        fs::copy(source, dest)?;
        Ok(1)
    } else if source.is_dir() {
        copy_directory_recursive(source, dest, depth)
    } else {
        Ok(0) // Skip special files
    }
}
