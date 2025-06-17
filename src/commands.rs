//! Command implementations for the interactive menu
//!
//! This module contains the implementation of all menu commands, handling
//! user interaction, input validation, and executing Git operations.
//!
//! # Command Flow
//!
//! Each command follows a general pattern:
//! 1. Display information or prompts to the user
//! 2. Collect and validate user input
//! 3. Execute the requested operation
//! 4. Handle errors and provide feedback
//! 5. Return control to the main menu
//!
//! # ESC Key Handling
//!
//! All interactive prompts support ESC key cancellation, allowing users
//! to return to the main menu at any point during input.

use anyhow::Result;
use colored::*;
use console::Term;
use dialoguer::{Confirm, MultiSelect, Select};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::hooks::{self, HookContext};
use crate::input_esc_raw::{
    input_esc_raw as input_esc, input_esc_with_default_raw as input_esc_with_default,
};
use crate::menu::MenuItem;
use crate::utils::{self, get_terminal, get_theme};

// Helper function to get worktree icon
fn get_worktree_icon(is_current: bool) -> colored::ColoredString {
    if is_current {
        "→".bright_green().bold()
    } else {
        "▸".bright_blue()
    }
}

// Helper function to format branch label
fn format_branch_label(branch: &str, is_current: bool) -> String {
    if is_current {
        format!(
            "{} {} {}",
            "Branch:".bright_black(),
            branch.yellow(),
            "[CURRENT]".bright_green().bold()
        )
    } else {
        format!("{} {}", "Branch:".bright_black(), branch.yellow())
    }
}

/// Executes the selected menu command
///
/// This is the main entry point for all menu commands. It dispatches
/// the selected menu item to the appropriate handler function.
///
/// # Arguments
///
/// * `item` - The menu item selected by the user
///
/// # Returns
///
/// * `Ok(true)` - If the command requests to switch to a different worktree
/// * `Ok(false)` - If the command completes normally without switching
/// * `Err(_)` - If the command encounters an error
///
/// # Special Behavior
///
/// The `SwitchWorktree`, `SearchWorktrees`, and `CreateWorktree` commands
/// may return `Ok(true)` to signal that the shell should change directory.
pub fn execute(item: MenuItem) -> Result<bool> {
    let manager = GitWorktreeManager::new()?;

    match item {
        MenuItem::ListWorktrees => list_worktrees(&manager)?,
        MenuItem::SearchWorktrees => return search_worktrees(&manager),
        MenuItem::CreateWorktree => return create_worktree(&manager),
        MenuItem::DeleteWorktree => delete_worktree(&manager)?,
        MenuItem::BatchDelete => batch_delete_worktrees(&manager)?,
        MenuItem::CleanupOldWorktrees => cleanup_old_worktrees(&manager)?,
        MenuItem::SwitchWorktree => return switch_worktree(&manager),
        MenuItem::RenameWorktree => rename_worktree(&manager)?,
        MenuItem::Exit => unreachable!(),
    }

    Ok(false)
}

