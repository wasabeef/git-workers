use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Confirm, MultiSelect};

use crate::adapters::git::GitWorktreeManager;
use crate::adapters::hooks::{self, HookContext};
use crate::constants::{section_header, DEFAULT_MENU_SELECTION, HOOK_PRE_REMOVE};
use crate::domain::worktree::WorktreeInfo;
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::{self, get_theme, press_any_key_to_continue};

/// Validate deletion target
#[allow(dead_code)]
pub fn validate_deletion_target(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Worktree name cannot be empty"));
    }

    if name == "main" || name == "master" {
        return Err(anyhow!("Cannot delete main worktree"));
    }

    Ok(())
}

/// Check if orphaned branch should be deleted
#[allow(dead_code)]
pub fn should_delete_orphaned_branch(
    is_branch_unique: bool,
    branch_name: &str,
    worktree_name: &str,
) -> bool {
    is_branch_unique && branch_name == worktree_name
}

/// Configuration for batch delete operations
#[derive(Debug, Clone)]
pub struct BatchDeleteConfig {
    pub selected_worktrees: Vec<String>,
    pub delete_orphaned_branches: bool,
}

/// Configuration for worktree deletion
#[derive(Debug, Clone)]
pub struct WorktreeDeleteConfig {
    pub name: String,
    pub path: std::path::PathBuf,
    pub branch: String,
    pub delete_branch: bool,
}

/// Result of deletion analysis
#[derive(Debug, Clone)]
pub struct DeletionAnalysis {
    pub worktree: WorktreeInfo,
    pub is_branch_unique: bool,
    pub delete_branch_recommended: bool,
}

/// Pure business logic for filtering deletable worktrees
pub fn get_deletable_worktrees(worktrees: &[WorktreeInfo]) -> Vec<&WorktreeInfo> {
    worktrees.iter().filter(|w| !w.is_current).collect()
}

/// Pure business logic for filtering deletable worktrees for batch operations
pub fn prepare_batch_delete_items(worktrees: &[WorktreeInfo]) -> Vec<String> {
    worktrees
        .iter()
        .filter(|w| !w.is_current)
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect()
}

/// Pure business logic for analyzing deletion requirements
pub fn analyze_deletion(
    worktree: &WorktreeInfo,
    manager: &GitWorktreeManager,
) -> Result<DeletionAnalysis> {
    let is_branch_unique =
        manager.is_branch_unique_to_worktree(&worktree.branch, &worktree.name)?;

    Ok(DeletionAnalysis {
        worktree: worktree.clone(),
        is_branch_unique,
        delete_branch_recommended: is_branch_unique,
    })
}

/// Pure business logic for executing deletion
pub fn execute_deletion(config: &WorktreeDeleteConfig, manager: &GitWorktreeManager) -> Result<()> {
    if let Err(e) = hooks::execute_hooks(
        HOOK_PRE_REMOVE,
        &HookContext {
            worktree_name: config.name.clone(),
            worktree_path: config.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {e}"));
    }

    manager
        .remove_worktree(&config.name)
        .map_err(|e| anyhow!("Failed to delete worktree: {e}"))?;

    let name_red = config.name.bright_red();
    utils::print_success(&format!("Deleted worktree '{name_red}'"));

    if config.delete_branch {
        match manager.delete_branch(&config.branch) {
            Ok(_) => {
                let branch_red = config.branch.bright_red();
                utils::print_success(&format!("Deleted branch '{branch_red}'"));
            }
            Err(e) => {
                utils::print_warning(&format!("Failed to delete branch: {e}"));
            }
        }
    }

    Ok(())
}

/// Deletes a single worktree interactively
pub fn delete_worktree() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    let ui = DialoguerUI;
    delete_worktree_with_ui(&manager, &ui)
}

