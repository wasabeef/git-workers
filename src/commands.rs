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
use unicode_width::UnicodeWidthStr;

use crate::config::Config;
use crate::constants::{section_header, CONFIG_FILE_NAME, GIT_REMOTE_PREFIX, WORKTREES_SUBDIR};
use crate::file_copy;
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::hooks::{self, HookContext};
use crate::input_esc_raw::{
    input_esc_raw as input_esc, input_esc_with_default_raw as input_esc_with_default,
};
use crate::utils::{self, get_theme, press_any_key_to_continue, write_switch_path};

/// Gets the appropriate icon for a worktree based on its status
///
/// # Arguments
///
/// * `is_current` - Whether this is the currently active worktree
///
/// # Returns
///
/// Returns a colored icon:
/// - `→` in bright green for the current worktree
/// - `▸` in bright blue for other worktrees
fn get_worktree_icon(is_current: bool) -> colored::ColoredString {
    if is_current {
        "→".bright_green().bold()
    } else {
        "▸".bright_blue()
    }
}

// ===== Public API =====

/// Lists all worktrees with pagination support
///
/// Displays a formatted table of all worktrees in the repository, including:
/// - Worktree name
/// - Associated branch
/// - Modified status (whether the worktree has uncommitted changes)
/// - Full path to the worktree
///
/// The current worktree is highlighted and shown first, followed by others
/// in alphabetical order. For repositories with many worktrees, the list
/// is paginated based on terminal height.
///
/// # Navigation
///
/// - Arrow keys (← →): Navigate between pages
/// - ESC: Return to main menu
/// - Any other key: Return to main menu
///
/// # Returns
///
/// Returns `Ok(())` on successful completion or when the user exits.
///
/// # Errors
///
/// Returns an error if:
/// - Git repository operations fail
/// - Terminal operations fail
pub fn list_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    list_worktrees_internal(&manager)
}