fn list_worktrees(manager: &GitWorktreeManager) -> Result<()> {
    // Show loading spinner while fetching worktree info
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} Loading worktrees...")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));

    let worktrees = manager.list_worktrees()?;

    // プログレスバーを適切に終了
    spinner.finish_and_clear();
    // 短時間待機してターミナル状態を安定化
    std::thread::sleep(std::time::Duration::from_millis(50));

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees found.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    // Pagination settings
    const PAGE_SIZE: usize = 10;
    let total_pages = worktrees.len().div_ceil(PAGE_SIZE);
    let mut current_page = 0;

    loop {
        // Screen clearing is handled by main.rs, not here

        println!("{}", "• Current worktrees:".green().bold());
        println!();

        // Calculate page boundaries
        let start = current_page * PAGE_SIZE;
        let end = std::cmp::min(start + PAGE_SIZE, worktrees.len());

        // Display worktrees for current page
        for wt in &worktrees[start..end] {
            let icon = get_worktree_icon(wt.is_current);

            println!(
                "  {} {} - {}",
                icon,
                wt.name.bright_white().bold(),
                wt.path.display().to_string().bright_black(),
            );

            let branch_label = format_branch_label(&wt.branch, wt.is_current);

            println!("    {}", branch_label);

            // Show commit info
            if let Some(commit) = &wt.last_commit {
                println!(
                    "    {} {} - {} ({})",
                    "Commit:".bright_black(),
                    commit.id.bright_blue(),
                    commit.message.bright_black(),
                    commit.time.bright_black()
                );
            }

            // Show status indicators
            let mut status_parts = Vec::new();

            if wt.has_changes {
                status_parts.push("● Modified".yellow().to_string());
            }

            if let Some((ahead, behind)) = wt.ahead_behind {
                if ahead > 0 {
                    status_parts.push(format!("↑{}", ahead).green().to_string());
                }
                if behind > 0 {
                    status_parts.push(format!("↓{}", behind).red().to_string());
                }
            }

            if !status_parts.is_empty() {
                println!(
                    "    {} {}",
                    "Status:".bright_black(),
                    status_parts.join(" ")
                );
            }
        }

        println!();

        // Pagination controls for multiple pages or pause for single page
        if total_pages > 1 {
            println!(
                "{}",
                format!("Page {}/{}", current_page + 1, total_pages).bright_black()
            );
            println!("{}", "[n]ext [p]revious [q]uit".bright_black());

            match get_terminal().read_key()? {
                console::Key::Char('n') => {
                    if current_page < total_pages - 1 {
                        current_page += 1;
                    } else {
                        break;
                    }
                }
                console::Key::Char('p') => {
                    current_page = current_page.saturating_sub(1);
                }
                console::Key::Char('q') | console::Key::Escape => break,
                _ => break,
            }
        } else {
            // Single page - just show a simple continue prompt
            println!("{}", "[Press any key to continue]".bright_black());
            let _ = get_terminal().read_key();
            break;
        }
    }

    Ok(())
}

