//! Tests for custom path behavior in worktree creation
//!
//! This test suite ensures that the custom path feature behaves correctly:
//! - Always treats input as directory path and appends worktree name
//! - Handles special cases like "./" and "../" correctly
//! - Validates paths for security and compatibility

use anyhow::Result;
use git_workers::commands::create_worktree_with_ui;
use tempfile::TempDir;

mod common;
use common::{TestRepo, TestUI};

/// Test that custom path always appends worktree name
#[test]
fn test_custom_path_appends_worktree_name() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // Test case 1: "branch/" -> "branch/feature-x"
    let ui = TestUI::new()
        .with_input("feature-x") // worktree name
        .with_selection(2) // custom path option
        .with_input("branch/") // directory path
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result); // didn't switch

    // Verify worktree was created at correct location
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "feature-x"));
    assert!(worktrees
        .iter()
        .any(|w| w.path.ends_with("branch/feature-x")));

    Ok(())
}

/// Test that "./" creates worktree in project root  
#[test]
fn test_dot_slash_creates_in_project_root() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    let ui = TestUI::new()
        .with_input("my-feature") // worktree name
        .with_selection(2) // custom path option
        .with_input("./") // current directory
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    // Verify worktree was created at ./my-feature
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "my-feature"));

    // The path should be in the project root (contains repo name and worktree name)
    let worktree = worktrees.iter().find(|w| w.name == "my-feature").unwrap();
    let path_str = worktree.path.to_string_lossy();
    assert!(path_str.contains("test-repo/my-feature") || path_str.ends_with("/my-feature"));

    Ok(())
}

/// Test that "../" creates worktree outside project
#[test]
fn test_parent_directory_creates_outside_project() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    let ui = TestUI::new()
        .with_input("external-feature") // worktree name
        .with_selection(2) // custom path option
        .with_input("../") // parent directory
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    // Verify worktree was created at ../external-feature
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "external-feature"));

    let worktree = worktrees
        .iter()
        .find(|w| w.name == "external-feature")
        .unwrap();
    let repo_parent = test_repo.path().parent().unwrap();
    let expected_path = repo_parent.join("external-feature");
    assert_eq!(worktree.path.canonicalize()?, expected_path.canonicalize()?);

    Ok(())
}

/// Test nested directory paths
#[test]
fn test_nested_directory_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // Test case: "features/ui/" -> "features/ui/button"
    let ui = TestUI::new()
        .with_input("button") // worktree name
        .with_selection(2) // custom path option
        .with_input("features/ui/") // nested directory
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "button"));
    assert!(worktrees
        .iter()
        .any(|w| w.path.ends_with("features/ui/button")));

    Ok(())
}

/// Test that paths without trailing slash still work as directories
#[test]
fn test_path_without_trailing_slash_treated_as_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // "hotfix" (no slash) should behave like "hotfix/"
    let ui = TestUI::new()
        .with_input("urgent-fix") // worktree name
        .with_selection(2) // custom path option
        .with_input("hotfix") // no trailing slash
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "urgent-fix"));
    assert!(worktrees
        .iter()
        .any(|w| w.path.ends_with("hotfix/urgent-fix")));

    Ok(())
}

/// Test empty input defaults to worktree name only
#[test]
fn test_empty_path_uses_worktree_name_only() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    let ui = TestUI::new()
        .with_input("simple") // worktree name
        .with_selection(2) // custom path option
        .with_input("") // empty path
        .with_error(); // should error on empty path

    let result = create_worktree_with_ui(&manager, &ui);
    assert!(result.is_ok()); // Function succeeds but returns false
    assert!(!result.unwrap()); // Operation was cancelled due to empty path

    // Verify no worktree was created
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.iter().any(|w| w.name == "simple"));

    Ok(())
}

/// Test path validation prevents dangerous paths
#[test]
fn test_path_validation_prevents_dangerous_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // Test absolute path (should fail)
    let ui = TestUI::new()
        .with_input("test") // worktree name
        .with_selection(2) // custom path option
        .with_input("/tmp/evil") // absolute path
        .with_error(); // should error

    let result = create_worktree_with_ui(&manager, &ui);
    assert!(result.is_ok() && !result.unwrap());

    // Test path traversal (should fail)
    let ui = TestUI::new()
        .with_input("test")
        .with_selection(2)
        .with_input("../../../../../../etc") // path traversal
        .with_error();

    let result = create_worktree_with_ui(&manager, &ui);
    assert!(result.is_ok() && !result.unwrap());

    // Verify no worktrees were created
    let worktrees = manager.list_worktrees()?;
    assert_eq!(worktrees.len(), 0);

    Ok(())
}

/// Test special case: just "/" becomes worktree name
#[test]
fn test_single_slash_becomes_worktree_name() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    let ui = TestUI::new()
        .with_input("root-level") // worktree name
        .with_selection(2) // custom path option
        .with_input("/") // just a slash
        .with_selection(0) // create from HEAD
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    // Should create at the default location with just the worktree name
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "root-level"));

    Ok(())
}

/// Test that custom paths work with branch selection too
#[test]
fn test_custom_path_with_branch_selection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // Create a test branch
    test_repo.create_branch("test-branch")?;

    let ui = TestUI::new()
        .with_input("branch-feature") // worktree name
        .with_selection(2) // custom path option
        .with_input("branches/") // directory for branches
        .with_selection(1) // select branch
        .with_selection(0) // select first branch (test-branch)
        .with_confirmation(false); // don't switch

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    let worktrees = manager.list_worktrees()?;
    let worktree = worktrees
        .iter()
        .find(|w| w.name == "branch-feature")
        .unwrap();
    assert!(worktree.path.ends_with("branches/branch-feature"));
    // New branch was created with worktree name, not using test-branch directly
    assert_eq!(worktree.branch, "branch-feature");

    Ok(())
}

