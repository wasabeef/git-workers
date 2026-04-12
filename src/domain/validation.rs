use anyhow::{anyhow, Result};

use crate::constants::{
    GIT_RESERVED_NAMES, INVALID_FILESYSTEM_CHARS, MAX_WORKTREE_NAME_LENGTH, WINDOWS_RESERVED_CHARS,
};

pub fn validate_worktree_name(name: &str) -> Result<String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Worktree name cannot be empty"));
    }

    if trimmed.len() > MAX_WORKTREE_NAME_LENGTH {
        return Err(anyhow!(
            "Worktree name cannot exceed {MAX_WORKTREE_NAME_LENGTH} characters"
        ));
    }

    let trimmed_lower = trimmed.to_lowercase();
    for reserved in GIT_RESERVED_NAMES {
        if trimmed_lower == reserved.to_lowercase() {
            return Err(anyhow!(
                "'{}' is a reserved Git name and cannot be used as a worktree name",
                trimmed
            ));
        }
    }

    for &ch in INVALID_FILESYSTEM_CHARS {
        if trimmed.contains(ch) {
            return Err(anyhow!(
                "Worktree name cannot contain '{}' (filesystem incompatible)",
                ch
            ));
        }
    }

    for &ch in WINDOWS_RESERVED_CHARS {
        if trimmed.contains(ch) {
            return Err(anyhow!(
                "Worktree name cannot contain '{}' (Windows incompatible)",
                ch
            ));
        }
    }

    if trimmed.contains('\0') {
        return Err(anyhow!("Worktree name cannot contain null bytes"));
    }

    if trimmed.starts_with('.') {
        return Err(anyhow!(
            "Worktree name cannot start with '.' (hidden files not allowed)"
        ));
    }

    if !trimmed.is_ascii() {
        return Err(anyhow!(
            "Worktree name must contain only ASCII characters for compatibility"
        ));
    }

    Ok(trimmed.to_string())
}

pub fn validate_custom_path(path: &str) -> Result<()> {
    let trimmed = path.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Custom path cannot be empty"));
    }

    if trimmed.starts_with('/') || (trimmed.len() > 1 && trimmed.chars().nth(1) == Some(':')) {
        return Err(anyhow!("Custom path must be relative, not absolute"));
    }

    if trimmed.starts_with("\\\\") {
        return Err(anyhow!("UNC paths are not supported"));
    }

    if trimmed.ends_with('/') || trimmed.ends_with('\\') {
        return Err(anyhow!("Custom path cannot end with a path separator"));
    }

    let components: Vec<&str> = trimmed.split('/').collect();

    let mut depth = 0;
    for component in &components {
        if *component == ".." {
            depth -= 1;
            if depth < -1 {
                return Err(anyhow!(
                    "Excessive directory traversal (..) is not allowed for security reasons"
                ));
            }
        } else if *component != "." && !component.is_empty() {
            depth += 1;
        }
    }

    for component in &components {
        if component.is_empty() || *component == "." || *component == ".." {
            continue;
        }

        if GIT_RESERVED_NAMES.contains(component) {
            return Err(anyhow!(
                "Path component '{}' is a reserved Git name",
                component
            ));
        }

        for &ch in INVALID_FILESYSTEM_CHARS {
            if ch == '\\' {
                continue;
            }
            if component.contains(ch) {
                return Err(anyhow!(
                    "Path component '{}' contains invalid character '{}'",
                    component,
                    ch
                ));
            }
        }

        for &ch in WINDOWS_RESERVED_CHARS {
            if component.contains(ch) {
                return Err(anyhow!(
                    "Path component '{}' contains Windows-incompatible character '{}'",
                    component,
                    ch
                ));
            }
        }

        if component.len() > MAX_WORKTREE_NAME_LENGTH {
            return Err(anyhow!(
                "Path component '{}' exceeds maximum length of {} characters",
                component,
                MAX_WORKTREE_NAME_LENGTH
            ));
        }
    }

    Ok(())
}
