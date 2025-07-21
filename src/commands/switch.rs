use anyhow::{anyhow, Result};
use colored::*;

use crate::constants::{
    section_header, DEFAULT_MENU_SELECTION, HOOK_POST_SWITCH, MSG_ALREADY_IN_WORKTREE,
};
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::hooks::{self, HookContext};
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::{self, press_any_key_to_continue, write_switch_path};

/// Validate switch target
#[allow(dead_code)]
pub fn validate_switch_target(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Switch target cannot be empty"));
    }

    if name.contains(' ') || name.contains('\t') || name.contains('\n') {
        return Err(anyhow!("Invalid worktree name: contains whitespace"));
    }

    Ok(())
}

/// Check if already in the target worktree
#[allow(dead_code)]
pub fn is_already_in_worktree(current: &Option<String>, target: &str) -> bool {
    match current {
        Some(current_name) => current_name == target,
        None => false,
    }
}

/// Configuration for worktree switching (simplified for tests)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SwitchConfig {
    pub target_name: String,
    pub target_path: std::path::PathBuf,
    pub source_name: Option<String>,
    pub save_changes: bool,
}

/// Configuration for worktree switching
#[derive(Debug, Clone)]
pub struct WorktreeSwitchConfig {
    pub target_name: String,
    pub target_path: std::path::PathBuf,
    pub target_branch: String,
}

/// Result of switch analysis
#[derive(Debug, Clone)]
pub struct SwitchAnalysis {
    pub worktrees: Vec<WorktreeInfo>,
    pub current_worktree_index: Option<usize>,
    pub is_already_current: bool,
}

/// Pure business logic for sorting worktrees for display
pub fn sort_worktrees_for_display(mut worktrees: Vec<WorktreeInfo>) -> Vec<WorktreeInfo> {
    worktrees.sort_by(|a, b| {
        if a.is_current && !b.is_current {
            std::cmp::Ordering::Less
        } else if !a.is_current && b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });
    worktrees
}

/// Pure business logic for analyzing switch target
pub fn analyze_switch_target(
    worktrees: &[WorktreeInfo],
    selected_index: usize,
) -> Result<SwitchAnalysis> {
    if selected_index >= worktrees.len() {
        return Err(anyhow!("Invalid selection index"));
    }

    let current_index = worktrees.iter().position(|w| w.is_current);
    let selected_worktree = &worktrees[selected_index];

    Ok(SwitchAnalysis {
        worktrees: worktrees.to_vec(),
        current_worktree_index: current_index,
        is_already_current: selected_worktree.is_current,
    })
}

