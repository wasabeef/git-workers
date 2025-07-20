//! Git worktree management functionality
//!
//! This module provides the core Git operations for managing worktrees,
//! including creation, deletion, listing, and renaming. It handles both
//! regular and bare repositories with proper path resolution.
//!
//! # Features
//!
//! - **Worktree Creation**: Create worktrees from branches or current HEAD
//! - **Worktree Listing**: List all worktrees with detailed status information
//! - **Worktree Deletion**: Safely remove worktrees and optionally their branches
//! - **Worktree Renaming**: Complete rename including directory, metadata, and optional branch
//! - **Pattern Detection**: Automatically detect and follow worktree organization patterns
//! - **Bare Repository Support**: Special handling for bare repositories
//! - **Path Resolution**: Proper handling of relative and absolute paths for worktree creation
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
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::constants::{
    COMMIT_ID_SHORT_LENGTH, DEFAULT_AUTHOR_UNKNOWN, DEFAULT_BRANCH_DETACHED,
    DEFAULT_BRANCH_UNKNOWN, DEFAULT_MESSAGE_NONE, ERROR_LOCK_CREATE, ERROR_LOCK_EXISTS,
    ERROR_NO_PARENT_BARE_REPO, ERROR_NO_PARENT_DIR, ERROR_NO_REPO_DIR, ERROR_NO_REPO_WORKING_DIR,
    ERROR_NO_WORKING_DIR, ERROR_WORKTREE_CREATE, ERROR_WORKTREE_PATH_EXISTS, GIT_ADD, GIT_BRANCH,
    GIT_BRANCH_NOT_FOUND_MSG, GIT_CANNOT_FIND_PARENT, GIT_CANNOT_RENAME_CURRENT,
    GIT_CANNOT_RENAME_DETACHED, GIT_CMD, GIT_COMMIT_AUTHOR_UNKNOWN, GIT_COMMIT_MESSAGE_NONE,
    GIT_DEFAULT_MAIN_WORKTREE, GIT_DIR, GIT_GITDIR_PREFIX, GIT_GITDIR_SUFFIX, GIT_HEAD_INDEX,
    GIT_NEW_NAME_NO_SPACES, GIT_OPT_BRANCH, GIT_OPT_GIT_COMMON_DIR, GIT_OPT_RENAME, GIT_ORIGIN,
    GIT_REFS_REMOTES, GIT_REFS_TAGS, GIT_REPAIR, GIT_RESERVED_NAMES, GIT_REV_PARSE, GIT_WORKTREE,
    LOCK_FILE_NAME, STALE_LOCK_TIMEOUT_SECS, TIME_FORMAT, WINDOW_FIRST_INDEX, WINDOW_SECOND_INDEX,
    WINDOW_SIZE_PAIRS,
};
use crate::filesystem::FileSystem;

// Create Duration from constant for stale lock timeout
const STALE_LOCK_TIMEOUT: Duration = Duration::from_secs(STALE_LOCK_TIMEOUT_SECS);

/// Simple lock structure for worktree operations
pub struct WorktreeLock {
    lock_path: PathBuf,
    _file: Option<File>,
}

impl WorktreeLock {
    /// Attempts to acquire a lock for worktree operations
    pub fn acquire(git_dir: &Path) -> Result<Self> {
        let lock_path = git_dir.join(LOCK_FILE_NAME);

        // Check for stale lock
        if lock_path.exists() {
            if let Ok(metadata) = lock_path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > STALE_LOCK_TIMEOUT {
                            // Remove stale lock
                            let _ = fs::remove_file(&lock_path);
                        }
                    }
                }
            }
        }

        // Try to create lock file exclusively
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    anyhow!(ERROR_LOCK_EXISTS)
                } else {
                    anyhow!("{}", ERROR_LOCK_CREATE.replace("{}", &e.to_string()))
                }
            })?;

        Ok(WorktreeLock {
            lock_path,
            _file: Some(file),
        })
    }
}

impl Drop for WorktreeLock {
    fn drop(&mut self) {
        // Clean up lock file when lock is released
        let _ = fs::remove_file(&self.lock_path);
    }
}

