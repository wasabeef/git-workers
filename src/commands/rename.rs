use anyhow::{anyhow, Result};
use colored::*;

use crate::constants::{
    section_header, DEFAULT_BRANCH_DETACHED, DEFAULT_BRANCH_UNKNOWN, DEFAULT_MENU_SELECTION,
};
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::{self, press_any_key_to_continue};

// Use validation function from core module
use super::super::core::validate_worktree_name;

/// Check if branch should be renamed
#[allow(dead_code)]
pub fn should_rename_branch(
    worktree_name: &str,
    branch_name: &str,
    user_wants_rename: bool,
) -> bool {
    user_wants_rename && worktree_name == branch_name
}

/// Configuration for worktree renaming (simplified for tests)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RenameConfig {
    pub old_name: String,
    pub new_name: String,
    pub old_path: std::path::PathBuf,
    pub new_path: std::path::PathBuf,
    pub rename_branch: bool,
}

/// Configuration for worktree renaming
#[derive(Debug, Clone)]
pub struct WorktreeRenameConfig {
    pub old_name: String,
    pub new_name: String,
    pub old_path: std::path::PathBuf,
    pub new_path: std::path::PathBuf,
    pub old_branch: String,
    pub new_branch: Option<String>,
    pub rename_branch: bool,
}

/// Result of rename analysis
#[derive(Debug, Clone)]
pub struct RenameAnalysis {
    pub worktree: WorktreeInfo,
    pub can_rename_branch: bool,
    pub suggested_branch_name: Option<String>,
    pub is_feature_branch: bool,
}

/// Pure business logic for filtering renameable worktrees
pub fn get_renameable_worktrees(worktrees: &[WorktreeInfo]) -> Vec<&WorktreeInfo> {
    worktrees.iter().filter(|w| !w.is_current).collect()
}

/// Pure business logic for analyzing rename requirements
pub fn analyze_rename_requirements(worktree: &WorktreeInfo) -> Result<RenameAnalysis> {
    let can_rename_branch = worktree.branch != DEFAULT_BRANCH_DETACHED
        && worktree.branch != DEFAULT_BRANCH_UNKNOWN
        && (worktree.branch == worktree.name
            || worktree.branch == format!("feature/{}", worktree.name));

    let is_feature_branch = worktree.branch.starts_with("feature/");
    let suggested_branch_name = if can_rename_branch {
        Some(if is_feature_branch {
            format!("feature/{}", worktree.name)
        } else {
            worktree.name.clone()
        })
    } else {
        None
    };

    Ok(RenameAnalysis {
        worktree: worktree.clone(),
        can_rename_branch,
        suggested_branch_name,
        is_feature_branch,
    })
}

/// Pure business logic for validating rename operation
pub fn validate_rename_operation(old_name: &str, new_name: &str) -> Result<()> {
    if old_name.is_empty() {
        return Err(anyhow!("Old name cannot be empty"));
    }

    if new_name.is_empty() {
        return Err(anyhow!("New name cannot be empty"));
    }

    if new_name == old_name {
        return Err(anyhow!("New name must be different from the current name"));
    }

    if old_name == "main" || old_name == "master" {
        return Err(anyhow!("Cannot rename main worktree"));
    }

    if new_name == "main" || new_name == "master" {
        return Err(anyhow!("Cannot rename to main"));
    }

    Ok(())
}

/// Pure business logic for executing rename operation
pub fn execute_rename(config: &WorktreeRenameConfig, manager: &GitWorktreeManager) -> Result<()> {
    // Rename worktree
    manager
        .rename_worktree(&config.old_name, &config.new_name)
        .map_err(|e| anyhow!("Failed to rename worktree: {e}"))?;

    utils::print_success(&format!(
        "Worktree renamed from '{}' to '{}'!",
        config.old_name.yellow(),
        config.new_name.bright_green()
    ));

    // Rename branch if requested
    if config.rename_branch {
        if let Some(ref new_branch) = config.new_branch {
            utils::print_progress(&format!("Renaming branch to '{new_branch}'..."));

            match manager.rename_branch(&config.old_branch, new_branch) {
                Ok(_) => {
                    utils::print_success(&format!(
                        "Branch renamed from '{}' to '{}'!",
                        config.old_branch.yellow(),
                        new_branch.bright_green()
                    ));
                }
                Err(e) => {
                    return Err(anyhow!("Failed to rename branch: {e}"));
                }
            }
        }
    }

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
    let ui = DialoguerUI;
    rename_worktree_with_ui(&manager, &ui)
}