/// Pure business logic for executing switch operation
pub fn execute_switch(config: &WorktreeSwitchConfig) -> Result<()> {
    // Write switch path for shell integration
    write_switch_path(&config.target_path);

    // Execute post-switch hooks
    if let Err(e) = hooks::execute_hooks(
        HOOK_POST_SWITCH,
        &HookContext {
            worktree_name: config.target_name.clone(),
            worktree_path: config.target_path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {e}"));
    }

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
    let ui = DialoguerUI;
    switch_worktree_with_ui(&manager, &ui)
}

/// Internal implementation of switch_worktree with dependency injection
///
/// # Arguments
///
/// * `manager` - Git worktree manager instance
/// * `ui` - User interface implementation for testability
///
/// # Returns
///
/// Returns `true` if a switch occurred, `false` if cancelled or already in selected worktree
pub fn switch_worktree_with_ui(
    manager: &GitWorktreeManager,
    ui: &dyn UserInterface,
) -> Result<bool> {
    let worktrees = manager.list_worktrees()?;

    if worktrees.is_empty() {
        println!();
        let msg = "• No worktrees available.".yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    println!();
    let header = section_header("Switch Worktree");
    println!("{header}");
    println!();

    // Use business logic to sort worktrees for display
    let sorted_worktrees = sort_worktrees_for_display(worktrees);

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

    let selection = match ui.select_with_default(
        "Select a worktree to switch to (ESC to cancel)",
        &items,
        DEFAULT_MENU_SELECTION,
    ) {
        Ok(selection) => selection,
        Err(_) => return Ok(false),
    };

    // Use business logic to analyze the switch target
    let analysis = analyze_switch_target(&sorted_worktrees, selection)?;

    if analysis.is_already_current {
        println!();
        let msg = MSG_ALREADY_IN_WORKTREE.yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    let selected_worktree = &sorted_worktrees[selection];

    // Create switch configuration
    let config = WorktreeSwitchConfig {
        target_name: selected_worktree.name.clone(),
        target_path: selected_worktree.path.clone(),
        target_branch: selected_worktree.branch.clone(),
    };

    println!();
    let plus_sign = "+".green();
    let worktree_name = config.target_name.bright_white().bold();
    println!("{plus_sign} Switching to worktree '{worktree_name}'");
    let path_label = "Path:".bright_black();
    let path_display = config.target_path.display();
    println!("  {path_label} {path_display}");
    let branch_label = "Branch:".bright_black();
    let branch_name = config.target_branch.yellow();
    println!("  {branch_label} {branch_name}");

    // Execute switch using business logic
    execute_switch(&config)?;

    Ok(true)
}

#[cfg(test)] // Re-enabled tests with corrected WorktreeInfo fields
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_switch_target_valid() {
        assert!(validate_switch_target("feature-branch").is_ok());
        assert!(validate_switch_target("main").is_ok());
        assert!(validate_switch_target("valid_name").is_ok());
        assert!(validate_switch_target("123").is_ok());
    }

    #[test]
    fn test_validate_switch_target_invalid() {
        assert!(validate_switch_target("").is_err());
        assert!(validate_switch_target("name with space").is_err());
        assert!(validate_switch_target("name\twith\ttab").is_err());
        assert!(validate_switch_target("name\nwith\nnewline").is_err());
    }

    #[test]
    fn test_is_already_in_worktree_true() {
        let current = Some("feature".to_string());
        assert!(is_already_in_worktree(&current, "feature"));
    }

    #[test]
    fn test_is_already_in_worktree_false_different_name() {
        let current = Some("main".to_string());
        assert!(!is_already_in_worktree(&current, "feature"));
    }

    #[test]
    fn test_is_already_in_worktree_false_no_current() {
        let current = None;
        assert!(!is_already_in_worktree(&current, "feature"));
    }

    #[test]
    fn test_switch_config_creation() {
        let config = SwitchConfig {
            target_name: "feature".to_string(),
            target_path: PathBuf::from("/tmp/feature"),
            source_name: Some("main".to_string()),
            save_changes: true,
        };

        assert_eq!(config.target_name, "feature");
        assert_eq!(config.source_name, Some("main".to_string()));
        assert!(config.save_changes);
    }

    #[test]
    fn test_worktree_switch_config_creation() {
        let config = WorktreeSwitchConfig {
            target_name: "feature".to_string(),
            target_path: PathBuf::from("/tmp/feature"),
            target_branch: "feature-branch".to_string(),
        };

        assert_eq!(config.target_name, "feature");
        assert_eq!(config.target_branch, "feature-branch");
        assert_eq!(config.target_path, PathBuf::from("/tmp/feature"));
    }

    #[test]
    fn test_sort_worktrees_for_display() {
        let worktrees = vec![
            WorktreeInfo {
                name: "zzz-last".to_string(),
                path: PathBuf::from("/tmp/zzz"),
                branch: "zzz-branch".to_string(),
                is_locked: false,
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
            WorktreeInfo {
                name: "aaa-first".to_string(),
                path: PathBuf::from("/tmp/aaa"),
                branch: "aaa-branch".to_string(),
                is_locked: false,
                is_current: true,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            },
        ];

        let sorted = sort_worktrees_for_display(worktrees);
        assert_eq!(sorted[0].name, "aaa-first"); // Current worktree should be first
        assert_eq!(sorted[1].name, "zzz-last");
        assert!(sorted[0].is_current);
        assert!(!sorted[1].is_current);
    }

    #[test]
    fn test_analyze_switch_target_basic() {
        let worktrees = vec![WorktreeInfo {
            name: "main".to_string(),
            path: PathBuf::from("/tmp/main"),
            branch: "main".to_string(),
            is_locked: false,
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        }];

        let analysis = analyze_switch_target(&worktrees, 0).unwrap();
        assert_eq!(analysis.worktrees[0].name, "main");
        assert!(!analysis.is_already_current);
        assert_eq!(analysis.current_worktree_index, None);
    }

    #[test]
    fn test_analyze_switch_target_invalid_index() {
        let worktrees = vec![];
        let result = analyze_switch_target(&worktrees, 0);
        assert!(result.is_err());
    }
}
