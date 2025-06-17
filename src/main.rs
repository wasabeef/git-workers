//! Git Workers - Interactive Git Worktree Manager
//!
//! This is the main entry point for the Git Workers CLI application.
//! It provides an interactive menu-driven interface for managing Git worktrees.

use anyhow::Result;
use clap::Parser;
use colored::*;
use console::{style, Term};
use dialoguer::Select;
use std::env;
use std::io::{self, Write};

mod commands;
mod config;
mod git;
mod hooks;
mod input_esc_raw;
mod menu;
mod repository_info;
mod utils;

use menu::MenuItem;
use repository_info::get_repository_info;
use utils::get_theme;

/// Command-line arguments for Git Workers
#[derive(Parser)]
#[command(name = "gw")]
#[command(about = "Interactive Git Worktree Manager", long_about = None)]
struct Cli {
    /// Print version information
    #[arg(short, long)]
    version: bool,

    /// List all worktrees (non-interactive)
    #[arg(short, long)]
    list: bool,
}

/// Main entry point for Git Workers
///
/// Initializes the CLI, handles version flag, and runs the main interactive loop.
/// The application will continue running until the user selects "Exit" or presses ESC.
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("git-workers v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.list {
        // Non-interactive mode: just list worktrees
        match git::list_worktrees() {
            Ok(worktrees) => {
                for worktree in worktrees {
                    println!("{}", worktree);
                }
            }
            Err(e) => {
                eprintln!("Error listing worktrees: {}", e);
                return Err(e);
            }
        }
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
        println!("{}", "=".repeat(50).bright_blue());
        println!(
            "{} {}",
            "Repository:".bright_white(),
            repo_info.bright_yellow().bold()
        );
        println!();
        let items = vec![
            MenuItem::ListWorktrees,
            MenuItem::SwitchWorktree,
            MenuItem::SearchWorktrees,
            MenuItem::CreateWorktree,
            MenuItem::DeleteWorktree,
            MenuItem::BatchDelete,
            MenuItem::CleanupOldWorktrees,
            MenuItem::RenameWorktree,
            MenuItem::Exit,
        ];

        let selection = match Select::with_theme(&get_theme())
            .with_prompt("? What would you like to do?")
            .items(&items)
            .default(0)
            .interact_on_opt(&term)
        {
            Ok(selection) => selection,
            Err(e) => {
                // Handle terminal detection errors gracefully
                if e.to_string().contains("terminal") || e.to_string().contains("Pipes") {
                    eprintln!("Error: git-workers must be run in a terminal environment.");
                    eprintln!("Pipes and non-interactive environments are not supported.");
                    eprintln!();
                    eprintln!("If you're using the 'gw' shell function, try:");
                    eprintln!("  - Running 'source ~/.bashrc' or 'source ~/.zshrc'");
                    eprintln!(
                        "  - Using the full path: {}",
                        env::current_exe().unwrap_or_default().display()
                    );
                    eprintln!("  - Setting GW_BINARY environment variable");
                    return Err(e.into());
                }
                return Err(e.into());
            }
        };

        let selection = match selection {
            Some(s) => s,
            None => {
                // User pressed ESC
                println!("{}", "» Goodbye! Happy coding!".bright_green().bold());
                break;
            }
        };

        match items[selection] {
            MenuItem::Exit => {
                println!("{}", "» Goodbye! Happy coding!".bright_green().bold());
                break;
            }
            item => {
                match commands::execute(item) {
                    Ok(should_exit) => {
                        if should_exit {
                            // Exit after switch to allow shell to update directory
                            break;
                        }
                        // Continue directly to menu without pause
                        continue;
                    }
                    Err(e) => {
                        handle_command_error(&term, e);
                        // Error was already handled, continue to menu
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Sets up terminal configuration for optimal display
fn setup_terminal_config() {
    // Force colored output even when piped
    // This helps when running through shell functions that use tee
    colored::control::set_override(true);

    // Enable terminal features if supported
    let term = Term::stdout();
    // Ensure terminal is in a clean state
    let _ = term.clear_screen();
}

/// Clears the screen with robust fallback handling
fn clear_screen(term: &Term) {
    // Try the standard terminal clear first
    if term.clear_screen().is_ok() {
        // Success - also ensure cursor is at top
        let _ = term.move_cursor_to(0, 0);
        return;
    }

    // Fallback 1: Direct ANSI escape sequence to stdout
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();

    // Fallback 2: Try terminal reset if still having issues
    if term.is_term() {
        // Only use reset on actual terminals, not pipes
        print!("\x1Bc");
        let _ = io::stdout().flush();
    }
}

/// Handles command execution errors with consistent UI
fn handle_command_error(term: &Term, error: anyhow::Error) {
    // Display error message cleanly
    println!();
    println!("{} {}", style("✗ Error:").red().bold(), error);
    println!();

    // Wait for user input before continuing
    println!("{}", "Press any key to continue...".bright_black());
    // Only read key if we're in a terminal
    if term.is_term() {
        let _ = term.read_key();
    }
}
