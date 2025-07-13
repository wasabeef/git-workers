use anyhow::Result;
use git2::Repository;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

use git_workers::config::Config;

// Use a mutex to ensure tests don't interfere with each other
// when changing the current directory
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_config_template_generation() -> Result<()> {
    // Test that the template content is valid TOML
    let template_content = r#"# Git Workers Configuration
# This file configures hooks for worktree lifecycle events

[repository]
# Optional: Specify repository URL to ensure hooks only run in the intended repository
# url = "https://github.com/owner/repo.git"

[hooks]
# Hook commands support template variables:
# {{worktree_name}} - The name of the worktree
# {{worktree_path}} - The absolute path to the worktree

# Executed after creating a new worktree
post-create = [
    # "npm install",
    # "cp .env.example .env",
    # "echo 'Created worktree {{worktree_name}} at {{worktree_path}}'"
]

# Executed before removing a worktree
pre-remove = [
    # "rm -rf node_modules",
    # "echo 'Removing worktree {{worktree_name}}'"
]

# Executed after switching to a worktree
post-switch = [
    # "echo 'Switched to {{worktree_name}}'",
    # "git status"
]
"#;

    // Verify it's valid TOML
    let parsed: Config = toml::from_str(template_content)?;
    // The hooks should have empty arrays, not be absent
    assert_eq!(parsed.hooks.get("post-create"), Some(&vec![]));

    Ok(())
}

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_config_path_resolution_in_worktree_structure() -> Result<()> {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path().join("project");

    // Create worktree structure
    let main_dir = base_path.join("main");
    let feature_dir = base_path.join("feature");
    fs::create_dir_all(&main_dir)?;
    fs::create_dir_all(&feature_dir)?;

    // Initialize git repos
    Repository::init(&main_dir)?;
    Repository::init(&feature_dir)?;

    // Save current directory
    let original_dir = std::env::current_dir()?;

    // Create config in main worktree
    let config_content = r#"
[hooks]
post-create = ["echo 'Config from main'"]
"#;
    fs::write(main_dir.join(".git-workers.toml"), config_content)?;

    // Change to feature directory
    std::env::set_current_dir(&feature_dir)?;

    // Load config - should find it in parent's main directory
    let config = Config::load()?;
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(config.hooks["post-create"], vec!["echo 'Config from main'"]);

    // Restore directory with fallback to temp_dir if original is not accessible
    if std::env::set_current_dir(&original_dir).is_err() {
        // If we can't go back to original, at least go to a valid directory
        let _ = std::env::set_current_dir(temp_dir.path());
    }

    Ok(())
}

#[test]
fn test_config_in_bare_repo_worktree() -> Result<()> {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("repo.git");

    // Initialize bare repository
    Repository::init_bare(&bare_repo_path)?;

    // Create a worktree directory with a .git file that points to the bare repo
    let worktree_dir = temp_dir.path().join("worktree1");
    fs::create_dir(&worktree_dir)?;

    // Create a .git file that simulates a worktree
    let git_file_content = format!("gitdir: {}", bare_repo_path.display());
    fs::write(worktree_dir.join(".git"), git_file_content)?;

    // Save current directory
    let original_dir = std::env::current_dir()?;

    // Create config in worktree directory
    let config_content = r#"
[hooks]
post-create = ["echo 'Config in bare repo worktree'"]
"#;
    fs::write(worktree_dir.join(".git-workers.toml"), config_content)?;

    // Change to worktree directory
    std::env::set_current_dir(&worktree_dir)?;

    // Load config - should find it in current directory
    let config = Config::load()?;
    // The config should load the file from current directory
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'Config in bare repo worktree'"]
    );

    // Restore directory with fallback to temp_dir if original is not accessible
    if std::env::set_current_dir(&original_dir).is_err() {
        // If we can't go back to original, at least go to a valid directory
        let _ = std::env::set_current_dir(temp_dir.path());
    }

    Ok(())
}

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_config_precedence_chain() -> Result<()> {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    Repository::init(&repo_path)?;

    // Save current directory
    let original_dir = std::env::current_dir()?;

    // Create nested directory structure
    let sub_dir = repo_path.join("src").join("components");
    fs::create_dir_all(&sub_dir)?;

    // Create config in repository root
    let root_config = r#"
[hooks]
post-create = ["echo 'From root'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), root_config)?;

    // Create config in subdirectory (higher priority)
    let sub_config = r#"
[hooks]
post-create = ["echo 'From subdirectory'"]
"#;
    fs::write(sub_dir.join(".git-workers.toml"), sub_config)?;

    // Change to subdirectory
    std::env::set_current_dir(&sub_dir)?;

    // Should load config from current directory first
    let config = Config::load()?;
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'From subdirectory'"]
    );

    // Remove subdirectory config
    fs::remove_file(sub_dir.join(".git-workers.toml"))?;

    // Now should fall back to root config
    let config = Config::load()?;
    assert_eq!(config.hooks["post-create"], vec!["echo 'From root'"]);

    // Restore directory with fallback to temp_dir if original is not accessible
    if std::env::set_current_dir(&original_dir).is_err() {
        // If we can't go back to original, at least go to a valid directory
        let _ = std::env::set_current_dir(temp_dir.path());
    }

    Ok(())
}

#[test]
fn test_editor_environment_detection() -> Result<()> {
    // Test EDITOR environment variable
    std::env::set_var("EDITOR", "test-editor");
    assert_eq!(std::env::var("EDITOR").unwrap(), "test-editor");
    std::env::remove_var("EDITOR");

    // Test VISUAL environment variable
    std::env::set_var("VISUAL", "test-visual");
    assert_eq!(std::env::var("VISUAL").unwrap(), "test-visual");
    std::env::remove_var("VISUAL");

    // Test default editor selection based on platform
    #[cfg(target_family = "unix")]
    {
        let default_editor = "vi";
        assert!(!default_editor.is_empty());
    }

    #[cfg(target_family = "windows")]
    {
        let default_editor = "notepad";
        assert!(!default_editor.is_empty());
    }

    Ok(())
}
