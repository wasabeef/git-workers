use anyhow::Result;
use colored::*;

use crate::constants::{section_header, DEFAULT_WORKTREE_CLEANUP_DAYS};
use crate::git::GitWorktreeManager;
use crate::input_esc_raw::input_esc_with_default_raw as input_esc_with_default;
use crate::utils::{self, press_any_key_to_continue};

pub fn cleanup_old_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    cleanup_old_worktrees_internal(&manager)
}

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

    let _days = match input_esc_with_default(
        "Delete worktrees older than (days)",
        DEFAULT_WORKTREE_CLEANUP_DAYS,
    ) {
        Some(days_str) => match days_str.parse::<u64>() {
            Ok(days) => days,
            Err(_) => {
                utils::print_error("Invalid number");
                return Ok(());
            }
        },
        None => return Ok(()),
    };

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
