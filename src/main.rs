//! Git Workers - Interactive Git Worktree Manager
//!
//! This is the main entry point for the Git Workers CLI application.
//! It provides an interactive menu-driven interface for managing Git worktrees.
//!
//! # Overview
//!
//! Git Workers simplifies the management of Git worktrees by providing an intuitive
//! TUI (Terminal User Interface) that handles common worktree operations:
//!
//! - Creating new worktrees from branches or HEAD
//! - Switching between worktrees with automatic directory changes
//! - Deleting worktrees (individually or in batch)
//! - Renaming worktrees and their associated branches
//! - Searching through worktrees with fuzzy matching
//! - Managing lifecycle hooks for automation
//!
//! # Architecture
//!
//! The application follows a simple architecture:
//!
//! 1. **Main Loop**: Displays the menu and handles user selection
//! 2. **Command Handlers**: Execute the selected operations (see [`commands`] module)
//! 3. **Git Integration**: Interfaces with Git via git2 and process commands (see [`git`] module)
//! 4. **Shell Integration**: Enables automatic directory switching when changing worktrees
//!
//! # Shell Integration
//!
//! The application supports automatic directory switching through shell functions.
//! When switching worktrees, it writes the target path to a file specified by
//! the `GW_SWITCH_FILE` environment variable. The shell wrapper then reads this
//! file and executes the `cd` command.
//!
//! # Exit Codes
//!
//! - `0`: Successful execution
//! - `1`: Error during execution (displayed to user)

use anyhow::Result;
use clap::Parser;
use colored::*;
use console::Term;
use dialoguer::Select;
use std::env;
use std::io::{self, Write};

mod commands;
mod config;
mod constants;
mod file_copy;
mod git;
mod hooks;
mod input_esc_raw;
mod menu;
mod repository_info;
mod utils;

use constants::header_separator;
use menu::MenuItem;
use repository_info::get_repository_info;
use utils::get_theme;

/// Command-line arguments for Git Workers
///
/// Currently supports minimal CLI arguments as the application is primarily
/// interactive. Future versions may add support for direct command execution.
#[derive(Parser)]
#[command(name = "gw")]
#[command(about = "Interactive Git Worktree Manager", long_about = None)]
struct Cli {
    /// Print version information and exit
    ///
    /// When specified, prints the version number from Cargo.toml and exits
    /// without entering the interactive mode.
    #[arg(short, long)]
    version: bool,
}

/// Main entry point for Git Workers
///
/// Initializes the CLI, handles version flag, and runs the main interactive loop.
/// The application will continue running until the user selects "Exit" or presses ESC.
///
/// # Flow
///
/// 1. Parse command-line arguments
/// 2. Handle version flag if present
/// 3. Configure terminal settings for optimal display
/// 4. Enter the main menu loop:
///    - Clear screen and display header
///    - Show repository information
///    - Display menu options
///    - Handle user selection
///    - Execute selected command
///    - Repeat until exit
///
/// # Errors
///
/// Returns an error if:
/// - Terminal operations fail (rare)
/// - Command execution encounters an unrecoverable error
///
/// Most errors are handled gracefully within the loop and displayed to the user.
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("git-workers v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Terminal check removed - we'll handle errors gracefully when they occur
    // Some terminal environments may not be detected correctly by is_terminal()

    // Create terminal instance for consistent handling
    let term = console::Term::stdout();

    // Configure terminal and color output
    setup_terminal_config();

    loop {
        // Clear screen and show header for each iteration
        clear_screen(&term);

        // Force flush to ensure screen is cleared before new content
        let _ = io::stdout().flush();

        // Print clean header with repository info
        let repo_info = get_repository_info();

        println!();
        println!(
            "{}",
            "Git Workers - Interactive Worktree Manager"
                .bright_cyan()
                .bold()
        );
        println!("{}", header_separator());
        println!(
            "{} {}",
            "Repository:".bright_white(),
            repo_info.bright_yellow().bold()
        );
        println!();

        // Build menu items
        let menu_items = [
            MenuItem::ListWorktrees,
            MenuItem::SwitchWorktree,
            MenuItem::SearchWorktrees,
            MenuItem::CreateWorktree,
            MenuItem::DeleteWorktree,
            MenuItem::BatchDelete,
            MenuItem::CleanupOldWorktrees,
            MenuItem::RenameWorktree,
            MenuItem::EditHooks,
            MenuItem::Exit,
        ];

        // Convert to display strings
        let display_items: Vec<String> = menu_items.iter().map(|item| item.to_string()).collect();

        // Show menu with List worktrees as default selection
        let selection = match Select::with_theme(&get_theme())
            .with_prompt("What would you like to do?")
            .items(&display_items)
            .default(0) // Set List worktrees (index 0) as default
            .interact_opt()?
        {
            Some(selection) => selection,
            None => {
                // User pressed ESC - exit cleanly
                clear_screen(&term);
                println!("{}", "Exiting Git Workers...".bright_black());
                break;
            }
        };

        let selected_item = &menu_items[selection];

        match handle_menu_item(selected_item, &term)? {
            MenuAction::Continue => continue,
            MenuAction::Exit => {
                clear_screen(&term);
                println!("{}", "Exiting Git Workers...".bright_black());
                break;
            }
            MenuAction::ExitAfterSwitch => {
                // Exit without clearing screen (to preserve switch message)
                break;
            }
        }
    }

    Ok(())
}

