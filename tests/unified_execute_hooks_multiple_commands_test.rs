use anyhow::Result;
use git2::{Repository, Signature};
use std::fs;
use tempfile::TempDir;

use git_workers::hooks::{execute_hooks, HookContext};

/// Helper function to create initial commit
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    // Setup repository with git2
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create worktree directory
    fs::create_dir_all(&worktree_path)?;

    // Create hook context
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test configuration with multiple commands
    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'", 
    "echo 'Third command'"
]
"#;

    // Change to repo directory and write config file there
    std::env::set_current_dir(&repo_path)?;
    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute hooks - should succeed
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Multiple hook commands should execute successfully"
    );

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_with_git_cli() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Setup repository with git CLI commands (like public API test)
    std::env::set_current_dir(&temp_dir)?;

    std::process::Command::new("git").args(["init"]).output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .output()?;

    // Use temp directory as worktree path (simpler structure)
    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Test configuration with multiple commands
    let config_content = r#"
[hooks]
post-create = [
    "echo 'First command'",
    "echo 'Second command'",
    "echo 'Third command'"
]
"#;

    // Write config file (we're already in temp_dir)
    let config_path = temp_dir.path().join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute hooks - should succeed
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Multiple hook commands should execute successfully with git CLI setup"
    );

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_different_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    // Setup repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test configuration with different command types
    let config_content = r#"
[hooks]
post-create = [
    "echo 'Echo command'",
    "pwd",
    "ls -la"
]
"#;

    std::env::set_current_dir(&repo_path)?;
    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute hooks with mixed command types
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Mixed command types should execute successfully"
    );

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_with_template_variables() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("feature-branch");

    // Setup repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    let context = HookContext {
        worktree_name: "feature-branch".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test configuration with template variables
    let config_content = r#"
[hooks]
post-create = [
    "echo 'Creating worktree: {{worktree_name}}'",
    "echo 'Path: {{worktree_path}}'",
    "echo 'Done with {{worktree_name}}'"
]
"#;

    std::env::set_current_dir(&repo_path)?;
    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute hooks with template variables
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Commands with template variables should execute successfully"
    );

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_empty_array() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    // Setup repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test configuration with empty commands array
    let config_content = r#"
[hooks]
post-create = []
"#;

    std::env::set_current_dir(&repo_path)?;
    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute empty hooks array - should succeed (no-op)
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Empty hooks array should execute successfully"
    );

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands_nonexistent_hook() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    // Setup repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Test configuration without the requested hook
    let config_content = r#"
[hooks]
post-remove = [
    "echo 'This is a different hook'"
]
"#;

    std::env::set_current_dir(&repo_path)?;
    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Execute non-existent hook - should succeed (no-op)
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Non-existent hook should execute successfully (no-op)"
    );

    Ok(())
}
