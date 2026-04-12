use colored::*;

pub fn app_title(version: &str) -> String {
    format!("Git Workers v{version} - Interactive Worktree Manager")
        .bright_cyan()
        .bold()
        .to_string()
}