/// Test UI examples match actual behavior
#[test]
fn test_ui_examples_are_accurate() -> Result<()> {
    // Test each example independently to avoid state pollution
    let examples = vec![
        ("example1", "branch/", "branch/example1"),
        ("example2", "hotfix/", "hotfix/example2"),
        ("example3", "../", "/example3"), // Absolute path will end with just the worktree name
        ("example4", "./", "example4"),   // Relative to root ends with just the worktree name
    ];

    for (name, input, expected_suffix) in examples.into_iter() {
        // Create a fresh test environment for each example
        let temp_dir = TempDir::new()?;
        let test_repo = TestRepo::new(&temp_dir)?;
        let manager = test_repo.manager()?;

        let ui = TestUI::new()
            .with_input(name) // worktree name
            .with_selection(2) // custom path option
            .with_input(input) // directory path
            .with_selection(0) // create from HEAD
            .with_confirmation(false); // don't switch

        let result = create_worktree_with_ui(&manager, &ui)?;
        assert!(!result);

        let worktrees = manager.list_worktrees()?;
        let worktree = worktrees.iter().find(|w| w.name == name).unwrap();

        // Normalize paths for comparison
        let path_str = worktree.path.to_string_lossy();
        let path_str = path_str.replace('\\', "/"); // Handle Windows paths

        assert!(
            path_str.ends_with(expected_suffix),
            "Expected path to end with '{expected_suffix}', but got '{path_str}'"
        );
    }

    Ok(())
}

/// Test that the same behavior works for subsequent worktrees
#[test]
fn test_custom_path_for_subsequent_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    // Create first worktree with custom path
    let ui = TestUI::new()
        .with_input("first")
        .with_selection(2) // custom path
        .with_input("work/")
        .with_selection(0)
        .with_confirmation(false);

    create_worktree_with_ui(&manager, &ui)?;

    // Create second worktree (should still offer custom path option)
    let ui = TestUI::new()
        .with_input("second")
        .with_selection(2) // custom path should still be available
        .with_input("work/") // same directory
        .with_selection(0)
        .with_confirmation(false);

    let result = create_worktree_with_ui(&manager, &ui)?;
    assert!(!result);

    // Both should be in the work/ directory
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.path.ends_with("work/first")));
    assert!(worktrees.iter().any(|w| w.path.ends_with("work/second")));

    Ok(())
}

/// Test edge case: single dot "." behaves like "./"
#[test]
fn test_single_dot_behaves_like_dot_slash() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_repo = TestRepo::new(&temp_dir)?;
    let manager = test_repo.manager()?;

    let ui = TestUI::new()
        .with_input("dot-test")
        .with_selection(2) // custom path
        .with_input(".") // just a dot
        .with_selection(0)
        .with_confirmation(false);

    create_worktree_with_ui(&manager, &ui)?;

    // Should behave same as "./"
    let worktrees = manager.list_worktrees()?;
    let worktree = worktrees.iter().find(|w| w.name == "dot-test").unwrap();

    // Should be in project root
    let path_str = worktree.path.to_string_lossy();
    assert!(path_str.contains("test-repo/dot-test") || path_str.ends_with("/dot-test"));

    Ok(())
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use git_workers::core::validate_custom_path;

    #[test]
    fn test_validate_custom_path_accepts_valid_paths() {
        // Valid relative paths
        assert!(validate_custom_path("features/ui").is_ok());
        assert!(validate_custom_path("../external").is_ok());
        assert!(validate_custom_path("./local").is_ok());
        assert!(validate_custom_path("work/2024/january").is_ok());

        // Paths with dots
        assert!(validate_custom_path("versions/v1.2.3").is_ok());
        assert!(validate_custom_path("config.old/backup").is_ok());
    }

    #[test]
    fn test_validate_custom_path_rejects_invalid_paths() {
        // Absolute paths
        assert!(validate_custom_path("/absolute/path").is_err());
        assert!(validate_custom_path("C:\\Windows\\Path").is_err());

        // Invalid characters
        assert!(validate_custom_path("path*with*asterisk").is_err());
        assert!(validate_custom_path("path?with?question").is_err());
        assert!(validate_custom_path("path:with:colon").is_err());

        // Git reserved names - these should be allowed as directory names
        // Only top-level git reserved names are blocked
        assert!(validate_custom_path("gitignore").is_ok()); // This should be ok
        assert!(validate_custom_path("HEAD").is_err()); // This should be blocked
        assert!(validate_custom_path("refs").is_err()); // This should be blocked

        // Path traversal
        assert!(validate_custom_path("../../../../../../../etc/passwd").is_err());
    }

    #[test]
    fn test_path_normalization() {
        // The implementation should handle these cases correctly
        let temp_dir = TempDir::new().unwrap();
        let test_repo = TestRepo::new(&temp_dir).unwrap();
        let manager = test_repo.manager().unwrap();

        // Test trailing slashes are handled
        let ui = TestUI::new()
            .with_input("test")
            .with_selection(2)
            .with_input("path/with/trailing/////") // Multiple trailing slashes
            .with_selection(0)
            .with_confirmation(false);

        let result = create_worktree_with_ui(&manager, &ui).unwrap();
        assert!(!result);

        let worktrees = manager.list_worktrees().unwrap();
        let worktree = worktrees.iter().find(|w| w.name == "test").unwrap();
        assert!(worktree.path.ends_with("path/with/trailing/test"));
    }
}
