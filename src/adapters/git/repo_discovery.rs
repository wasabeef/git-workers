use anyhow::Result;
use std::path::Path;

use crate::adapters::git::git_worktree_repository::GitWorktreeManager;

pub fn open_repository_from_env() -> Result<GitWorktreeManager> {
    GitWorktreeManager::new()
}

pub fn open_repository_at_path(path: &Path) -> Result<GitWorktreeManager> {
    GitWorktreeManager::new_from_path(path)
}
