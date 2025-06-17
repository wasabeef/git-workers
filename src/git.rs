//! Git worktree management functionality
//!
//! This module provides the core Git operations for managing worktrees,
//! including creation, deletion, listing, and renaming. It handles both
//! regular and bare repositories.
//!
//! # Features
//!
//! - **Worktree Creation**: Create worktrees from branches or current HEAD
//! - **Worktree Listing**: List all worktrees with detailed status information
//! - **Worktree Deletion**: Safely remove worktrees and optionally their branches
//! - **Worktree Renaming**: Complete rename including directory, metadata, and optional branch
//! - **Pattern Detection**: Automatically detect and follow worktree organization patterns
//! - **Bare Repository Support**: Special handling for bare repositories
//!
//! # Examples
//!
//! ```no_run
//! use git_workers::git::GitWorktreeManager;
//!
//! let manager = GitWorktreeManager::new().unwrap();
//!
//! // List all worktrees
//! let worktrees = manager.list_worktrees().unwrap();
//! for wt in worktrees {
//!     println!("{}: {} ({})", wt.name, wt.branch,
//!              if wt.is_current { "current" } else { "" });
//! }
//!
//! // Create a new worktree
//! let path = manager.create_worktree("feature-x", Some("feature/new-feature")).unwrap();
//! println!("Created worktree at: {}", path.display());
//! ```

use anyhow::{anyhow, Result};
use git2::{BranchType, Repository};
use std::path::{Path, PathBuf};

// Constants for default values
const DEFAULT_BRANCH_UNKNOWN: &str = "unknown";
const DEFAULT_BRANCH_DETACHED: &str = "detached";
const DEFAULT_AUTHOR_UNKNOWN: &str = "Unknown";
const DEFAULT_MESSAGE_NONE: &str = "No message";
const COMMIT_ID_SHORT_LENGTH: usize = 8;
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

/// Finds the common parent directory of all worktrees
///
/// Returns `None` if worktrees don't share a common parent or if the list is empty
fn find_common_parent(worktrees: &[WorktreeInfo]) -> Option<PathBuf> {
    if worktrees.is_empty() {
        return None;
    }

    let parent_dirs: Vec<_> = worktrees
        .iter()
        .filter_map(|w| w.path.parent())
        .map(|p| p.to_path_buf())
        .collect();

    // Check if all parent directories are the same
    if parent_dirs.windows(2).all(|w| w[0] == w[1]) {
        parent_dirs.first().cloned()
    } else {
        None
    }
}

/// High-level Git worktree manager
///
/// Provides a convenient interface for common worktree operations,
/// wrapping both git2 library calls and git command-line operations.
///
/// # Thread Safety
///
/// `GitWorktreeManager` is not thread-safe. Each thread should create
/// its own instance.
///
/// # Error Handling
///
/// All methods return `Result<T, anyhow::Error>` for consistent error handling.
/// Errors include Git operation failures, I/O errors, and validation errors.
///
/// This struct wraps a git2::Repository and provides high-level
/// operations for worktree management.
pub struct GitWorktreeManager {
    repo: Repository,
}

impl GitWorktreeManager {
    /// Creates a new GitWorktreeManager by discovering the repository from the current directory
    ///
    /// This will discover the Git repository by searching upward from
    /// the current working directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current directory is not inside a Git repository
    /// - The repository cannot be opened
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use git_workers::git::GitWorktreeManager;
    ///
    /// let manager = GitWorktreeManager::new().expect("Failed to open repository");
    /// ```
    pub fn new() -> Result<Self> {
        let repo = Repository::open_from_env()?;
        Ok(Self { repo })
    }

