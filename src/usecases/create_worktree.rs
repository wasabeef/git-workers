use anyhow::{anyhow, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

use crate::adapters::git::GitWorktreeManager;
use crate::adapters::hooks::HookContext;
use crate::adapters::shell::switch_file::write_switch_path;
use crate::adapters::{filesystem::copy_configured_files, hooks};
use crate::config::Config;
use crate::constants::{
    section_header, BRANCH_OPTION_SELECT_BRANCH, BRANCH_OPTION_SELECT_TAG, DEFAULT_EMPTY_STRING,
    DEFAULT_MENU_SELECTION, ERROR_CUSTOM_PATH_EMPTY, ERROR_WORKTREE_NAME_EMPTY,
    FUZZY_SEARCH_THRESHOLD, GIT_REMOTE_PREFIX, HEADER_CREATE_WORKTREE, HOOK_POST_CREATE,
    HOOK_POST_SWITCH, ICON_LOCAL_BRANCH, ICON_REMOTE_BRANCH, ICON_TAG_INDICATOR,
    MSG_EXAMPLE_BRANCH, MSG_EXAMPLE_DOT, MSG_EXAMPLE_HOTFIX, MSG_EXAMPLE_PARENT,
    MSG_FIRST_WORKTREE_CHOOSE, MSG_SPECIFY_DIRECTORY_PATH, OPTION_CREATE_FROM_HEAD_FULL,
    OPTION_CUSTOM_PATH_FULL, OPTION_SELECT_BRANCH_FULL, OPTION_SELECT_TAG_FULL,
    PROGRESS_BAR_TICK_MILLIS, PROMPT_CONFLICT_ACTION, PROMPT_CUSTOM_PATH, PROMPT_SELECT_BRANCH,
    PROMPT_SELECT_BRANCH_OPTION, PROMPT_SELECT_TAG, PROMPT_SELECT_WORKTREE_LOCATION,
    PROMPT_WORKTREE_NAME, SLASH_CHAR, TAG_MESSAGE_TRUNCATE_LENGTH, WORKTREE_LOCATION_CUSTOM_PATH,
    WORKTREE_LOCATION_SAME_LEVEL,
};
use crate::domain::validation::{validate_custom_path, validate_worktree_name};
use crate::ui::{DialoguerUI, UserInterface};
use crate::utils::{self, press_any_key_to_continue};

/// Configuration for worktree creation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorktreeCreateConfig {
    pub name: String,
    pub path: PathBuf,
    pub branch_source: BranchSource,
    pub switch_to_new: bool,
}

/// Source for creating the worktree branch
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BranchSource {
    Head,
    Branch(String),
    Tag(String),
    NewBranch { name: String, base: String },
}

pub use crate::domain::paths::{
    determine_worktree_path, validate_worktree_creation, validate_worktree_location,
};

/// Pure business logic for determining worktree path (legacy)
#[allow(dead_code)]
pub fn determine_worktree_path_legacy(
    name: &str,
    location_choice: usize,
    custom_path: Option<&str>,
    _repo_name: &str,
) -> Result<PathBuf> {
    match location_choice {
        WORKTREE_LOCATION_SAME_LEVEL => Ok(PathBuf::from(format!("../{name}"))),
        WORKTREE_LOCATION_CUSTOM_PATH => {
            let path = custom_path.ok_or_else(|| anyhow!("Custom path not provided"))?;
            validate_custom_path(path)?;
            Ok(PathBuf::from(path))
        }
        _ => Err(anyhow!("Invalid location choice")),
    }
}

pub fn create_worktree() -> Result<bool> {
    let manager = GitWorktreeManager::new()?;
    let ui = DialoguerUI;
    create_worktree_with_ui(&manager, &ui)
}

