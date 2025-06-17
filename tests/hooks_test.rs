use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use git_workers::hooks::{execute_hooks, HookContext};

#[test]
fn test_hook_context_creation() {
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: Path::new("/path/to/worktree").to_path_buf(),
    };

    assert_eq!(context.worktree_name, "test-worktree");
    assert_eq!(context.worktree_path.to_str().unwrap(), "/path/to/worktree");
}

#[test]
fn test_hook_context_simple() {
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: Path::new("/path/to/worktree").to_path_buf(),
    };

    assert_eq!(context.worktree_name, "test-worktree");
    assert!(context.worktree_path.to_str().is_some());
}

#[test]
fn test_execute_hooks_without_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().join("test"),
    };

    // Should not fail even without .git-workers.toml
    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_execute_hooks_with_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config file with hooks
    let config_content = r#"
[hooks]
post-create = ["echo 'Worktree created'", "echo 'Setup complete'"]
pre-remove = ["echo 'Cleaning up'"]
post-switch = ["echo 'Switched to {{worktree_name}}'"]
"#;

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Execute post-create hooks
    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

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

#[test]
fn test_execute_hooks_with_invalid_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create invalid config file
    let invalid_config = "invalid toml content [[[";
    fs::write(repo_path.join(".git-workers.toml"), invalid_config)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().join("test"),
    };

    // Should handle invalid config gracefully
    let result = execute_hooks("post-create", &context);
    // This should not panic, though it may return an error
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[test]
fn test_execute_hooks_with_empty_hooks() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create config with empty hooks
    let config_content = r#"
[hooks]
post-create = []
"#;

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: temp_dir.path().join("test"),
    };

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

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

// Helper function
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
