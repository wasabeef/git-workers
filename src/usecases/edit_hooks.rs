use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use std::process::Command;

use crate::adapters::config::loader::find_config_file_path_internal;
use crate::adapters::shell::editor::preferred_editor;
use crate::constants::{section_header, CONFIG_FILE_NAME};
use crate::utils::{self, get_theme, press_any_key_to_continue};

pub fn edit_hooks() -> Result<()> {
    println!();
    println!("{}", section_header("Edit Hooks Configuration"));
    println!();

    let config_path = if let Ok(repo) = git2::Repository::discover(".") {
        find_config_file_path_internal(&repo)?
    } else {
        utils::print_error("Not in a git repository");
        println!();
        press_any_key_to_continue()?;
        return Ok(());
    };

    if !config_path.exists() {
        println!("{}", "• No configuration file found.".yellow());
        println!();

        let create = Confirm::with_theme(&get_theme())
            .with_prompt(format!("Create {CONFIG_FILE_NAME}?"))
            .default(true)
            .interact_opt()?
            .unwrap_or(false);

        if create {
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

    let editor = preferred_editor();
    println!(
        "{} Opening {} with {}...",
        "•".bright_blue(),
        config_path.display().to_string().bright_white(),
        editor.bright_yellow()
    );
    println!();

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
            println!("  {}", config_path.display().to_string().bright_white());
        }
    }

    println!();
    press_any_key_to_continue()?;

    Ok(())
}
