//! Legacy validation facade kept for backward compatibility.

pub use crate::domain::validation::{validate_custom_path, validate_worktree_name};

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