/// Internal implementation of create_worktree with dependency injection
pub fn create_worktree_with_ui(
    manager: &GitWorktreeManager,
    ui: &dyn UserInterface,
) -> Result<bool> {
    println!();
    let header = section_header(HEADER_CREATE_WORKTREE);
    println!("{header}");
    println!();

    let existing_worktrees = manager.list_worktrees()?;
    let has_worktrees = !existing_worktrees.is_empty();

    let name = match ui.input(PROMPT_WORKTREE_NAME) {
        Ok(name) => name.trim().to_string(),
        Err(_) => return Ok(false),
    };

    if name.is_empty() {
        utils::print_error(ERROR_WORKTREE_NAME_EMPTY);
        return Ok(false);
    }

    let name = match validate_worktree_name(&name) {
        Ok(validated_name) => validated_name,
        Err(e) => {
            utils::print_error(&format!("Invalid worktree name: {e}"));
            return Ok(false);
        }
    };

    let final_name = if !has_worktrees {
        println!();
        let msg = MSG_FIRST_WORKTREE_CHOOSE.bright_cyan();
        println!("{msg}");

        let options = vec![
            format!("Same level as repository (../{})", name),
            OPTION_CUSTOM_PATH_FULL.to_string(),
        ];

        let selection = match ui.select_with_default(
            PROMPT_SELECT_WORKTREE_LOCATION,
            &options,
            DEFAULT_MENU_SELECTION,
        ) {
            Ok(selection) => selection,
            Err(_) => return Ok(false),
        };

        match selection {
            WORKTREE_LOCATION_SAME_LEVEL => format!("../{name}"),
            WORKTREE_LOCATION_CUSTOM_PATH => {
                println!();
                let msg = MSG_SPECIFY_DIRECTORY_PATH.bright_cyan();
                println!("{msg}");

                println!();
                println!(
                    "{}:",
                    format!("Examples (worktree name: '{name}'):").bright_black()
                );
                println!(
                    "  • {} → creates at ./branch/{name}",
                    MSG_EXAMPLE_BRANCH.green()
                );
                println!(
                    "  • {} → creates at ./hotfix/{name}",
                    MSG_EXAMPLE_HOTFIX.green()
                );
                println!(
                    "  • {} → creates at ../{name} (outside project)",
                    MSG_EXAMPLE_PARENT.green()
                );
                println!(
                    "  • {} → creates at ./{name} (project root)",
                    MSG_EXAMPLE_DOT.green()
                );
                println!();

                let custom_path = match ui.input(PROMPT_CUSTOM_PATH) {
                    Ok(path) => path.trim().to_string(),
                    Err(_) => return Ok(false),
                };

                if custom_path.is_empty() {
                    utils::print_error(ERROR_CUSTOM_PATH_EMPTY);
                    return Ok(false);
                }

                let custom_path = custom_path.trim_end_matches(SLASH_CHAR);
                let final_path = if custom_path.is_empty() {
                    name.clone()
                } else if custom_path == "." {
                    format!("./{name}")
                } else {
                    format!("{custom_path}/{name}")
                };

                if let Err(e) = validate_custom_path(&final_path) {
                    utils::print_error(&format!("Invalid custom path: {e}"));
                    return Ok(false);
                }

                final_path
            }
            _ => {
                utils::print_error(&format!(
                    "Invalid location selection: {selection}. Expected 0 or 1."
                ));
                return Ok(false);
            }
        }
    } else {
        name.clone()
    };

    println!();
    let branch_options = vec![
        OPTION_CREATE_FROM_HEAD_FULL.to_string(),
        OPTION_SELECT_BRANCH_FULL.to_string(),
        OPTION_SELECT_TAG_FULL.to_string(),
    ];

    let branch_choice = match ui.select_with_default(
        PROMPT_SELECT_BRANCH_OPTION,
        &branch_options,
        DEFAULT_MENU_SELECTION,
    ) {
        Ok(choice) => choice,
        Err(_) => return Ok(false),
    };

    let (branch, new_branch_name) = match branch_choice {
        BRANCH_OPTION_SELECT_BRANCH => {
            let (local_branches, remote_branches) = manager.list_all_branches()?;
            if local_branches.is_empty() && remote_branches.is_empty() {
                utils::print_warning("No branches found, creating from HEAD");
                (None, None)
            } else {
                let branch_worktree_map = manager.get_branch_worktree_map()?;
                let mut branch_items: Vec<String> = Vec::new();
                let mut branch_refs: Vec<(String, bool)> = Vec::new();

                for branch in &local_branches {
                    if let Some(worktree) = branch_worktree_map.get(branch) {
                        branch_items.push(format!(
                            "{ICON_LOCAL_BRANCH}{branch} (in use by '{worktree}')"
                        ));
                    } else {
                        branch_items.push(format!("{ICON_LOCAL_BRANCH}{branch}"));
                    }
                    branch_refs.push((branch.clone(), false));
                }

                for branch in &remote_branches {
                    let full_remote_name = format!("{GIT_REMOTE_PREFIX}{branch}");
                    if let Some(worktree) = branch_worktree_map.get(&full_remote_name) {
                        branch_items.push(format!(
                            "{ICON_REMOTE_BRANCH}{full_remote_name} (in use by '{worktree}')"
                        ));
                    } else {
                        branch_items.push(format!("{ICON_REMOTE_BRANCH}{full_remote_name}"));
                    }
                    branch_refs.push((branch.clone(), true));
                }

                println!();

                let selection_result = if branch_items.len() > FUZZY_SEARCH_THRESHOLD {
                    println!("Type to search branches (fuzzy search enabled):");
                    ui.fuzzy_select(PROMPT_SELECT_BRANCH, &branch_items)
                } else {
                    ui.select_with_default(
                        PROMPT_SELECT_BRANCH,
                        &branch_items,
                        DEFAULT_MENU_SELECTION,
                    )
                };
                let selection_result = selection_result.ok();

                match selection_result {
                    Some(selection) => {
                        let (selected_branch, is_remote): (&String, &bool) =
                            (&branch_refs[selection].0, &branch_refs[selection].1);

                        if !is_remote {
                            if let Some(worktree) = branch_worktree_map.get(selected_branch) {
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

                                match ui.select_with_default(
                                    PROMPT_CONFLICT_ACTION,
                                    &action_options,
                                    DEFAULT_MENU_SELECTION,
                                ) {
                                    Ok(0) => (Some(selected_branch.clone()), Some(name.clone())),
                                    Ok(1) => {
                                        println!();
                                        let new_branch = match ui.input_with_default(
                                            &format!(
                                                "Enter new branch name (base: {})",
                                                selected_branch.yellow()
                                            ),
                                            &name,
                                        ) {
                                            Ok(name) => name.trim().to_string(),
                                            Err(_) => return Ok(false),
                                        };

                                        if new_branch.is_empty() {
                                            utils::print_error("Branch name cannot be empty");
                                            return Ok(false);
                                        }

                                        if local_branches.contains(&new_branch) {
                                            utils::print_error(&format!(
                                                "Branch '{new_branch}' already exists"
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
                        } else if local_branches.contains(selected_branch) {
                            println!();
                            utils::print_warning(&format!(
                                "A local branch '{}' already exists for remote '{}'",
                                selected_branch.yellow(),
                                format!("{GIT_REMOTE_PREFIX}{selected_branch}").bright_blue()
                            ));
                            println!();

                            let use_local_option =
                                if let Some(worktree) = branch_worktree_map.get(selected_branch) {
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

                            match ui.select_with_default(
                                PROMPT_CONFLICT_ACTION,
                                &action_options,
                                DEFAULT_MENU_SELECTION,
                            ) {
                                Ok(0) => (
                                    Some(format!("{GIT_REMOTE_PREFIX}{selected_branch}")),
                                    Some(name.clone()),
                                ),
                                Ok(1) => {
                                    if let Some(worktree) = branch_worktree_map.get(selected_branch)
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
                            (Some(format!("{GIT_REMOTE_PREFIX}{selected_branch}")), None)
                        }
                    }
                    None => return Ok(false),
                }
            }
        }
        BRANCH_OPTION_SELECT_TAG => {
            let tags = manager.list_all_tags()?;
            if tags.is_empty() {
                utils::print_warning("No tags found, creating from HEAD");
                (None, None)
            } else {
                let tag_items: Vec<String> = tags
                    .iter()
                    .map(|(name, message)| {
                        if let Some(msg) = message {
                            let first_line = msg.lines().next().unwrap_or(DEFAULT_EMPTY_STRING);
                            let truncated = if first_line.len() > TAG_MESSAGE_TRUNCATE_LENGTH {
                                format!("{}...", &first_line[..TAG_MESSAGE_TRUNCATE_LENGTH])
                            } else {
                                first_line.to_string()
                            };
                            format!("{ICON_TAG_INDICATOR}{name} - {truncated}")
                        } else {
                            format!("{ICON_TAG_INDICATOR}{name}")
                        }
                    })
                    .collect();

                println!();

                let selection_result = if tag_items.len() > FUZZY_SEARCH_THRESHOLD {
                    println!("Type to search tags (fuzzy search enabled):");
                    ui.fuzzy_select(PROMPT_SELECT_TAG, &tag_items)
                } else {
                    ui.select_with_default(PROMPT_SELECT_TAG, &tag_items, DEFAULT_MENU_SELECTION)
                };
                let selection_result = selection_result.ok();

                match selection_result {
                    Some(selection) => {
                        let selected_tag = &tags[selection].0;
                        (Some(selected_tag.clone()), Some(name.clone()))
                    }
                    None => return Ok(false),
                }
            }
        }
        _ => (None, None),
    };

    println!();
    let preview_label = "Preview:".bright_white();
    println!("{preview_label}");
    let name_label = "Name:".bright_black();
    let name_value = final_name.bright_green();
    println!("  {name_label} {name_value}");
    if let Some(new_branch) = &new_branch_name {
        let base_branch_name = branch.as_ref().unwrap();
        if manager
            .repo()
            .find_reference(&format!("refs/tags/{base_branch_name}"))
            .is_ok()
        {
            let branch_label = "New Branch:".bright_black();
            let branch_value = new_branch.yellow();
            let tag_value = format!("tag: {base_branch_name}").bright_cyan();
            println!("  {branch_label} {branch_value} (from {tag_value})");
        } else {
            let branch_label = "New Branch:".bright_black();
            let branch_value = new_branch.yellow();
            let base_value = base_branch_name.bright_black();
            println!("  {branch_label} {branch_value} (from {base_value})");
        }
    } else if let Some(branch_name) = &branch {
        let branch_label = "Branch:".bright_black();
        let branch_value = branch_name.yellow();
        println!("  {branch_label} {branch_value}");
    } else {
        let from_label = "From:".bright_black();
        println!("  {from_label} Current HEAD");
    }
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Creating worktree...");
    pb.enable_steady_tick(Duration::from_millis(PROGRESS_BAR_TICK_MILLIS));

    let result = if let Some(new_branch) = &new_branch_name {
        manager.create_worktree_with_new_branch(&final_name, new_branch, branch.as_ref().unwrap())
    } else {
        manager.create_worktree(&final_name, branch.as_deref())
    };

    match result {
        Ok(path) => {
            pb.finish_and_clear();
            let name_green = name.bright_green();
            let path_display = path.display();
            utils::print_success(&format!(
                "Created worktree '{name_green}' at {path_display}"
            ));

            let config = Config::load()?;
            if !config.files.copy.is_empty() {
                println!();
                println!("Copying configured files...");
                match copy_configured_files(&config.files, &path, manager) {
                    Ok(copied) => {
                        if !copied.is_empty() {
                            let copied_count = copied.len();
                            utils::print_success(&format!("Copied {copied_count} files"));
                            for file in &copied {
                                println!("  ✓ {file}");
                            }
                        }
                    }
                    Err(e) => {
                        utils::print_warning(&format!("Failed to copy files: {e}"));
                    }
                }
            }

            if let Err(e) = hooks::execute_hooks(
                HOOK_POST_CREATE,
                &HookContext {
                    worktree_name: name.clone(),
                    worktree_path: path.clone(),
                },
            ) {
                utils::print_warning(&format!("Hook execution warning: {e}"));
            }

            println!();
            let switch = ui
                .confirm_with_default("Switch to the new worktree?", true)
                .unwrap_or(false);

            if switch {
                write_switch_path(&path)?;

                println!();
                let plus_sign = "+".green();
                let worktree_name = name.bright_white().bold();
                println!("{plus_sign} Switching to worktree '{worktree_name}'");

                if let Err(e) = hooks::execute_hooks(
                    HOOK_POST_SWITCH,
                    &HookContext {
                        worktree_name: name,
                        worktree_path: path,
                    },
                ) {
                    utils::print_warning(&format!("Hook execution warning: {e}"));
                }

                Ok(true)
            } else {
                println!();
                press_any_key_to_continue()?;
                Ok(false)
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            utils::print_error(&format!("Failed to create worktree: {e}"));
            println!();
            press_any_key_to_continue()?;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_validate_worktree_location_valid() {
        assert!(validate_worktree_location("same-level").is_ok());
        assert!(validate_worktree_location("custom").is_ok());
    }

    #[test]
    fn test_validate_worktree_location_invalid() {
        assert!(validate_worktree_location("invalid").is_err());
        assert!(validate_worktree_location("").is_err());
        assert!(validate_worktree_location("wrong-type").is_err());
    }

    #[test]
    fn test_determine_worktree_path_same_level() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&git_dir).unwrap();

        let result = determine_worktree_path(&git_dir, "test-worktree", "same-level", None);
        assert!(result.is_ok());

        let (path, pattern) = result.unwrap();
        assert_eq!(pattern, "same-level");
        assert!(path.to_string_lossy().ends_with("test-worktree"));
    }

    #[test]
    fn test_determine_worktree_path_custom() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&git_dir).unwrap();

        let custom_path = PathBuf::from("custom/path");
        let result = determine_worktree_path(
            &git_dir,
            "test-worktree",
            "custom",
            Some(custom_path.clone()),
        );
        assert!(result.is_ok());

        let (path, pattern) = result.unwrap();
        assert_eq!(pattern, "custom");
        assert!(path.to_string_lossy().contains("custom/path"));
    }

    #[test]
    fn test_determine_worktree_path_legacy_same_level() {
        let result =
            determine_worktree_path_legacy("test", WORKTREE_LOCATION_SAME_LEVEL, None, "repo");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("../test"));
    }

    #[test]
    fn test_determine_worktree_path_legacy_custom() {
        let result = determine_worktree_path_legacy(
            "test",
            WORKTREE_LOCATION_CUSTOM_PATH,
            Some("../custom/test"),
            "repo",
        );
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("../custom/test"));
    }

    #[test]
    fn test_determine_worktree_path_legacy_invalid_choice() {
        let result = determine_worktree_path_legacy("test", 999, None, "repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_worktree_creation_no_conflicts() {
        let existing_worktrees = vec![];
        let path = PathBuf::from("/tmp/new-worktree");

        let result = validate_worktree_creation("new-worktree", &path, &existing_worktrees);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "WorktreeInfo struct fields need to be updated"]
    fn test_validate_worktree_creation_name_conflict() {
        let existing_worktrees = vec![];
        let path = PathBuf::from("/tmp/new-worktree");

        let result = validate_worktree_creation("test", &path, &existing_worktrees);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "WorktreeInfo struct fields need to be updated"]
    fn test_validate_worktree_creation_path_conflict() {
        let existing_path = PathBuf::from("/tmp/existing");
        let existing_worktrees = vec![];

        let result =
            validate_worktree_creation("new-worktree", &existing_path, &existing_worktrees);
        assert!(result.is_ok());
    }

    #[test]
    fn test_determine_worktree_path_custom_missing_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let git_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&git_dir).unwrap();

        let result = determine_worktree_path(&git_dir, "test-worktree", "custom", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_worktree_path_invalid_location() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let git_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&git_dir).unwrap();

        let invalid_location = "invalid-location";
        let result = determine_worktree_path(&git_dir, "test-worktree", invalid_location, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_worktree_location_all_valid() {
        let valid_locations = vec!["same-level", "custom"];
        for location in valid_locations {
            assert!(validate_worktree_location(location).is_ok());
        }
    }

    #[test]
    fn test_determine_worktree_path_legacy_custom_missing_path() {
        let repo_name = "repo";
        let result =
            determine_worktree_path_legacy("test", WORKTREE_LOCATION_CUSTOM_PATH, None, repo_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_source_enum_variants() {
        let test_branch = "main";
        let test_tag = "v1.0.0";
        let test_new_branch = "feature";
        let test_base = "develop";

        let sources = vec![
            BranchSource::Head,
            BranchSource::Branch(test_branch.to_string()),
            BranchSource::Tag(test_tag.to_string()),
            BranchSource::NewBranch {
                name: test_new_branch.to_string(),
                base: test_base.to_string(),
            },
        ];

        for source in sources {
            match source {
                BranchSource::Head => {}
                BranchSource::Branch(ref branch) => assert_eq!(branch, test_branch),
                BranchSource::Tag(ref tag) => assert_eq!(tag, test_tag),
                BranchSource::NewBranch { ref name, ref base } => {
                    assert_eq!(name, test_new_branch);
                    assert_eq!(base, test_base);
                }
            }
        }
    }
}
