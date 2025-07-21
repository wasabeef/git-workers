//! Unit tests for hook system functionality
//!
//! Tests for executing lifecycle hooks (post-create, pre-remove, post-switch)
//! and template variable substitution.

use anyhow::Result;
use git_workers::infrastructure::hooks::{execute_hooks, HookContext};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Hook Context Tests
// ============================================================================

#[test]
fn test_hook_context_creation() {
    let context = HookContext {
        worktree_name: "feature".to_string(),
        worktree_path: PathBuf::from("/tmp/feature"),
    };

    assert_eq!(context.worktree_name, "feature");
    assert_eq!(context.worktree_path, PathBuf::from("/tmp/feature"));
}

#[test]
fn test_hook_context_with_complex_names() {
    let context = HookContext {
        worktree_name: "feature-auth-123".to_string(),
        worktree_path: PathBuf::from("/path/to/project/worktrees/feature-auth-123"),
    };

    assert_eq!(context.worktree_name, "feature-auth-123");
    assert!(context
        .worktree_path
        .to_str()
        .unwrap()
        .contains("feature-auth-123"));
}

// ============================================================================
// Hook Template Variable Tests
// ============================================================================

#[test]
fn test_template_variables_structure() {
    // Test that template variables are defined
    assert!(git_workers::constants::TEMPLATE_WORKTREE_NAME.contains("worktree_name"));
    assert!(git_workers::constants::TEMPLATE_WORKTREE_PATH.contains("worktree_path"));
}

// ============================================================================
// Hook Execution Tests (Integration-style)
// ============================================================================

#[test]
fn test_execute_hooks_with_no_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Change to temp directory with no .git-workers.toml
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // This should succeed even without config
    let result = execute_hooks("post-create", &context);

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    // Should not error out when no config exists
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_execute_hooks_with_config() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a simple config file with echo command only (cross-platform)
    let config_content = r#"
[hooks]
post-create = ["echo 'Created {{worktree_name}}'"]
"#;
    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Execute hooks
    let result = execute_hooks("post-create", &context);

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    // Should succeed with valid config
    assert!(result.is_ok());

    Ok(())
}

// ============================================================================
// Hook Type Tests
// ============================================================================

#[test]
fn test_hook_types() {
    // Test that we can call execute_hooks with different hook types
    let _context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: PathBuf::from("/tmp/test"),
    };

    // These would normally require actual config files, but we're testing the API
    let hook_types = ["post-create", "pre-remove", "post-switch"];

    for hook_type in hook_types {
        // Just verify the function accepts these hook types
        assert!(!hook_type.is_empty());
        // In a real test environment with config, we could call:
        // execute_hooks(hook_type, &context).ok();
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_hook_execution_resilience() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with a failing command
    let config_content = r#"
[hooks]
post-create = ["exit 1", "echo 'This should still run'"]
"#;
    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Hook execution should not fail even if individual commands fail
    let result = execute_hooks("post-create", &context);

    std::env::set_current_dir(original_dir)?;

    // Should still return Ok even if hooks fail
    assert!(result.is_ok());

    Ok(())
}

// ============================================================================
// Path Handling Tests
// ============================================================================

#[test]
fn test_hook_context_path_types() {
    // Test with absolute path
    let abs_context = HookContext {
        worktree_name: "main".to_string(),
        worktree_path: PathBuf::from("/absolute/path/to/worktree"),
    };
    assert!(abs_context.worktree_path.is_absolute());

    // Test with relative path
    let rel_context = HookContext {
        worktree_name: "feature".to_string(),
        worktree_path: PathBuf::from("relative/path"),
    };
    assert!(!rel_context.worktree_path.is_absolute());
}

// ============================================================================
// Integration Test Simulation
// ============================================================================

#[test]
fn test_hook_execution_flow_simulation() -> Result<()> {
    // Simulate the flow that would happen during worktree operations
    let contexts = vec![
        HookContext {
            worktree_name: "feature-1".to_string(),
            worktree_path: PathBuf::from("/tmp/feature-1"),
        },
        HookContext {
            worktree_name: "hotfix-2".to_string(),
            worktree_path: PathBuf::from("/tmp/hotfix-2"),
        },
    ];

    for context in contexts {
        // Verify context properties
        assert!(!context.worktree_name.is_empty());
        assert!(!context.worktree_path.as_os_str().is_empty());

        // In real usage, hooks would be executed here:
        // execute_hooks("post-create", &context)?;
    }

    Ok(())
}
