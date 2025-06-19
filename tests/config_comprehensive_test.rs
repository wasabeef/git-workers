use anyhow::Result;
use git2::Repository;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

use git_workers::config::Config;

#[test]
fn test_config_load_from_different_locations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Test loading from repo directory
    std::env::set_current_dir(&repo_path)?;

    // Test loading without config file (should return default)
    let default_config = Config::load()?;
    assert!(default_config.hooks.is_empty());

    Ok(())
}

#[test]
fn test_config_with_complex_hooks() -> Result<()> {
    // Test parsing complex config content directly
    let config_content = r#"
[hooks]
post-create = [
    "echo 'Creating worktree'",
    "npm install"
]
pre-remove = [
    "echo 'Removing'",
    "rm -rf node_modules"
]
"#;

    let config: Config = toml::from_str(config_content)?;

    // Verify hooks were parsed correctly
    assert!(!config.hooks.is_empty());
    assert!(config.hooks.contains_key("post-create"));
    assert!(config.hooks.contains_key("pre-remove"));

    // Check hook contents
    let post_create_hooks = config.hooks.get("post-create").unwrap();
    assert_eq!(post_create_hooks.len(), 2);
    assert!(post_create_hooks[0].contains("Creating worktree"));
    assert!(post_create_hooks[1].contains("npm install"));

    let pre_remove_hooks = config.hooks.get("pre-remove").unwrap();
    assert_eq!(pre_remove_hooks.len(), 2);

    Ok(())
}

#[test]
fn test_config_with_empty_hooks() -> Result<()> {
    // Test parsing config with empty hook arrays directly
    let config_content = r#"
[hooks]
post-create = []
pre-remove = []
"#;

    let config: Config = toml::from_str(config_content)?;

    assert!(config.hooks.contains_key("post-create"));
    assert!(config.hooks.contains_key("pre-remove"));
    assert!(config.hooks.get("post-create").unwrap().is_empty());
    assert!(config.hooks.get("pre-remove").unwrap().is_empty());

    Ok(())
}

#[test]
fn test_config_with_no_hooks_section() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[other_section]
key = "value"
"#;

    fs::write(".git-workers.toml", config_content)?;

    let config = Config::load()?;
    assert!(config.hooks.is_empty());

    Ok(())
}

#[test]
fn test_config_struct_creation() {
    // Test Config struct can be created manually
    let mut hooks = HashMap::new();
    hooks.insert("test-hook".to_string(), vec!["echo test".to_string()]);

    let config = Config {
        repository: git_workers::config::RepositoryConfig::default(),
        hooks,
    };

    assert!(!config.hooks.is_empty());
    assert!(config.hooks.contains_key("test-hook"));
    assert_eq!(config.hooks.get("test-hook").unwrap()[0], "echo test");
}

#[test]
fn test_config_load_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo-error"); // Unique name

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;
    std::env::set_current_dir(&repo_path)?;

    // Create invalid TOML
    let invalid_config = "invalid toml content [[[";
    fs::write(".git-workers.toml", invalid_config)?;

    // Should handle invalid TOML gracefully
    let result = Config::load();
    // Either succeeds with default config or returns an error
    match result {
        Ok(config) => {
            // If it succeeds, should be default empty config
            assert!(config.hooks.is_empty());
        }
        Err(_) => {
            // Error is also acceptable for invalid TOML
        }
    }

    Ok(())
}

#[test]
fn test_config_with_special_characters() -> Result<()> {
    // Test parsing config with special characters directly
    let config_content = r#"
[hooks]
post-create = [
    "echo 'Special chars: àáâãäåæçèéêë'",
    "echo 'Symbols: !@#$%^&*()_+-=[]{}|;:,.<>?'",
    "echo 'Unicode: 🎉 ✅ ❌ 🔍'"
]
"#;

    let config: Config = toml::from_str(config_content)?;
    let hooks = config.hooks.get("post-create").unwrap();

    assert!(hooks[0].contains("Special chars"));
    assert!(hooks[1].contains("Symbols"));
    assert!(hooks[2].contains("Unicode"));

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
