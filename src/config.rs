//! Configuration management for Git Workers
//!
//! This module handles loading and parsing configuration files for Git Workers.
//! The configuration loading strategy differs between bare and non-bare repositories
//! to ensure the most appropriate configuration file is found.
//!
//! # Configuration File Loading Priority
//!
//! ## Bare Repositories
//!
//! 1. Current directory
//! 2. Default branch directory within current directory (e.g., `./main/`)
//! 3. Detected worktree pattern (using `git worktree list`)
//! 4. Common subdirectories (`branch/`, `worktrees/`)
//! 5. Sibling directories at parent level
//!
//! ## Non-bare Repositories
//!
//! 1. Current directory
//! 2. Main repository directory (where `.git` is a directory)
//! 3. Parent directories for `main/` or `master/`
//!
//! # Repository Identification
//!
//! The configuration file can include a repository URL to ensure hooks
//! are only executed in the intended repository. This prevents accidentally
//! running project-specific hooks in the wrong repository.
//!
//! # File Format
//!
//! Configuration files use TOML format. Currently, only hook definitions
//! are supported, but the format is extensible for future features.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::CONFIG_FILE_NAME;

/// Main configuration structure for Git Workers
///
/// Currently supports hook definitions for various worktree lifecycle events.
///
/// # Example Configuration
///
/// ```toml
/// [repository]
/// url = "https://github.com/owner/repo.git"
///
/// [hooks]
/// post-create = ["npm install", "cp .env.example .env"]
/// pre-remove = ["rm -rf node_modules"]
/// post-switch = ["echo 'Switched to {{worktree_name}}'"]
/// ```
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    /// Repository identification
    #[serde(default)]
    pub repository: RepositoryConfig,

    /// Hook definitions mapping hook names to lists of commands
    ///
    /// Supported hooks:
    /// - `post-create`: Run after creating a new worktree
    /// - `pre-remove`: Run before removing a worktree
    /// - `post-switch`: Run after switching to a worktree
    ///
    /// Commands can include placeholders:
    /// - `{{worktree_name}}`: Replaced with the worktree name
    /// - `{{worktree_path}}`: Replaced with the full worktree path
    #[serde(default)]
    pub hooks: HashMap<String, Vec<String>>,

    /// File copy configuration
    #[serde(default)]
    pub files: FilesConfig,
}

/// File copy configuration for worktree creation
///
/// This configuration allows specifying files that should be copied
/// from the main worktree to new worktrees during creation.
/// This is useful for files that are gitignored but necessary for
/// the project to function (e.g., `.env` files).
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FilesConfig {
    /// List of files to copy when creating a new worktree
    ///
    /// Paths are relative to the source directory (usually main worktree).
    /// Supports both files and directories.
    ///
    /// # Example
    ///
    /// ```toml
    /// [files]
    /// # source = "./templates"  # Optional: custom source directory
    /// copy = [".env", ".env.local", "config/local.json"]
    /// ```
    #[serde(default)]
    pub copy: Vec<String>,

    /// Source directory for files to copy
    ///
    /// If not specified, defaults to the main worktree directory.
    /// Must be an absolute path or relative to the repository root.
    #[serde(default)]
    pub source: Option<String>,
}

/// Repository-specific configuration
///
/// This configuration section allows specifying repository metadata
/// to ensure hooks and other configuration only apply to the intended
/// repository.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RepositoryConfig {
    /// Repository URL for identification
    ///
    /// When specified, Git Workers will verify that the current repository's
    /// origin URL matches this value before executing hooks. This prevents
    /// accidentally running project-specific commands in the wrong repository.
    ///
    /// # Example
    ///
    /// ```toml
    /// [repository]
    /// url = "https://github.com/mycompany/myproject.git"
    /// ```
    ///
    /// # URL Matching
    ///
    /// URLs are normalized for comparison:
    /// - Trailing `.git` is ignored
    /// - Trailing slashes are ignored
    /// - Comparison is case-insensitive
    pub url: Option<String>,
}

