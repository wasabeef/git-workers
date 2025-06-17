//! Hook system for executing custom commands on worktree events
//!
//! This module provides functionality to execute user-defined commands
//! at specific points in the worktree lifecycle. Hooks are configured
//! via the configuration file and can access context information about
//! the worktree being operated on.
//!
//! # Configuration
//!
//! Hooks are configured in the `.git-workers.toml` file in the repository root:
//!
//! ```toml
//! [hooks]
//! post-create = ["npm install", "cp .env.example .env"]
//! pre-remove = ["rm -rf node_modules"]
//! post-switch = ["echo 'Switched to {{worktree_name}}'"]
//! ```
//!
//! # Hook Types
//!
//! - `post-create`: Executed after a worktree is created
//! - `pre-remove`: Executed before a worktree is removed
//! - `post-switch`: Executed after switching to a different worktree
//!
//! # Template Variables
//!
//! Hook commands support template variables:
//! - `{{worktree_name}}`: The name of the worktree
//! - `{{worktree_path}}`: The absolute path to the worktree

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;

/// Context information passed to hook commands
///
/// This struct contains information about the worktree that hooks
/// can use via template placeholders in their command strings.
pub struct HookContext {
    /// The name of the worktree being operated on
    pub worktree_name: String,
    /// The full filesystem path to the worktree
    pub worktree_path: PathBuf,
}

/// Executes configured hooks for a specific event type
///
/// This function loads the configuration, looks up hooks for the specified
/// event type, and executes them in order. Each command is run in a shell
/// with the worktree directory as the working directory.
///
/// # Arguments
///
/// * `hook_type` - The type of hook to execute (e.g., "post-create", "pre-remove")
/// * `context` - Context information about the worktree
///
/// # Hook Types
///
/// - `post-create`: Run after a worktree is created
/// - `pre-remove`: Run before a worktree is removed
/// - `post-switch`: Run after switching to a worktree
///
/// # Template Placeholders
///
/// Commands can include the following placeholders:
/// - `{{worktree_name}}`: Replaced with the worktree name
/// - `{{worktree_path}}`: Replaced with the full worktree path
///
/// # Example
///
/// ```no_run
/// use git_workers::hooks::{execute_hooks, HookContext};
/// use std::path::PathBuf;
///
/// let context = HookContext {
///     worktree_name: "feature-branch".to_string(),
///     worktree_path: PathBuf::from("/path/to/worktree"),
/// };
///
/// // Execute post-create hooks
/// execute_hooks("post-create", &context).ok();
/// ```
///
/// # Error Handling
///
/// Hook failures are logged to stderr but do not stop execution of
/// subsequent hooks or the main operation. This ensures that a failing
/// hook doesn't prevent worktree operations from completing.
pub fn execute_hooks(hook_type: &str, context: &HookContext) -> Result<()> {
    let config = Config::load()?;

    if let Some(commands) = config.hooks.get(hook_type) {
        println!("Running {} hooks...", hook_type);

        for cmd in commands {
            // Replace template placeholders with actual values
            let expanded_cmd = cmd
                .replace("{{worktree_name}}", &context.worktree_name)
                .replace(
                    "{{worktree_path}}",
                    &context.worktree_path.display().to_string(),
                );

            println!("  > {}", expanded_cmd);

            // Execute the command in a shell for maximum compatibility
            // This allows complex commands with pipes, redirects, etc.
            let output = Command::new("sh")
                .arg("-c")
                .arg(&expanded_cmd)
                .current_dir(&context.worktree_path)
                .output()?;

            if !output.status.success() {
                // Log hook failures but don't stop execution
                // This prevents a misconfigured hook from breaking worktree operations
                eprintln!(
                    "Hook command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    Ok(())
}
