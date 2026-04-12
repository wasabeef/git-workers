use colored::*;

use crate::constants::{
    header_separator, DEFAULT_BRANCH_DETACHED, EMOJI_DETACHED, EMOJI_FOLDER, EMOJI_HOME,
    EMOJI_LOCKED,
};
use crate::domain::repo_context;
use crate::git::WorktreeInfo;

pub fn build_header_lines() -> Vec<String> {
    let version = env!("CARGO_PKG_VERSION");
    let title = format!("Git Workers v{version} - Interactive Worktree Manager")
        .bright_cyan()
        .bold()
        .to_string();
    let separator = header_separator();
    let repo_info = repo_context::get_repository_info();
    let repository_line = format!(
        "{} {}",
        "Repository:".bright_white(),
        repo_info.bright_yellow().bold()
    );

    vec![
        String::new(),
        title,
        separator,
        repository_line,
        String::new(),
    ]
}

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