/// Finds the common parent directory of all worktrees
///
/// This function is used to detect the pattern for organizing worktrees.
/// If all existing worktrees share a common parent directory, new worktrees
/// should be created in the same location to maintain consistency.
///
/// # Arguments
///
/// * `worktrees` - Slice of worktree information structs
///
/// # Returns
///
/// * `Some(PathBuf)` - The common parent directory if all worktrees share one
/// * `None` - If worktrees don't share a common parent or the list is empty
///
/// # Example
///
/// If worktrees are at:
/// - `/home/user/projects/myrepo/feature1`
/// - `/home/user/projects/myrepo/feature2`
///
/// Returns: `Some("/home/user/projects/myrepo")`
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
    if parent_dirs
        .windows(WINDOW_SIZE_PAIRS)
        .all(|w| w[WINDOW_FIRST_INDEX] == w[WINDOW_SECOND_INDEX])
    {
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
    pub(crate) repo: Repository,
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

    /// Creates a new GitWorktreeManager from a specific path
    ///
    /// This method is primarily used for testing but is available for any code
    /// that needs to create a manager from a specific repository path.
    #[allow(dead_code)]
    pub fn new_from_path(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    /// Returns a reference to the underlying git2::Repository
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// Gets the directory to use as working directory for git commands
    ///
    /// For bare repositories, returns the repository path itself.
    /// For normal repositories, returns the working directory.
    ///
    /// # Returns
    ///
    /// The appropriate directory path for executing git commands
    ///
    /// # Errors
    ///
    /// Returns an error if the repository has no working directory
    /// (should not happen in practice)
    pub fn get_git_dir(&self) -> Result<&Path> {
        if self.repo.is_bare() {
            Ok(self.repo.path())
        } else {
            self.repo
                .workdir()
                .ok_or_else(|| anyhow!(ERROR_NO_WORKING_DIR))
        }
    }

    /// Determines the default base path for creating new worktrees
    ///
    /// This method provides the fallback location when no existing worktree
    /// pattern can be detected.
    ///
    /// # Behavior
    ///
    /// - **Bare repositories**: Uses the parent of the repository path
    /// - **Normal repositories**: Uses the parent of the working directory
    ///
    /// # Returns
    ///
    /// The default base path for creating worktrees
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot find the parent directory
    /// - Repository has no working directory (for non-bare repos)
    pub fn get_default_worktree_base_path(&self) -> Result<PathBuf> {
        if self.repo.is_bare() {
            self.repo
                .path()
                .parent()
                .ok_or_else(|| anyhow!(ERROR_NO_PARENT_BARE_REPO))
                .map(|p| p.to_path_buf())
        } else {
            self.repo
                .workdir()
                .ok_or_else(|| anyhow!(ERROR_NO_REPO_WORKING_DIR))?
                .parent()
                .ok_or_else(|| anyhow!(ERROR_NO_PARENT_DIR))
                .map(|p| p.to_path_buf())
        }
    }

    /// Lists all worktrees in the repository with their status information
    ///
    /// This method uses parallel processing to gather worktree information
    /// efficiently, including branch names, modification status, and commit info.
    /// Each worktree is processed in a separate thread to minimize latency when
    /// accessing multiple repository directories.
    ///
    /// # Returns
    ///
    /// A vector of [`WorktreeInfo`] structs sorted alphabetically by name
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot enumerate worktrees from the repository
    /// - Thread operations fail (rare)
    ///
    /// # Performance
    ///
    /// This method spawns one thread per worktree for parallel processing.
    /// For repositories with many worktrees, this significantly reduces the
    /// total time compared to sequential processing.
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

    /// Checks if the given path is the current worktree
    ///
    /// # Arguments
    ///
    /// * `path` - The worktree path to check
    ///
    /// # Returns
    ///
    /// `true` if the current working directory is within the given worktree path
    fn is_current_worktree(&self, path: &std::path::Path) -> bool {
        std::env::current_dir()
            .map(|dir| dir.starts_with(path))
            .unwrap_or(false)
    }

    /// Gets the current branch name for a worktree
    ///
    /// # Arguments
    ///
    /// * `worktree` - The git2 worktree object
    ///
    /// # Returns
    ///
    /// The branch name, or "detached" if in detached HEAD state,
    /// or "unknown" if the branch cannot be determined
    ///
    /// # Errors
    ///
    /// Returns an error if the worktree repository cannot be opened
    #[allow(dead_code)]
    fn get_worktree_branch(&self, worktree: &git2::Worktree) -> Result<String> {
        let worktree_repo = Repository::open(worktree.path())?;
        let head = worktree_repo.head()?;

        if head.is_branch() {
            Ok(head
                .shorthand()
                .unwrap_or(DEFAULT_BRANCH_UNKNOWN)
                .to_string())
        } else {
            Ok(DEFAULT_BRANCH_DETACHED.to_string())
        }
    }

    /// Creates a new worktree with the specified name and optional branch
    ///
    /// This method intelligently determines the base path for the new worktree
    /// by analyzing existing worktree patterns. It supports three path patterns:
    ///
    /// 1. **Relative paths** (`../name`): Creates at same level as repository
    /// 2. **Subdirectory paths** (`worktrees/name`): Creates within repository
    /// 3. **Simple names**: Uses existing pattern detection or defaults
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the new worktree (used as directory name)
    /// * `branch` - Optional branch name to create the worktree from
    ///
    /// # Path Resolution
    ///
    /// - Paths starting with `../` are resolved relative to the repository parent
    /// - Paths containing `/` (e.g., "worktrees/feature") are created within the repository directory
    /// - Simple names use pattern detection from existing worktrees
    /// - All paths are canonicalized to resolve `..` components for clean display
    ///
    /// # Returns
    ///
    /// The canonicalized path to the newly created worktree
    ///
    /// # Thread Safety
    ///
    /// This method uses file-based locking to prevent concurrent worktree creation
    /// from multiple git-workers processes. A lock file is created in the `.git`
    /// directory and automatically removed when the operation completes.
    ///
    /// # Errors
    ///
    /// * If another git-workers process is currently creating a worktree
    /// * If the worktree path already exists
    /// * If the branch name is invalid
    /// * If Git operations fail
    /// * If path canonicalization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// // Create worktree from existing branch
    /// let path = manager.create_worktree("feature", Some("feature/auth")).unwrap();
    ///
    /// // Create worktree from current HEAD with subdirectory pattern
    /// let path = manager.create_worktree("worktrees/experiment", None).unwrap();
    ///
    /// // Create worktree at same level as repository
    /// let path = manager.create_worktree("../sibling", None).unwrap();
    /// ```
    pub fn create_worktree(&self, name: &str, branch: Option<&str>) -> Result<PathBuf> {
        // Acquire lock to prevent concurrent worktree creation
        let _lock = WorktreeLock::acquire(self.repo.path())?;

        let base_path = self.determine_worktree_base_path()?;

        // Handle different path patterns
        let worktree_path = if name.starts_with("../") {
            // Relative path from repository (e.g., "../feature")
            // This creates worktrees at the same level as the repository
            let repo_dir = self
                .repo
                .workdir()
                .or_else(|| self.repo.path().parent())
                .ok_or_else(|| anyhow!(ERROR_NO_REPO_DIR))?;
            repo_dir.join(name)
        } else if name.contains('/') {
            // Name includes a path pattern (e.g., "worktrees/feature")
            // This is for subdirectory pattern - use repository directory as base
            let repo_dir = self.repo.workdir().unwrap_or_else(|| self.repo.path());
            repo_dir.join(name)
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
                "{}",
                ERROR_WORKTREE_PATH_EXISTS.replace("{}", &worktree_path.display().to_string())
            ));
        }

        // Extract the actual worktree name (last component)
        let worktree_name = worktree_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);

        // Canonicalize the path to resolve .. components
        let canonical_path = worktree_path
            .canonicalize()
            .or_else(|_| -> Result<PathBuf> {
                // If canonicalize fails (path doesn't exist yet), manually resolve
                if let Some(parent) = worktree_path.parent() {
                    if let Ok(canonical_parent) = parent.canonicalize() {
                        return Ok(canonical_parent.join(worktree_path.file_name().unwrap()));
                    }
                }
                Ok(worktree_path.clone())
            })
            .unwrap_or_else(|_| worktree_path.clone());

        // Create worktree with git2
        if let Some(branch_name) = branch {
            // Use git CLI for branch-based worktree creation
            // (git2's worktree API has limitations)
            self.create_worktree_with_branch(&canonical_path, branch_name)
        } else {
            // Create worktree from current HEAD
            self.create_worktree_from_head(&canonical_path, worktree_name)
        }
    }

    /// Creates a worktree with a new branch from a base branch
    ///
    /// This method creates a new worktree with a new branch that branches off from
    /// the specified base branch. This is useful for feature development workflows
    /// where you want to create a new feature branch from main/develop.
    ///
    /// # Arguments
    ///
    /// * `name` - The worktree name/path (can include path patterns like "../name" or "worktrees/name")
    /// * `new_branch` - The name of the new branch to create
    /// * `base_branch` - The base branch to create from (can be local or remote like "origin/main")
    ///
    /// # Returns
    ///
    /// The canonical path to the created worktree
    ///
    /// # Thread Safety
    ///
    /// This method uses file-based locking to prevent concurrent worktree creation
    /// from multiple git-workers processes. A lock file is created in the `.git`
    /// directory and automatically removed when the operation completes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Another git-workers process is currently creating a worktree
    /// - The worktree path already exists
    /// - The new branch name already exists
    /// - Git command execution fails
    /// - Parent directory creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// // Create a new feature branch from main
    /// let path = manager.create_worktree_with_new_branch(
    ///     "feature-x",
    ///     "feature-x",
    ///     "main"
    /// ).unwrap();
    /// ```
    pub fn create_worktree_with_new_branch(
        &self,
        name: &str,
        new_branch: &str,
        base_branch: &str,
    ) -> Result<PathBuf> {
        // Acquire lock to prevent concurrent worktree creation
        let _lock = WorktreeLock::acquire(self.repo.path())?;

        let base_path = self.determine_worktree_base_path()?;

        // Handle different path patterns (same as create_worktree)
        let worktree_path = if name.starts_with("../") {
            let repo_dir = self
                .repo
                .workdir()
                .or_else(|| self.repo.path().parent())
                .ok_or_else(|| anyhow!(ERROR_NO_REPO_DIR))?;
            repo_dir.join(name)
        } else if name.contains('/') {
            let repo_dir = self.repo.workdir().unwrap_or_else(|| self.repo.path());
            repo_dir.join(name)
        } else {
            base_path.join(name)
        };

        // Ensure parent directories exist
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if worktree_path.exists() {
            return Err(anyhow!(
                "{}",
                ERROR_WORKTREE_PATH_EXISTS.replace("{}", &worktree_path.display().to_string())
            ));
        }

        // Extract the actual worktree name (unused but kept for consistency)
        let _worktree_name = worktree_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);

        // Canonicalize the path
        let canonical_path = worktree_path
            .canonicalize()
            .or_else(|_| -> Result<PathBuf> {
                if let Some(parent) = worktree_path.parent() {
                    if let Ok(canonical_parent) = parent.canonicalize() {
                        return Ok(canonical_parent.join(worktree_path.file_name().unwrap()));
                    }
                }
                Ok(worktree_path.clone())
            })
            .unwrap_or_else(|_| worktree_path.clone());

        // Use git CLI to create worktree with new branch
        use std::process::Command;
        let output = Command::new(GIT_CMD)
            .current_dir(self.get_git_dir()?)
            .arg(GIT_WORKTREE)
            .arg(GIT_ADD)
            .arg(GIT_OPT_BRANCH)
            .arg(new_branch)
            .arg(&canonical_path)
            .arg(base_branch)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "{}",
                ERROR_WORKTREE_CREATE.replace("{}", &String::from_utf8_lossy(&output.stderr))
            ));
        }

        // Return the canonicalized path
        canonical_path
            .canonicalize()
            .or_else(|_| Ok(canonical_path))
    }

    /// Gets the branch that is checked out in each worktree
    ///
    /// This method maps branch names to their corresponding worktree names,
    /// including both the main worktree (repository itself) and all linked worktrees.
    /// This is useful for preventing multiple checkouts of the same branch.
    ///
    /// # Returns
    ///
    /// A HashMap where:
    /// - Key: Branch name (e.g., "main", "feature-x", "origin/remote-branch")
    /// - Value: Worktree name (e.g., "git-workers", "feature-worktree")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// let map = manager.get_branch_worktree_map().unwrap();
    /// if let Some(worktree) = map.get("main") {
    ///     println!("Branch 'main' is checked out in worktree '{}'", worktree);
    /// }
    /// ```
    pub fn get_branch_worktree_map(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut map = std::collections::HashMap::new();

        // First, check the main worktree (the repository itself)
        if let Ok(head) = self.repo.head() {
            if head.is_branch() {
                if let Some(branch_name) = head.shorthand() {
                    // For main worktree, use the repository path as the name
                    let main_worktree_name = if let Some(workdir) = self.repo.workdir() {
                        workdir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(GIT_DEFAULT_MAIN_WORKTREE)
                            .to_string()
                    } else {
                        GIT_DEFAULT_MAIN_WORKTREE.to_string()
                    };
                    map.insert(branch_name.to_string(), main_worktree_name);
                }
            } else if let Some(shorthand) = head.shorthand() {
                let main_worktree_name = if let Some(workdir) = self.repo.workdir() {
                    workdir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(crate::constants::DEFAULT_BRANCH_MAIN)
                        .to_string()
                } else {
                    crate::constants::DEFAULT_BRANCH_MAIN.to_string()
                };
                map.insert(shorthand.to_string(), main_worktree_name);
            }
        }

        // Then check all linked worktrees
        let worktree_names = self.repo.worktrees()?;
        for name in worktree_names.iter().flatten() {
            if let Ok(worktree) = self.repo.find_worktree(name) {
                if let Ok(repo) = Repository::open(worktree.path()) {
                    if let Ok(head) = repo.head() {
                        // Check if HEAD is a direct reference to a branch
                        if head.is_branch() {
                            // It's a local branch
                            if let Some(branch_name) = head.shorthand() {
                                map.insert(branch_name.to_string(), name.to_string());
                            }
                        } else if let Some(shorthand) = head.shorthand() {
                            // It might be a remote tracking branch or detached HEAD
                            // For remote tracking branches, the shorthand will be like "origin/branch"
                            // We'll include these in the map as well
                            map.insert(shorthand.to_string(), name.to_string());
                        }
                    }
                }
            }
        }

        Ok(map)
    }

    /// Determines the base path for creating new worktrees
    ///
    /// This method implements the pattern detection logic:
    /// 1. First, check if existing worktrees share a common parent
    /// 2. If yes, use that parent to maintain consistency
    /// 3. If no, fall back to the default base path
    ///
    /// # Returns
    ///
    /// The base path where new worktrees should be created
    ///
    /// # Errors
    ///
    /// Returns an error if the default base path cannot be determined
    fn determine_worktree_base_path(&self) -> Result<PathBuf> {
        if let Ok(existing_worktrees) = self.list_worktrees() {
            if let Some(common_parent) = find_common_parent(&existing_worktrees) {
                return Ok(common_parent);
            }
        }
        self.get_default_worktree_base_path()
    }

    /// Creates a worktree with a specific branch or tag
    ///
    /// Uses the git CLI command for branch-based worktree creation because
    /// git2's worktree API has limitations with branch handling.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path for the new worktree
    /// * `branch_name` - The branch or tag to check out in the worktree
    ///
    /// # Behavior
    ///
    /// - If a tag reference: Creates worktree at the tag commit (detached HEAD)
    /// - If the branch exists: Creates worktree with that branch checked out
    /// - If the branch doesn't exist: Creates a new branch and worktree
    ///
    /// # Returns
    ///
    /// The path to the created worktree
    ///
    /// # Errors
    ///
    /// Returns an error if the git command fails
    pub fn create_worktree_with_branch(&self, path: &Path, branch_name: &str) -> Result<PathBuf> {
        use std::process::Command;

        let mut cmd = Command::new(GIT_CMD);
        cmd.current_dir(self.get_git_dir()?);

        // Check if this is a tag reference
        if self
            .repo
            .find_reference(&format!("{GIT_REFS_TAGS}{branch_name}"))
            .is_ok()
        {
            // For tags, we need to create a detached HEAD worktree
            // git worktree add <path> <tag>
            cmd.arg(GIT_WORKTREE)
                .arg(GIT_ADD)
                .arg(path)
                .arg(branch_name);
        } else if branch_name.starts_with(GIT_ORIGIN) {
            // For remote branches, we need to create a local branch
            // Extract the branch name without "origin/" prefix
            let local_branch_name = branch_name.strip_prefix(GIT_ORIGIN).unwrap_or(branch_name);

            // Check if a local branch with this name already exists
            let local_exists = self
                .repo
                .find_branch(local_branch_name, BranchType::Local)
                .is_ok();

            if local_exists {
                // Local branch exists - this might fail if it's already checked out
                // Let's return a more helpful error message
                return Err(anyhow!(
                    "Cannot create worktree from '{}': A local branch '{}' already exists.\n\
                     If you want to use the existing local branch, please select it from the local branches list.\n\
                     If you want to create a new worktree from the remote branch, consider using a different name.",
                    branch_name,
                    local_branch_name
                ));
            } else {
                // Create new local branch from remote
                cmd.arg(GIT_WORKTREE)
                    .arg(GIT_ADD)
                    .arg(GIT_OPT_BRANCH)
                    .arg(local_branch_name)
                    .arg(path)
                    .arg(branch_name);
            }
        } else {
            // Check if local branch exists
            let branch_exists = self
                .repo
                .find_branch(branch_name, BranchType::Local)
                .is_ok();

            if branch_exists {
                // If branch exists, create worktree pointing to that branch
                cmd.arg(GIT_WORKTREE)
                    .arg(GIT_ADD)
                    .arg(path)
                    .arg(branch_name);
            } else {
                // If branch doesn't exist, create new branch with worktree
                cmd.arg(GIT_WORKTREE)
                    .arg(GIT_ADD)
                    .arg(GIT_OPT_BRANCH)
                    .arg(branch_name)
                    .arg(path);
            }
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "{}",
                ERROR_WORKTREE_CREATE.replace("{}", &String::from_utf8_lossy(&output.stderr))
            ));
        }

        // Return the canonicalized path
        path.canonicalize().or_else(|_| Ok(path.to_path_buf()))
    }

    /// Creates a worktree from the current HEAD
    ///
    /// This method creates a new worktree from the current HEAD commit, automatically
    /// creating a new branch named after the worktree. It handles path resolution
    /// to ensure consistent behavior across different working directories.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path for the new worktree (can be relative or absolute)
    /// * `name` - The name for the worktree (currently unused, kept for API compatibility)
    ///
    /// # Path Resolution
    ///
    /// - Relative paths are resolved from the current working directory
    /// - Absolute paths are used as-is
    /// - This ensures that paths like `../worktree` work correctly regardless of
    ///   whether the command is run from the repository root or a subdirectory
    ///
    /// # Implementation Notes
    ///
    /// - Uses git CLI command for both bare and non-bare repositories
    /// - Automatically creates a new branch with the worktree name
    /// - The git command is executed from the repository's git directory
    ///
    /// # Returns
    ///
    /// The canonicalized absolute path to the created worktree
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current directory cannot be determined
    /// - The git command fails (e.g., path already exists, no commits)
    /// - Path canonicalization fails after creation
    pub fn create_worktree_from_head(&self, path: &Path, _name: &str) -> Result<PathBuf> {
        use std::process::Command;

        // Convert to absolute path to ensure consistent interpretation by git command
        // This prevents issues when path is relative and current_dir is different
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Resolve relative path from current working directory, not from get_git_dir()
            std::env::current_dir()?.join(path)
        };

        // For both bare and non-bare repositories, use git command without specifying HEAD
        // This will create a new branch with the worktree name automatically
        let output = Command::new(GIT_CMD)
            .current_dir(self.get_git_dir()?)
            .arg(GIT_WORKTREE)
            .arg(GIT_ADD)
            .arg(&absolute_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "{}",
                ERROR_WORKTREE_CREATE.replace("{}", &String::from_utf8_lossy(&output.stderr))
            ));
        }

        // Return the canonicalized path, or the absolute path if canonicalization fails
        absolute_path.canonicalize().or_else(|_| Ok(absolute_path))
    }

    /// Removes a worktree by name
    ///
    /// This prunes the worktree, removing both the Git metadata and
    /// the working directory. The operation is atomic - either everything
    /// is removed or nothing is.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the worktree to remove
    ///
    /// # Safety
    ///
    /// - Cannot remove the current worktree (checked by caller)
    /// - Removes all files in the worktree directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The worktree doesn't exist
    /// - The worktree is locked
    /// - File system operations fail
    pub fn remove_worktree(&self, name: &str) -> Result<()> {
        let worktree = self.repo.find_worktree(name)?;
        worktree.prune(Some(
            &mut git2::WorktreePruneOptions::new()
                .valid(true)
                .working_tree(true),
        ))?;
        Ok(())
    }

    /// Lists all branches (local and remote) in the repository
    ///
    /// This method provides a comprehensive list of all branches, separated by type.
    /// Remote branch names have the "origin/" prefix stripped for cleaner display,
    /// and HEAD references are excluded from the remote branches list.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<String>` of local branch names (sorted alphabetically)
    /// - `Vec<String>` of remote branch names without "origin/" prefix (sorted alphabetically)
    ///
    /// # Errors
    ///
    /// Returns an error if branch enumeration fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// let (local_branches, remote_branches) = manager.list_all_branches().unwrap();
    /// println!("Local branches: {:?}", local_branches);
    /// println!("Remote branches: {:?}", remote_branches);
    /// ```
    pub fn list_all_branches(&self) -> Result<(Vec<String>, Vec<String>)> {
        let mut local_branches = Vec::new();
        let mut remote_branches = Vec::new();

        // Get local branches
        let local_iter = self.repo.branches(Some(BranchType::Local))?;
        for (branch, _) in local_iter.flatten() {
            if let Some(name) = branch.name()? {
                local_branches.push(name.to_string());
            }
        }

        // Get remote branches
        let remote_iter = self.repo.branches(Some(BranchType::Remote))?;
        for (branch, _) in remote_iter.flatten() {
            if let Some(name) = branch.name()? {
                // Remove "origin/" prefix for cleaner display
                let clean_name = name.strip_prefix(GIT_ORIGIN).unwrap_or(name);
                // Skip HEAD references
                if clean_name != GIT_RESERVED_NAMES[GIT_HEAD_INDEX] {
                    remote_branches.push(clean_name.to_string());
                }
            }
        }

        local_branches.sort();
        remote_branches.sort();

        Ok((local_branches, remote_branches))
    }

    /// Lists all tags in the repository
    ///
    /// This method retrieves all tags, including both lightweight and annotated tags.
    /// For annotated tags, it includes the tag message if available.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing:
    /// - Tag name (`String`)
    /// - Tag message (`Option<String>`) - Some for annotated tags, None for lightweight tags
    ///
    /// # Errors
    ///
    /// Returns an error if tag enumeration fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use git_workers::git::GitWorktreeManager;
    /// # let manager = GitWorktreeManager::new().unwrap();
    /// let tags = manager.list_all_tags().unwrap();
    /// for (name, message) in tags {
    ///     if let Some(msg) = message {
    ///         println!("Tag: {} - {}", name, msg);
    ///     } else {
    ///         println!("Tag: {}", name);
    ///     }
    /// }
    /// ```
    pub fn list_all_tags(&self) -> Result<Vec<(String, Option<String>)>> {
        let mut tags = Vec::new();

        self.repo.tag_foreach(|oid, name| {
            if let Ok(name_str) = std::str::from_utf8(name) {
                // Remove refs/tags/ prefix
                let tag_name = name_str
                    .strip_prefix(GIT_REFS_TAGS)
                    .unwrap_or(name_str)
                    .to_string();

                // Try to get tag message for annotated tags
                let tag_message = if let Ok(obj) = self.repo.find_object(oid, None) {
                    if let Ok(tag) = obj.peel_to_tag() {
                        tag.message().map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                tags.push((tag_name, tag_message));
            }
            true
        })?;

        // Sort tags by name (reverse order to show newest versions first)
        tags.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(tags)
    }

    /// Deletes a local branch by name
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch to delete
    ///
    /// # Safety
    ///
    /// This performs a force delete. The caller should ensure:
    /// - The branch is not currently checked out in any worktree
    /// - Any important changes have been merged
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The branch doesn't exist
    /// - The branch cannot be deleted (e.g., currently checked out)
    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        match self.repo.find_branch(branch_name, BranchType::Local) {
            Ok(mut branch) => {
                branch.delete()?;
                Ok(())
            }
            Err(_) => Err(anyhow!(GIT_BRANCH_NOT_FOUND_MSG.replace("{}", branch_name))),
        }
    }

    /// Checks if a branch is unique to a specific worktree
    ///
    /// This is used to determine if a branch should be offered for deletion
    /// when removing a worktree.
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch to check
    /// * `worktree_name` - The name of the worktree to check against
    ///
    /// # Returns
    ///
    /// `true` if the branch is only checked out in the specified worktree
    /// and not in any other worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if worktree enumeration fails
    pub fn is_branch_unique_to_worktree(
        &self,
        branch_name: &str,
        worktree_name: &str,
    ) -> Result<bool> {
        let worktrees = self.list_worktrees()?;
        let mut count = 0;
        let mut found_in_target = false;

        for worktree in &worktrees {
            if worktree.branch == branch_name {
                count += 1;
                if worktree.name == worktree_name {
                    found_in_target = true;
                }
            }
        }

        Ok(found_in_target && count == 1)
    }

    /// Renames a branch
    ///
    /// Uses the git CLI for more robust branch renaming, as it handles
    /// all the edge cases better than the git2 API.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current name of the branch
    /// * `new_name` - The new name for the branch
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The old branch doesn't exist
    /// - The new branch name already exists
    /// - The branch is currently checked out (in some Git versions)
    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        use std::process::Command;

        // Use git CLI for more robust branch renaming
        let output = Command::new(GIT_CMD)
            .current_dir(self.get_git_dir()?)
            .arg(GIT_BRANCH)
            .arg(GIT_OPT_RENAME)
            .arg(old_name)
            .arg(new_name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_msg = stderr.trim();
            return Err(anyhow!("Failed to rename branch: {error_msg}"));
        }

        Ok(())
    }

    /// Checks if a worktree has uncommitted changes
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository object for the worktree
    ///
    /// # Returns
    ///
    /// `true` if there are any uncommitted changes (including untracked files)
    ///
    /// # Errors
    ///
    /// Returns an error if status enumeration fails
    #[allow(dead_code)]
    fn check_worktree_changes(&self, repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .include_ignored(false),
        ))?;

        Ok(!statuses.is_empty())
    }

    /// Gets information about the last commit in a repository
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository to query
    ///
    /// # Returns
    ///
    /// A [`CommitInfo`] struct with commit details
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The repository has no commits
    /// - Cannot access HEAD
    #[allow(dead_code)]
    fn get_last_commit(&self, repo: &Repository) -> Result<CommitInfo> {
        let head = repo.head()?.peel_to_commit()?;
        let time = chrono::DateTime::from_timestamp(head.time().seconds(), 0)
            .unwrap_or_default()
            .format(TIME_FORMAT)
            .to_string();

        let id = head.id().to_string()[..COMMIT_ID_SHORT_LENGTH].to_string();
        let message = head
            .summary()
            .unwrap_or(GIT_COMMIT_MESSAGE_NONE)
            .to_string();
        let author = head
            .author()
            .name()
            .unwrap_or(GIT_COMMIT_AUTHOR_UNKNOWN)
            .to_string();

        Ok(CommitInfo {
            id,
            message,
            author,
            time,
        })
    }

    /// Renames a worktree, including all associated Git metadata
    ///
    /// This is a complex operation that involves:
    /// 1. Moving the worktree directory
    /// 2. Renaming the `.git/worktrees/<name>` metadata directory
    /// 3. Updating the `gitdir` file to point to the new location
    /// 4. Updating the `.git` file in the worktree
    /// 5. Running `git worktree repair` to fix any remaining references
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current name of the worktree
    /// * `new_name` - The desired new name
    ///
    /// # Returns
    ///
    /// The new path to the renamed worktree
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The new name contains whitespace
    /// - The target path already exists
    /// - The worktree is currently active
    /// - The worktree has a detached HEAD
    /// - Any file system operations fail
    ///
    /// # Note
    ///
    /// Branch renaming is handled separately by the caller if needed.
    pub fn rename_worktree(&self, old_name: &str, new_name: &str) -> Result<PathBuf> {
        self.rename_worktree_with_fs(
            old_name,
            new_name,
            &crate::filesystem::RealFileSystem::new(),
        )
    }

    /// Internal implementation of rename_worktree with filesystem abstraction
    pub fn rename_worktree_with_fs(
        &self,
        old_name: &str,
        new_name: &str,
        fs: &dyn FileSystem,
    ) -> Result<PathBuf> {
        use std::process::Command;

        // Validate new name
        if new_name.contains(char::is_whitespace) {
            return Err(anyhow!(GIT_NEW_NAME_NO_SPACES));
        }

        // Get the old worktree info
        let old_worktree = self.repo.find_worktree(old_name)?;
        let old_path = old_worktree.path().to_path_buf();

        // Generate new path
        let new_path = old_path
            .parent()
            .ok_or_else(|| anyhow!(GIT_CANNOT_FIND_PARENT))?
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
            return Err(anyhow!(GIT_CANNOT_RENAME_CURRENT));
        }

        // Validate the worktree is not in detached HEAD state
        if let Ok(wt_repo) = Repository::open(&old_path) {
            if let Ok(head) = wt_repo.head() {
                if !head.is_branch() {
                    return Err(anyhow!(GIT_CANNOT_RENAME_DETACHED));
                }
            } else {
                return Err(anyhow!("Cannot read worktree HEAD"));
            }
        } else {
            return Err(anyhow!("Cannot open worktree repository"));
        }

        // Step 1: Move the directory
        fs.rename(&old_path, &new_path)?;

        // Step 2: Rename the git metadata directory
        // Use git rev-parse to find the common git directory
        let output = Command::new(GIT_CMD)
            .current_dir(self.get_git_dir()?)
            .arg(GIT_REV_PARSE)
            .arg(GIT_OPT_GIT_COMMON_DIR)
            .output()?;

        let git_common_dir = if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            PathBuf::from(path_str)
        } else {
            // Fallback to repo path
            self.repo.path().to_path_buf()
        };

        let old_worktree_git_dir = git_common_dir
            .join(crate::constants::WORKTREES_SUBDIR)
            .join(old_name);
        let new_worktree_git_dir = git_common_dir
            .join(crate::constants::WORKTREES_SUBDIR)
            .join(new_name);

        if old_worktree_git_dir.exists() {
            fs.rename(&old_worktree_git_dir, &new_worktree_git_dir)?;

            // Update the gitdir file
            let gitdir_file = new_worktree_git_dir.join("gitdir");
            if gitdir_file.exists() {
                let new_path_str = new_path.display();
                fs.write(&gitdir_file, &format!("{new_path_str}{GIT_GITDIR_SUFFIX}"))?;
            }
        }

        // Step 3: Update the .git file in the worktree
        let git_file_path = new_path.join(GIT_DIR);
        if git_file_path.exists() {
            let git_dir_str = new_worktree_git_dir.display();
            let git_file_content = format!("{GIT_GITDIR_PREFIX}{git_dir_str}\n");
            fs.write(&git_file_path, &git_file_content)?;
        }

        // Step 4: Run git worktree repair to update Git's internal tracking
        // Note: This won't rename the worktree in Git's tracking, but will ensure
        // the paths are correct
        let repair_output = Command::new(GIT_CMD)
            .current_dir(self.get_git_dir()?)
            .args([GIT_WORKTREE, GIT_REPAIR])
            .output()?;

        if !repair_output.status.success() {
            eprintln!(
                "Warning: git worktree repair failed: {}",
                String::from_utf8_lossy(&repair_output.stderr)
            );
        }

        // Branch renaming is handled separately by the caller

        Ok(new_path)
    }

    /// Gets the ahead/behind count relative to the upstream branch
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository to check
    ///
    /// # Returns
    ///
    /// A tuple of (ahead, behind) counts:
    /// - `ahead`: Number of commits ahead of upstream
    /// - `behind`: Number of commits behind upstream
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not on a branch (detached HEAD)
    /// - No upstream branch is configured
    /// - Cannot compute the graph difference
    #[allow(dead_code)]
    fn get_ahead_behind(&self, repo: &Repository) -> Result<(usize, usize)> {
        let head = repo.head()?;
        if !head.is_branch() {
            return Err(anyhow!("Not on a branch"));
        }

        let local_oid = head.target().ok_or_else(|| anyhow!("No target"))?;
        let branch_name = head.shorthand().ok_or_else(|| anyhow!("No branch name"))?;

        // Try to find upstream branch
        let upstream_name = format!("{GIT_ORIGIN}{branch_name}");
        if let Ok(upstream) = repo.find_reference(&format!("{GIT_REFS_REMOTES}{upstream_name}")) {
            let upstream_oid = upstream.target().ok_or_else(|| anyhow!("No target"))?;
            let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
            Ok((ahead, behind))
        } else {
            Err(anyhow!("No upstream branch"))
        }
    }
}

