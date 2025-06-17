//! Configuration management for Git Workers
//!
//! This module handles loading and parsing configuration files for Git Workers.
//! Configuration can be stored in two locations:
//! 1. Local: `.git-workers.toml` in the current directory
//! 2. Global: `~/.config/git-workers/config.toml`
//!
//! Local configuration takes precedence over global configuration.
//!
//! # Configuration Search Order
//!
//! 1. `.git-workers.toml` in the current directory
//! 2. `.git-workers.toml` in parent directories up to the repository root
//! 3. `~/.config/git-workers/config.toml` (global configuration)
//!
//! # File Format
//!
//! Configuration files use TOML format. Currently, only hook definitions
//! are supported, but the format is extensible for future features.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for Git Workers
///
/// Currently supports hook definitions for various worktree lifecycle events.
///
/// # Example Configuration
///
/// ```toml
/// [hooks]
/// post-create = ["npm install", "cp .env.example .env"]
/// pre-remove = ["rm -rf node_modules"]
/// post-switch = ["echo 'Switched to {{worktree_name}}'"]
/// ```
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
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

impl Config {
    /// Loads configuration from the filesystem
    ///
    /// This method searches for configuration files in the following order:
    /// 1. `.git-workers.toml` in the current directory (project-specific)
    /// 2. `~/.config/git-workers/config.toml` (user global)
    ///
    /// The first configuration file found is used. If no configuration
    /// file exists, a default empty configuration is returned.
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
        // Try to load from current directory first
        let local_config = PathBuf::from(".git-workers.toml");
        if local_config.exists() {
            let content = fs::read_to_string(&local_config)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }

        // Try to load from global config
        if let Some(config_dir) = dirs::config_dir() {
            let global_config = config_dir.join("git-workers").join("config.toml");
            if global_config.exists() {
                let content = fs::read_to_string(&global_config)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        // Return default config if no config file found
        Ok(Config::default())
    }
}
