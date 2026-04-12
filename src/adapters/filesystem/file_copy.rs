use anyhow::Result;
use std::path::Path;

use crate::adapters::git::GitWorktreeManager;
use crate::config::FilesConfig;

pub fn copy_configured_files(
    config: &FilesConfig,
    destination_path: &Path,
    manager: &GitWorktreeManager,
) -> Result<Vec<String>> {
    crate::infrastructure::file_copy::copy_configured_files(config, destination_path, manager)
}
