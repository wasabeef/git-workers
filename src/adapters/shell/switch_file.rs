use std::path::Path;

use anyhow::Result;

use crate::constants::{ENV_GW_SWITCH_FILE, MSG_SWITCH_FILE_WARNING_PREFIX, SWITCH_TO_PREFIX};

pub fn write_switch_path(path: &Path) -> Result<()> {
    if let Ok(switch_file) = std::env::var(ENV_GW_SWITCH_FILE) {
        if let Err(e) = std::fs::write(&switch_file, path.display().to_string()) {
            eprintln!("{MSG_SWITCH_FILE_WARNING_PREFIX}{e}");
        }
    } else {
        let path_display = path.display();
        println!("{SWITCH_TO_PREFIX}{path_display}");
    }

    Ok(())
}
