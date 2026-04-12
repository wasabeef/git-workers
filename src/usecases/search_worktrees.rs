use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::FuzzySelect;

use crate::adapters::git::GitWorktreeManager;
use crate::adapters::hooks::{self, HookContext};
use crate::constants::{
    section_header, HEADER_SEARCH_WORKTREES, HOOK_POST_SWITCH, MSG_ALREADY_IN_WORKTREE,
    MSG_NO_WORKTREES_TO_SEARCH, MSG_SEARCH_FUZZY_ENABLED, PROMPT_SELECT_WORKTREE_SWITCH,
    SEARCH_CURRENT_INDICATOR,
};
use crate::domain::worktree::{BasicWorktreeInfo, WorktreeInfo};
use crate::utils::{self, get_theme, press_any_key_to_continue};

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub query: String,
    pub show_current_indicator: bool,
}

#[derive(Debug, Clone)]
pub struct SearchAnalysis {
    pub items: Vec<String>,
    pub total_count: usize,
    pub has_current: bool,
}

fn lightweight_worktrees_to_display(worktrees: Vec<BasicWorktreeInfo>) -> Vec<WorktreeInfo> {
    worktrees
        .into_iter()
        .map(|worktree| WorktreeInfo {
            name: worktree.name,
            git_name: worktree.git_name,
            path: worktree.path,
            branch: worktree.branch,
            is_locked: worktree.is_locked,
            is_current: worktree.is_current,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        })
        .collect()
}

pub fn create_search_items(worktrees: &[WorktreeInfo]) -> SearchAnalysis {
    let items: Vec<String> = worktrees
        .iter()
        .map(|wt| {
            let mut item = format!("{} ({})", wt.name, wt.branch);
            if wt.is_current {
                item.push_str(SEARCH_CURRENT_INDICATOR);
            }
            item
        })
        .collect();

    let has_current = worktrees.iter().any(|w| w.is_current);

    SearchAnalysis {
        items,
        total_count: worktrees.len(),
        has_current,
    }
}

pub fn validate_search_selection(
    worktrees: &[WorktreeInfo],
    selection_index: usize,
) -> Result<&WorktreeInfo> {
    if selection_index >= worktrees.len() {
        return Err(anyhow!("Invalid selection index"));
    }

    Ok(&worktrees[selection_index])
}

pub fn search_worktrees() -> Result<bool> {
    let manager = GitWorktreeManager::new()?;
    search_worktrees_internal(&manager)
}

fn search_worktrees_internal(manager: &GitWorktreeManager) -> Result<bool> {
    let worktrees = lightweight_worktrees_to_display(manager.list_worktrees_basic()?);

    if worktrees.is_empty() {
        println!();
        println!("{}", MSG_NO_WORKTREES_TO_SEARCH.yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    println!();
    println!("{}", section_header(HEADER_SEARCH_WORKTREES));
    println!();

    let analysis = create_search_items(&worktrees);

    println!("{MSG_SEARCH_FUZZY_ENABLED}");
    let selection = match FuzzySelect::with_theme(&get_theme())
        .with_prompt(PROMPT_SELECT_WORKTREE_SWITCH)
        .items(&analysis.items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(false),
    };

    let selected_worktree = validate_search_selection(&worktrees, selection)?;

    if selected_worktree.is_current {
        println!();
        println!("{}", MSG_ALREADY_IN_WORKTREE.yellow());
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    crate::adapters::shell::switch_file::write_switch_path(&selected_worktree.path)?;

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

    if let Err(e) = hooks::execute_hooks(
        HOOK_POST_SWITCH,
        &HookContext {
            worktree_name: selected_worktree.name.clone(),
            worktree_path: selected_worktree.path.clone(),
        },
    ) {
        utils::print_warning(&format!("Hook execution warning: {e}"));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_search_items() {
        let worktrees = vec![WorktreeInfo {
            name: "feature-branch".to_string(),
            git_name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: true,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        }];

        let analysis = create_search_items(&worktrees);

        assert_eq!(analysis.total_count, 1);
        assert!(analysis.has_current);
        assert_eq!(analysis.items.len(), 1);
        assert!(analysis.items[0].contains("feature-branch"));
        assert!(analysis.items[0].contains("feature/test"));
        assert!(analysis.items[0].contains(SEARCH_CURRENT_INDICATOR));
    }

    #[test]
    fn test_validate_search_selection() -> Result<()> {
        let worktrees = vec![WorktreeInfo {
            name: "feature-branch".to_string(),
            git_name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: false,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        }];

        let selected = validate_search_selection(&worktrees, 0)?;
        assert_eq!(selected.name, "feature-branch");
        assert!(validate_search_selection(&worktrees, 1).is_err());

        Ok(())
    }
}
