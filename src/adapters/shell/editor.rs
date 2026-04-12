use std::process::Command;

use anyhow::Result;

use crate::constants::{DEFAULT_EDITOR_UNIX, DEFAULT_EDITOR_WINDOWS, ENV_EDITOR, ENV_VISUAL};

pub fn preferred_editor() -> String {
    std::env::var(ENV_EDITOR)
        .or_else(|_| std::env::var(ENV_VISUAL))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                DEFAULT_EDITOR_WINDOWS.to_string()
            } else {
                DEFAULT_EDITOR_UNIX.to_string()
            }
        })
}

pub fn open_in_editor(path: &std::path::Path) -> Result<std::process::ExitStatus> {
    Ok(Command::new(preferred_editor()).arg(path).status()?)
}