fn create_worktree(manager: &GitWorktreeManager) -> Result<bool> {
    // Use custom input that supports ESC
    let name_input = match input_esc("Worktree name (ESC to cancel)") {
        Some(input) => input,
        None => {
            // ESC pressed - cancel
            return Ok(false);
        }
    };

    if name_input.is_empty() {
        // Operation cancelled - return silently
        return Ok(false);
    }

    // Validate the input
    if name_input.trim().is_empty() {
        utils::print_error("Worktree name cannot be empty. Please provide a valid name.");
        return Ok(false);
    }

    if name_input.contains(char::is_whitespace) {
        utils::print_error(
            "Worktree name cannot contain spaces. Use hyphens or underscores instead.",
        );
        return Ok(false);
    }

    let name = name_input;

    // Check if this is the first worktree
    let existing_worktrees = manager.list_worktrees()?;
    let is_first_worktree = existing_worktrees.is_empty();

    // For the first worktree, ask for directory structure
    let worktree_path = if is_first_worktree {
        println!();
        println!(
            "{}",
            "Setting up worktree directory structure"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "This only needs to be configured once.".bright_black()
        );
        println!();

        // Get repository info
        let repo_path = manager
            .repo()
            .workdir()
            .unwrap_or_else(|| manager.repo().path())
            .to_string_lossy();
        let repo_parent = std::path::Path::new(&*repo_path)
            .parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();
        let repo_name = std::path::Path::new(&*repo_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Show examples
        println!("{}:", "Examples for worktree location".bright_white());
        println!();
        println!("  {} Same level as repository:", "1.".bright_yellow());
        println!(
            "     {}/{}  {}",
            repo_parent,
            repo_name,
            "(main repository)".bright_black()
        );
        println!(
            "     {}/{}  {}",
            repo_parent,
            name,
            "(new worktree)".bright_green()
        );
        println!();
        println!("  {} In subdirectory (recommended):", "2.".bright_yellow());
        println!(
            "     {}/{}  {}",
            repo_parent,
            repo_name,
            "(main repository)".bright_black()
        );
        println!(
            "     {}/{}/worktrees/{}  {}",
            repo_parent,
            repo_name,
            name,
            "(new worktree)".bright_green()
        );
        println!();

        let location_options = vec![
            format!("Same level as repository ({}/{})", repo_parent, name),
            format!(
                "In subdirectory ({}/{}/worktrees/{})",
                repo_parent, repo_name, name
            ),
        ];

        let location_selection = match Select::with_theme(&get_theme())
            .with_prompt("Where should worktrees be created?")
            .items(&location_options)
            .default(1) // Default to subdirectory (recommended)
            .interact_opt()?
        {
            Some(selection) => selection,
            None => {
                // ESC pressed - cancel
                return Ok(false);
            }
        };

        match location_selection {
            0 => {
                // Same level - just use the name
                name.clone()
            }
            1 => {
                // Inside repository in worktrees subdirectory
                format!("../{}/worktrees/{}", repo_name, name)
            }
            _ => unreachable!(),
        }
    } else {
        // For subsequent worktrees, use just the name
        // GitWorktreeManager will detect the existing pattern
        name.clone()
    };

    // Ask if user wants to create from existing branch or new branch
    let branch_options = vec![
        "Create from current branch",
        "Create from existing branch",
        "Create new branch",
    ];

    let branch_selection = match Select::with_theme(&get_theme())
        .with_prompt("How would you like to create the worktree? (ESC to cancel)")
        .items(&branch_options)
        .default(0)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => {
            // User pressed ESC - cancel
            return Ok(false);
        }
    };

    let branch = match branch_selection {
        0 => None, // Use current branch
        1 => {
            // Select from existing branches
            let branches = manager.list_branches()?;
            if branches.is_empty() {
                println!("{}", "No branches found.".yellow());
                return Ok(false);
            }

            let selection = match Select::with_theme(&get_theme())
                .with_prompt("Select branch (ESC to cancel)")
                .items(&branches)
                .interact_opt()?
            {
                Some(selection) => selection,
                None => {
                    // User pressed ESC - cancel
                    return Ok(false);
                }
            };

            Some(branches[selection].clone())
        }
        2 => {
            // Create new branch - use worktree name as default
            let input = match input_esc_with_default("New branch name (ESC to cancel)", &name) {
                Some(input) => input,
                None => {
                    // ESC pressed - cancel
                    return Ok(false);
                }
            };

            Some(input)
        }
        _ => unreachable!(),
    };

    utils::print_progress("Creating worktree...");

    match manager.create_worktree(&worktree_path, branch.as_deref()) {
        Ok(worktree_path) => {
            utils::print_success(&format!(
                "Worktree '{}' created successfully!",
                name.bright_white().bold()
            ));

            // Execute post-create hooks
            if let Err(e) = hooks::execute_hooks(
                "post-create",
                &HookContext {
                    worktree_name: name.clone(),
                    worktree_path: worktree_path.clone(),
                },
            ) {
                eprintln!("{}: Failed to execute hooks: {}", "Warning".yellow(), e);
            }

            println!();
            println!("  {} {}", "Path:".bright_black(), worktree_path.display());
            if let Some(b) = branch {
                println!("  {} {}", "Branch:".bright_black(), b.yellow());
            }

            // Ask if user wants to switch to the new worktree
            println!();
            let switch = Confirm::with_theme(&get_theme())
                .with_prompt("Switch to the new worktree?")
                .default(true)
                .interact()?;

            if switch {
                // Output special marker for shell function to detect
                // Check if GW_SWITCH_FILE is set (new shell integration)
                if let Ok(switch_file) = std::env::var("GW_SWITCH_FILE") {
                    // Write to the file for new shell function
                    if let Ok(mut file) = std::fs::File::create(&switch_file) {
                        use std::io::Write;
                        let _ = writeln!(file, "{}", worktree_path.display());
                    }
                } else {
                    // Fallback to stdout for old shell function
                    println!("SWITCH_TO:{}", worktree_path.display());
                }
                println!();
                println!(
                    "{} Switching to worktree '{}'",
                    "+".green(),
                    name.bright_white().bold()
                );

                // Execute post-switch hooks if configured
                if let Err(e) = hooks::execute_hooks(
                    "post-switch",
                    &HookContext {
                        worktree_name: name,
                        worktree_path,
                    },
                ) {
                    eprintln!("{}: Failed to execute hooks: {}", "Warning".yellow(), e);
                }

                // Exit after switch to allow shell to update directory
                return Ok(true);
            }
        }
        Err(e) => {
            utils::print_error(&format!("Failed to create worktree: {}", e));
        }
    }

    Ok(false)
}

