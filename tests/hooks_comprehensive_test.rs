use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use git_workers::hooks::{execute_hooks, HookContext};

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

#[test]
fn test_execute_hooks_with_placeholders() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("my-worktree");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    // Create config with hooks using placeholders
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
        worktree_name: "my-worktree".to_string(),
        worktree_path: worktree_path.clone(),
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_execute_hooks_failing_command() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

    // Create config with a failing hook command
    let config_content = r#"
[hooks]
post-create = ["false", "echo 'This should still run'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path,
    };

    // Should not fail even if a hook command fails
    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_execute_hooks_multiple_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let worktree_path = temp_dir.path().join("test-worktree");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&worktree_path)?;

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
        worktree_path,
    };

    // Create the worktree directory for hook execution
    fs::create_dir_all(&context.worktree_path)?;

    let result = execute_hooks("post-create", &context);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_execute_hooks_unknown_hook_type() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().join("test-worktree"),
    };

    // Should handle unknown hook types gracefully
    let result = execute_hooks("unknown-hook", &context);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_hook_context_with_absolute_path() {
    let context = HookContext {
        worktree_name: "feature-xyz".to_string(),
        worktree_path: PathBuf::from("/home/user/projects/repo/worktrees/feature-xyz"),
    };

    assert_eq!(context.worktree_name, "feature-xyz");
    assert!(context.worktree_path.is_absolute());
    assert_eq!(context.worktree_path.file_name().unwrap(), "feature-xyz");
}

#[test]
fn test_hook_context_with_relative_path() {
    let context = HookContext {
        worktree_name: "bugfix".to_string(),
        worktree_path: PathBuf::from("./worktrees/bugfix"),
    };

    assert_eq!(context.worktree_name, "bugfix");
    assert!(!context.worktree_path.is_absolute());
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