/// Internal implementation of rename_worktree with dependency injection
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
/// * `ui` - User interface implementation for testability
///
/// # Implementation Details
///
/// - Updates worktree directory name
/// - Updates .git/worktrees/`<name>` metadata
/// - Updates gitdir references
/// - Optionally renames associated branch
pub fn rename_worktree_with_ui(manager: &GitWorktreeManager, ui: &dyn UserInterface) -> Result<()> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        let msg = "• No worktrees to rename.".yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    // Use business logic to filter renameable worktrees
    let renameable_worktrees = get_renameable_worktrees(&worktrees);

    if renameable_worktrees.is_empty() {
        println!();
        let msg = "• No worktrees available for renaming.".yellow();
        println!("{msg}");
        println!(
            "{}",
            "  (Cannot rename the current worktree)".bright_black()
        );
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    let header = section_header("Rename Worktree");
    println!("{header}");
    println!();

    let items: Vec<String> = renameable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

    let selection = match ui.select_with_default(
        "Select a worktree to rename (ESC to cancel)",
        &items,
        DEFAULT_MENU_SELECTION,
    ) {
        Ok(selection) => selection,
        Err(_) => return Ok(()),
    };

    let worktree = renameable_worktrees[selection];

    // Get new name
    println!();
    let new_name = match ui.input(&format!("New name for '{}' (ESC to cancel)", worktree.name)) {
        Ok(name) => name.trim().to_string(),
        Err(_) => return Ok(()),
    };

    // Use business logic to validate rename operation
    if let Err(e) = validate_rename_operation(&worktree.git_name, &new_name) {
        utils::print_warning(&e.to_string());
        return Ok(());
    }

    // Validate new name format
    let new_name = match validate_worktree_name(&new_name) {
        Ok(validated_name) => validated_name,
        Err(e) => {
            utils::print_error(&format!("Invalid worktree name: {e}"));
            println!();
            press_any_key_to_continue()?;
            return Ok(());
        }
    };

    // Use business logic to analyze rename requirements
    let analysis = analyze_rename_requirements(worktree)?;

    // Ask about branch renaming if applicable
    let rename_branch = if analysis.can_rename_branch {
        println!();
        match ui.confirm_with_default("Also rename the associated branch?", true) {
            Ok(confirm) => confirm,
            Err(_) => return Ok(()),
        }
    } else {
        false
    };

    let new_branch = if rename_branch {
        if analysis.is_feature_branch {
            Some(format!("feature/{new_name}"))
        } else {
            Some(new_name.clone())
        }
    } else {
        None
    };

    // Show preview
    println!();
    let preview_label = "Preview:".bright_white();
    println!("{preview_label}");
    let worktree_label = "Worktree:".bright_white();
    let old_name = &worktree.name;
    let new_name_green = new_name.bright_green();
    println!("  {worktree_label} {old_name} → {new_name_green}");

    let new_path = worktree.path.parent().unwrap().join(&new_name);
    let path_label = "Path:".bright_white();
    let old_path = worktree.path.display();
    let new_path_green = new_path.display().to_string().bright_green();
    println!("  {path_label} {old_path} → {new_path_green}");

    if let Some(ref new_branch_name) = new_branch {
        let branch_label = "Branch:".bright_white();
        let old_branch = &worktree.branch;
        let new_branch_green = new_branch_name.bright_green();
        println!("  {branch_label} {old_branch} → {new_branch_green}");
    }

    println!();
    let confirm = match ui.confirm_with_default("Proceed with rename?", false) {
        Ok(confirm) => confirm,
        Err(_) => return Ok(()),
    };

    if !confirm {
        return Ok(());
    }

    // Create rename configuration
    let config = WorktreeRenameConfig {
        old_name: worktree.git_name.clone(), // Use git_name for internal operations
        new_name: new_name.clone(),
        old_path: worktree.path.clone(),
        new_path,
        old_branch: worktree.branch.clone(),
        new_branch,
        rename_branch,
    };

    // Perform the rename using business logic
    utils::print_progress(&format!("Renaming worktree to '{new_name}'..."));

    match execute_rename(&config, manager) {
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
    fn test_should_rename_branch_true() {
        let test_name = "feature";
        // User wants rename and names match
        assert!(should_rename_branch(test_name, test_name, true));
    }

    #[test]
    fn test_should_rename_branch_false_user_doesnt_want() {
        let test_name = "feature";
        // User doesn't want rename even if names match
        assert!(!should_rename_branch(test_name, test_name, false));
    }

    #[test]
    fn test_should_rename_branch_false_names_mismatch() {
        let feature_name = "feature";
        let main_name = "main";
        // Names don't match even if user wants rename
        assert!(!should_rename_branch(feature_name, main_name, true));
    }

    #[test]
    fn test_rename_config_creation() {
        let old_name = "old-name";
        let new_name = "new-name";
        let config = RenameConfig {
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
            old_path: PathBuf::from("/tmp/old"),
            new_path: PathBuf::from("/tmp/new"),
            rename_branch: true,
        };

        assert_eq!(config.old_name, old_name);
        assert_eq!(config.new_name, new_name);
        assert!(config.rename_branch);
    }

    #[test]
    fn test_worktree_rename_config_creation() {
        let old_worktree = "old-worktree";
        let new_worktree = "new-worktree";
        let old_branch = "old-branch";
        let new_branch = "new-branch";
        let config = WorktreeRenameConfig {
            old_name: old_worktree.to_string(),
            new_name: new_worktree.to_string(),
            old_path: PathBuf::from("/tmp/old"),
            new_path: PathBuf::from("/tmp/new"),
            old_branch: old_branch.to_string(),
            new_branch: Some(new_branch.to_string()),
            rename_branch: true,
        };

        assert_eq!(config.old_name, old_worktree);
        assert_eq!(config.new_name, new_worktree);
        assert_eq!(config.old_branch, old_branch);
        assert_eq!(config.new_branch, Some(new_branch.to_string()));
        assert!(config.rename_branch);
    }

    #[test]
    fn test_get_renameable_worktrees_filter_current() {
        let main_name = "main";
        let feature_name = "feature";
        let worktrees = vec![
            WorktreeInfo {
                name: main_name.to_string(),
                git_name: main_name.to_string(),
                path: PathBuf::from("/tmp/main"),
                branch: main_name.to_string(),
                is_current: true, // Current worktree - should be filtered out
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
            WorktreeInfo {
                name: feature_name.to_string(),
                git_name: feature_name.to_string(),
                path: PathBuf::from("/tmp/feature"),
                branch: feature_name.to_string(),
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
                is_locked: false,
            },
        ];

        let renameable = get_renameable_worktrees(&worktrees);
        assert_eq!(renameable.len(), 1);
        assert_eq!(renameable[0].name, feature_name);
    }

    #[test]
    fn test_analyze_rename_requirements_basic() {
        let feature_name = "feature";
        let worktree = WorktreeInfo {
            name: feature_name.to_string(),
            git_name: feature_name.to_string(),
            path: PathBuf::from("/tmp/feature"),
            branch: feature_name.to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let analysis = analyze_rename_requirements(&worktree).unwrap();

        assert_eq!(analysis.worktree.name, feature_name);
        assert!(analysis.can_rename_branch);
        assert_eq!(
            analysis.suggested_branch_name,
            Some(feature_name.to_string())
        );
    }

    #[test]
    fn test_validate_rename_operation_valid() {
        let old_name = "old-name";
        let new_name = "new-name";
        let result = validate_rename_operation(old_name, new_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rename_operation_same_name() {
        let same_name = "same-name";
        let result = validate_rename_operation(same_name, same_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rename_operation_main_worktree() {
        let main_name = "main";
        let new_name = "new-name";
        let result = validate_rename_operation(main_name, new_name);
        assert!(result.is_err());

        let old_name = "old-name";
        let result = validate_rename_operation(old_name, main_name);
        assert!(result.is_err());
    }

    // Add 5 new tests for better coverage
    #[test]
    fn test_analyze_rename_requirements_feature_branch() {
        let worktree_name = "auth";
        let feature_branch = "feature/auth";
        let worktree = WorktreeInfo {
            name: worktree_name.to_string(),
            git_name: worktree_name.to_string(),
            path: PathBuf::from("/tmp/auth"),
            branch: feature_branch.to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let analysis = analyze_rename_requirements(&worktree).unwrap();
        assert!(analysis.can_rename_branch);
        assert!(analysis.is_feature_branch);
        assert_eq!(
            analysis.suggested_branch_name,
            Some(format!("feature/{worktree_name}"))
        );
    }

    #[test]
    fn test_analyze_rename_requirements_detached_head() {
        let worktree = WorktreeInfo {
            name: "detached".to_string(),
            git_name: "detached".to_string(),
            path: PathBuf::from("/tmp/detached"),
            branch: DEFAULT_BRANCH_DETACHED.to_string(),
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
            is_locked: false,
        };

        let analysis = analyze_rename_requirements(&worktree).unwrap();
        assert!(!analysis.can_rename_branch);
        assert!(analysis.suggested_branch_name.is_none());
    }

    #[test]
    fn test_validate_rename_operation_empty_names() {
        let empty_string = "";
        let valid_name = "valid-name";

        // Empty old name
        assert!(validate_rename_operation(empty_string, valid_name).is_err());

        // Empty new name
        assert!(validate_rename_operation(valid_name, empty_string).is_err());
    }

    #[test]
    fn test_validate_rename_operation_master_worktree() {
        let master_name = "master";
        let new_name = "new-name";
        let old_name = "old-name";

        // Cannot rename master worktree
        assert!(validate_rename_operation(master_name, new_name).is_err());

        // Cannot rename to master
        assert!(validate_rename_operation(old_name, master_name).is_err());
    }

    #[test]
    fn test_get_renameable_worktrees_empty_list() {
        let worktrees: Vec<WorktreeInfo> = vec![];
        let renameable = get_renameable_worktrees(&worktrees);
        assert!(renameable.is_empty());
    }
}