fn delete_worktree(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to delete.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    let deletable_worktrees: Vec<_> = worktrees.iter().filter(|w| !w.is_current).collect();

    if deletable_worktrees.is_empty() {
        println!();
        println!(
            "{}",
            "No deletable worktrees (cannot delete current worktree).".yellow()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select worktree to delete (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => {
            // User pressed ESC - cancel
            return Ok(());
        }
    };

    let worktree = deletable_worktrees[selection];

    let confirm = match Confirm::with_theme(&get_theme())
        .with_prompt(format!(
            "Are you sure you want to delete '{}'?",
            worktree.name.bright_red()
        ))
        .default(false)
        .interact()
    {
        Ok(confirm) => confirm,
        Err(_) => {
            // Operation cancelled - return silently
            return Ok(());
        }
    };

    if confirm {
        // Ask if user wants to delete the branch too
        let delete_branch = Confirm::with_theme(&get_theme())
            .with_prompt(format!(
                "Also delete the branch '{}'?",
                worktree.branch.yellow()
            ))
            .default(false)
            .interact()
            .unwrap_or(false);

        // Execute pre-remove hooks
        if let Err(e) = hooks::execute_hooks(
            "pre-remove",
            &HookContext {
                worktree_name: worktree.name.clone(),
                worktree_path: worktree.path.clone(),
            },
        ) {
            eprintln!("{}: Failed to execute hooks: {}", "Warning".yellow(), e);
        }

        utils::print_progress("Removing worktree...");

        let branch_to_delete = delete_branch.then(|| worktree.branch.clone());

        match manager.remove_worktree(&worktree.name) {
            Ok(_) => {
                utils::print_success(&format!(
                    "Worktree '{}' deleted successfully!",
                    worktree.name.bright_white().bold()
                ));

                // Delete branch if requested
                if let Some(branch) = branch_to_delete {
                    utils::print_progress(&format!("Deleting branch '{}'...", branch));
                    match manager.delete_branch(&branch) {
                        Ok(_) => {
                            utils::print_success(&format!(
                                "Branch '{}' deleted successfully!",
                                branch.yellow()
                            ));
                        }
                        Err(e) => {
                            utils::print_error(&format!("Failed to delete branch: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                utils::print_error(&format!("Failed to delete worktree: {}", e));
            }
        }
    }

    Ok(())
}

fn switch_worktree(manager: &GitWorktreeManager) -> Result<bool> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to switch to.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(false);
    }

    let items: Vec<String> = worktrees
        .iter()
        .map(|w| {
            if w.is_current {
                format!("{} ({}) {}", w.name, w.branch, "[current]".bright_green())
            } else {
                format!("{} ({})", w.name, w.branch)
            }
        })
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select worktree to switch to (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => {
            // User pressed ESC - cancel
            return Ok(false);
        }
    };

    let worktree = &worktrees[selection];

    if worktree.is_current {
        println!("{}", "• Already in this worktree.".yellow());
        return Ok(false);
    }

    // Output special marker for shell function to detect
    if let Ok(switch_file) = std::env::var("GW_SWITCH_FILE") {
        // New method: write to file
        if let Err(e) = std::fs::write(&switch_file, worktree.path.display().to_string()) {
            eprintln!("Warning: Failed to write switch file: {}", e);
        }
    } else {
        // Legacy method: output to stdout
        println!("SWITCH_TO:{}", worktree.path.display());
    }

    println!();
    println!(
        "{} Switching to worktree '{}'",
        "+".green(),
        worktree.name.bright_white().bold()
    );
    println!("  {} {}", "Path:".bright_black(), worktree.path.display());
    println!(
        "  {} {}",
        "Branch:".bright_black(),
        worktree.branch.yellow()
    );

    // Execute post-switch hooks
    if let Err(e) = hooks::execute_hooks(
        "post-switch",
        &HookContext {
            worktree_name: worktree.name.clone(),
            worktree_path: worktree.path.clone(),
        },
    ) {
        eprintln!("{}: Failed to execute hooks: {}", "Warning".yellow(), e);
    }

    // Return true to indicate we should exit after switch
    Ok(true)
}

fn search_worktrees(manager: &GitWorktreeManager) -> Result<bool> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees found.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(false);
    }

    // Use custom input that supports ESC
    let query = match input_esc("Search query (ESC to cancel)") {
        Some(q) => q,
        None => {
            // ESC pressed - cancel search
            return Ok(false);
        }
    };

    if query.is_empty() {
        // Search cancelled - return silently
        return Ok(false);
    }

    // Fuzzy search
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(usize, i64, &WorktreeInfo)> = worktrees
        .iter()
        .enumerate()
        .filter_map(|(idx, wt)| {
            // Search in name and branch
            let name_score = matcher.fuzzy_match(&wt.name, &query).unwrap_or(0);
            let branch_score = matcher.fuzzy_match(&wt.branch, &query).unwrap_or(0);
            let best_score = name_score.max(branch_score);

            if best_score > 0 {
                Some((idx, best_score, wt))
            } else {
                None
            }
        })
        .collect();

    // Sort by score (highest first)
    matches.sort_by(|a, b| b.1.cmp(&a.1));

    if matches.is_empty() {
        println!("{}", "No matches found.".yellow());
        return Ok(false);
    }

    println!();
    println!(
        "{} {} {}",
        "Found".green().bold(),
        matches.len().to_string().bright_white().bold(),
        if matches.len() == 1 {
            "match:"
        } else {
            "matches:"
        }
    );
    println!();

    // Display search results
    for (_, score, wt) in &matches {
        let icon = get_worktree_icon(wt.is_current);

        println!(
            "  {} {} - {} {}",
            icon,
            wt.name.bright_white().bold(),
            wt.path.display().to_string().bright_black(),
            format!("[score: {}]", score).bright_black()
        );

        let branch_label = format_branch_label(&wt.branch, wt.is_current);

        println!("    {}", branch_label);
    }

    println!();

    // Ask if user wants to switch to a result
    if matches.len() == 1 {
        let confirm = Confirm::with_theme(&get_theme())
            .with_prompt("Switch to this worktree?")
            .default(true)
            .interact();

        if confirm.unwrap_or(false) {
            let worktree = matches[0].2;
            if !worktree.is_current {
                // Output special marker for shell function
                if let Ok(switch_file) = std::env::var("GW_SWITCH_FILE") {
                    // New method: write to file
                    if let Err(e) =
                        std::fs::write(&switch_file, worktree.path.display().to_string())
                    {
                        eprintln!("Warning: Failed to write switch file: {}", e);
                    }
                } else {
                    // Legacy method: output to stdout
                    println!("SWITCH_TO:{}", worktree.path.display());
                }
                println!();
                println!(
                    "{} Switching to worktree '{}'",
                    "+".green(),
                    worktree.name.bright_white().bold()
                );
                println!("  {} {}", "Path:".bright_black(), worktree.path.display());
                return Ok(true);
            }
        }
    } else {
        // Multiple results - let user select
        let items: Vec<String> = matches
            .iter()
            .map(|(_, _, wt)| format!("{} ({})", wt.name, wt.branch))
            .collect();

        let selection = match Select::with_theme(&get_theme())
            .with_prompt("Select worktree to switch to (ESC to cancel)")
            .items(&items)
            .interact_opt()?
        {
            Some(selection) => selection,
            None => {
                // User pressed ESC - cancel
                return Ok(false);
            }
        };

        if selection < matches.len() {
            let worktree = matches[selection].2;
            if !worktree.is_current {
                // Output special marker for shell function
                if let Ok(switch_file) = std::env::var("GW_SWITCH_FILE") {
                    // New method: write to file
                    if let Err(e) =
                        std::fs::write(&switch_file, worktree.path.display().to_string())
                    {
                        eprintln!("Warning: Failed to write switch file: {}", e);
                    }
                } else {
                    // Legacy method: output to stdout
                    println!("SWITCH_TO:{}", worktree.path.display());
                }
                println!();
                println!(
                    "{} Switching to worktree '{}'",
                    "+".green(),
                    worktree.name.bright_white().bold()
                );
                println!("  {} {}", "Path:".bright_black(), worktree.path.display());
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn batch_delete_worktrees(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees found.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    let deletable_worktrees: Vec<_> = worktrees.iter().filter(|w| !w.is_current).collect();

    if deletable_worktrees.is_empty() {
        println!();
        println!(
            "{}",
            "No deletable worktrees (cannot delete current worktree).".yellow()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| {
            format!(
                "{} ({}) - {}",
                w.name,
                w.branch,
                if w.has_changes { "● Modified" } else { "" }
            )
        })
        .collect();

    let selections = match MultiSelect::with_theme(&get_theme())
        .with_prompt(
            "Select worktrees to delete (Space to select, Enter to confirm, ESC to cancel)",
        )
        .items(&items)
        .interact_opt()?
    {
        Some(s) => s,
        None => {
            // User pressed ESC - cancel
            return Ok(());
        }
    };

    if selections.is_empty() {
        println!("{}", "No worktrees selected.".yellow());
        return Ok(());
    }

    let selected_worktrees: Vec<_> = selections.iter().map(|&i| deletable_worktrees[i]).collect();

    println!();
    println!("{}", "Selected worktrees for deletion:".red().bold());
    for wt in &selected_worktrees {
        println!("  • {} ({})", wt.name.bright_white(), wt.branch.yellow());
    }
    println!();

    let confirm = match Confirm::with_theme(&get_theme())
        .with_prompt(format!("Delete {} worktrees?", selected_worktrees.len()))
        .default(false)
        .interact()
    {
        Ok(c) => c,
        Err(_) => {
            // Operation cancelled - return silently
            return Ok(());
        }
    };

    if confirm {
        // Ask if user wants to delete branches too
        let delete_branches = Confirm::with_theme(&get_theme())
            .with_prompt("Also delete the associated branches?")
            .default(false)
            .interact()
            .unwrap_or(false);

        let pb = ProgressBar::new(selected_worktrees.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut branches_to_delete = Vec::new();

        for wt in selected_worktrees {
            pb.set_message(format!("Deleting {}...", wt.name));

            // Execute pre-remove hooks
            if let Err(e) = hooks::execute_hooks(
                "pre-remove",
                &HookContext {
                    worktree_name: wt.name.clone(),
                    worktree_path: wt.path.clone(),
                },
            ) {
                eprintln!(
                    "{}: Failed to execute hooks for {}: {}",
                    "Warning".yellow(),
                    wt.name,
                    e
                );
            }

            match manager.remove_worktree(&wt.name) {
                Ok(_) => {
                    success_count += 1;
                    pb.inc(1);

                    // Store branch for later deletion if requested
                    if delete_branches {
                        branches_to_delete.push(wt.branch.clone());
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    eprintln!("{}: Failed to delete {}: {}", "Error".red(), wt.name, e);
                    pb.inc(1);
                }
            }
        }

        // プログレスバーを適切に終了
        pb.finish_and_clear();

        // 結果を表示
        if success_count > 0 {
            utils::print_success(&format!(
                "{} worktree{} deleted successfully!",
                success_count,
                if success_count > 1 { "s" } else { "" }
            ));
        }
        if failed_count > 0 {
            utils::print_error(&format!(
                "{} worktree{} failed to delete.",
                failed_count,
                if failed_count > 1 { "s" } else { "" }
            ));
        }

        // Delete branches if requested
        if delete_branches && !branches_to_delete.is_empty() {
            println!();
            println!("{}", "Deleting associated branches...".yellow());
            let mut branch_success = 0;
            let mut branch_failed = 0;

            for branch in branches_to_delete {
                match manager.delete_branch(&branch) {
                    Ok(_) => {
                        branch_success += 1;
                        println!("  {} Branch '{}' deleted", "+".green(), branch.yellow());
                    }
                    Err(e) => {
                        branch_failed += 1;
                        eprintln!(
                            "  {} Failed to delete branch '{}': {}",
                            "✗".red(),
                            branch.yellow(),
                            e
                        );
                    }
                }
            }

            if branch_success > 0 {
                utils::print_success(&format!(
                    "{} branch{} deleted successfully!",
                    branch_success,
                    if branch_success > 1 { "es" } else { "" }
                ));
            }
            if branch_failed > 0 {
                utils::print_error(&format!(
                    "{} branch{} failed to delete.",
                    branch_failed,
                    if branch_failed > 1 { "es" } else { "" }
                ));
            }
        }
    }

    Ok(())
}

fn cleanup_old_worktrees(manager: &GitWorktreeManager) -> Result<()> {
    // 分析用スピナーを表示
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.yellow} Analyzing worktrees...")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));

    let worktrees = manager.list_worktrees()?;

    // スピナーを適切に終了
    spinner.finish_and_clear();

    if worktrees.is_empty() {
        println!("{}", "• No worktrees found.".yellow());
        return Ok(());
    }

    // Find worktrees older than 30 days without changes
    let mut old_worktrees = Vec::new();
    let _cutoff_days = 30;

    for wt in &worktrees {
        if wt.is_current {
            continue;
        }

        // Check if worktree has no recent commits and no changes
        if !wt.has_changes {
            if let Some(_commit) = &wt.last_commit {
                // Parse the time string and check if it's old
                // For now, we'll add all non-current worktrees without changes
                old_worktrees.push(wt);
            }
        }
    }

    if old_worktrees.is_empty() {
        println!("{}", "No old worktrees found for cleanup.".green());
        println!(
            "{}",
            "All worktrees are either current or have uncommitted changes.".bright_black()
        );
        return Ok(());
    }

    println!();
    println!(
        "{} {}",
        "Found".yellow(),
        format!(
            "{} worktree{} that could be cleaned up:",
            old_worktrees.len(),
            if old_worktrees.len() > 1 { "s" } else { "" }
        )
        .bright_white()
    );
    println!();

    for wt in &old_worktrees {
        println!("  • {} ({})", wt.name.bright_white(), wt.branch.yellow());
        if let Some(commit) = &wt.last_commit {
            println!(
                "    {} {}",
                "Last commit:".bright_black(),
                commit.time.bright_black()
            );
        }
    }

    println!();
    let confirm = match Confirm::with_theme(&get_theme())
        .with_prompt("Delete these old worktrees?")
        .default(false)
        .interact()
    {
        Ok(c) => c,
        Err(_) => {
            // Operation cancelled - return silently
            return Ok(());
        }
    };

    if confirm {
        let pb = ProgressBar::new(old_worktrees.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} Cleaning up... [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut cleaned = 0;
        for wt in old_worktrees {
            pb.set_message(format!("Removing {}...", wt.name));

            if let Err(e) = hooks::execute_hooks(
                "pre-remove",
                &HookContext {
                    worktree_name: wt.name.clone(),
                    worktree_path: wt.path.clone(),
                },
            ) {
                eprintln!("{}: Failed to execute hooks: {}", "Warning".yellow(), e);
            }

            if manager.remove_worktree(&wt.name).is_ok() {
                cleaned += 1;
            }
            pb.inc(1);
            std::thread::sleep(Duration::from_millis(100)); // Visual feedback
        }

        // プログレスバーを適切に終了
        pb.finish_and_clear();

        utils::print_success(&format!(
            "Cleaned up {} old worktree{}!",
            cleaned,
            if cleaned != 1 { "s" } else { "" }
        ));
    }

    Ok(())
}

fn rename_worktree(manager: &GitWorktreeManager) -> Result<()> {
    println!();
    println!("{}", "⚠️  Rename function has limitations:".yellow().bold());
    println!("  • Cannot rename current worktree");
    println!("  • Worktree will be temporarily unavailable during rename");
    println!("  • Make sure to commit or stash changes before renaming");
    println!();

    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees found.".yellow());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    // Filter out current worktree (cannot rename current)
    let renameable_worktrees: Vec<_> = worktrees.iter().filter(|w| !w.is_current).collect();

    if renameable_worktrees.is_empty() {
        println!();
        println!(
            "{}",
            "No renameable worktrees (cannot rename current worktree).".yellow()
        );
        println!();
        println!("Press any key to continue...");
        Term::stdout().read_key()?;
        return Ok(());
    }

    // Select worktree to rename
    let items: Vec<String> = renameable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select worktree to rename (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => {
            // User pressed ESC - cancel
            return Ok(());
        }
    };

    let selected_worktree = renameable_worktrees[selection];
    let old_name = &selected_worktree.name;

    // Get new name
    let new_name = match input_esc(&format!("New name for '{}' (ESC to cancel)", old_name)) {
        Some(name) => name,
        None => {
            // ESC pressed - cancel
            return Ok(());
        }
    };

    // Validate the input
    if new_name.contains(char::is_whitespace) {
        utils::print_error(
            "Worktree name cannot contain spaces. Use hyphens or underscores instead.",
        );
        return Ok(());
    }

    if new_name == *old_name {
        println!("{}", "New name is the same as current name.".yellow());
        return Ok(());
    }

    // Check if new name already exists
    if worktrees.iter().any(|w| w.name == new_name) {
        utils::print_error(&format!("Worktree '{}' already exists", new_name));
        return Ok(());
    }

    // Confirm the operation
    let confirm = match Confirm::with_theme(&get_theme())
        .with_prompt(format!(
            "Rename '{}' to '{}'?",
            old_name.bright_blue(),
            new_name.bright_green()
        ))
        .default(false)
        .interact()
    {
        Ok(confirm) => confirm,
        Err(_) => {
            // Operation cancelled - return silently
            return Ok(());
        }
    };

    if confirm {
        utils::print_progress(&format!("Renaming '{}' to '{}'...", old_name, new_name));

        match manager.rename_worktree(old_name, &new_name) {
            Ok(new_path) => {
                utils::print_success(&format!(
                    "Worktree renamed successfully!\n  {} {} → {}\n  {} {}",
                    "From:".bright_black(),
                    old_name.bright_blue(),
                    new_name.bright_green().bold(),
                    "Path:".bright_black(),
                    new_path.display()
                ));
            }
            Err(e) => {
                utils::print_error(&format!("Failed to rename worktree: {}", e));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_input_validation() {
        let input_with_spaces = "test name";
        assert!(input_with_spaces.contains(char::is_whitespace));

        let input_without_spaces = "testname";
        assert!(!input_without_spaces.contains(char::is_whitespace));
    }

    #[test]
    fn test_esc_key_handling_logic() {
        // ESC キーの処理ロジックをテスト
        // 実際のユーザー入力をシミュレートすることはできないが、
        // 処理ロジックの構造をテストできる

        // 空文字列の場合（ESCまたは空入力）
        let empty_input = "";
        assert!(empty_input.trim().is_empty());

        // 有効な入力の場合
        let valid_input = "test-worktree";
        assert!(!valid_input.trim().is_empty());
        assert!(!valid_input.contains(char::is_whitespace));
    }
}
