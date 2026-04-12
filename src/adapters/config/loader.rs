use anyhow::{anyhow, Result};

use crate::constants::{CONFIG_FILE_NAME, GIT_DIR};
use crate::git::GitWorktreeManager;

pub use crate::config::Config;

pub fn find_config_file_path(manager: &GitWorktreeManager) -> Result<std::path::PathBuf> {
    find_config_file_path_internal(manager.repo())
}

pub fn find_config_file_path_internal(repo: &git2::Repository) -> Result<std::path::PathBuf> {
    if repo.is_bare() {
        if let Ok(cwd) = std::env::current_dir() {
            let current_config = cwd.join(CONFIG_FILE_NAME);
            if current_config.exists() {
                return Ok(current_config);
            }

            Ok(cwd.join(CONFIG_FILE_NAME))
        } else {
            Err(anyhow!("Cannot determine current directory"))
        }
    } else if let Ok(cwd) = std::env::current_dir() {
        let current_config = cwd.join(CONFIG_FILE_NAME);
        if current_config.exists() {
            return Ok(current_config);
        }

        if let Some(workdir) = repo.workdir() {
            let workdir_path = workdir.to_path_buf();

            if cwd == workdir_path {
                return Ok(workdir_path.join(CONFIG_FILE_NAME));
            }

            let git_path = workdir_path.join(GIT_DIR);
            if git_path.is_dir() && workdir_path.exists() {
                let config_path = workdir_path.join(CONFIG_FILE_NAME);
                if config_path.exists() {
                    return Ok(config_path);
                }
            }
        }

        Ok(cwd.join(CONFIG_FILE_NAME))
    } else {
        repo.workdir()
            .map(|path| path.join(CONFIG_FILE_NAME))
            .ok_or_else(|| anyhow!("No working directory found"))
    }
}