/// Status information for a worktree
///
/// This struct is used internally to collect status information
/// about a worktree in a single pass.
struct WorktreeStatus {
    has_changes: bool,
    last_commit: Option<CommitInfo>,
    ahead_behind: Option<(usize, usize)>,
}

/// Gets the status information for a worktree
///
/// This function opens the worktree repository and collects various
/// status information. It's designed to be called from multiple threads
/// in parallel.
///
/// # Arguments
///
/// * `path` - The filesystem path to the worktree
///
/// # Returns
///
/// A [`WorktreeStatus`] struct with the collected information.
/// If the repository cannot be opened, returns a status with all
/// fields set to their default/empty values.
///
/// # Performance
///
/// This function is optimized for speed over completeness. Some
/// expensive operations (like ahead/behind calculation) are skipped.
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
    #[allow(dead_code)]
    pub last_commit: Option<CommitInfo>,
    /// Number of commits ahead and behind the upstream branch
    #[allow(dead_code)]
    pub ahead_behind: Option<(usize, usize)>, // (ahead, behind)
}

/// Information about a Git commit
///
/// Contains basic information about a commit for display purposes.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Short commit ID (first 8 characters)
    #[allow(dead_code)]
    pub id: String,
    /// First line of the commit message
    #[allow(dead_code)]
    pub message: String,
    /// Commit author name
    #[allow(dead_code)]
    pub author: String,
    /// Formatted commit time (YYYY-MM-DD HH:MM)
    #[allow(dead_code)]
    pub time: String,
}

/// Convenience function to list worktrees from the current directory
///
/// This is a wrapper around GitWorktreeManager for simple CLI usage.
/// It discovers the repository from the current directory and returns
/// a formatted list of worktrees.
///
/// # Returns
///
/// A vector of formatted strings in the format "name (branch)"
///
/// # Errors
///
/// Returns an error if:
/// - Not in a Git repository
/// - Cannot enumerate worktrees
///
/// # Example
///
/// ```no_run
/// # use git_workers::git::list_worktrees;
/// match list_worktrees() {
///     Ok(worktrees) => {
///         for wt in worktrees {
///             println!("{}", wt);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[allow(dead_code)]
pub fn list_worktrees() -> Result<Vec<String>> {
    let manager = GitWorktreeManager::new()?;
    let worktrees = manager.list_worktrees()?;

    Ok(worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect())
}