/// Represents the action to take after handling a menu item
///
/// This enum controls the flow of the main loop, determining whether
/// to continue showing the menu or exit the application.
enum MenuAction {
    /// Continue the main loop and show the menu again
    Continue,
    /// Exit the application with a farewell message
    Exit,
    /// Exit after switching worktree (preserves switch message)
    ///
    /// This special exit mode is used when the user switches to a different
    /// worktree. It exits without clearing the screen to preserve the switch
    /// information for the shell wrapper to process.
    ExitAfterSwitch,
}

/// Handles the selected menu item and returns the next action
///
/// This function is the central dispatcher for all menu commands. It clears
/// the screen, executes the appropriate command, and determines the next
/// action based on the result.
///
/// # Arguments
///
/// * `item` - The selected menu item to execute
/// * `term` - Terminal instance for screen operations
///
/// # Returns
///
/// Returns a [`MenuAction`] indicating whether to:
/// - Continue showing the menu
/// - Exit the application
/// - Exit after a worktree switch (special case)
///
/// # Errors
///
/// Propagates any errors from the command execution. These are typically
/// handled by displaying an error message to the user.
fn handle_menu_item(item: &MenuItem, term: &Term) -> Result<MenuAction> {
    clear_screen(term);

    match item {
        MenuItem::ListWorktrees => commands::list_worktrees()?,
        MenuItem::CreateWorktree => {
            if commands::create_worktree()? {
                // User created and switched to new worktree
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::DeleteWorktree => commands::delete_worktree()?,
        MenuItem::SwitchWorktree => {
            if commands::switch_worktree()? {
                // User switched worktree - exit to apply the change
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::SearchWorktrees => {
            if commands::search_worktrees()? {
                // User switched worktree via search
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::BatchDelete => commands::batch_delete_worktrees()?,
        MenuItem::CleanupOldWorktrees => commands::cleanup_old_worktrees()?,
        MenuItem::RenameWorktree => commands::rename_worktree()?,
        MenuItem::EditHooks => commands::edit_hooks()?,
        MenuItem::Exit => return Ok(MenuAction::Exit),
    }

    Ok(MenuAction::Continue)
}

/// Clears the terminal screen with proper error handling
///
/// This function wraps the terminal clear operation to gracefully handle
/// any potential errors. Errors are ignored as screen clearing is not
/// critical to the application's functionality.
///
/// # Arguments
///
/// * `term` - Terminal instance to clear
fn clear_screen(term: &Term) {
    let _ = term.clear_screen();
}

/// Configures terminal settings for optimal display
///
/// This function sets up the terminal environment for the best possible
/// user experience across different platforms and terminal emulators.
///
/// # Configuration
///
/// 1. **Windows**: Enables ANSI color support on Windows terminals
/// 2. **Color Mode**: Respects environment variables for color output:
///    - `NO_COLOR`: Disables all color output when set
///    - `FORCE_COLOR` or `CLICOLOR_FORCE=1`: Forces color output even in pipes
///
/// # Environment Variables
///
/// The following environment variables control color output:
/// - `NO_COLOR`: When set (any value), disables colored output
/// - `FORCE_COLOR`: When set (any value), forces colored output
/// - `CLICOLOR_FORCE`: When set to "1", forces colored output
fn setup_terminal_config() {
    // Enable ANSI colors on Windows
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    // Set color mode based on environment
    if env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    } else if env::var("FORCE_COLOR").is_ok()
        || env::var("CLICOLOR_FORCE").unwrap_or_default() == "1"
    {
        colored::control::set_override(true);
    }
}
