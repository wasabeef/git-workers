use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::constants::{STRING_CUSTOM, STRING_SAME_LEVEL};
use crate::domain::worktree::WorktreeInfo;

pub fn validate_worktree_location(location: &str) -> Result<()> {
    match location {
        STRING_SAME_LEVEL | STRING_CUSTOM => Ok(()),
        _ => Err(anyhow!("Invalid worktree location type: {location}")),
    }
}

pub fn determine_worktree_path(
    git_dir: &Path,
    name: &str,
    location: &str,
    custom_path: Option<PathBuf>,
) -> Result<(PathBuf, String)> {
    validate_worktree_location(location)?;

    match location {
        STRING_SAME_LEVEL => {
            let path = git_dir
                .parent()
                .ok_or_else(|| anyhow!("Cannot determine parent directory"))?
                .join(name);
            Ok((path, STRING_SAME_LEVEL.to_string()))
        }
        STRING_CUSTOM => {
            let path = custom_path
                .ok_or_else(|| anyhow!("Custom path required when location is 'custom'"))?;
            Ok((git_dir.join(path), STRING_CUSTOM.to_string()))
        }
        _ => Err(anyhow!("Invalid location type: {location}")),
    }
}

pub fn validate_worktree_creation(
    name: &str,
    path: &PathBuf,
    existing_worktrees: &[WorktreeInfo],
) -> Result<()> {
    if existing_worktrees.iter().any(|w| w.name == name) {
        return Err(anyhow!("Worktree '{name}' already exists"));
    }

    if existing_worktrees.iter().any(|w| w.path == *path) {
        return Err(anyhow!("Path '{}' already in use", path.display()));
    }

    Ok(())
}
