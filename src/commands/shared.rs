use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Confirm, FuzzySelect, MultiSelect};
use std::process::Command;

/// Configuration for search operations
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub query: String,
    pub show_current_indicator: bool,
}

/// Configuration for batch delete operations
#[derive(Debug, Clone)]
pub struct BatchDeleteConfig {
    pub selected_worktrees: Vec<String>,
    pub delete_orphaned_branches: bool,
}

/// Result of search analysis
#[derive(Debug, Clone)]
pub struct SearchAnalysis {
    pub items: Vec<String>,
    pub total_count: usize,
    pub has_current: bool,
}
use crate::constants::{
    section_header, CONFIG_FILE_NAME, DEFAULT_BRANCH_DETACHED, DEFAULT_EDITOR_UNIX,
    DEFAULT_EDITOR_WINDOWS, DEFAULT_WORKTREE_CLEANUP_DAYS, EMOJI_DETACHED, EMOJI_FOLDER,
    EMOJI_HOME, EMOJI_LOCKED, ENV_EDITOR, ENV_VISUAL, GIT_DIR, HEADER_SEARCH_WORKTREES,
    HOOK_POST_SWITCH, HOOK_PRE_REMOVE, MSG_ALREADY_IN_WORKTREE, MSG_NO_WORKTREES_TO_SEARCH,
    MSG_SEARCH_FUZZY_ENABLED, PROMPT_SELECT_WORKTREE_SWITCH, SEARCH_CURRENT_INDICATOR,
};
use crate::git::{GitWorktreeManager, WorktreeInfo};
use crate::hooks::{self, HookContext};
use crate::input_esc_raw::input_esc_with_default_raw as input_esc_with_default;
use crate::utils::{self, get_theme, press_any_key_to_continue, write_switch_path};

/// Pure business logic for creating search items
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
        total_count: worktrees.len(),
        has_current,
        items,
    }
}

/// Pure business logic for validating search selection
pub fn validate_search_selection(
    worktrees: &[WorktreeInfo],
    selection_index: usize,
) -> Result<&WorktreeInfo> {
    if selection_index >= worktrees.len() {
        return Err(anyhow!("Invalid selection index"));
    }

    let selected = &worktrees[selection_index];
    Ok(selected)
}

/// Pure business logic for filtering deletable worktrees for batch operations
pub fn prepare_batch_delete_items(worktrees: &[WorktreeInfo]) -> Vec<String> {
    worktrees
        .iter()
        .filter(|w| !w.is_current)
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect()
}

