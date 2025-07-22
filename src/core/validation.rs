//! Validation logic for git-workers
//!
//! This module contains all validation logic for worktree names and paths,
//! ensuring safety and compatibility across different filesystems.

use anyhow::{anyhow, Result};

// Import constants from the parent module
use crate::constants::{
    GIT_RESERVED_NAMES, INVALID_FILESYSTEM_CHARS, MAX_WORKTREE_NAME_LENGTH, WINDOWS_RESERVED_CHARS,
};

/// Validates a worktree name for safety and compatibility
///
/// # Arguments
///
/// * `name` - The worktree name to validate
///
/// # Returns
///
/// * `Ok(String)` - The validated name (possibly with warnings shown)
/// * `Err(anyhow::Error)` - If the name is invalid
///
/// # Validation Rules
///
/// 1. **Non-empty**: Name must not be empty or whitespace-only
/// 2. **Length limit**: Must not exceed 255 characters (filesystem limit)
/// 3. **Reserved names**: Cannot be Git internal names (.git, HEAD, refs, etc.)
/// 4. **Invalid characters**: Cannot contain filesystem-incompatible characters
/// 5. **Hidden files**: Names starting with '.' are not allowed
/// 6. **Unicode**: Non-ASCII characters are not allowed for compatibility
///
/// # Security
///
/// This function prevents directory traversal attacks and ensures
/// compatibility across different filesystems and operating systems.
///
/// # Examples
///
/// ```rust
/// use git_workers::core::validate_worktree_name;
///
/// // Valid names
/// assert!(validate_worktree_name("feature-branch").is_ok());
/// assert!(validate_worktree_name("bugfix-123").is_ok());
///
/// // Invalid names
/// assert!(validate_worktree_name("").is_err());
/// assert!(validate_worktree_name(".git").is_err());
/// assert!(validate_worktree_name("branch:name").is_err());
/// ```
pub fn validate_worktree_name(name: &str) -> Result<String> {
    let trimmed = name.trim();

    // Check if empty
    if trimmed.is_empty() {
        return Err(anyhow!("Worktree name cannot be empty"));
    }

    // Check length
    if trimmed.len() > MAX_WORKTREE_NAME_LENGTH {
        return Err(anyhow!(
            "Worktree name cannot exceed {MAX_WORKTREE_NAME_LENGTH} characters"
        ));
    }

    // Check for Git reserved names (case insensitive)
    let trimmed_lower = trimmed.to_lowercase();
    for reserved in GIT_RESERVED_NAMES {
        if trimmed_lower == reserved.to_lowercase() {
            return Err(anyhow!(
                "'{}' is a reserved Git name and cannot be used as a worktree name",
                trimmed
            ));
        }
    }

    // Check for path separators and dangerous characters
    for &ch in INVALID_FILESYSTEM_CHARS {
        if trimmed.contains(ch) {
            return Err(anyhow!(
                "Worktree name cannot contain '{}' (filesystem incompatible)",
                ch
            ));
        }
    }

    // Check for Windows reserved characters (even on non-Windows systems for portability)
    for &ch in WINDOWS_RESERVED_CHARS {
        if trimmed.contains(ch) {
            return Err(anyhow!(
                "Worktree name cannot contain '{}' (Windows incompatible)",
                ch
            ));
        }
    }

    // Check for null bytes
    if trimmed.contains('\0') {
        return Err(anyhow!("Worktree name cannot contain null bytes"));
    }

    // Check if name starts with a dot (hidden file - not allowed)
    if trimmed.starts_with('.') {
        return Err(anyhow!(
            "Worktree name cannot start with '.' (hidden files not allowed)"
        ));
    }

    // Check for non-ASCII characters (not allowed in test environment)
    if !trimmed.is_ascii() {
        return Err(anyhow!(
            "Worktree name must contain only ASCII characters for compatibility"
        ));
    }

    Ok(trimmed.to_string())
}