impl Config {
    /// Loads configuration from the default branch
    ///
    /// This method loads the configuration file from the default branch (main or master).
    /// If no configuration file exists in the default branch, a default empty
    /// configuration is returned.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration or an error if
    /// the configuration file exists but cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use git_workers::config::Config;
    ///
    /// let config = Config::load().expect("Failed to load config");
    /// if let Some(post_create_hooks) = config.hooks.get("post-create") {
    ///     for command in post_create_hooks {
    ///         println!("Will run: {}", command);
    ///     }
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        if let Ok(repo) = git2::Repository::discover(".") {
            // Only load from the main repository directory
            if let Some(config) = Self::load_from_main_repository_only(&repo)? {
                return Ok(config);
            }
        }

        // Return default config if no config file found
        Ok(Config::default())
    }

    /// Loads configuration from a specific path context
    ///
    /// This method loads configuration following the same rules as the main load method:
    /// - For bare repositories: checks main/master worktree only
    /// - For non-bare repositories: checks current worktree first, then main/master
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to use as context for finding the repository
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration or a default configuration
    /// if no config file is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use git_workers::config::Config;
    /// use std::path::Path;
    ///
    /// let worktree_path = Path::new("/path/to/worktree");
    /// let config = Config::load_from_path(worktree_path)
    ///     .expect("Failed to load config");
    /// ```
    #[allow(dead_code)]
    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        if let Ok(repo) = git2::Repository::discover(path) {
            // Use the same loading logic as load()
            if let Some(config) = Self::load_from_main_repository_only(&repo)? {
                return Ok(config);
            }
        }

        // Return default config if no repo found
        Ok(Config::default())
    }

    /// Loads configuration with repository-aware strategy
    ///
    /// This method implements different loading strategies for bare and non-bare repositories:
    ///
    /// # Bare Repositories
    ///
    /// For bare repositories (e.g., `/path/to/repo.git`), the method:
    /// 1. Checks the current directory for a config file
    /// 2. Looks for config in default branch subdirectories (e.g., `./main/.git-workers.toml`)
    /// 3. Detects existing worktree patterns using `git worktree list`
    /// 4. Falls back to common directory names (`branch/`, `worktrees/`)
    ///
    /// # Non-bare Repositories
    ///
    /// For regular repositories, the method:
    /// 1. Checks the current directory
    /// 2. Finds the main repository directory (where `.git` is a directory, not a file)
    /// 3. Checks for `main/` or `master/` subdirectories in parent paths
    ///
    /// # Arguments
    ///
    /// * `repo` - The Git repository reference
    ///
    /// # Returns
    ///
    /// * `Ok(Some(config))` - Configuration was found and loaded
    /// * `Ok(None)` - No configuration file exists
    /// * `Err(...)` - An error occurred while loading
    fn load_from_main_repository_only(repo: &git2::Repository) -> Result<Option<Self>> {
        if repo.is_bare() {
            // For bare repositories:
            // Get the default branch name from HEAD
            let default_branch = if let Ok(head) = repo.head() {
                head.shorthand()
                    .unwrap_or(crate::constants::DEFAULT_BRANCH_MAIN)
                    .to_string()
            } else {
                // Fallback to common default branch names
                crate::constants::DEFAULT_BRANCH_MAIN.to_string()
            };

            if let Ok(cwd) = std::env::current_dir() {
                // 1. First check current directory for config
                let current_config = cwd.join(CONFIG_FILE_NAME);
                if current_config.exists() {
                    return Self::load_from_file(&current_config, repo);
                }

                // 2. If we're in a directory that might contain worktrees, check for default branch
                let default_config_in_current = cwd.join(&default_branch).join(CONFIG_FILE_NAME);
                if default_config_in_current.exists() {
                    return Self::load_from_file(&default_config_in_current, repo);
                }

                // Also check main/master if different from default
                if let Some(config_path) = crate::utils::find_config_in_default_branches(
                    &cwd,
                    &default_branch,
                    CONFIG_FILE_NAME,
                ) {
                    return Self::load_from_file(&config_path, repo);
                }

                // 2. Try to detect worktree pattern by listing existing worktrees
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

                    // Find common parent directory of worktrees
                    if !worktree_paths.is_empty() {
                        let parent_dirs: Vec<_> = worktree_paths
                            .iter()
                            .filter_map(|p| std::path::Path::new(p).parent())
                            .collect();

                        // Check if all worktrees share a common parent
                        if let Some(first_parent) = parent_dirs.first() {
                            if parent_dirs.iter().all(|p| p == first_parent) {
                                // Look for default branch in the common parent
                                let default_config =
                                    first_parent.join(&default_branch).join(CONFIG_FILE_NAME);
                                if default_config.exists() {
                                    return Self::load_from_file(&default_config, repo);
                                }

                                // Fallback to main/master
                                if let Some(config_path) =
                                    crate::utils::find_config_in_default_branches(
                                        first_parent,
                                        &default_branch,
                                        CONFIG_FILE_NAME,
                                    )
                                {
                                    return Self::load_from_file(&config_path, repo);
                                }
                            }
                        }
                    }
                }

                // 3. Fallback: Check common subdirectories
                for subdir in &[
                    crate::constants::BRANCH_SUBDIR,
                    crate::constants::WORKTREES_SUBDIR,
                ] {
                    let branch_path = cwd
                        .join(subdir)
                        .join(&default_branch)
                        .join(CONFIG_FILE_NAME);
                    if branch_path.exists() {
                        return Self::load_from_file(&branch_path, repo);
                    }
                }

                // 4. Check sibling directories (same level as current)
                if let Some(parent) = cwd.parent() {
                    let default_path = parent.join(&default_branch).join(CONFIG_FILE_NAME);
                    if default_path.exists() {
                        return Self::load_from_file(&default_path, repo);
                    }
                }
            }

            // No config found
            Ok(None)
        } else {
            // For non-bare repositories:
            // 1. First check current directory (current worktree)
            if let Ok(cwd) = std::env::current_dir() {
                let config_path = cwd.join(CONFIG_FILE_NAME);
                if config_path.exists() {
                    return Self::load_from_file(&config_path, repo);
                }

                // 2. Then check main/master default branch worktree
                // Check if current directory is named "worktrees" or inside a worktree
                if cwd
                    .file_name()
                    .is_some_and(|n| n == crate::constants::WORKTREES_SUBDIR)
                {
                    // We're in the worktrees directory itself
                    if let Some(parent) = cwd.parent() {
                        let main_config = parent.join(CONFIG_FILE_NAME);
                        if main_config.exists() && parent.join(".git").is_dir() {
                            return Self::load_from_file(&main_config, repo);
                        }
                    }
                } else {
                    let git_path = cwd.join(".git");
                    if !git_path.is_dir() {
                        // This is a linked worktree, find the main/master worktree
                        if let Some(parent) = cwd.parent() {
                            // Check if we're in a worktrees subdirectory
                            if parent
                                .file_name()
                                .is_some_and(|n| n == crate::constants::WORKTREES_SUBDIR)
                            {
                                // Go up one more level to repository root
                                if let Some(repo_root) = parent.parent() {
                                    // Check for main worktree
                                    let main_config = repo_root.join(CONFIG_FILE_NAME);
                                    if main_config.exists() && repo_root.join(".git").is_dir() {
                                        return Self::load_from_file(&main_config, repo);
                                    }

                                    // Also check main/master subdirectories
                                    let main_path = repo_root
                                        .join(crate::constants::DEFAULT_BRANCH_MAIN)
                                        .join(CONFIG_FILE_NAME);
                                    if main_path.exists() {
                                        return Self::load_from_file(&main_path, repo);
                                    }

                                    let master_path = repo_root
                                        .join(crate::constants::DEFAULT_BRANCH_MASTER)
                                        .join(CONFIG_FILE_NAME);
                                    if master_path.exists() {
                                        return Self::load_from_file(&master_path, repo);
                                    }
                                }
                            } else {
                                // Not in worktrees subdirectory, check parent for main/master
                                let main_path = parent
                                    .join(crate::constants::DEFAULT_BRANCH_MAIN)
                                    .join(CONFIG_FILE_NAME);
                                if main_path.exists() {
                                    return Self::load_from_file(&main_path, repo);
                                }

                                let master_path = parent
                                    .join(crate::constants::DEFAULT_BRANCH_MASTER)
                                    .join(CONFIG_FILE_NAME);
                                if master_path.exists() {
                                    return Self::load_from_file(&master_path, repo);
                                }
                            }
                        }
                    }
                }
            }

            // No config found
            Ok(None)
        }
    }

    /// Loads configuration from a specific file path
    ///
    /// This method:
    /// 1. Reads the TOML file from disk
    /// 2. Parses it into a [`Config`] struct
    /// 3. Validates the repository URL if specified
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `repo` - Repository for URL validation
    ///
    /// # Returns
    ///
    /// * `Ok(Some(config))` - Configuration loaded and validated
    /// * `Ok(None)` - File couldn't be read or parsed (with warning)
    /// * `Ok(Some(default))` - Config loaded but repository URL mismatch
    ///
    /// # Error Handling
    ///
    /// This method logs warnings instead of returning errors for:
    /// - File read failures
    /// - TOML parse errors
    /// - Repository URL mismatches
    ///
    /// This ensures Git Workers continues to function even with invalid config.
    fn load_from_file(path: &std::path::Path, repo: &git2::Repository) -> Result<Option<Self>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", CONFIG_FILE_NAME, e);
                return Ok(None);
            }
        };

        let config = match toml::from_str::<Config>(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", CONFIG_FILE_NAME, e);
                return Ok(None);
            }
        };

        // Validate repository URL if specified
        if let Some(expected_url) = &config.repository.url {
            if !Self::validate_repository_url(repo, expected_url) {
                return Ok(Some(Config::default()));
            }
        }

        Ok(Some(config))
    }

    /// Validates that the repository URL matches the expected URL
    ///
    /// This security feature ensures that hooks defined in a configuration
    /// file only run in the intended repository. This prevents accidentally
    /// running project-specific commands (like database migrations) in the
    /// wrong repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The current Git repository
    /// * `expected_url` - The URL specified in the configuration
    ///
    /// # Returns
    ///
    /// * `true` - URLs match or validation should be skipped
    /// * `false` - URLs don't match (with warning output)
    ///
    /// # Validation Rules
    ///
    /// - Returns `true` if no origin remote exists (local-only repo)
    /// - Returns `true` if origin has no URL configured
    /// - URLs are normalized before comparison:
    ///   - Trailing `.git` suffix is removed
    ///   - Trailing slashes are removed
    ///   - Case-insensitive comparison
    ///
    /// # Example
    ///
    /// These URLs are considered equivalent:
    /// - `https://github.com/owner/repo.git`
    /// - `https://github.com/owner/repo`
    /// - `HTTPS://GITHUB.COM/OWNER/REPO/`
    fn validate_repository_url(repo: &git2::Repository, expected_url: &str) -> bool {
        let remote = match repo.find_remote("origin") {
            Ok(r) => r,
            Err(_) => return true, // No origin remote, skip validation
        };

        let actual_url = match remote.url() {
            Some(u) => u,
            None => return true, // No URL, skip validation
        };

        // Normalize URLs for comparison
        let normalize = |url: &str| {
            url.trim_end_matches(".git")
                .trim_end_matches('/')
                .to_lowercase()
        };

        if normalize(expected_url) != normalize(actual_url) {
            eprintln!("Warning: Repository URL mismatch!");
            eprintln!("  Expected: {}", expected_url);
            eprintln!("  Actual: {}", actual_url);
            eprintln!("  Hooks will not be executed.");
            return false;
        }

        true
    }
}