/// Searches and switches to worktrees using fuzzy search
///
/// Provides an interactive fuzzy search interface for finding and switching
/// to worktrees by name or branch. This is useful when you have many worktrees
/// and want to quickly navigate between them.
///
/// # Search Features
///
/// - **Fuzzy Search**: Type partial matches to filter worktrees
/// - **Current Indicator**: Current worktree is marked with indicator
/// - **Branch Display**: Shows both worktree name and branch name
/// - **Quick Navigation**: Switch directly without going through menus
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
        let msg = MSG_NO_WORKTREES_TO_SEARCH.yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    println!();
    let header = section_header(HEADER_SEARCH_WORKTREES);
    println!("{header}");
    println!();

    // Use business logic to create search items
    let analysis = create_search_items(&worktrees);

    // Use FuzzySelect for interactive search
    println!("{MSG_SEARCH_FUZZY_ENABLED}");
    let selection = match FuzzySelect::with_theme(&get_theme())
        .with_prompt(PROMPT_SELECT_WORKTREE_SWITCH)
        .items(&analysis.items)
        .interact_opt()?
    {
        Some(selection) => selection,
        None => return Ok(false),
    };

    // Use business logic to validate selection
    let selected_worktree = validate_search_selection(&worktrees, selection)?;

    if selected_worktree.is_current {
        println!();
        let msg = MSG_ALREADY_IN_WORKTREE.yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(false);
    }

    // Switch to the selected worktree
    write_switch_path(&selected_worktree.path);

    println!();
    let plus_sign = "+".green();
    let worktree_name = selected_worktree.name.bright_white().bold();
    println!("{plus_sign} Switching to worktree '{worktree_name}'");
    let path_label = "Path:".bright_black();
    let path_display = selected_worktree.path.display();
    println!("  {path_label} {path_display}");
    let branch_label = "Branch:".bright_black();
    let branch_name = selected_worktree.branch.yellow();
    println!("  {branch_label} {branch_name}");

    // Execute post-switch hooks
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
/// Uses dialoguer's MultiSelect for the selection interface and provides
/// comprehensive feedback during the deletion process. The function handles
/// errors gracefully and continues with remaining deletions even if some fail.
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

    // Filter out current worktree
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

    let items: Vec<String> = deletable_worktrees
        .iter()
        .map(|w| format!("{} ({})", w.name, w.branch))
        .collect();

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

    // Check for branches that will become orphaned
    let mut branches_to_delete = Vec::new();
    for wt in &selected_worktrees {
        if manager.is_branch_unique_to_worktree(&wt.branch, &wt.name)? {
            branches_to_delete.push((wt.branch.clone(), wt.name.clone()));
        }
    }

    // Show summary
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

    // Ask about branch deletion if there are orphaned branches
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

    // Delete worktrees
    println!();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut deleted_worktrees = Vec::new();

    for wt in &selected_worktrees {
        // Execute pre-remove hooks
        if let Err(e) = hooks::execute_hooks(
            HOOK_PRE_REMOVE,
            &HookContext {
                worktree_name: wt.name.clone(),
                worktree_path: wt.path.clone(),
            },
        ) {
            utils::print_warning(&format!("Hook execution warning: {e}"));
        }

        match manager.remove_worktree(&wt.name) {
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
        let msg = "• No worktrees to clean up.".yellow();
        println!("{msg}");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    }

    println!();
    let header = section_header("Cleanup Old Worktrees");
    println!("{header}");
    println!();

    // Get age threshold
    let _days = match input_esc_with_default(
        "Delete worktrees older than (days)",
        DEFAULT_WORKTREE_CLEANUP_DAYS,
    ) {
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

/// Edits the hooks configuration file
///
/// Opens the `.git-workers.toml` configuration file in the user's
/// preferred editor, allowing them to configure lifecycle hooks.
///
/// # Configuration File Location
///
/// The function uses the exact same configuration file discovery logic as `Config::load()`,
/// ensuring consistency across all features. The search order depends on repository type:
///
/// ## Bare Repositories
/// 1. Current directory
/// 2. Default branch subdirectories (e.g., `./main/.git-workers.toml`)
/// 3. Existing worktree pattern detection via `git worktree list`
/// 4. Common directory fallbacks (`branch/`, `worktrees/`)
/// 5. Sibling directories
///
/// ## Non-bare Repositories
/// 1. Current directory (current worktree)
/// 2. Main repository directory (where `.git` is a directory)
/// 3. `main/` or `master/` subdirectories in parent paths
///
/// This ensures hooks configuration is found in the same location as other
/// configurations, maintaining consistency across all git-workers features.
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
    println!();
    let header = section_header("Edit Hooks Configuration");
    println!("{header}");
    println!();

    // Find the config file location using the same logic as Config::load()
    let config_path = if let Ok(repo) = git2::Repository::discover(".") {
        find_config_file_path_internal(&repo)?
    } else {
        utils::print_error("Not in a git repository");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    };

    // Create the file if it doesn't exist
    if !config_path.exists() {
        let msg = "• No configuration file found.".yellow();
        println!("{msg}");
        println!();

        let create = Confirm::with_theme(&get_theme())
            .with_prompt(format!("Create {CONFIG_FILE_NAME}?"))
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
            utils::print_success(&format!("Created {CONFIG_FILE_NAME} with template"));
        } else {
            return Ok(());
        }
    }

    // Get the user's preferred editor
    let editor = std::env::var(ENV_EDITOR)
        .or_else(|_| std::env::var(ENV_VISUAL))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                DEFAULT_EDITOR_WINDOWS.to_string()
            } else {
                DEFAULT_EDITOR_UNIX.to_string()
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
            utils::print_error(&format!("Failed to open editor: {e}"));
            println!();
            println!("You can manually edit the file at:");
            let path_str = config_path.display().to_string().bright_white();
            println!("  {path_str}");
        }
    }

    println!();
    press_any_key_to_continue()?;

    Ok(())
}

/// Finds the configuration file path using GitWorktreeManager
///
/// This is a convenience wrapper around find_config_file_path_internal
/// that works with GitWorktreeManager instances.
pub fn find_config_file_path(manager: &GitWorktreeManager) -> Result<std::path::PathBuf> {
    find_config_file_path_internal(manager.repo())
}

