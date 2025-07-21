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

use crate::constants::*;
use colored::*;
use console::Term;
use dialoguer::theme::ColorfulTheme;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
    let spinner = ICON_SPINNER.yellow();
    print!("{spinner} {message}");
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
    let checkmark = ICON_SUCCESS.green();
    println!("\r{checkmark} {message}");
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
    let cross = ICON_ERROR.red();
    println!("\r{cross} {message}");
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
    let warning = ICON_WARNING.yellow();
    println!("{warning} {message}");
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
        eprintln!("{ERROR_TERMINAL_REQUIRED}");
        eprintln!("{ERROR_NON_INTERACTIVE}");
        std::process::exit(EXIT_FAILURE);
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
    println!();
    println!("{MSG_PRESS_ANY_KEY}");
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
    if let Ok(switch_file) = std::env::var(ENV_GW_SWITCH_FILE) {
        if let Err(e) = std::fs::write(&switch_file, path.display().to_string()) {
            eprintln!("{MSG_SWITCH_FILE_WARNING_PREFIX}{e}");
        }
    } else {
        let path_display = path.display();
        println!("{SWITCH_TO_PREFIX}{path_display}");
    }
}

/// Checks alternative default branch names for configuration files
///
/// Given a current default branch and a directory path, this function checks
/// for configuration files in the common default branch directories (main/master).
///
/// # Arguments
///
/// * `dir` - The directory to search in
/// * `default_branch` - The current default branch name
/// * `config_file` - The configuration filename to look for
///
/// # Returns
///
/// The path to the found configuration file, or None if not found
pub fn find_config_in_default_branches(
    dir: &Path,
    default_branch: &str,
    config_file: &str,
) -> Option<PathBuf> {
    // Check main branch if it's not the current default
    if default_branch != DEFAULT_BRANCH_MAIN {
        let main_config = dir.join(DEFAULT_BRANCH_MAIN).join(config_file);
        if main_config.exists() {
            return Some(main_config);
        }
    }

    // Check master branch if it's not the current default
    if default_branch != DEFAULT_BRANCH_MASTER {
        let master_config = dir.join(DEFAULT_BRANCH_MASTER).join(config_file);
        if master_config.exists() {
            return Some(master_config);
        }
    }

    None
}

/// Finds alternative default branch directories
///
/// Given a current default branch and a parent directory, this function checks
/// for existing directories with common default branch names (main/master).
///
/// # Arguments
///
/// * `dir` - The directory to search in
/// * `default_branch` - The current default branch name
///
/// # Returns
///
/// The path to the found directory, or None if not found
pub fn find_default_branch_directory(dir: &Path, default_branch: &str) -> Option<PathBuf> {
    // Check main branch if it's not the current default
    if default_branch != DEFAULT_BRANCH_MAIN {
        let main_dir = dir.join(DEFAULT_BRANCH_MAIN);
        if main_dir.exists() && main_dir.is_dir() {
            return Some(main_dir);
        }
    }

    // Check master branch if it's not the current default
    if default_branch != DEFAULT_BRANCH_MASTER {
        let master_dir = dir.join(DEFAULT_BRANCH_MASTER);
        if master_dir.exists() && master_dir.is_dir() {
            return Some(master_dir);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_theme_creation() {
        // Test that we can create a theme instance
        let theme = get_theme();
        // Just verify it's the expected type - we can't easily test the actual theme properties
        let _ = theme; // Use the theme to avoid unused variable warning
    }

    #[test]
    fn test_write_switch_path_with_env_var() {
        let temp_dir = TempDir::new().unwrap();
        let switch_file = temp_dir.path().join("switch.txt");
        let test_path = std::path::Path::new("/test/path");

        // Set the environment variable
        std::env::set_var(ENV_GW_SWITCH_FILE, &switch_file);

        write_switch_path(test_path);

        // Verify the file was written with the correct content
        let content = fs::read_to_string(&switch_file).unwrap();
        assert_eq!(content, "/test/path");

        // Clean up
        std::env::remove_var(ENV_GW_SWITCH_FILE);
    }

    #[test]
    fn test_find_config_in_default_branches_main() {
        let temp_dir = TempDir::new().unwrap();
        let config_filename = "test.toml";

        // Create main directory with config file
        let main_dir = temp_dir.path().join(DEFAULT_BRANCH_MAIN);
        fs::create_dir_all(&main_dir).unwrap();
        let config_file = main_dir.join(config_filename);
        fs::write(&config_file, "test config").unwrap();

        // Search should find the main branch config when current default is master
        let result = find_config_in_default_branches(
            temp_dir.path(),
            DEFAULT_BRANCH_MASTER, // current default is master
            config_filename,
        );

        assert!(result.is_some());
        assert_eq!(result.unwrap(), config_file);
    }

    #[test]
    fn test_find_config_in_default_branches_master() {
        let temp_dir = TempDir::new().unwrap();
        let config_filename = "test.toml";

        // Create master directory with config file
        let master_dir = temp_dir.path().join(DEFAULT_BRANCH_MASTER);
        fs::create_dir_all(&master_dir).unwrap();
        let config_file = master_dir.join(config_filename);
        fs::write(&config_file, "test config").unwrap();

        // Search should find the master branch config when current default is main
        let result = find_config_in_default_branches(
            temp_dir.path(),
            DEFAULT_BRANCH_MAIN, // current default is main
            config_filename,
        );

        assert!(result.is_some());
        assert_eq!(result.unwrap(), config_file);
    }

    #[test]
    fn test_find_config_in_default_branches_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Search should return None when no config files exist
        let result = find_config_in_default_branches(
            temp_dir.path(),
            DEFAULT_BRANCH_MAIN,
            "nonexistent.toml",
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_find_default_branch_directory_main() {
        let temp_dir = TempDir::new().unwrap();

        // Create main directory
        let main_dir = temp_dir.path().join(DEFAULT_BRANCH_MAIN);
        fs::create_dir_all(&main_dir).unwrap();

        // Search should find the main directory when current default is master
        let result = find_default_branch_directory(temp_dir.path(), DEFAULT_BRANCH_MASTER);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), main_dir);
    }

    #[test]
    fn test_find_default_branch_directory_master() {
        let temp_dir = TempDir::new().unwrap();

        // Create master directory
        let master_dir = temp_dir.path().join(DEFAULT_BRANCH_MASTER);
        fs::create_dir_all(&master_dir).unwrap();

        // Search should find the master directory when current default is main
        let result = find_default_branch_directory(temp_dir.path(), DEFAULT_BRANCH_MAIN);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), master_dir);
    }

    #[test]
    fn test_find_default_branch_directory_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Search should return None when no directories exist
        let result = find_default_branch_directory(temp_dir.path(), DEFAULT_BRANCH_MAIN);

        assert!(result.is_none());
    }

    #[test]
    fn test_find_default_branch_directory_skips_current_default() {
        let temp_dir = TempDir::new().unwrap();

        // Create both main and master directories
        let main_dir = temp_dir.path().join(DEFAULT_BRANCH_MAIN);
        let master_dir = temp_dir.path().join(DEFAULT_BRANCH_MASTER);
        fs::create_dir_all(&main_dir).unwrap();
        fs::create_dir_all(&master_dir).unwrap();

        // When current default is main, should find master
        let result = find_default_branch_directory(temp_dir.path(), DEFAULT_BRANCH_MAIN);
        assert_eq!(result.unwrap(), master_dir);

        // When current default is master, should find main
        let result = find_default_branch_directory(temp_dir.path(), DEFAULT_BRANCH_MASTER);
        assert_eq!(result.unwrap(), main_dir);
    }
}
