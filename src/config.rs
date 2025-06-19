//! Configuration management for Git Workers
//!
//! This module handles loading and parsing configuration files for Git Workers.
//! Configuration is loaded from `.git-workers.toml` in the default branch (main or master).
//!
//! This ensures all worktrees use the same configuration, preventing inconsistencies
//! between different worktrees.
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
    /// This method loads `.git-workers.toml` from the default branch (main or master).
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
    pub fn load() -> Result<Self> {
        if let Ok(repo) = git2::Repository::discover(".") {
            // Try to find .git-workers.toml in the default branch (main or master)
            if let Some(config) = Self::load_from_default_branch(&repo)? {
                return Ok(config);
            }
        }

        // Return default config if no config file found
        Ok(Config::default())
    }

    /// Loads configuration from the default branch (main or master)
    ///
    /// This method implements the configuration lookup strategy:
    ///
    /// 1. **Current directory**: First checks the current directory for `.git-workers.toml`
    ///    (useful for bare repository worktrees)
    /// 2. **Parent main/master**: If in a worktree structure, looks for config in the
    ///    main/master worktree in the parent directory
    /// 3. **Repository root**: Falls back to checking the current repository's
    ///    working directory
    ///
    /// This ensures all worktrees share the same configuration by loading it
    /// from a consistent location, while also supporting bare repository workflows.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Git repository to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(config))` - Configuration was found and loaded
    /// * `Ok(None)` - No configuration file exists
    /// * `Err(...)` - An error occurred while loading
    fn load_from_default_branch(repo: &git2::Repository) -> Result<Option<Self>> {
        // First, check current directory (useful for bare repo worktrees)
        if let Ok(cwd) = std::env::current_dir() {
            let current_config = cwd.join(".git-workers.toml");
            if current_config.exists() {
                return Self::load_from_file(&current_config, repo);
            }

            // Check if we're in a worktree structure like /path/to/repo/branch/worktree-name
            if let Some(parent) = cwd.parent() {
                // Look for main or master directories in the parent
                let main_path = parent.join("main").join(".git-workers.toml");
                let master_path = parent.join("master").join(".git-workers.toml");

                if main_path.exists() {
                    return Self::load_from_file(&main_path, repo);
                } else if master_path.exists() {
                    return Self::load_from_file(&master_path, repo);
                }
            }
        }

        // If not found in parent, check the current repository
        if let Some(workdir) = repo.workdir() {
            let config_path = workdir.join(".git-workers.toml");
            if config_path.exists() {
                return Self::load_from_file(&config_path, repo);
            }
        }

        Ok(None)
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
                eprintln!("Warning: Failed to read .git-workers.toml: {}", e);
                return Ok(None);
            }
        };

        let config = match toml::from_str::<Config>(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to parse .git-workers.toml: {}", e);
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