// Helper function to find config file path with the same logic as Config::load()
pub fn find_config_file_path_internal(repo: &git2::Repository) -> Result<std::path::PathBuf> {
    if repo.is_bare() {
        // For bare repositories - use complex discovery logic
        if let Ok(cwd) = std::env::current_dir() {
            // 1. Check current directory first
            let current_config = cwd.join(CONFIG_FILE_NAME);
            if current_config.exists() {
                return Ok(current_config);
            }

            // Default: use current directory for creation
            Ok(cwd.join(CONFIG_FILE_NAME))
        } else {
            // Can't get current directory
            Err(anyhow::anyhow!("Cannot determine current directory"))
        }
    } else {
        // For non-bare repositories - same logic as Config::load_from_main_repository_only()
        if let Ok(cwd) = std::env::current_dir() {
            // 1. First check current directory
            let current_config = cwd.join(CONFIG_FILE_NAME);
            if current_config.exists() {
                return Ok(current_config);
            }

            // 2. Check if this is the main worktree
            if let Some(workdir) = repo.workdir() {
                let workdir_path = workdir.to_path_buf();

                // Check if current directory is the main worktree
                if cwd == workdir_path {
                    return Ok(workdir_path.join(CONFIG_FILE_NAME));
                }

                // If not, check if the main worktree exists
                let git_path = workdir_path.join(GIT_DIR);
                if git_path.is_dir() && workdir_path.exists() {
                    let config_path = workdir_path.join(CONFIG_FILE_NAME);
                    if config_path.exists() {
                        return Ok(config_path);
                    }
                }
            }

            // Default: use current directory for creation
            Ok(cwd.join(CONFIG_FILE_NAME))
        } else {
            // Final fallback: use repository working directory
            repo.workdir()
                .map(|p| p.join(CONFIG_FILE_NAME))
                .ok_or_else(|| anyhow::anyhow!("No working directory found"))
        }
    }
}

/// Returns an appropriate icon for a worktree based on its status
///
/// This function provides visual indicators for different worktree states
/// using emoji icons. The returned icon helps users quickly identify the
/// state of each worktree in listings and menus.
///
/// # Arguments
///
/// * `worktree` - The worktree information containing status flags
///
/// # Returns
///
/// Returns a static string containing the appropriate emoji:
/// - `🏠` for current worktree
/// - `🔒` for locked worktree
/// - `🔗` for detached HEAD worktree
/// - `📁` for regular worktree
#[allow(dead_code)]
pub fn get_worktree_icon(worktree: &WorktreeInfo) -> &'static str {
    if worktree.is_current {
        EMOJI_HOME
    } else if worktree.is_locked {
        EMOJI_LOCKED
    } else if worktree.branch == DEFAULT_BRANCH_DETACHED {
        EMOJI_DETACHED
    } else {
        EMOJI_FOLDER
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::git::GitWorktreeManager;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper to create a test repository
    #[allow(dead_code)]
    fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
        let temp_dir = TempDir::new()?;

        // Initialize repository
        Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .output()?;

        // Configure git
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()?;

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()?;

        // Create initial commit
        fs::write(temp_dir.path().join("README.md"), "# Test")?;
        Command::new("git")
            .arg("add")
            .arg("README.md")
            .current_dir(temp_dir.path())
            .output()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(temp_dir.path())
            .output()?;

        let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
        Ok((temp_dir, manager))
    }

    #[test]
    fn test_create_search_items() -> Result<()> {
        let worktree_info = WorktreeInfo {
            name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: true,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        };
        let worktrees = vec![worktree_info];

        let analysis = create_search_items(&worktrees);

        assert_eq!(analysis.total_count, 1);
        assert!(analysis.has_current);
        assert_eq!(analysis.items.len(), 1);
        assert!(analysis.items[0].contains("feature-branch"));
        assert!(analysis.items[0].contains("feature/test"));
        assert!(analysis.items[0].contains(SEARCH_CURRENT_INDICATOR));

        Ok(())
    }

    #[test]
    fn test_validate_search_selection() -> Result<()> {
        let worktree_info = WorktreeInfo {
            name: "feature-branch".to_string(),
            path: std::path::PathBuf::from("/test/feature-branch"),
            branch: "feature/test".to_string(),
            is_current: false,
            is_locked: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        };
        let worktrees = vec![worktree_info];

        // Valid selection
        let selected = validate_search_selection(&worktrees, 0)?;
        assert_eq!(selected.name, "feature-branch");

        // Invalid selection
        let result = validate_search_selection(&worktrees, 1);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_prepare_batch_delete_items() -> Result<()> {
        let worktrees = vec![
            WorktreeInfo {
                name: "main".to_string(),
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

        // Should only include non-current worktrees
        assert_eq!(items.len(), 1);
        assert!(items[0].contains("feature-branch"));
        assert!(items[0].contains("feature/test"));
        assert!(!items[0].contains("main"));

        Ok(())
    }
}
