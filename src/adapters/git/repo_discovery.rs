use anyhow::Result;
use git2::Repository;
use std::path::Path;

use crate::adapters::git::git_worktree_repository::GitWorktreeManager;

pub(crate) fn discover_repository_from_env() -> Result<Repository> {
    Ok(Repository::open_from_env()?)
}

pub(crate) fn open_repository_at_path_raw(path: &Path) -> Result<Repository> {
    Ok(Repository::open(path)?)
}

pub fn open_repository_from_env() -> Result<GitWorktreeManager> {
    let repo = discover_repository_from_env()?;
    Ok(GitWorktreeManager { repo })
}

pub fn open_repository_at_path(path: &Path) -> Result<GitWorktreeManager> {
    let repo = open_repository_at_path_raw(path)?;
    Ok(GitWorktreeManager { repo })
}
