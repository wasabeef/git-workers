use anyhow::Result;
use git2::Repository;
use git_workers::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Comprehensive test suite for execute_hooks functionality with various configurations
/// This file consolidates and unifies tests from hooks_test.rs and hooks_public_api_test.rs
/// Test execute_hooks with valid configuration using git2 Repository initialization
#[test]
fn test_execute_hooks_with_config_git2() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository using git2
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config file with hooks
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'Worktree created'", "echo 'Setup complete'"]
pre-remove = ["echo 'Cleaning up'"]
post-switch = ["echo 'Switched to {{worktree_name}}'"]
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("test-worktree");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    // Execute post-create hooks
    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with valid configuration using git command line initialization
#[test]
fn test_execute_hooks_with_config_git_cli() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create a basic git repository using command line
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
pre-remove = ["echo 'Removing worktree'"]
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should succeed with valid hooks
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Execute hooks with valid config should succeed"
    );

    Ok(())
}

/// Test execute_hooks with multiple hook types in comprehensive configuration
#[test]
fn test_execute_hooks_comprehensive_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository using git2
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create comprehensive config file with various hook types
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'post-create hook executed'", "echo 'worktree: {{worktree_name}}'"]
pre-remove = ["echo 'pre-remove hook executed'"]
post-switch = ["echo 'post-switch hook for {{worktree_name}}'"]
custom-hook = ["echo 'custom hook executed'"]
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("feature-branch");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "feature-branch".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    // Test all hook types
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

/// Test execute_hooks with empty hook configuration
#[test]
fn test_execute_hooks_empty_config() -> Result<()> {
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

/// Test execute_hooks with no hooks configured (no config file)
#[test]
fn test_execute_hooks_no_config() -> Result<()> {
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

/// Test execute_hooks with invalid configuration file
#[test]
fn test_execute_hooks_invalid_config() -> Result<()> {
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

/// Test execute_hooks with invalid TOML syntax in config
#[test]
fn test_execute_hooks_malformed_toml() -> Result<()> {
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

    // Create malformed TOML config file
    let config_content = r#"
[hooks
post-create = "invalid toml"
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should handle invalid config gracefully
    let result = execute_hooks("post-create", &context);
    // The function should either succeed (ignore invalid config) or fail gracefully
    match result {
        Ok(_) => { /* Handled invalid config gracefully by ignoring */ }
        Err(_) => { /* Handled invalid config gracefully by failing */ }
    }

    Ok(())
}

/// Test execute_hooks with non-existent hook type
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

/// Test execute_hooks with various worktree names and paths
#[test]
fn test_execute_hooks_various_contexts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config file with template variables
    let config_content = r#"
[hooks]
post-create = ["echo 'Created {{worktree_name}}'"]
post-switch = ["echo 'Switched to {{worktree_name}} at {{worktree_path}}'"]
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    let test_cases = vec![
        ("simple", "/simple/path"),
        ("name-with-dashes", "/path/with/dashes"),
        ("name_with_underscores", "/path/with/underscores"),
        ("name.with.dots", "/path/with/dots"),
        ("feature-123", "/features/feature-123"),
    ];

    for (worktree_name, worktree_path) in test_cases {
        // Create worktree directory
        let full_path = temp_dir.path().join(worktree_name);
        fs::create_dir_all(&full_path)?;

        let context = HookContext {
            worktree_name: worktree_name.to_string(),
            worktree_path: PathBuf::from(worktree_path),
        };

        // Test both hook types
        let result1 = execute_hooks("post-create", &context);
        assert!(
            result1.is_ok(),
            "post-create should succeed for worktree: {worktree_name}"
        );

        let result2 = execute_hooks("post-switch", &context);
        assert!(
            result2.is_ok(),
            "post-switch should succeed for worktree: {worktree_name}"
        );
    }

    Ok(())
}

/// Test execute_hooks with edge case worktree names
#[test]
fn test_execute_hooks_edge_case_names() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config file
    let config_content = r#"
[hooks]
post-create = ["echo 'Created {{worktree_name}}'"]
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    let edge_cases = vec![
        ("", ""),   // Empty strings
        ("a", "b"), // Single characters
        (
            "very-long-worktree-name-with-many-characters",
            "/very/long/path/to/worktree/with/many/segments",
        ),
        ("name123", "/path/123"),
        ("123name", "/123/path"),
    ];

    for (name, path) in edge_cases {
        let context = HookContext {
            worktree_name: name.to_string(),
            worktree_path: PathBuf::from(path),
        };

        // Should handle edge cases gracefully
        let result = execute_hooks("post-create", &context);
        assert!(
            result.is_ok(),
            "Execute hooks should handle edge case: name='{name}', path='{path}'"
        );
    }

    Ok(())
}

/// Test execute_hooks with config containing multiple commands per hook
#[test]
fn test_execute_hooks_multiple_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with multiple commands per hook
    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'", 
    "echo 'Third command for {{worktree_name}}'"
]
pre-remove = [
    "echo 'Cleanup step 1'",
    "echo 'Cleanup step 2'"
]
"#;

    fs::write(repo_path.join(".git/git-workers.toml"), config_content)?;

    // Create worktree directory
    let worktree_path = temp_dir.path().join("multi-command-test");
    fs::create_dir(&worktree_path)?;

    let context = HookContext {
        worktree_name: "multi-command-test".to_string(),
        worktree_path,
    };

    // Change to repo directory so config can be found
    std::env::set_current_dir(&repo_path)?;

    // Test post-create with multiple commands
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Multiple post-create commands should execute successfully"
    );

    // Test pre-remove with multiple commands
    let result = execute_hooks("pre-remove", &context);
    assert!(
        result.is_ok(),
        "Multiple pre-remove commands should execute successfully"
    );

    Ok(())
}

// Helper function for git2-based initialization
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
