//! Utility functions for terminal output formatting
//!
//! This module provides consistent, colored output functions for displaying
//! progress, success, and error messages in the terminal.

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

/// Gets a consistent terminal instance for UI operations
///
/// Returns a terminal instance using stderr to avoid conflicts with
/// command output that might be piped or processed by shell functions.
///
/// # Panics
///
/// Panics if not running in a terminal environment, as the application
/// requires interactive terminal capabilities.
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
pub fn get_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}