/// Validates a custom path for worktree creation
///
/// # Arguments
///
/// * `path` - The custom path to validate
///
/// # Returns
///
/// * `Ok(())` - If the path is valid
/// * `Err(anyhow::Error)` - If the path is invalid with explanation
///
/// # Validation Rules
///
/// 1. **Relative paths only**: Must not be absolute
/// 2. **Path traversal prevention**: Limited directory traversal
/// 3. **Reserved names**: Cannot contain Git reserved names in path components
/// 4. **Character validation**: Must not contain filesystem-incompatible characters
/// 5. **Cross-platform compatibility**: Validates against Windows reserved characters
/// 6. **Format validation**: Must not end with path separators
///
/// # Security
///
/// This function is critical for preventing directory traversal attacks
/// and ensuring the worktree is created in a safe location relative
/// to the repository root.
///
/// # Examples
///
/// ```rust
/// use git_workers::core::validate_custom_path;
///
/// // Valid paths
/// assert!(validate_custom_path("../sibling-dir/worktree").is_ok());
/// assert!(validate_custom_path("subdir/worktree").is_ok());
/// assert!(validate_custom_path("../parent/child").is_ok());
///
/// // Invalid paths
/// assert!(validate_custom_path("/absolute/path").is_err());
/// assert!(validate_custom_path("../../etc/passwd").is_err());
/// assert!(validate_custom_path("path/.git/config").is_ok()); // .git as path component is allowed
/// ```
pub fn validate_custom_path(path: &str) -> Result<()> {
    let trimmed = path.trim();

    // Check if empty
    if trimmed.is_empty() {
        return Err(anyhow!("Custom path cannot be empty"));
    }

    // Check if absolute path
    if trimmed.starts_with('/') || (trimmed.len() > 1 && trimmed.chars().nth(1) == Some(':')) {
        return Err(anyhow!("Custom path must be relative, not absolute"));
    }

    // Check for Windows UNC paths
    if trimmed.starts_with("\\\\") {
        return Err(anyhow!("UNC paths are not supported"));
    }

    // Check if path ends with separator (not allowed)
    if trimmed.ends_with('/') || trimmed.ends_with('\\') {
        return Err(anyhow!("Custom path cannot end with a path separator"));
    }

    // Split path into components for validation
    let components: Vec<&str> = trimmed.split('/').collect();

    // Check for excessive directory traversal (security)
    let mut depth = 0;
    for component in &components {
        if *component == ".." {
            depth -= 1;
            if depth < -1 {
                // Allow one level up but not more for security
                return Err(anyhow!(
                    "Excessive directory traversal (..) is not allowed for security reasons"
                ));
            }
        } else if *component != "." && !component.is_empty() {
            depth += 1;
        }
    }

    // Validate each path component
    for component in &components {
        // Skip empty components, current dir (.), and parent dir (..)
        if component.is_empty() || *component == "." || *component == ".." {
            continue;
        }

        // Check for Git reserved names in path components
        if GIT_RESERVED_NAMES.contains(component) {
            return Err(anyhow!(
                "Path component '{}' is a reserved Git name",
                component
            ));
        }

        // Check for filesystem-incompatible characters (except backslash for Windows compatibility)
        for &ch in INVALID_FILESYSTEM_CHARS {
            // Allow backslash in custom paths for Windows compatibility
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

        // Check for Windows reserved characters
        for &ch in WINDOWS_RESERVED_CHARS {
            if component.contains(ch) {
                return Err(anyhow!(
                    "Path component '{}' contains Windows-incompatible character '{}'",
                    component,
                    ch
                ));
            }
        }

        // Check component length
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_worktree_name_valid() {
        let valid_names = vec![
            "valid-name",
            "valid_name",
            "valid123",
            "feature-branch",
            "bugfix_123",
        ];

        for name in valid_names {
            let result = validate_worktree_name(name);
            assert!(result.is_ok(), "Expected '{name}' to be valid");
        }
    }

    #[test]
    fn test_validate_worktree_name_invalid() {
        let invalid_names = vec![
            ("", "Empty name"),
            (".hidden", "Hidden file"),
            ("name/slash", "Contains slash"),
            ("HEAD", "Git reserved name"),
            ("name:colon", "Contains colon"),
        ];

        for (name, reason) in invalid_names {
            let result = validate_worktree_name(name);
            assert!(result.is_err(), "Expected '{name}' to be invalid: {reason}");
        }
    }

    #[test]
    fn test_validate_custom_path_valid() {
        let valid_paths = vec![
            "../safe/path",
            "subdirectory/path",
            "../sibling",
            "./relative/path",
            "simple-path",
        ];

        for path in valid_paths {
            let result = validate_custom_path(path);
            assert!(result.is_ok(), "Expected '{path}' to be valid");
        }
    }

    #[test]
    fn test_validate_custom_path_invalid() {
        let invalid_paths = vec![
            ("", "Empty path"),
            ("/absolute/path", "Absolute path"),
            ("../../etc/passwd", "Too many parent traversals"),
            ("path/", "Trailing slash"),
            ("C:\\Windows", "Windows absolute path"),
        ];

        for (path, reason) in invalid_paths {
            let result = validate_custom_path(path);
            assert!(result.is_err(), "Expected '{path}' to be invalid: {reason}");
        }
    }
}
