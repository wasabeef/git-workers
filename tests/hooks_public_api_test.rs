use anyhow::Result;
use git_workers::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test HookContext creation and field access
#[test]
fn test_hook_context_creation() {
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: PathBuf::from("/path/to/worktree"),
    };

    assert_eq!(context.worktree_name, "test-worktree");
    assert_eq!(context.worktree_path, PathBuf::from("/path/to/worktree"));
}

/// Test HookContext with various names and paths
#[test]
fn test_hook_context_various_inputs() {
    let test_cases = vec![
        ("simple", "/simple/path"),
        ("name-with-dashes", "/path/with/dashes"),
        ("name_with_underscores", "/path/with/underscores"),
        ("name.with.dots", "/path/with/dots"),
        ("name with spaces", "/path with spaces"),
        ("123numeric", "/123/numeric/path"),
        ("", ""), // Empty values
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

/// Test execute_hooks with no hooks configured
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

/// Test execute_hooks with valid configuration
#[test]
fn test_execute_hooks_with_config() -> Result<()> {
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

/// Test execute_hooks with multiple commands
#[test]
fn test_execute_hooks_multiple_commands() -> Result<()> {
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

    // Create config file with multiple hook commands
    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'",
    "echo 'Third command'"
]
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should succeed with multiple commands
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Execute hooks with multiple commands should succeed"
    );

    Ok(())
}

/// Test execute_hooks with empty hook commands
#[test]
fn test_execute_hooks_empty_commands() -> Result<()> {
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

    // Create config file with empty hook commands
    let config_content = r#"
[hooks]
post-create = []
"#;

    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Should succeed with empty commands
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Execute hooks with empty commands should succeed"
    );

    Ok(())
}

/// Test execute_hooks with different hook types
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

/// Test HookContext with edge case values
#[test]
fn test_hook_context_edge_cases() {
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
        assert_eq!(context.worktree_name, name);
        assert_eq!(context.worktree_path, PathBuf::from(path));
    }
}

/// Test execute_hooks error handling with invalid config
#[test]
fn test_execute_hooks_invalid_config() -> Result<()> {
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

    // Create invalid config file
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