/// Internal implementation of delete_worktree with dependency injection
pub fn delete_worktree_with_ui(manager: &GitWorktreeManager, ui: &dyn UserInterface) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        let msg = "• No worktrees to delete.".yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    let deletable_worktrees = get_deletable_worktrees(&worktrees);

    if deletable_worktrees.is_empty() {
        println!();
        let msg = "• No worktrees available for deletion.".yellow();
        println!("{msg}");
        println!(
            "{}",
            "  (Cannot delete the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    let header = section_header("Delete Worktree");
    println!("{header}");
    println!();

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match ui.select_with_default(
        "Select a worktree to delete (ESC to cancel)",
        &items,
        DEFAULT_MENU_SELECTION,
    ) {
        Ok(selection) => selection,
        Err(_) => return Ok(()),
    };

    let worktree_to_delete = deletable_worktrees[selection];
    let analysis = analyze_deletion(worktree_to_delete, manager)?;

    println!();
    let warning = "⚠ Warning".red().bold();
    println!("{warning}");
    let name_label = "Name:".bright_white();
    let name_value = analysis.worktree.name.yellow();
    println!("  {name_label} {name_value}");
    let path_label = "Path:".bright_white();
    let path_value = analysis.worktree.path.display();
    println!("  {path_label} {path_value}");
    let branch_label = "Branch:".bright_white();
    let branch_value = analysis.worktree.branch.yellow();
    println!("  {branch_label} {branch_value}");
    println!();

    let mut delete_branch = false;
    if analysis.is_branch_unique {
        let msg = "This branch is only used by this worktree.".yellow();
        println!("{msg}");
        delete_branch = ui
            .confirm_with_default("Also delete the branch?", false)
            .unwrap_or(false);
        println!();
    }

    let confirm = ui
        .confirm_with_default("Are you sure you want to delete this worktree?", false)
        .unwrap_or(false);

    if !confirm {
        return Ok(());
    }

    let config = WorktreeDeleteConfig {
        name: analysis.worktree.git_name.clone(),
        path: analysis.worktree.path.clone(),
        branch: analysis.worktree.branch.clone(),
        delete_branch,
    };

    match execute_deletion(&config, manager) {
        Ok(_) => {
            println!();
            press_any_key_to_continue()?;
            Ok(())
        }
        Err(e) => {
            utils::print_error(&format!("{e}"));
            println!();
            press_any_key_to_continue()?;
            Ok(())
        }
    }
}

/// Batch deletes multiple worktrees with optional branch cleanup
pub fn batch_delete_worktrees() -> Result<()> {
    let manager = GitWorktreeManager::new()?;
    batch_delete_worktrees_internal(&manager)
}

fn batch_delete_worktrees_internal(manager: &GitWorktreeManager) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        let msg = "• No worktrees to delete.".yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    let deletable_worktrees: Vec<&WorktreeInfo> =
        worktrees.iter().filter(|w| !w.is_current).collect();

    if deletable_worktrees.is_empty() {
        println!();
        let msg = "• No worktrees available for deletion.".yellow();
        println!("{msg}");
        println!(
            "{}",
            "  (Cannot delete the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    let header = section_header("Batch Delete Worktrees");
    println!("{header}");
    println!();

    let items = prepare_batch_delete_items(&worktrees);

    let selections = MultiSelect::with_theme(&get_theme())
        .with_prompt(
            "Select worktrees to delete (Space to toggle, Enter to confirm, ESC to cancel)",
        )
        .items(&items)
        .interact_opt()?;

    let selections = match selections {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(()),
    };

    let selected_worktrees: Vec<&WorktreeInfo> =
        selections.iter().map(|&i| deletable_worktrees[i]).collect();

    let mut branches_to_delete = Vec::new();
    for wt in &selected_worktrees {
        if manager.is_branch_unique_to_worktree(&wt.branch, &wt.name)? {
            branches_to_delete.push((wt.branch.clone(), wt.name.clone()));
        }
    }

    println!();
    let summary_label = "Summary:".bright_white();
    println!("{summary_label}");
    println!();
    let worktrees_label = "Selected worktrees:".bright_cyan();
    println!("{worktrees_label}");
    for wt in &selected_worktrees {
        let name = wt.name.bright_red();
        let branch = &wt.branch;
        println!("  • {name} ({branch})");
    }

    if !branches_to_delete.is_empty() {
        println!();
        let branches_label = "Branches that will become orphaned:".bright_yellow();
        println!("{branches_label}");
        for (branch, _) in &branches_to_delete {
            let branch_yellow = branch.bright_yellow();
            println!("  • {branch_yellow}");
        }
    }

    println!();
    let warning = "⚠ Warning".red().bold();
    println!("{warning}");
    let selected_count = selected_worktrees.len();
    println!("This will delete {selected_count} worktree(s) and their files.");
    if !branches_to_delete.is_empty() {
        let branch_count = branches_to_delete.len();
        println!("This action will also make {branch_count} branch(es) orphaned.");
    }
    println!();

    let confirm = Confirm::with_theme(&get_theme())
        .with_prompt("Are you sure you want to delete these worktrees?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false);

    if !confirm {
        return Ok(());
    }

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

    println!();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut deleted_worktrees = Vec::new();

    for wt in &selected_worktrees {
        if let Err(e) = hooks::execute_hooks(
            HOOK_PRE_REMOVE,
            &HookContext {
                worktree_name: wt.name.clone(),
                worktree_path: wt.path.clone(),
            },
        ) {
            utils::print_warning(&format!("Hook execution warning: {e}"));
        }

        match manager.remove_worktree(&wt.git_name) {
            Ok(_) => {
                let name_red = wt.name.bright_red();
                utils::print_success(&format!("Deleted worktree '{name_red}'"));
                deleted_worktrees.push((wt.branch.clone(), wt.name.clone()));
                success_count += 1;
            }
            Err(e) => {
                let name = &wt.name;
                utils::print_error(&format!("Failed to delete '{name}': {e}"));
                error_count += 1;
            }
        }
    }

    if delete_branches {
        let mut branch_success = 0;
        let mut branch_error = 0;

        println!();
        for (branch, worktree_name) in &branches_to_delete {
            if deleted_worktrees
                .iter()
                .any(|(b, w)| b == branch && w == worktree_name)
            {
                match manager.delete_branch(branch) {
                    Ok(_) => {
                        let branch_red = branch.bright_red();
                        utils::print_success(&format!("Deleted branch '{branch_red}'"));
                        branch_success += 1;
                    }
                    Err(e) => {
                        utils::print_error(&format!("Failed to delete branch '{branch}': {e}"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_deletion_target_valid() {
        assert!(validate_deletion_target("feature-branch").is_ok());
        assert!(validate_deletion_target("bugfix-123").is_ok());
        assert!(validate_deletion_target("valid-name").is_ok());
    }

    #[test]
    fn test_validate_deletion_target_invalid() {
        assert!(validate_deletion_target("").is_err());
        assert!(validate_deletion_target("main").is_err());
        assert!(validate_deletion_target("master").is_err());
    }

    #[test]
    fn test_should_delete_orphaned_branch_true() {
        assert!(should_delete_orphaned_branch(true, "feature", "feature"));
    }

    #[test]
    fn test_should_delete_orphaned_branch_false_not_unique() {
        assert!(!should_delete_orphaned_branch(false, "feature", "feature"));
    }

    #[test]
    fn test_should_delete_orphaned_branch_false_name_mismatch() {
        assert!(!should_delete_orphaned_branch(true, "main", "feature"));
    }

    #[test]
    fn test_get_deletable_worktrees_filter_main() {
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
                git_name: "main".to_string(),
                path: PathBuf::from("/tmp/main"),
                branch: "main".to_string(),
                is_current: true,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: "feature".to_string(),
                git_name: "feature".to_string(),
                path: PathBuf::from("/tmp/feature"),
                branch: "feature".to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
        ];
        let deletable = get_deletable_worktrees(&worktrees);
        assert_eq!(deletable.len(), 1);
        assert_eq!(deletable[0].name, "feature");
    }

    #[test]
    fn test_get_deletable_worktrees_empty() {
        let worktrees = vec![];
        let deletable = get_deletable_worktrees(&worktrees);
        assert!(deletable.is_empty());
    }

    #[test]
    fn test_deletion_analysis_creation() {
        let worktree = WorktreeInfo {
            name: "feature".to_string(),
            git_name: "feature".to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: "feature".to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let analysis = DeletionAnalysis {
            worktree: worktree.clone(),
            is_branch_unique: true,
            delete_branch_recommended: true,
        };

        assert_eq!(analysis.worktree.name, "feature");
        assert!(analysis.is_branch_unique);
        assert!(analysis.delete_branch_recommended);
    }

    #[test]
    fn test_execute_deletion_config() {
        let config = WorktreeDeleteConfig {
            name: "test-worktree".to_string(),
            path: PathBuf::from("/tmp/test"),
            branch: "test-branch".to_string(),
            delete_branch: false,
        };

        assert_eq!(config.name, "test-worktree");
        assert_eq!(config.branch, "test-branch");
        assert!(!config.delete_branch);
    }

    #[test]
    fn test_prepare_batch_delete_items() {
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
                git_name: "main".to_string(),
                path: std::path::PathBuf::from("/test/main"),
                branch: "main".to_string(),
                is_current: true,
                is_locked: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
            WorktreeInfo {
                name: "feature-branch".to_string(),
                git_name: "feature-branch".to_string(),
                path: std::path::PathBuf::from("/test/feature-branch"),
                branch: "feature/test".to_string(),
                is_current: false,
                is_locked: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
        ];

        let items = prepare_batch_delete_items(&worktrees);

        assert_eq!(items.len(), 1);
        assert!(items[0].contains("feature-branch"));
        assert!(items[0].contains("feature/test"));
        assert!(!items[0].contains("main"));
    }
}
