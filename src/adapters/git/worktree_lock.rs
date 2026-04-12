use anyhow::{anyhow, Result};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::constants::{
    ERROR_LOCK_CREATE, ERROR_LOCK_EXISTS, LOCK_FILE_NAME, STALE_LOCK_TIMEOUT_SECS,
};

const STALE_LOCK_TIMEOUT: Duration = Duration::from_secs(STALE_LOCK_TIMEOUT_SECS);

pub struct WorktreeLock {
    lock_path: PathBuf,
    _file: Option<File>,
}

impl WorktreeLock {
    pub fn acquire(git_dir: &Path) -> Result<Self> {
        let lock_path = git_dir.join(LOCK_FILE_NAME);

        if lock_path.exists() {
            if let Ok(metadata) = lock_path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > STALE_LOCK_TIMEOUT {
                            let _ = fs::remove_file(&lock_path);
                        }
                    }
                }
            }
        }

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    anyhow!(ERROR_LOCK_EXISTS)
                } else {
                    anyhow!("{}", ERROR_LOCK_CREATE.replace("{}", &e.to_string()))
                }
            })?;

        Ok(Self {
            lock_path,
            _file: Some(file),
        })
    }
}

impl Drop for WorktreeLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}
