use anyhow::{anyhow, Result};
use colored::*;

use crate::constants::{section_header, DEFAULT_MENU_SELECTION, HOOK_PRE_REMOVE};
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::hooks::{self, HookContext};
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::{self, press_any_key_to_continue};

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
    // Execute pre-remove hooks
    if let Err(e) = hooks::execute_hooks(
        HOOK_PRE_REMOVE,
        &HookContext {
            worktree_name: config.name.clone(),
            worktree_path: config.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {e}"));
    }

    // Delete the worktree
    manager
        .remove_worktree(&config.name)
        .map_err(|e| anyhow!("Failed to delete worktree: {e}"))?;

    let name_red = config.name.bright_red();
    utils::print_success(&format!("Deleted worktree '{name_red}'"));

    // Delete branch if requested
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
    let ui = DialoguerUI;
    delete_worktree_with_ui(&manager, &ui)
}

/// Internal implementation of delete_worktree with dependency injection
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
/// * `ui` - User interface implementation for testability
///
/// # Deletion Process
///
/// 1. Filters out current worktree (cannot be deleted)
/// 2. Presents selection list to user
/// 3. Checks if branch is unique to the worktree
/// 4. Confirms deletion with detailed preview
/// 5. Executes pre-remove hooks
/// 6. Performs deletion of worktree and optionally branch
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

    // Use business logic to filter deletable worktrees
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

    // Use business logic to analyze deletion requirements
    let analysis = analyze_deletion(worktree_to_delete, manager)?;

    // Show confirmation with details
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

    // Ask about branch deletion if it's unique to this worktree
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

    // Create deletion configuration
    let config = WorktreeDeleteConfig {
        name: analysis.worktree.name.clone(),
        path: analysis.worktree.path.clone(),
        branch: analysis.worktree.branch.clone(),
        delete_branch,
    };

    // Execute deletion using business logic
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
        // Branch name matches worktree name and is unique
        assert!(should_delete_orphaned_branch(true, "feature", "feature"));
    }

    #[test]
    fn test_should_delete_orphaned_branch_false_not_unique() {
        // Branch is not unique
        assert!(!should_delete_orphaned_branch(false, "feature", "feature"));
    }

    #[test]
    fn test_should_delete_orphaned_branch_false_name_mismatch() {
        // Branch name doesn't match worktree name
        assert!(!should_delete_orphaned_branch(true, "main", "feature"));
    }

    #[test]
    fn test_get_deletable_worktrees_filter_main() {
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
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

        // Basic config creation test
        assert_eq!(config.name, "test-worktree");
        assert_eq!(config.branch, "test-branch");
        assert!(!config.delete_branch);
    }
}
