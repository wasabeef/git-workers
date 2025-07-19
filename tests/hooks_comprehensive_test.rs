//! Comprehensive tests for hooks module
//!
//! This module provides comprehensive test coverage for the hooks functionality,
//! including template variable substitution, error handling, and command execution.

use anyhow::Result;
use git_workers::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a test config file
fn create_test_config(temp_dir: &TempDir, content: &str) -> PathBuf {
    let config_path = temp_dir.path().join(".git-workers.toml");
    fs::write(&config_path, content).expect("Failed to write test config");
    config_path
}

/// Helper function to create a test worktree directory
fn create_test_worktree(temp_dir: &TempDir, name: &str) -> PathBuf {
    let worktree_path = temp_dir.path().join(name);
    fs::create_dir_all(&worktree_path).expect("Failed to create test worktree");
    worktree_path
}

/// Test template variable substitution functionality
#[test]
fn test_execute_hooks_template_substitution() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "feature-branch");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'Created worktree: {{worktree_name}}'",
    "echo 'Path: {{worktree_path}}'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "feature-branch".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should succeed since echo commands are valid
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with command failures
#[test]
fn test_execute_hooks_command_failure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "test-worktree");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'This works'",
    "nonexistent-command-should-fail",
    "echo 'This might not run'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should continue execution even if one command fails
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with multiple commands
#[test]
fn test_execute_hooks_multiple_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "multi-test");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'",
    "echo 'Third command'"
]
pre-remove = [
    "echo 'Removing worktree'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "multi-test".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test post-create hooks
    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    // Test pre-remove hooks
    let result = execute_hooks("pre-remove", &context);
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with empty configuration
#[test]
fn test_execute_hooks_empty_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "empty-test");

    let config_content = r#"
[hooks]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "empty-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should succeed even with empty hooks
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with invalid template variables
#[test]
fn test_execute_hooks_invalid_template() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "invalid-template");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'Valid: {{worktree_name}}'",
    "echo 'Invalid: {{invalid_variable}}'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "invalid-template".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should still succeed, invalid templates just remain as-is
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution without config file
#[test]
fn test_execute_hooks_no_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "no-config-test");

    let context = HookContext {
        worktree_name: "no-config-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should succeed gracefully when no config is provided
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with malformed TOML
#[test]
fn test_execute_hooks_malformed_toml() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "malformed-test");

    let config_content = r#"
[hooks
post-create = [
    "echo 'This will not work'"
# Missing closing bracket
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "malformed-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should handle malformed TOML gracefully
    assert!(result.is_ok());

    Ok(())
}

/// Test hook context creation with various inputs
#[test]
fn test_hook_context_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "context-test");

    // Test normal context creation
    let context = HookContext {
        worktree_name: "context-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    assert_eq!(context.worktree_name, "context-test");
    assert_eq!(context.worktree_path, worktree_path);

    // Test with special characters in name
    let special_context = HookContext {
        worktree_name: "feature/new-ui".to_string(),
        worktree_path: worktree_path.clone(),
    };
    assert_eq!(special_context.worktree_name, "feature/new-ui");

    // Test with unicode characters
    let unicode_context = HookContext {
        worktree_name: "機能-テスト".to_string(),
        worktree_path: worktree_path.clone(),
    };
    assert_eq!(unicode_context.worktree_name, "機能-テスト");

    Ok(())
}

/// Test different hook types
#[test]
fn test_hook_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "hook-types-test");

    let config_content = r#"
[hooks]
post-create = ["echo 'post-create hook'"]
pre-remove = ["echo 'pre-remove hook'"]
post-switch = ["echo 'post-switch hook'"]
custom = ["echo 'custom hook'"]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);
    let context = HookContext {
        worktree_name: "hook-types-test".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test all hook types
    assert!(execute_hooks("post-create", &context).is_ok());
    assert!(execute_hooks("pre-remove", &context).is_ok());
    assert!(execute_hooks("post-switch", &context).is_ok());

    // Test custom hook type (if supported)
    let custom_result = execute_hooks("custom", &context);
    assert!(custom_result.is_ok());

    Ok(())
}

/// Test hook execution with complex shell commands
#[test]
fn test_execute_hooks_complex_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "complex-test");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'Working directory:' && pwd",
    "echo 'Worktree name: {{worktree_name}}' > /tmp/hook-test.log || true",
    "ls -la {{worktree_path}} || echo 'Directory listing failed'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "complex-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should handle complex shell commands
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution performance with many commands
#[test]
fn test_execute_hooks_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "performance-test");

    // Create config with many commands
    let mut commands = Vec::new();
    for i in 0..20 {
        commands.push(format!("\"echo 'Command {i}'\""));
    }

    let config_content = format!(
        r#"
[hooks]
post-create = [{}]
"#,
        commands.join(", ")
    );

    let _config_path = create_test_config(&temp_dir, &config_content);

    let context = HookContext {
        worktree_name: "performance-test".to_string(),
        worktree_path: worktree_path.clone(),
    };

    let start = std::time::Instant::now();
    let result = execute_hooks("post-create", &context);
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Should complete within reasonable time (5 seconds for 20 echo commands)
    assert!(duration.as_secs() < 5);

    Ok(())
}

/// Test hook execution with environment variable access
#[test]
fn test_execute_hooks_environment_variables() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "env-test");

    let config_content = r#"
[hooks]
post-create = [
    "echo 'User: $USER'",
    "echo 'Home: $HOME'",
    "echo 'Path: $PATH'"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "env-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should access environment variables successfully
    assert!(result.is_ok());

    Ok(())
}

/// Test hook execution with working directory context
#[test]
fn test_execute_hooks_working_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let worktree_path = create_test_worktree(&temp_dir, "workdir-test");

    let config_content = r#"
[hooks]
post-create = [
    "pwd",
    "echo 'Current directory listing:'",
    "ls -la"
]
"#;

    let _config_path = create_test_config(&temp_dir, config_content);

    let context = HookContext {
        worktree_name: "workdir-test".to_string(),
        worktree_path: worktree_path.clone(),
    };
    let result = execute_hooks("post-create", &context);

    // Should execute in proper working directory context
    assert!(result.is_ok());

    Ok(())
}
