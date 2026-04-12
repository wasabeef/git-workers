//! Repository information display compatibility facade.

pub use crate::domain::repo_context::{get_repository_info, get_repository_info_at_path};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{MAIN_SUFFIX, UNKNOWN_VALUE};

    #[test]
    fn test_get_repository_info_non_git() {
        let info = get_repository_info();
        assert!(!info.is_empty());
    }

    #[test]
    fn test_constants_are_used() {
        assert_eq!(UNKNOWN_VALUE, "unknown");
        assert_eq!(MAIN_SUFFIX, " (main)");
    }

    #[test]
    fn test_repository_info_function_exists() {
        let _info = get_repository_info();
    }

    #[test]
    fn test_bare_repository_name_extraction() {
        use std::path::PathBuf;

        let bare_repo_path = PathBuf::from("/path/to/jump-app.bare/.git");
        let extracted_name = bare_repo_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        assert_eq!(extracted_name, "jump-app.bare");
    }

    #[test]
    fn test_non_bare_repository_name_extraction() {
        use std::path::PathBuf;

        let worktree_path = PathBuf::from("/path/to/worktree-name");
        let extracted_name = worktree_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        assert_eq!(extracted_name, "worktree-name");
    }
}
