//! Utility functions for terminal output formatting
//!
//! This module provides consistent, colored output functions for displaying
//! progress, success, and error messages in the terminal. All output functions
//! use consistent emoji icons and color schemes to create a cohesive user
//! experience.
//!
//! # Design Principles
//!
//! - **Consistent Icons**: Each message type has a unique emoji icon
//! - **Color Coding**: Green for success, red for errors, yellow for warnings/progress
//! - **Line Overwriting**: Progress messages can be overwritten by results
//! - **Immediate Feedback**: All output is flushed immediately

use colored::*;
use console::Term;
use dialoguer::theme::ColorfulTheme;
use std::io::{self, Write};

/// Displays a progress message with a spinning hourglass emoji
///
/// This function prints a message without a newline, allowing it to be
/// overwritten by subsequent success or error messages. The output is
/// immediately flushed to ensure it appears even without a newline.
///
/// # Arguments
///
/// * `message` - The progress message to display
///
/// # Example
///
/// ```no_run
/// use git_workers::utils::print_progress;
///
/// print_progress("Creating worktree...");
/// // Perform operation
/// // Then call print_success or print_error to overwrite
/// ```
pub fn print_progress(message: &str) {
    print!("{} {}", "⏳".yellow(), message);
    io::stdout().flush().unwrap();
}

/// Displays a success message with a green checkmark
///
/// This function uses a carriage return to overwrite any previous progress
/// message on the same line, providing a clean transition from progress
/// to completion status.
///
/// # Arguments
///
/// * `message` - The success message to display
///
/// # Example
///
/// ```no_run
/// use git_workers::utils::{print_progress, print_success};
///
/// print_progress("Creating worktree...");
/// // Operation succeeds
/// print_success("Worktree created successfully!");
/// ```
pub fn print_success(message: &str) {
    println!("\r{} {}", "✓".green(), message);
}

/// Displays an error message with a red X mark
///
/// Similar to `print_success`, this function overwrites any previous
/// progress message to show an error status.
///
/// # Arguments
///
/// * `message` - The error message to display
///
/// # Example
///
/// ```no_run
/// use git_workers::utils::{print_progress, print_error};
///
/// print_progress("Creating worktree...");
/// // Operation fails
/// print_error("Failed to create worktree: permission denied");
/// ```
pub fn print_error(message: &str) {
    println!("\r{} {}", "✗".red(), message);
}

/// Displays a warning message with a yellow exclamation mark
///
/// This function shows non-fatal issues that the user should be aware of
/// but that don't prevent the operation from continuing.
///
/// # Arguments
///
/// * `message` - The warning message to display
///
/// # Example
///
/// ```no_run
/// use git_workers::utils::print_warning;
///
/// print_warning("Hook execution failed: command not found");
/// ```
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

/// Gets a consistent terminal instance for UI operations
///
/// Returns a terminal instance using stderr to avoid conflicts with
/// command output that might be piped or processed by shell functions.
///
/// # Panics
///
/// Panics if not running in a terminal environment, as the application
/// requires interactive terminal capabilities.
#[allow(dead_code)]
pub fn get_terminal() -> Term {
    let term = Term::stderr();
    if !term.is_term() {
        eprintln!("Error: git-workers requires a terminal environment.");
        eprintln!("Non-interactive environments are not supported.");
        std::process::exit(1);
    }
    term
}

/// Returns a consistent theme for all dialoguer prompts
///
/// This ensures consistent styling across all interactive prompts in the application.
/// The theme controls colors for:
/// - Selection highlights
/// - Active item indicators
/// - Prompt text
/// - Input validation messages
///
/// # Returns
///
/// A [`ColorfulTheme`] instance configured with the application's color scheme
///
/// # Example
///
/// ```no_run
/// use dialoguer::Select;
/// use git_workers::utils::get_theme;
///
/// let selection = Select::with_theme(&get_theme())
///     .with_prompt("Choose an option")
///     .items(&["Option 1", "Option 2"])
///     .interact()
///     .unwrap();
/// ```
pub fn get_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Prompts user to press any key to continue
///
/// This is a common pattern used throughout the application to pause
/// execution and allow the user to read output before returning to the menu.
///
/// # Returns
///
/// Returns `Ok(())` after the user presses a key, or an error if the
/// terminal operation fails.
///
/// # Example
///
/// ```no_run
/// use git_workers::utils::press_any_key_to_continue;
///
/// println!("Operation completed!");
/// press_any_key_to_continue().unwrap();
/// ```
pub fn press_any_key_to_continue() -> io::Result<()> {
    use crate::constants::MSG_PRESS_ANY_KEY;
    println!();
    println!("{}", MSG_PRESS_ANY_KEY);
    Term::stdout().read_key()?;
    Ok(())
}

/// Writes the worktree path for shell integration
///
/// This function handles the logic for communicating with the shell wrapper
/// to enable automatic directory switching when changing worktrees.
///
/// # Arguments
///
/// * `path` - The path to write for the shell to switch to
///
/// # Shell Integration
///
/// Two methods are supported:
/// 1. File-based: Writes to file specified by `GW_SWITCH_FILE` environment variable
/// 2. Stdout marker: Prints `SWITCH_TO:` prefix followed by the path
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_workers::utils::write_switch_path;
///
/// let worktree_path = Path::new("/home/user/project/feature");
/// write_switch_path(worktree_path);
/// ```
pub fn write_switch_path(path: &std::path::Path) {
    use crate::constants::{MSG_SWITCH_FILE_WARNING, SWITCH_TO_PREFIX};

    if let Ok(switch_file) = std::env::var("GW_SWITCH_FILE") {
        if let Err(e) = std::fs::write(&switch_file, path.display().to_string()) {
            eprintln!("{}", MSG_SWITCH_FILE_WARNING.replace("{}", &e.to_string()));
        }
    } else {
        println!("{}{}", SWITCH_TO_PREFIX, path.display());
    }
}