    /// Creates a new GitWorktreeManager from a specific path (for testing)
    #[allow(dead_code)]
    pub fn new_from_path(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    /// Returns a reference to the underlying git2::Repository
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// Get the directory to use as working directory for git commands
    fn get_git_dir(&self) -> Result<&Path> {
        if self.repo.is_bare() {
            Ok(self.repo.path())
        } else {
            self.repo
                .workdir()
                .ok_or_else(|| anyhow!("No working directory"))
        }
    }

    /// Determines the default base path for creating new worktrees
    ///
    /// For bare repositories, uses the parent of the repository path.
    /// For normal repositories, uses the parent of the working directory.
    fn get_default_worktree_base_path(&self) -> Result<PathBuf> {
        if self.repo.is_bare() {
            self.repo
                .path()
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory of bare repository"))
                .map(|p| p.to_path_buf())
        } else {
            self.repo
                .workdir()
                .ok_or_else(|| anyhow!("Cannot find repository working directory"))?
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))
                .map(|p| p.to_path_buf())
        }
    }

    /// Lists all worktrees in the repository with their status information
    ///
    /// This method uses parallel processing to gather worktree information
    /// efficiently, including branch names, modification status, and commit info.
    ///
    /// # Returns
    ///
    /// A vector of `WorktreeInfo` structs sorted by name
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let mut worktrees;
        let worktree_names = self.repo.worktrees()?;

        // Use parallel processing for better performance
        use std::sync::{Arc, Mutex};
        use std::thread;

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for name in worktree_names.iter().flatten() {
            if let Ok(worktree) = self.repo.find_worktree(name) {
                let path = worktree.path();
                let is_current = self.is_current_worktree(path);
                let name_clone = name.to_string();
                let path_clone = path.to_path_buf();
                let is_locked = worktree.is_locked().is_ok();
                let results_clone = Arc::clone(&results);

                // Spawn thread for each worktree to parallelize repository operations
                let handle = thread::spawn(move || {
                    let branch = if let Ok(wt_repo) = Repository::open(&path_clone) {
                        wt_repo
                            .head()
                            .ok()
                            .and_then(|h| h.shorthand().map(|s| s.to_string()))
                            .unwrap_or_else(|| String::from(DEFAULT_BRANCH_DETACHED))
                    } else {
                        String::from(DEFAULT_BRANCH_UNKNOWN)
                    };

                    // Get additional status info for the worktree
                    let worktree_status = get_worktree_status(&path_clone);

                    let info = WorktreeInfo {
                        name: name_clone,
                        path: path_clone,
                        branch,
                        is_locked,
                        is_current,
                        has_changes: worktree_status.has_changes,
                        last_commit: worktree_status.last_commit,
                        ahead_behind: worktree_status.ahead_behind,
                    };

                    results_clone.lock().unwrap().push(info);
                });

                handles.push(handle);
            }
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        worktrees = Arc::try_unwrap(results).unwrap().into_inner().unwrap();

        // Sort by name for consistent ordering
        worktrees.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(worktrees)
    }

    fn is_current_worktree(&self, path: &std::path::Path) -> bool {
        std::env::current_dir()
            .map(|dir| dir.starts_with(path))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    fn get_worktree_branch(&self, worktree: &git2::Worktree) -> Result<String> {
        let worktree_repo = Repository::open(worktree.path())?;
        let head = worktree_repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("unknown").to_string())
        } else {
            Ok("detached".to_string())
        }
    }

    /// Creates a new worktree with the specified name and optional branch
    ///
    /// This method intelligently determines the base path for the new worktree
    /// by analyzing existing worktree patterns. For bare repositories, it
    /// typically creates worktrees in a `branch/` subdirectory.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the new worktree (used as directory name)
    /// * `branch` - Optional branch name to create the worktree from
    ///
    /// # Returns
    ///
    /// The path to the newly created worktree
    ///
    /// # Errors
    ///
    /// * If the worktree path already exists
    /// * If the branch name is invalid
    /// * If Git operations fail
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// // Create worktree from existing branch
    /// let path = manager.create_worktree("feature", Some("feature/auth")).unwrap();
    ///
    /// // Create worktree from current HEAD
    /// let path = manager.create_worktree("experiment", None).unwrap();
    /// ```
    pub fn create_worktree(&self, name: &str, branch: Option<&str>) -> Result<PathBuf> {
        let base_path = self.determine_worktree_base_path()?;

        // Handle different path patterns
        let worktree_path = if name.starts_with("../") {
            // Relative path from repository (e.g., "../emu/branch/feature")
            // This is used for creating worktrees inside the repository structure
            let repo_dir = self
                .repo
                .workdir()
                .or_else(|| self.repo.path().parent())
                .ok_or_else(|| anyhow!("Cannot determine repository directory"))?;
            repo_dir.join(name)
        } else if name.contains('/') {
            // Name includes a path pattern (e.g., "branch/feature")
            // Use the parent directory as base
            base_path.join(name)
        } else {
            // Simple name - use existing pattern detection
            base_path.join(name)
        };

        // Ensure parent directories exist
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if worktree_path.exists() {
            return Err(anyhow!(
                "Worktree path already exists: {}",
                worktree_path.display()
            ));
        }

        // Extract the actual worktree name (last component)
        let worktree_name = worktree_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);

        // Create worktree with git2
        if let Some(branch_name) = branch {
            // Use git CLI for branch-based worktree creation
            // (git2's worktree API has limitations)
            self.create_worktree_with_branch(&worktree_path, branch_name)
        } else {
            // Create worktree from current HEAD
            self.create_worktree_from_head(&worktree_path, worktree_name)
        }
    }

    /// Determines the base path for creating new worktrees
    fn determine_worktree_base_path(&self) -> Result<PathBuf> {
        if let Ok(existing_worktrees) = self.list_worktrees() {
            if let Some(common_parent) = find_common_parent(&existing_worktrees) {
                return Ok(common_parent);
            }
        }
        self.get_default_worktree_base_path()
    }

    /// Creates a worktree with a specific branch
    fn create_worktree_with_branch(&self, path: &Path, branch_name: &str) -> Result<PathBuf> {
        use std::process::Command;

        // Check if branch exists
        let branch_exists = self
            .repo
            .find_branch(branch_name, BranchType::Local)
            .is_ok();

        let mut cmd = Command::new("git");

        // Set the current directory to the repository path
        cmd.current_dir(self.get_git_dir()?);

        if branch_exists {
            // If branch exists, create worktree pointing to that branch
            cmd.arg("worktree").arg("add").arg(path).arg(branch_name);
        } else {
            // If branch doesn't exist, create new branch with worktree
            cmd.arg("worktree")
                .arg("add")
                .arg("-b")
                .arg(branch_name)
                .arg(path);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to create worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(path.to_path_buf())
    }

    /// Creates a worktree from the current HEAD
    fn create_worktree_from_head(&self, path: &Path, name: &str) -> Result<PathBuf> {
        if self.repo.is_bare() {
            // For bare repositories, use git command
            use std::process::Command;
            let output = Command::new("git")
                .current_dir(self.get_git_dir()?)
                .arg("worktree")
                .arg("add")
                .arg(path)
                .arg("HEAD")
                .output()?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to create worktree: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        } else {
            // For non-bare repositories, use git2 API
            self.repo.worktree(name, path, None)?;
        }
        Ok(path.to_path_buf())
    }

    /// Removes a worktree by name
    ///
    /// This prunes the worktree, removing both the Git metadata and
    /// the working directory.
    pub fn remove_worktree(&self, name: &str) -> Result<()> {
        let worktree = self.repo.find_worktree(name)?;
        worktree.prune(Some(
            &mut git2::WorktreePruneOptions::new()
                .valid(true)
                .working_tree(true),
        ))?;
        Ok(())
    }

    /// Lists all local branches in the repository
    ///
    /// # Returns
    ///
    /// A sorted vector of branch names
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        let branch_iter = self.repo.branches(Some(BranchType::Local))?;

        for (branch, _) in branch_iter.flatten() {
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        branches.sort();
        Ok(branches)
    }

    /// Deletes a local branch by name
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch to delete
    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        match self.repo.find_branch(branch_name, BranchType::Local) {
            Ok(mut branch) => {
                branch.delete()?;
                Ok(())
            }
            Err(_) => Err(anyhow!("Branch '{}' not found", branch_name)),
        }
    }

    #[allow(dead_code)]
    fn check_worktree_changes(&self, repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .include_ignored(false),
        ))?;

        Ok(!statuses.is_empty())
    }

    #[allow(dead_code)]
    fn get_last_commit(&self, repo: &Repository) -> Result<CommitInfo> {
        let head = repo.head()?.peel_to_commit()?;
        let time = chrono::DateTime::from_timestamp(head.time().seconds(), 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M")
            .to_string();

        let id = head.id().to_string()[..8].to_string();
        let message = head.summary().unwrap_or("No message").to_string();
        let author = head.author().name().unwrap_or("Unknown").to_string();

        Ok(CommitInfo {
            id,
            message,
            author,
            time,
        })
    }

    pub fn rename_worktree(&self, old_name: &str, new_name: &str) -> Result<PathBuf> {
        use std::fs;
        use std::process::Command;

        // Validate new name
        if new_name.contains(char::is_whitespace) {
            return Err(anyhow!("New name cannot contain spaces"));
        }

        // Get the old worktree info
        let old_worktree = self.repo.find_worktree(old_name)?;
        let old_path = old_worktree.path().to_path_buf();

        // Generate new path
        let new_path = old_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot find parent directory"))?
            .join(new_name);

        if new_path.exists() {
            return Err(anyhow!(
                "Target path already exists: {}",
                new_path.display()
            ));
        }

        // Check if it's the current worktree
        let is_current = self.is_current_worktree(&old_path);
        if is_current {
            return Err(anyhow!(
                "Cannot rename current worktree. Please switch to another worktree first."
            ));
        }

        // Get the branch name
        let branch_name = if let Ok(wt_repo) = Repository::open(&old_path) {
            if let Ok(head) = wt_repo.head() {
                if head.is_branch() {
                    head.shorthand().unwrap_or("").to_string()
                } else {
                    // Detached HEAD - cannot use branch rename
                    return Err(anyhow!("Cannot rename worktree with detached HEAD"));
                }
            } else {
                return Err(anyhow!("Cannot read worktree HEAD"));
            }
        } else {
            return Err(anyhow!("Cannot open worktree repository"));
        };

        // Step 1: Move the directory
        fs::rename(&old_path, &new_path)?;

        // Step 2: Get the git directory
        let git_dir = self.repo.path().to_path_buf();

        let old_worktree_git_dir = git_dir.join("worktrees").join(old_name);
        let new_worktree_git_dir = git_dir.join("worktrees").join(new_name);

        if old_worktree_git_dir.exists() {
            fs::rename(&old_worktree_git_dir, &new_worktree_git_dir)?;

            // Update the gitdir file
            let gitdir_file = new_worktree_git_dir.join("gitdir");
            if gitdir_file.exists() {
                fs::write(&gitdir_file, format!("{}/.git\n", new_path.display()))?;
            }
        }

        // Step 3: Update the .git file in the worktree
        let git_file_path = new_path.join(".git");
        if git_file_path.exists() {
            let git_file_content = format!("gitdir: {}\n", new_worktree_git_dir.display());
            fs::write(&git_file_path, git_file_content)?;
        }

        // Step 4: If the branch name matches the old worktree name, rename it
        if branch_name == old_name {
            let output = Command::new("git")
                .current_dir(&new_path)
                .arg("branch")
                .arg("-m")
                .arg(&branch_name)
                .arg(new_name)
                .output()?;

            if !output.status.success() {
                eprintln!(
                    "Warning: Could not rename branch: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(new_path)
    }

    #[allow(dead_code)]
    fn get_ahead_behind(&self, repo: &Repository) -> Result<(usize, usize)> {
        let head = repo.head()?;
        if !head.is_branch() {
            return Err(anyhow!("Not on a branch"));
        }

        let local_oid = head.target().ok_or_else(|| anyhow!("No target"))?;
        let branch_name = head.shorthand().ok_or_else(|| anyhow!("No branch name"))?;

        // Try to find upstream branch
        let upstream_name = format!("origin/{}", branch_name);
        if let Ok(upstream) = repo.find_reference(&format!("refs/remotes/{}", upstream_name)) {
            let upstream_oid = upstream.target().ok_or_else(|| anyhow!("No target"))?;
            let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
            Ok((ahead, behind))
        } else {
            Err(anyhow!("No upstream branch"))
        }
    }
}

/// Status information for a worktree
struct WorktreeStatus {
    has_changes: bool,
    last_commit: Option<CommitInfo>,
    ahead_behind: Option<(usize, usize)>,
}

/// Gets the status information for a worktree
fn get_worktree_status(path: &Path) -> WorktreeStatus {
    if let Ok(repo) = Repository::open(path) {
        let has_changes = repo
            .statuses(Some(
                git2::StatusOptions::new()
                    .include_untracked(true)
                    .include_ignored(false),
            ))
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        let last_commit = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .map(|commit| {
                let time = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_default()
                    .format(TIME_FORMAT)
                    .to_string();

                CommitInfo {
                    id: commit.id().to_string()[..COMMIT_ID_SHORT_LENGTH].to_string(),
                    message: commit.summary().unwrap_or(DEFAULT_MESSAGE_NONE).to_string(),
                    author: commit
                        .author()
                        .name()
                        .unwrap_or(DEFAULT_AUTHOR_UNKNOWN)
                        .to_string(),
                    time,
                }
            });

        WorktreeStatus {
            has_changes,
            last_commit,
            ahead_behind: None, // Skip for performance
        }
    } else {
        WorktreeStatus {
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        }
    }
}

/// Information about a Git worktree
///
/// This struct contains all the relevant information about a worktree
/// that is displayed in the UI or used for decision making.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// The name of the worktree (derived from the directory name)
    pub name: String,
    /// The absolute filesystem path to the worktree
    pub path: PathBuf,
    /// The current branch name or "detached" if in detached HEAD state
    pub branch: String,
    /// Whether the worktree is locked (prevents deletion)
    #[allow(dead_code)]
    pub is_locked: bool,
    /// Whether this is the currently active worktree
    pub is_current: bool,
    /// Whether the worktree has uncommitted changes
    pub has_changes: bool,
    /// Information about the last commit in the worktree
    pub last_commit: Option<CommitInfo>,
    /// Number of commits ahead and behind the upstream branch
    pub ahead_behind: Option<(usize, usize)>, // (ahead, behind)
}

/// Information about a Git commit
///
/// Contains basic information about a commit for display purposes.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Short commit ID (first 8 characters)
    pub id: String,
    /// First line of the commit message
    pub message: String,
    /// Commit author name
    #[allow(dead_code)]
    pub author: String,
    /// Formatted commit time (YYYY-MM-DD HH:MM)
    pub time: String,
}

/// Convenience function to list worktrees from the current directory
///
/// This is a wrapper around GitWorktreeManager for simple CLI usage
pub fn list_worktrees() -> Result<Vec<String>> {
    let manager = GitWorktreeManager::new()?;
    let worktrees = manager.list_worktrees()?;

    Ok(worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect())
}