/// Internal implementation of list_worktrees
///
/// Separated for better testability and code organization.
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Implementation Details
///
/// 1. Retrieves all worktrees from the repository
/// 2. Sorts them (current first, then alphabetically)
/// 3. Calculates column widths for proper alignment
/// 4. Determines pagination based on terminal height
/// 5. Displays the table with navigation support
fn list_worktrees_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees found.".bright_black());
        println!();
        println!(
            "  {} Use '{}' to create your first worktree",
            "Tip:".bright_black(),
            "+ Create worktree".green()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Sort worktrees: current first, then alphabetically
    let mut sorted_worktrees = worktrees;
    sorted_worktrees.sort_by(|a, b| {
        if a.is_current && !b.is_current {
            std::cmp::Ordering::Less
        } else if !a.is_current && b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    // Find the longest name and branch for formatting (using display width)
    let max_name_len = sorted_worktrees
        .iter()
        .map(|w| w.name.width())
        .max()
        .unwrap_or(0);
    let max_branch_len = sorted_worktrees
        .iter()
        .map(|w| w.branch.width())
        .max()
        .unwrap_or(0);

    // Determine items per page dynamically based on terminal height
    let term_height = Term::stdout().size().0 as usize;
    let header_lines = 7; // Title + header + separator (increased for modified column)
    let footer_lines = 4; // Navigation help + prompt
    let available_lines = term_height.saturating_sub(header_lines + footer_lines);
    let items_per_page = available_lines.max(5); // At least 5 items per page

    let total_pages = sorted_worktrees.len().div_ceil(items_per_page);
    let mut current_page = 0;

    loop {
        let term = Term::stdout();
        term.clear_screen()?;

        // Print header
        println!();
        println!("{}", section_header("Worktrees"));

        let start_idx = current_page * items_per_page;
        let end_idx = ((current_page + 1) * items_per_page).min(sorted_worktrees.len());
        let page_worktrees = &sorted_worktrees[start_idx..end_idx];

        // Table header
        let name_width = max_name_len.max(8);
        let branch_width = max_branch_len.max(10) + 10; // Extra space for [current] marker
        let modified_width = 8;

        println!();
        println!(
            "  {:<name_width$} {:<branch_width$} {:<modified_width$} {}",
            "Name".bright_white().bold(),
            "Branch".bright_white().bold(),
            "Modified".bright_white().bold(),
            "Path".bright_white().bold(),
            name_width = name_width,
            branch_width = branch_width,
            modified_width = modified_width
        );
        println!(
            "  {:-<name_width$} {:-<branch_width$} {:-<modified_width$} {:-<path_width$}",
            "-",
            "-",
            "-",
            "-",
            name_width = name_width,
            branch_width = branch_width,
            modified_width = modified_width,
            path_width = 40
        );

        // Table rows
        for wt in page_worktrees {
            let icon = get_worktree_icon(wt.is_current);
            let branch_display = if wt.is_current {
                format!("{} [current]", wt.branch).bright_green()
            } else {
                wt.branch.yellow()
            };

            // Modified status
            let modified = if wt.has_changes {
                "Yes".bright_yellow()
            } else {
                "No".bright_black()
            };

            println!(
                "{} {:<name_width$} {:<branch_width$} {:<modified_width$} {}",
                icon,
                if wt.is_current {
                    wt.name.bright_green().bold()
                } else {
                    wt.name.normal()
                },
                branch_display,
                modified,
                wt.path.display().to_string().bright_black(),
                name_width = name_width,
                branch_width = branch_width,
                modified_width = modified_width
            );
        }

        // Navigation footer
        println!();
        if total_pages > 1 {
            println!(
                "  {} Page {} of {} (showing {}-{} of {})",
                "•".bright_blue(),
                current_page + 1,
                total_pages,
                start_idx + 1,
                end_idx,
                sorted_worktrees.len()
            );
            println!(
                "  {} Use ← → to navigate pages, ESC to return",
                "•".bright_blue()
            );
        }
        println!();

        // Wait for key press
        println!("Press any key to continue...");
        match term.read_key()? {
            console::Key::ArrowLeft if current_page > 0 => {
                current_page -= 1;
            }
            console::Key::ArrowRight if current_page < total_pages - 1 => {
                current_page += 1;
            }
            console::Key::Escape => break,
            _ => break,
        }
    }

    Ok(())
}

/// Searches through worktrees using fuzzy matching
///
/// Provides an interactive search interface that matches against both
/// worktree names and branch names. Uses the Skim fuzzy matcher algorithm
/// for flexible, typo-tolerant searching.
///
/// # Search Behavior
///
/// - Searches both worktree names and branch names
/// - Case-insensitive matching
/// - Supports partial matches and typos
/// - Results are sorted by match score (best matches first)
///
/// # Returns
///
/// Returns `true` if the user switched to a worktree, `false` otherwise.
///
/// # Errors
///
/// Returns an error if:
/// - Git repository operations fail
/// - Terminal operations fail
///
/// # Example Search Patterns
///
/// - `feat` matches "feature/login", "feature/logout"
/// - `lgn` matches "login", "feature/login" (fuzzy matching)
pub fn search_worktrees() -> Result<bool> {
    let manager = GitWorktreeManager::new()?;
    search_worktrees_internal(&manager)
}

/// Internal implementation of search_worktrees
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Returns
///
/// Returns `true` if a worktree was selected and switched to, `false` otherwise
/// (includes ESC cancellation or selecting current worktree).
fn search_worktrees_internal(manager: &GitWorktreeManager) -> Result<bool> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to search.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    // Get search query
    println!();
    println!("{}", section_header("Search Worktrees"));
    println!();

    let search_query = match input_esc("Enter search query (name or branch)") {
        Some(query) => query.trim().to_string(),
        None => return Ok(false),
    };

    if search_query.is_empty() {
        return Ok(false);
    }

    // Perform fuzzy search
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, String, &WorktreeInfo)> = worktrees
        .iter()
        .filter_map(|wt| {
            let name_score = matcher.fuzzy_match(&wt.name, &search_query).unwrap_or(0);
            let branch_score = matcher.fuzzy_match(&wt.branch, &search_query).unwrap_or(0);
            let max_score = name_score.max(branch_score);

            if max_score > 0 {
                let display = format!("{} ({})", wt.name, wt.branch);
                Some((max_score, display, wt))
            } else {
                None
            }
        })
        .collect();

    if results.is_empty() {
        println!();
        println!("{}", "• No matching worktrees found.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| b.0.cmp(&a.0));

    // Display results
    println!();
    println!(
        "{} Found {} matching worktree{}:",
        "•".bright_green(),
        results.len(),
        if results.len() == 1 { "" } else { "s" }
    );
    println!();

    let items: Vec<String> = results
        .iter()
        .map(|(score, display, wt)| {
            let mut item = format!("{} [score: {}]", display, score);
            if wt.is_current {
                item.push_str(" (current)");
            }
            item
        })
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select a worktree to switch to (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(false),
    };

    let selected_worktree = results[selection].2;

    if selected_worktree.is_current {
        println!();
        println!("{}", "• Already in this worktree.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    // Switch to the selected worktree
    write_switch_path(&selected_worktree.path);

    println!();
    println!(
        "{} Switching to worktree '{}'",
        "+".green(),
        selected_worktree.name.bright_white().bold()
    );
    println!(
        "  {} {}",
        "Path:".bright_black(),
        selected_worktree.path.display()
    );
    println!(
        "  {} {}",
        "Branch:".bright_black(),
        selected_worktree.branch.yellow()
    );

    // Execute post-switch hooks
    if let Err(e) = hooks::execute_hooks(
        "post-switch",
        &HookContext {
            worktree_name: selected_worktree.name.clone(),
            worktree_path: selected_worktree.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {}", e));
    }

    Ok(true)
}

/// Creates a new worktree interactively
///
/// Guides the user through creating a new worktree with the following steps:
///
/// 1. **Name Input**: Prompts for the worktree name
/// 2. **Location Selection** (first worktree only):
///    - Same level as repository: `../worktree-name`
///    - In subdirectory (recommended): `../repo/worktrees/worktree-name`
/// 3. **Branch Selection**:
///    - Create from current HEAD
///    - Create from existing branch (shows branch list)
/// 4. **Creation**: Creates the worktree with progress indication
/// 5. **Post-create Hooks**: Executes any configured post-create hooks
/// 6. **Switch Option**: Asks if user wants to switch to the new worktree
///
/// # Worktree Patterns
///
/// The first worktree establishes the pattern for subsequent worktrees.
/// If the first worktree is created at the same level as the repository,
/// all future worktrees follow that pattern.
///
/// # Path Handling
///
/// - "Same level" paths (`../name`) are canonicalized for clean display without `..`
/// - "In subdirectory" paths create worktrees in `worktrees/` folder within the repository
/// - All paths are resolved to absolute canonical paths before creation
/// - Parent directories are created automatically if needed
///
/// # Returns
///
/// Returns `true` if the user created and switched to a new worktree,
/// `false` otherwise.
///
/// # Errors
///
/// Returns an error if:
/// - Git repository operations fail
/// - File system operations fail
/// - User input is invalid
pub fn create_worktree() -> Result<bool> {
    let manager = GitWorktreeManager::new()?;
    create_worktree_internal(&manager)
}

/// Internal implementation of create_worktree
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Implementation Notes
///
/// - Validates worktree name (non-empty)
/// - Detects existing worktree patterns for consistency
/// - Handles both branch and HEAD-based creation
/// - Executes lifecycle hooks at appropriate times
fn create_worktree_internal(manager: &GitWorktreeManager) -> Result<bool> {
    println!();
    println!("{}", section_header("Create New Worktree"));
    println!();

    // Get existing worktrees to detect pattern
    let existing_worktrees = manager.list_worktrees()?;
    let has_worktrees = !existing_worktrees.is_empty();

    // Get worktree name
    let name = match input_esc("Enter worktree name") {
        Some(name) => name.trim().to_string(),
        None => return Ok(false),
    };

    if name.is_empty() {
        utils::print_error("Worktree name cannot be empty");
        return Ok(false);
    }

    // If this is the first worktree, let user choose the pattern
    let final_name = if !has_worktrees {
        println!();
        println!("{}", "First worktree - choose location:".bright_cyan());

        // Get repository name for display
        let repo_name = manager
            .repo()
            .workdir()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("repo");

        let options = vec![
            format!("Same level as repository (../{name})"),
            format!("In subdirectory ({repo_name}/{}/{name})", WORKTREES_SUBDIR),
        ];

        let selection = match Select::with_theme(&get_theme())
            .with_prompt("Select worktree location pattern")
            .items(&options)
            .default(1) // Default to subdirectory (recommended)
            .interact_opt()?
        {
            Some(selection) => selection,
            None => return Ok(false),
        };

        match selection {
            0 => format!("../{}", name),                   // Same level
            _ => format!("{}/{}", WORKTREES_SUBDIR, name), // Subdirectory pattern
        }
    } else {
        name.clone()
    };

    // Branch handling
    println!();
    let branch_options = vec!["Create from current HEAD", "Select branch (smart mode)"];

    let branch_choice = match Select::with_theme(&get_theme())
        .with_prompt("Select branch option")
        .items(&branch_options)
        .interact_opt()?
    {
        Some(choice) => choice,
        None => return Ok(false),
    };

    let (branch, new_branch_name) = match branch_choice {
        1 => {
            // Select branch (smart mode)
            let (local_branches, remote_branches) = manager.list_all_branches()?;
            if local_branches.is_empty() && remote_branches.is_empty() {
                utils::print_warning("No branches found, creating from HEAD");
                (None, None)
            } else {
                // Get branch to worktree mapping
                let branch_worktree_map = manager.get_branch_worktree_map()?;

                // Create display items without section headers
                let mut branch_items: Vec<String> = Vec::new();
                let mut branch_refs: Vec<(String, bool)> = Vec::new(); // (branch_name, is_remote)

                // Add local branches
                for branch in &local_branches {
                    if let Some(worktree) = branch_worktree_map.get(branch) {
                        branch_items.push(format!(
                            "  {} {}",
                            branch.white(),
                            format!("(in use by '{}')", worktree).bright_red()
                        ));
                    } else {
                        branch_items.push(format!("  {}", branch.white()));
                    }
                    branch_refs.push((branch.clone(), false));
                }

                // Add remote branches with clear distinction
                for branch in &remote_branches {
                    let full_remote_name = format!("{}{}", GIT_REMOTE_PREFIX, branch);
                    if let Some(worktree) = branch_worktree_map.get(&full_remote_name) {
                        branch_items.push(format!(
                            "↑ {} {}",
                            full_remote_name.bright_blue(),
                            format!("(in use by '{}')", worktree).bright_red()
                        ));
                    } else {
                        branch_items.push(format!("↑ {}", full_remote_name.bright_blue()));
                    }
                    branch_refs.push((branch.clone(), true));
                }

                println!();
                match Select::with_theme(&get_theme())
                    .with_prompt("Select a branch")
                    .items(&branch_items)
                    .interact_opt()?
                {
                    Some(selection) => {
                        let (selected_branch, is_remote) = &branch_refs[selection];

                        if !is_remote {
                            // Local branch - check if already checked out
                            if let Some(worktree) = branch_worktree_map.get(selected_branch) {
                                // Branch is in use, offer to create a new branch
                                println!();
                                utils::print_warning(&format!(
                                    "Branch '{}' is already checked out in worktree '{}'",
                                    selected_branch.yellow(),
                                    worktree.bright_red()
                                ));
                                println!();

                                let action_options = vec![
                                    format!(
                                        "Create new branch '{}' from '{}'",
                                        name, selected_branch
                                    ),
                                    "Change the branch name".to_string(),
                                    "Cancel".to_string(),
                                ];

                                match Select::with_theme(&get_theme())
                                    .with_prompt("What would you like to do?")
                                    .items(&action_options)
                                    .interact_opt()?
                                {
                                    Some(0) => {
                                        // Use worktree name as new branch name
                                        (Some(selected_branch.clone()), Some(name.clone()))
                                    }
                                    Some(1) => {
                                        // Ask for custom branch name
                                        println!();
                                        let new_branch = match input_esc_with_default(
                                            &format!(
                                                "Enter new branch name (base: {})",
                                                selected_branch.yellow()
                                            ),
                                            &name,
                                        ) {
                                            Some(name) => name.trim().to_string(),
                                            None => return Ok(false),
                                        };

                                        if new_branch.is_empty() {
                                            utils::print_error("Branch name cannot be empty");
                                            return Ok(false);
                                        }

                                        if local_branches.contains(&new_branch) {
                                            utils::print_error(&format!(
                                                "Branch '{}' already exists",
                                                new_branch
                                            ));
                                            return Ok(false);
                                        }

                                        (Some(selected_branch.clone()), Some(new_branch))
                                    }
                                    _ => return Ok(false),
                                }
                            } else {
                                (Some(selected_branch.clone()), None)
                            }
                        } else {
                            // Remote branch - check if local branch with same name exists
                            if local_branches.contains(selected_branch) {
                                // Local branch with same name exists
                                println!();
                                utils::print_warning(&format!(
                                    "A local branch '{}' already exists for remote '{}'",
                                    selected_branch.yellow(),
                                    format!("{}{}", GIT_REMOTE_PREFIX, selected_branch)
                                        .bright_blue()
                                ));
                                println!();

                                let use_local_option = if let Some(worktree) =
                                    branch_worktree_map.get(selected_branch)
                                {
                                    format!(
                                        "Use the existing local branch instead (in use by '{}')",
                                        worktree.bright_red()
                                    )
                                } else {
                                    "Use the existing local branch instead".to_string()
                                };

                                let action_options = vec![
                                    format!(
                                        "Create new branch '{}' from '{}{}'",
                                        name, GIT_REMOTE_PREFIX, selected_branch
                                    ),
                                    use_local_option,
                                    "Cancel".to_string(),
                                ];

                                match Select::with_theme(&get_theme())
                                    .with_prompt("What would you like to do?")
                                    .items(&action_options)
                                    .interact_opt()?
                                {
                                    Some(0) => {
                                        // Create new branch with worktree name
                                        (
                                            Some(format!(
                                                "{}{}",
                                                GIT_REMOTE_PREFIX, selected_branch
                                            )),
                                            Some(name.clone()),
                                        )
                                    }
                                    Some(1) => {
                                        // Use local branch instead - but check if it's already in use
                                        if let Some(worktree) =
                                            branch_worktree_map.get(selected_branch)
                                        {
                                            println!();
                                            utils::print_error(&format!(
                                                "Branch '{}' is already checked out in worktree '{}'",
                                                selected_branch.yellow(),
                                                worktree.bright_red()
                                            ));
                                            println!("Please select a different option.");
                                            return Ok(false);
                                        }
                                        (Some(selected_branch.clone()), None)
                                    }
                                    _ => return Ok(false),
                                }
                            } else {
                                // No conflict, proceed normally
                                (
                                    Some(format!("{}{}", GIT_REMOTE_PREFIX, selected_branch)),
                                    None,
                                )
                            }
                        }
                    }
                    None => return Ok(false),
                }
            }
        }
        _ => {
            // Create from current HEAD
            (None, None)
        }
    };

    // Show preview
    println!();
    println!("{}", "Preview:".bright_white());
    println!("  {} {}", "Name:".bright_black(), final_name.bright_green());
    if let Some(new_branch) = &new_branch_name {
        let base_branch_name = branch.as_ref().unwrap();
        println!(
            "  {} {} (from {})",
            "New Branch:".bright_black(),
            new_branch.yellow(),
            base_branch_name.bright_black()
        );
    } else if let Some(branch_name) = &branch {
        println!("  {} {}", "Branch:".bright_black(), branch_name.yellow());
    } else {
        println!("  {} Current HEAD", "From:".bright_black());
    }
    println!();

    // Create worktree with progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Creating worktree...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let result = if let Some(new_branch) = &new_branch_name {
        // Create worktree with new branch from base branch
        manager.create_worktree_with_new_branch(&final_name, new_branch, branch.as_ref().unwrap())
    } else {
        // Create worktree with existing branch or from HEAD
        manager.create_worktree(&final_name, branch.as_deref())
    };

    match result {
        Ok(path) => {
            pb.finish_and_clear();
            utils::print_success(&format!(
                "Created worktree '{}' at {}",
                name.bright_green(),
                path.display()
            ));

            // Copy configured files
            let config = Config::load()?;
            if !config.files.copy.is_empty() {
                println!();
                println!("Copying configured files...");
                match file_copy::copy_configured_files(&config.files, &path, manager) {
                    Ok(copied) => {
                        if !copied.is_empty() {
                            utils::print_success(&format!("Copied {} files", copied.len()));
                            for file in &copied {
                                println!("  ✓ {}", file);
                            }
                        }
                    }
                    Err(e) => {
                        utils::print_warning(&format!("Failed to copy files: {}", e));
                    }
                }
            }

            // Execute post-create hooks
            if let Err(e) = hooks::execute_hooks(
                "post-create",
                &HookContext {
                    worktree_name: name.clone(),
                    worktree_path: path.clone(),
                },
            ) {
                utils::print_warning(&format!("Hook execution warning: {}", e));
            }

            // Ask if user wants to switch to the new worktree
            println!();
            let switch = Confirm::with_theme(&get_theme())
                .with_prompt("Switch to the new worktree?")
                .default(true)
                .interact_opt()?
                .unwrap_or(false);

            if switch {
                // Switch to the new worktree
                write_switch_path(&path);

                println!();
                println!(
                    "{} Switching to worktree '{}'",
                    "+".green(),
                    name.bright_white().bold()
                );

                // Execute post-switch hooks
                if let Err(e) = hooks::execute_hooks(
                    "post-switch",
                    &HookContext {
                        worktree_name: name,
                        worktree_path: path,
                    },
                ) {
                    utils::print_warning(&format!("Hook execution warning: {}", e));
                }

                Ok(true) // Indicate that we switched
            } else {
                println!();
                press_any_key_to_continue()?;
                Ok(false)
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            utils::print_error(&format!("Failed to create worktree: {}", e));
            println!();
            press_any_key_to_continue()?;
            Ok(false)
        }
    }
}

/// Deletes a single worktree interactively
///
/// Presents a list of deletable worktrees (excluding the current one)
/// and guides the user through the deletion process:
///
/// 1. **Selection**: Choose a worktree from the list
/// 2. **Branch Check**: If the branch is unique to this worktree, offers to delete it
/// 3. **Confirmation**: Shows worktree details and confirms deletion
/// 4. **Pre-remove Hooks**: Executes any configured pre-remove hooks
/// 5. **Deletion**: Removes the worktree and optionally its branch
///
/// # Safety
///
/// - Cannot delete the current worktree
/// - Requires explicit confirmation
/// - Shows all relevant information before deletion
///
/// # Returns
///
/// Returns `Ok(())` on successful completion (including cancellation).
///
/// # Errors
///
/// Returns an error if:
/// - Git repository operations fail
/// - File system operations fail during deletion
pub fn delete_worktree() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    delete_worktree_internal(&manager)
}

/// Internal implementation of delete_worktree
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Deletion Process
///
/// 1. Filters out current worktree (cannot be deleted)
/// 2. Presents selection list to user
/// 3. Checks if branch is unique to the worktree
/// 4. Confirms deletion with detailed preview
/// 5. Executes pre-remove hooks
/// 6. Performs deletion of worktree and optionally branch
fn delete_worktree_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to delete.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Filter out current worktree
    let deletable_worktrees: Vec<&WorktreeInfo> =
        worktrees.iter().filter(|w| !w.is_current).collect();

    if deletable_worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees available for deletion.".yellow());
        println!(
            "{}",
            "  (Cannot delete the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    println!("{}", section_header("Delete Worktree"));
    println!();

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select a worktree to delete (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(()),
    };

    let worktree_to_delete = deletable_worktrees[selection];

    // Show confirmation with details
    println!();
    println!("{}", "⚠ Warning".red().bold());
    println!(
        "  {} {}",
        "Name:".bright_white(),
        worktree_to_delete.name.yellow()
    );
    println!(
        "  {} {}",
        "Path:".bright_white(),
        worktree_to_delete.path.display()
    );
    println!(
        "  {} {}",
        "Branch:".bright_white(),
        worktree_to_delete.branch.yellow()
    );
    println!();

    // Ask about branch deletion if it's unique to this worktree
    let mut delete_branch = false;
    if manager.is_branch_unique_to_worktree(&worktree_to_delete.branch, &worktree_to_delete.name)? {
        println!("{}", "This branch is only used by this worktree.".yellow());
        delete_branch = Confirm::with_theme(&get_theme())
            .with_prompt("Also delete the branch?")
            .default(false)
            .interact_opt()?
            .unwrap_or(false);
        println!();
    }

    let confirm = Confirm::with_theme(&get_theme())
        .with_prompt("Are you sure you want to delete this worktree?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false);

    if !confirm {
        return Ok(());
    }

    // Execute pre-remove hooks
    if let Err(e) = hooks::execute_hooks(
        "pre-remove",
        &HookContext {
            worktree_name: worktree_to_delete.name.clone(),
            worktree_path: worktree_to_delete.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {}", e));
    }

    // Delete the worktree
    match manager.remove_worktree(&worktree_to_delete.name) {
        Ok(_) => {
            utils::print_success(&format!(
                "Deleted worktree '{}'",
                worktree_to_delete.name.bright_red()
            ));

            // Delete branch if requested
            if delete_branch {
                match manager.delete_branch(&worktree_to_delete.branch) {
                    Ok(_) => {
                        utils::print_success(&format!(
                            "Deleted branch '{}'",
                            worktree_to_delete.branch.bright_red()
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

    println!();
    press_any_key_to_continue()?;

    Ok(())
}

/// Switches to a different worktree
///
/// Displays a list of all worktrees with the current one marked,
/// allowing the user to select and switch to a different worktree.
///
/// # Switching Process
///
/// 1. Shows all worktrees (current one marked)
/// 2. User selects target worktree
/// 3. Writes target path for shell integration
/// 4. Executes post-switch hooks
///
/// # Shell Integration
///
/// The actual directory change is handled by the shell wrapper.
/// This function writes the target path to either:
/// - File specified by `GW_SWITCH_FILE` environment variable
/// - Standard output with `SWITCH_TO:` prefix (legacy mode)
///
/// # Returns
///
/// Returns `true` if the user switched worktrees, `false` otherwise
/// (includes selecting current worktree or cancellation).
///
/// # Errors
///
/// Returns an error if:
/// - Git repository operations fail
/// - File write operations fail
pub fn switch_worktree() -> Result<bool> {
    let manager = GitWorktreeManager::new()?;
    switch_worktree_internal(&manager)
}

/// Internal implementation of switch_worktree
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Returns
///
/// Returns `true` if a switch occurred, `false` if cancelled or already in selected worktree
fn switch_worktree_internal(manager: &GitWorktreeManager) -> Result<bool> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees available.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    println!();
    println!("{}", section_header("Switch Worktree"));
    println!();

    // Sort worktrees for display (current first)
    let mut sorted_worktrees = worktrees;
    sorted_worktrees.sort_by(|a, b| {
        if a.is_current && !b.is_current {
            std::cmp::Ordering::Less
        } else if !a.is_current && b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    let items: Vec<String> = sorted_worktrees
        .iter()
        .map(|w| {
            if w.is_current {
                format!("{} ({}) [current]", w.name, w.branch)
            } else {
                format!("{} ({})", w.name, w.branch)
            }
        })
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select a worktree to switch to (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(false),
    };

    let selected_worktree = &sorted_worktrees[selection];

    if selected_worktree.is_current {
        println!();
        println!("{}", "• Already in this worktree.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    // Switch to the selected worktree
    write_switch_path(&selected_worktree.path);

    println!();
    println!(
        "{} Switching to worktree '{}'",
        "+".green(),
        selected_worktree.name.bright_white().bold()
    );
    println!(
        "  {} {}",
        "Path:".bright_black(),
        selected_worktree.path.display()
    );
    println!(
        "  {} {}",
        "Branch:".bright_black(),
        selected_worktree.branch.yellow()
    );

    // Execute post-switch hooks
    if let Err(e) = hooks::execute_hooks(
        "post-switch",
        &HookContext {
            worktree_name: selected_worktree.name.clone(),
            worktree_path: selected_worktree.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {}", e));
    }

    Ok(true)
}

/// Batch deletes multiple worktrees with optional branch cleanup
///
/// Provides a multi-select interface for deleting multiple worktrees
/// in a single operation. This is useful for cleaning up multiple
/// feature branches or experimental worktrees. The function automatically
/// detects branches that would become orphaned and offers to delete them.
///
/// # Selection Interface
///
/// - Space: Toggle selection on current item
/// - Enter: Confirm and proceed with deletion
/// - ESC: Cancel operation
///
/// # Deletion Process
///
/// 1. **Multi-select**: Choose multiple worktrees (current excluded)
/// 2. **Branch Analysis**: Identifies branches unique to selected worktrees
/// 3. **Summary**: Shows selected worktrees and orphaned branches
/// 4. **Confirmation**: Confirms worktree deletion
/// 5. **Branch Confirmation**: If orphaned branches exist, asks to delete them
/// 6. **Batch Execution**: Deletes worktrees and optionally their branches
///
/// # Branch Management
///
/// - Uses `is_branch_unique_to_worktree` to identify orphaned branches
/// - Lists orphaned branches separately in the summary
/// - Only deletes branches for successfully deleted worktrees
/// - Reports branch deletion results separately
///
/// # Safety
///
/// - Cannot select/delete the current worktree
/// - Shows comprehensive summary before deletion
/// - Separate confirmations for worktrees and branches
/// - Executes pre-remove hooks for each worktree
/// - Continues with remaining deletions if one fails
///
/// # Returns
///
/// Returns `Ok(())` on completion. Individual deletion failures are
/// reported but don't stop the batch operation.
///
/// # Errors
///
/// Returns an error only if the operation cannot start (e.g., repository access fails).
pub fn batch_delete_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    batch_delete_worktrees_internal(&manager)
}

/// Internal implementation of batch_delete_worktrees
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Implementation Details
///
/// - Identifies orphaned branches before deletion
/// - Processes deletions sequentially to avoid conflicts
/// - Tracks success/failure count for both worktrees and branches
/// - Each deletion is independent (failures don't affect others)
/// - Branch deletions only occur for successfully deleted worktrees
fn batch_delete_worktrees_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to delete.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Filter out current worktree
    let deletable_worktrees: Vec<&WorktreeInfo> =
        worktrees.iter().filter(|w| !w.is_current).collect();

    if deletable_worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees available for deletion.".yellow());
        println!(
            "{}",
            "  (Cannot delete the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    println!();
    println!(
        "{}",
        "Select worktrees to delete (Space to select, Enter to confirm, ESC to cancel)"
            .bright_cyan()
    );
    println!();

    let selections = match MultiSelect::with_theme(&get_theme())
        .with_prompt("Select worktrees")
        .items(&items)
        .interact_opt()?
    {
        Some(selections) => selections,
        None => return Ok(()),
    };

    if selections.is_empty() {
        println!();
        println!("{}", "• No worktrees selected.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Check which branches are unique to selected worktrees
    let mut branches_to_delete = Vec::new();
    for &idx in &selections {
        let wt = deletable_worktrees[idx];
        if let Ok(is_unique) = manager.is_branch_unique_to_worktree(&wt.branch, &wt.name) {
            if is_unique {
                branches_to_delete.push((wt.branch.clone(), wt.name.clone()));
            }
        }
    }

    // Show summary
    println!();
    println!("{}", "Selected worktrees for deletion:".bright_white());
    for &idx in &selections {
        let wt = deletable_worktrees[idx];
        println!("  {} {} ({})", "•".red(), wt.name, wt.branch);
    }

    if !branches_to_delete.is_empty() {
        println!();
        println!("{}", "Branches that will become orphaned:".yellow());
        for (branch, _) in &branches_to_delete {
            println!("  {} {}", "•".yellow(), branch);
        }
    }

    println!();

    let confirm = Confirm::with_theme(&get_theme())
        .with_prompt(format!("Delete {} worktree(s)?", selections.len()))
        .default(false)
        .interact_opt()?
        .unwrap_or(false);

    if !confirm {
        return Ok(());
    }

    // Ask about branch deletion if any
    let delete_branches = if !branches_to_delete.is_empty() {
        println!();
        Confirm::with_theme(&get_theme())
            .with_prompt("Also delete the orphaned branches?")
            .default(false)
            .interact_opt()?
            .unwrap_or(false)
    } else {
        false
    };

    // Delete selected worktrees
    let mut success_count = 0;
    let mut error_count = 0;
    let mut deleted_worktrees = Vec::new();

    for &idx in &selections {
        let wt = deletable_worktrees[idx];

        // Execute pre-remove hooks
        if let Err(e) = hooks::execute_hooks(
            "pre-remove",
            &HookContext {
                worktree_name: wt.name.clone(),
                worktree_path: wt.path.clone(),
            },
        ) {
            utils::print_warning(&format!("Hook execution warning: {}", e));
        }

        match manager.remove_worktree(&wt.name) {
            Ok(_) => {
                utils::print_success(&format!("Deleted worktree '{}'", wt.name.bright_red()));
                deleted_worktrees.push((wt.branch.clone(), wt.name.clone()));
                success_count += 1;
            }
            Err(e) => {
                utils::print_error(&format!("Failed to delete '{}': {}", wt.name, e));
                error_count += 1;
            }
        }
    }

    // Delete branches if requested
    if delete_branches {
        let mut branch_success = 0;
        let mut branch_error = 0;

        println!();
        for (branch, worktree_name) in &branches_to_delete {
            // Only delete branches for successfully deleted worktrees
            if deleted_worktrees
                .iter()
                .any(|(b, w)| b == branch && w == worktree_name)
            {
                match manager.delete_branch(branch) {
                    Ok(_) => {
                        utils::print_success(&format!("Deleted branch '{}'", branch.bright_red()));
                        branch_success += 1;
                    }
                    Err(e) => {
                        utils::print_error(&format!("Failed to delete branch '{}': {}", branch, e));
                        branch_error += 1;
                    }
                }
            }
        }

        if branch_success > 0 || branch_error > 0 {
            println!();
            println!(
                "{} Deleted {} branch(es), {} failed",
                "•".bright_green(),
                branch_success,
                branch_error
            );
        }
    }

    println!();
    println!(
        "{} Deleted {} worktree(s), {} failed",
        "•".bright_green(),
        success_count,
        error_count
    );

    println!();
    press_any_key_to_continue()?;

    Ok(())
}

/// Cleans up old worktrees based on age
///
/// **Note**: This feature is currently not implemented and serves as
/// a placeholder for future functionality.
///
/// When implemented, this function will:
/// - Identify worktrees older than a specified number of days
/// - Show a preview of worktrees to be deleted
/// - Allow batch deletion of old worktrees
///
/// # Current Behavior
///
/// Displays a message indicating the feature is not yet implemented.
///
/// # Returns
///
/// Always returns `Ok(())` after displaying the message.
pub fn cleanup_old_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    cleanup_old_worktrees_internal(&manager)
}

/// Internal implementation of cleanup_old_worktrees
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Future Implementation
///
/// Will require:
/// - Tracking worktree creation dates (possibly in .git-workers.toml)
/// - Age calculation logic
/// - Preview and confirmation UI
fn cleanup_old_worktrees_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to clean up.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    println!("{}", section_header("Cleanup Old Worktrees"));
    println!();

    // Get age threshold
    let _days = match input_esc_with_default("Delete worktrees older than (days)", "30") {
        Some(days_str) => match days_str.parse::<u64>() {
            Ok(d) => d,
            Err(_) => {
                utils::print_error("Invalid number");
                return Ok(());
            }
        },
        None => return Ok(()),
    };

    // Find old worktrees (mock implementation - would need actual age tracking)
    println!();
    utils::print_warning("Age-based cleanup is not yet implemented.");
    println!(
        "{}",
        "This feature requires tracking worktree creation dates.".bright_black()
    );

    println!();
    press_any_key_to_continue()?;

    Ok(())
}

/// Renames an existing worktree
///
/// Provides functionality to rename a worktree and optionally its
/// associated branch. This is useful when refactoring feature names
/// or reorganizing worktrees.
///
/// # Rename Process
///
/// 1. **Selection**: Choose a worktree to rename (current excluded)
/// 2. **New Name**: Enter the new name for the worktree
/// 3. **Branch Rename**: If branch name matches worktree name, offers to rename it
/// 4. **Preview**: Shows before/after comparison
/// 5. **Execution**: Renames worktree directory and updates Git metadata
///
/// # Branch Renaming Logic
///
/// The branch is offered for renaming if:
/// - Branch name equals worktree name
/// - Branch name equals `feature/{worktree-name}`
///
/// # Limitations
///
/// - Cannot rename the current worktree
/// - Cannot rename worktrees with detached HEAD
/// - New name must be unique
///
/// # Returns
///
/// Returns `Ok(())` on successful completion or cancellation.
///
/// # Errors
///
/// Returns an error if:
/// - File system operations fail
/// - Git metadata update fails
/// - New name conflicts with existing worktree
pub fn rename_worktree() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    rename_worktree_internal(&manager)
}

/// Internal implementation of rename_worktree
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
///
/// # Implementation Details
///
/// - Updates worktree directory name
/// - Updates .git/worktrees/`<name>` metadata
/// - Updates gitdir references
/// - Optionally renames associated branch
fn rename_worktree_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees to rename.".yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Filter out current worktree
    let renameable_worktrees: Vec<&WorktreeInfo> =
        worktrees.iter().filter(|w| !w.is_current).collect();

    if renameable_worktrees.is_empty() {
        println!();
        println!("{}", "• No worktrees available for renaming.".yellow());
        println!(
            "{}",
            "  (Cannot rename the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    println!("{}", section_header("Rename Worktree"));
    println!();

    let items: Vec<String> = renameable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match Select::with_theme(&get_theme())
        .with_prompt("Select a worktree to rename (ESC to cancel)")
        .items(&items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(()),
    };

    let worktree = renameable_worktrees[selection];

    // Get new name
    println!();
    let new_name =
        match input_esc(format!("New name for '{}' (ESC to cancel)", worktree.name).as_str()) {
            Some(name) => name.trim().to_string(),
            None => return Ok(()),
        };

    if new_name.is_empty() {
        utils::print_error("Name cannot be empty");
        return Ok(());
    }

    if new_name == worktree.name {
        utils::print_warning("New name is the same as the current name");
        return Ok(());
    }

    // Check if the worktree has a branch that could be renamed
    let rename_branch = if worktree.branch != "detached"
        && worktree.branch != "unknown"
        && (worktree.branch == worktree.name
            || worktree.branch == format!("feature/{}", worktree.name))
    {
        println!();
        Confirm::with_theme(&get_theme())
            .with_prompt("Also rename the associated branch?")
            .default(true)
            .interact_opt()?
            .unwrap_or(false)
    } else {
        false
    };

    // Show preview
    println!();
    println!("{}", "Preview:".bright_white());
    println!(
        "  {} {} → {}",
        "Worktree:".bright_white(),
        worktree.name,
        new_name.bright_green()
    );

    let new_path = worktree.path.parent().unwrap().join(&new_name);
    println!(
        "  {} {} → {}",
        "Path:".bright_white(),
        worktree.path.display(),
        new_path.display().to_string().bright_green()
    );

    if rename_branch {
        let new_branch = if worktree.branch.starts_with("feature/") {
            format!("feature/{}", new_name)
        } else {
            new_name.clone()
        };
        println!(
            "  {} {} → {}",
            "Branch:".bright_white(),
            worktree.branch,
            new_branch.bright_green()
        );
    }

    println!();
    let confirm = Confirm::with_theme(&get_theme())
        .with_prompt("Proceed with rename?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false);

    if !confirm {
        return Ok(());
    }

    // Perform the rename
    utils::print_progress(&format!("Renaming worktree to '{}'...", new_name));

    match manager.rename_worktree(&worktree.name, &new_name) {
        Ok(_) => {
            utils::print_success(&format!(
                "Worktree renamed from '{}' to '{}'!",
                worktree.name.yellow(),
                new_name.bright_green()
            ));

            // Rename branch if requested
            if rename_branch {
                let new_branch = if worktree.branch.starts_with("feature/") {
                    format!("feature/{}", new_name)
                } else {
                    new_name.clone()
                };

                utils::print_progress(&format!("Renaming branch to '{}'...", new_branch));

                match manager.rename_branch(&worktree.branch, &new_branch) {
                    Ok(_) => {
                        utils::print_success(&format!(
                            "Branch renamed from '{}' to '{}'!",
                            worktree.branch.yellow(),
                            new_branch.bright_green()
                        ));
                    }
                    Err(e) => {
                        utils::print_error(&format!("Failed to rename branch: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            utils::print_error(&format!("Failed to rename worktree: {}", e));
        }
    }

    println!();
    press_any_key_to_continue()?;

    Ok(())
}

/// Edits the hooks configuration file
///
/// Opens the `.git-workers.toml` configuration file in the user's
/// preferred editor, allowing them to configure lifecycle hooks.
///
/// # Configuration File Location
///
/// The function searches for the configuration file in the following order:
/// 1. Current directory (useful for bare repo worktrees)
/// 2. Parent directory's main/master worktree (for organized worktree structures)
/// 3. Repository root (for standard repos)
///
/// This flexible lookup strategy ensures hooks work correctly in both
/// regular and bare repositories, while maintaining consistency across
/// all worktrees in a project.
///
/// # Editor Selection
///
/// Uses the following priority for editor selection:
/// 1. `EDITOR` environment variable
/// 2. `VISUAL` environment variable
/// 3. Platform default (vi on Unix, notepad on Windows)
///
/// # File Creation
///
/// If the configuration file doesn't exist, offers to create it
/// with a template containing example hooks for all lifecycle events.
///
/// # Template
///
/// The generated template includes:
/// - Repository URL configuration (optional)
/// - Post-create hooks example
/// - Pre-remove hooks example
/// - Post-switch hooks example
/// - Documentation for template variables
///
/// # Returns
///
/// Returns `Ok(())` after editing is complete or cancelled.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a Git repository
/// - Cannot determine configuration file location
/// - Editor fails to launch
pub fn edit_hooks() -> Result<()> {
    use std::process::Command;

    println!();
    println!("{}", section_header("Edit Hooks Configuration"));
    println!();

    // Find the config file location
    let config_path = if let Ok(repo) = git2::Repository::discover(".") {
        // Try a simpler approach: look for main/master in parent directory
        if let Ok(cwd) = std::env::current_dir() {
            // Check if we're in a worktree structure like /path/to/repo/branch/worktree-name
            if let Some(parent) = cwd.parent() {
                // Look for main or master directories in the parent
                let main_path = parent.join("main").join(CONFIG_FILE_NAME);
                let master_path = parent.join("master").join(CONFIG_FILE_NAME);

                if main_path.exists() {
                    main_path
                } else if master_path.exists() {
                    master_path
                } else {
                    // For regular repositories, use workdir
                    let workdir = repo
                        .workdir()
                        .ok_or_else(|| anyhow::anyhow!("No working directory"))?;
                    workdir.join(CONFIG_FILE_NAME)
                }
            } else {
                // No parent directory, use workdir
                let workdir = repo
                    .workdir()
                    .ok_or_else(|| anyhow::anyhow!("No working directory"))?;
                workdir.join(".git-workers.toml")
            }
        } else {
            // Can't get current directory, use workdir
            let workdir = repo
                .workdir()
                .ok_or_else(|| anyhow::anyhow!("No working directory"))?;
            workdir.join(".git-workers.toml")
        }
    } else {
        utils::print_error("Not in a git repository");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    };

    // Create the file if it doesn't exist
    if !config_path.exists() {
        println!("{}", "• No configuration file found.".yellow());
        println!();

        let create = Confirm::with_theme(&get_theme())
            .with_prompt(format!("Create {}?", CONFIG_FILE_NAME))
            .default(true)
            .interact_opt()?
            .unwrap_or(false);

        if create {
            // Create a template configuration
            let template = r#"# Git Workers configuration file

[repository]
# Repository URL for identification (optional)
# This ensures hooks only run in the intended repository
# url = "https://github.com/owner/repo.git"

[hooks]
# Run after creating a new worktree
post-create = [
    # "npm install",
    # "cp .env.example .env"
]

# Run before removing a worktree
pre-remove = [
    # "rm -rf node_modules"
]

# Run after switching to a worktree
post-switch = [
    # "echo 'Switched to {{worktree_name}}'"
]

[files]
# Optional: Specify a custom source directory
# If not specified, automatically finds the main worktree
# source = "/path/to/custom/source"
# source = "./templates"  # Relative to repository root

# Files to copy when creating new worktrees
copy = [
    # ".env",
    # ".env.local"
]
"#;

            std::fs::write(&config_path, template)?;
            utils::print_success(&format!("Created {} with template", CONFIG_FILE_NAME));
        } else {
            return Ok(());
        }
    }

    // Get the user's preferred editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    println!(
        "{} Opening {} with {}...",
        "•".bright_blue(),
        config_path.display().to_string().bright_white(),
        editor.bright_yellow()
    );
    println!();

    // Open the editor
    let status = Command::new(&editor).arg(&config_path).status();

    match status {
        Ok(status) if status.success() => {
            utils::print_success("Configuration file edited successfully");
        }
        Ok(_) => {
            utils::print_warning("Editor exited with non-zero status");
        }
        Err(e) => {
            utils::print_error(&format!("Failed to open editor: {}", e));
            println!();
            println!("You can manually edit the file at:");
            println!("  {}", config_path.display().to_string().bright_white());
        }
    }

    println!();
    press_any_key_to_continue()?;

    Ok(())
}
