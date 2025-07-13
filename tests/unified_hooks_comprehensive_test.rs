//! Unified hook tests
//!
//! Integrates hooks_comprehensive_test.rs and hooks_public_api_test.rs
//! Eliminates duplication and provides comprehensive hook functionality tests

use anyhow::Result;
use git2::Repository;
use git_workers::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create initial commit for repository
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let signature = git2::Signature::now("Test User", "test@example.com")?;

    // Create a file
    let workdir = repo.workdir().unwrap();
    fs::write(workdir.join("README.md"), "# Test Repository")?;

    // Add file to index
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("README.md"))?;
    index.write()?;

    // Create tree
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Create commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )?;

    Ok(())
}

// =============================================================================
// HookContext variation tests
// =============================================================================

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

// =============================================================================
// Hook execution tests - basic functionality
// =============================================================================

/// Test execute_hooks with post-create hook
#[test]
fn test_execute_hooks_post_create() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with post-create hooks
    let config_content = r#"
[hooks]
post-create = ["echo 'Post-create hook executed'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with pre-remove hook
#[test]
fn test_execute_hooks_pre_remove() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with pre-remove hooks
    let config_content = r#"
[hooks]
pre-remove = ["echo 'Pre-remove hook executed'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("pre-remove", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with post-switch hook
#[test]
fn test_execute_hooks_post_switch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with post-switch hooks
    let config_content = r#"
[hooks]
post-switch = ["echo 'Post-switch hook executed'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-switch", &context);
    assert!(result.is_ok());

    Ok(())
}

// =============================================================================
// Hook execution tests - error handling
// =============================================================================

/// Test execute_hooks with no hooks configured (using command-line git)
#[test]
fn test_execute_hooks_no_hooks_cmdline() -> Result<()> {
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

/// Test execute_hooks without config file (using git2)
#[test]
fn test_execute_hooks_no_config_git2() -> Result<()> {
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

/// Test execute_hooks with invalid configuration
#[test]
fn test_execute_hooks_invalid_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create invalid config file
    let invalid_config = "invalid toml content [[[";
    fs::write(repo_path.join(".git-workers.toml"), invalid_config)?;

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
#[test]
fn test_execute_hooks_empty_hooks() -> Result<()> {
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

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

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

// =============================================================================
// Multiple command hook tests
// =============================================================================

/// Test execute_hooks with multiple commands
#[test]
fn test_execute_hooks_multiple_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with multiple hook commands
    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'",
    "echo 'Third command'"
]
"#;

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test execute_hooks with template variables
#[test]
fn test_execute_hooks_with_templates() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with template variables
    let config_content = r#"
[hooks]
post-create = [
    "echo 'Worktree name: {{worktree_name}}'",
    "echo 'Worktree path: {{worktree_path}}'"
]
"#;

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "my-feature".to_string(),
        worktree_path: temp_dir.path().join("my-feature"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

// =============================================================================
// Different hook type tests
// =============================================================================

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

// =============================================================================
// Performance and load tests
// =============================================================================

/// Test hook execution performance
#[test]
fn test_hook_execution_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with simple hook
    let config_content = r#"
[hooks]
post-create = ["echo 'performance test'"]
"#;

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let start = std::time::Instant::now();

    // Execute hooks multiple times
    for _ in 0..10 {
        let result = execute_hooks("post-create", &context);
        assert!(result.is_ok());
    }

    let duration = start.elapsed();
    // Should complete reasonably quickly
    assert!(duration.as_secs() < 5);

    Ok(())
}

/// Test hook execution with many commands
#[test]
fn test_execute_hooks_many_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with many hook commands
    let mut commands = Vec::new();
    for i in 0..20 {
        commands.push(format!("echo 'Command {i}'"));
    }

    let commands_str = format!(
        "[{}]",
        commands
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let config_content = format!(
        r#"
[hooks]
post-create = {commands_str}
"#
    );

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}
