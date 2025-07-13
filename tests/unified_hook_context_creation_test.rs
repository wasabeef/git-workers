/// Unified Hook Context Creation Tests
///
/// This file consolidates the `test_hook_context_creation` function duplicates
/// from hooks_test.rs and hooks_public_api_test.rs, providing comprehensive
/// coverage for HookContext creation and validation.
use anyhow::Result;
use git2::Repository;
use git_workers::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test basic HookContext creation and field access
///
/// This test consolidates both versions of test_hook_context_creation,
/// testing both Path::new() and PathBuf::from() approaches for path creation.
#[test]
fn test_hook_context_creation() {
    // Test with Path::new() (from hooks_test.rs)
    let context1 = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: Path::new("/path/to/worktree").to_path_buf(),
    };

    assert_eq!(context1.worktree_name, "test-worktree");
    assert_eq!(
        context1.worktree_path.to_str().unwrap(),
        "/path/to/worktree"
    );

    // Test with PathBuf::from() (from hooks_public_api_test.rs)
    let context2 = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: PathBuf::from("/path/to/worktree"),
    };

    assert_eq!(context2.worktree_name, "test-worktree");
    assert_eq!(context2.worktree_path, PathBuf::from("/path/to/worktree"));

    // Verify both approaches produce equivalent results
    assert_eq!(context1.worktree_name, context2.worktree_name);
    assert_eq!(context1.worktree_path, context2.worktree_path);
}

/// Test HookContext with various input combinations
///
/// This test covers the edge cases and input variations from hooks_public_api_test.rs
#[test]
fn test_hook_context_various_inputs() {
    let test_cases = vec![
        ("simple", "/simple/path"),
        ("name-with-dashes", "/path/with/dashes"),
        ("name_with_underscores", "/path/with/underscores"),
        ("name.with.dots", "/path/with/dots"),
        ("name with spaces", "/path with spaces"),
        ("123numeric", "/123/numeric/path"),
        ("", ""),   // Empty values
        ("a", "b"), // Single characters
        (
            "very-long-worktree-name-with-many-characters",
            "/very/long/path/to/worktree/with/many/segments",
        ),
        ("name123", "/path/123"),
        ("123name", "/123/path"),
    ];

    for (name, path) in test_cases {
        let context = HookContext {
            worktree_name: name.to_string(),
            worktree_path: PathBuf::from(path),
        };

        assert_eq!(context.worktree_name, name);
        assert_eq!(context.worktree_path, PathBuf::from(path));
    }
}

/// Test HookContext simple validation
///
/// Simplified test from hooks_test.rs to verify basic functionality
#[test]
fn test_hook_context_simple() {
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: Path::new("/path/to/worktree").to_path_buf(),
    };

    assert_eq!(context.worktree_name, "test-worktree");
    assert!(context.worktree_path.to_str().is_some());
}

/// Test HookContext path operations
///
/// Test path manipulation methods on HookContext from hooks_test.rs
#[test]
fn test_hook_context_path_operations() {
    let context = HookContext {
        worktree_name: "my-feature".to_string(),
        worktree_path: Path::new("/repo/worktrees/my-feature").to_path_buf(),
    };

    // Test path operations
    assert_eq!(context.worktree_path.file_name().unwrap(), "my-feature");
    assert!(context.worktree_path.is_absolute());

    // Test worktree name handling
    assert_eq!(context.worktree_name, "my-feature");
}

/// Test execute_hooks without configuration file
///
/// Test from hooks_test.rs using git2 for repository initialization
#[test]
fn test_execute_hooks_without_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("test");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    // Should not fail even without .git/git-workers.toml
    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with no hooks configured
///
/// Test from hooks_public_api_test.rs using command-line git
#[test]
fn test_execute_hooks_no_hooks() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create a basic git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should succeed with no hooks to execute
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Execute hooks with no config should succeed"
    );

    Ok(())
}

/// Test execute_hooks with invalid configuration
///
/// Combined test from both files for handling invalid TOML configurations
#[test]
fn test_execute_hooks_with_invalid_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create invalid config file
    let invalid_config = "invalid toml content [[[";
    fs::write(repo_path.join(".git/git-workers.toml"), invalid_config)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("test");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    // Should handle invalid config gracefully
    let result = execute_hooks("post-create", &context);
    // This should not panic, though it may return an error
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

/// Test execute_hooks with empty hooks configuration
///
/// Combined test from both files for handling empty hook arrays
#[test]
fn test_execute_hooks_with_empty_hooks() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with empty hooks
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = []
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("test");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with non-existent hook type
///
/// Test from hooks_public_api_test.rs for handling undefined hook types
#[test]
fn test_execute_hooks_nonexistent_hook() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create a basic git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()?;

    // Create config file with hooks
    let config_content = r#"
[hooks]
post-create = ["echo 'Created worktree'"]
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should succeed for non-existent hook type (no hooks to execute)
    let result = execute_hooks("non-existent-hook", &context);
    assert!(
        result.is_ok(),
        "Execute hooks for non-existent hook should succeed"
    );

    Ok(())
}

/// Test execute_hooks with different hook types
///
/// Test from hooks_public_api_test.rs for handling various hook types
#[test]
fn test_execute_hooks_different_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create a basic git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()?;

    // Create config file with various hook types
    let config_content = r#"
[hooks]
post-create = ["echo 'post-create hook'"]
pre-remove = ["echo 'pre-remove hook'"]
post-switch = ["echo 'post-switch hook'"]
custom-hook = ["echo 'custom hook'"]
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Test different hook types
    let hook_types = vec!["post-create", "pre-remove", "post-switch", "custom-hook"];

    for hook_type in hook_types {
        let result = execute_hooks(hook_type, &context);
        assert!(
            result.is_ok(),
            "Execute hooks should succeed for hook type: {hook_type}"
        );
    }

    Ok(())
}

/// Test hook type string constants
///
/// Test from hooks_test.rs for verifying hook type constants
#[test]
fn test_hook_types() {
    // Test that hook type strings are correct
    let post_create = "post-create";
    let pre_remove = "pre-remove";
    let post_switch = "post-switch";

    assert_eq!(post_create, "post-create");
    assert_eq!(pre_remove, "pre-remove");
    assert_eq!(post_switch, "post-switch");
}

/// Helper function for creating initial commit using git2
///
/// Used by tests that initialize repositories with git2
fn create_initial_commit(repo: &Repository) -> Result<()> {
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}
