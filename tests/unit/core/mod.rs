//! Unit tests for core business logic
//!
//! Tests for validation, configuration, utilities, and other core functionality.

mod constants;
mod utils;

use anyhow::Result;
use git_workers::config::{Config, FilesConfig};
use git_workers::constants as gw_constants;

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_default() {
    let config = Config::default();
    assert!(config.repository.url.is_none());
    assert!(config.hooks.is_empty());
    let default_files = FilesConfig::default();
    assert_eq!(config.files.copy, default_files.copy);
}

#[test]
fn test_config_from_toml() -> Result<()> {
    let toml_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["npm install", "cp .env.example .env"]
pre-remove = ["rm -rf node_modules"]

[files]
copy = [".env", "config/local.json"]
"#;

    let config: Config = toml::from_str(toml_content)?;

    // Verify repository config
    assert!(config.repository.url.is_some());
    assert_eq!(
        config.repository.url,
        Some("https://github.com/test/repo.git".to_string())
    );

    // Verify hooks config
    assert_eq!(config.hooks.get("post-create").unwrap().len(), 2);
    assert_eq!(config.hooks.get("pre-remove").unwrap().len(), 1);

    // Verify files config
    assert_eq!(config.files.copy.len(), 2);

    Ok(())
}

#[test]
fn test_config_partial() -> Result<()> {
    let toml_content = r#"
[repository]
url = "https://github.com/test/repo.git"
"#;

    let config: Config = toml::from_str(toml_content)?;
    assert!(config.repository.url.is_some());
    assert!(config.hooks.is_empty());
    assert!(config.files.copy.is_empty());

    Ok(())
}

// ============================================================================
// File Copy Tests
// ============================================================================

#[test]
fn test_file_copy_placeholder() -> Result<()> {
    // TODO: File copy API has changed significantly
    // These tests need to be rewritten with new API
    Ok(())
}

// ============================================================================
// Constants Validation Tests
// ============================================================================

#[test]
fn test_constants_not_empty() {
    // Ensure all constants have values
    assert!(!gw_constants::EMOJI_HOME.is_empty());
    assert!(!gw_constants::EMOJI_LOCKED.is_empty());
    assert!(!gw_constants::EMOJI_BRANCH.is_empty());
    assert!(!gw_constants::EMOJI_DETACHED.is_empty());
    assert!(!gw_constants::EMOJI_FOLDER.is_empty());
}

#[test]
fn test_constants_messages() {
    assert!(!gw_constants::MSG_CREATING_FIRST_WORKTREE.is_empty());
    assert!(!gw_constants::INFO_OPERATION_CANCELLED.is_empty());
}

// ============================================================================
// Utility Function Tests
// ============================================================================

#[test]
fn test_error_formatting() {
    let error = anyhow::anyhow!("Test error");
    let error_string = error.to_string();
    assert!(error_string.contains("Test error"));
}

#[test]
fn test_error_context_formatting() {
    let error = anyhow::anyhow!("Root cause")
        .context("Middle layer")
        .context("Top layer");
    let error_string = error.to_string();
    assert!(error_string.contains("Top layer"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_chain_display() {
    let base_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error = anyhow::Error::new(base_error)
        .context("Failed to read configuration")
        .context("Failed to initialize worktree");

    let error_string = error.to_string();
    assert!(error_string.contains("Failed to initialize worktree"));
    // Check if the error chain contains the expected context
    assert!(
        error_string.contains("Failed to")
            || error_string.contains("configuration")
            || error_string.contains("initialize")
    );
}
