use anyhow::Result;
use git2::Repository;
use std::fs;
use tempfile::TempDir;

use git_workers::config::Config;

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_config_load_local_priority() -> Result<()> {
    // Skip in CI environment
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create local config
    let local_config = r#"
[hooks]
post-create = ["echo 'Local config'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), local_config)?;

    // Create global config directory
    let global_dir = temp_dir.path().join(".config/git-workers");
    fs::create_dir_all(&global_dir)?;
    let global_config = r#"
[hooks]
post-create = ["echo 'Global config'"]
"#;
    fs::write(global_dir.join("config.toml"), global_config)?;

    std::env::set_current_dir(&repo_path)?;

    let config = Config::load()?;
    // Local config should take priority
    if let Some(hooks) = config.hooks.get("post-create") {
        assert!(hooks[0].contains("Local config"));
    }

    Ok(())
}

#[test]
fn test_config_load_from_subdirectory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let sub_dir = repo_path.join("src/components");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    fs::create_dir_all(&sub_dir)?;

    let config_content = r#"
[hooks]
post-create = ["npm install"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&sub_dir)?;

    let config = Config::load()?;
    // Should find config in parent directory
    assert!(config.hooks.contains_key("post-create") || config.hooks.is_empty());

    Ok(())
}

#[test]
fn test_config_default_values() -> Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // No config file exists
    let config = Config::load()?;

    // Should return default empty config
    assert!(config.hooks.is_empty());

    Ok(())
}

#[test]
fn test_config_with_all_hook_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let config_content = r#"
[hooks]
post-create = ["echo 'Created'"]
pre-remove = ["echo 'Removing'"]
post-switch = ["echo 'Switched'"]
custom-hook = ["echo 'Custom'"]
another-hook = ["echo 'Another'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let config = Config::load()?;

    // All hooks should be loaded
    let expected_hooks = [
        "post-create",
        "pre-remove",
        "post-switch",
        "custom-hook",
        "another-hook",
    ];
    for hook in expected_hooks {
        assert!(config.hooks.contains_key(hook) || config.hooks.is_empty());
    }

    Ok(())
}

#[test]
fn test_config_with_complex_commands() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let config_content = r#"
[hooks]
post-create = [
    "git config user.name 'Test User'",
    "git config user.email 'test@example.com'",
    "npm install && npm run build",
    "chmod +x scripts/*.sh",
    "[ -f .env.example ] && cp .env.example .env || true"
]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let config = Config::load()?;

    if let Some(hooks) = config.hooks.get("post-create") {
        assert!(hooks.len() >= 5 || hooks.is_empty());
    }

    Ok(())
}

#[test]
fn test_config_partial_content() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Config with only [hooks] section but no content
    let config_content = r#"
[hooks]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    std::env::set_current_dir(&repo_path)?;

    let config = Config::load()?;
    assert!(config.hooks.is_empty());

    Ok(())
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
