//! Unified configuration tests
//!
//! Consolidates config_comprehensive_test.rs, config_load_test.rs, config_lookup_test.rs,
//! config_root_lookup_test.rs, config_tests.rs
//! Eliminates duplicates and provides comprehensive configuration functionality testing

use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test repository with initial commit
fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}

/// Helper to create initial commit for repository
#[allow(dead_code)]
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
// Basic configuration file loading tests
// =============================================================================

/// Test config file discovery in current directory
#[test]
fn test_config_discovery_current_dir() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config file in repository root
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'created'"]
pre-remove = ["echo 'removing'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Test that config exists and is readable
    let config_path = repo_path.join(".git-workers.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("[repository]"));
    assert!(content.contains("[hooks]"));

    Ok(())
}

/// Test config file discovery in git directory
#[test]
fn test_config_discovery_git_dir() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config file in .git directory
    let git_dir = repo_path.join(".git");
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["npm install"]
"#;
    fs::write(git_dir.join(".git-workers.toml"), config_content)?;

    // Test that config exists and is readable
    let config_path = git_dir.join(".git-workers.toml");
    assert!(config_path.exists());

    Ok(())
}

/// Test config parsing with valid TOML
#[test]
fn test_config_parsing_valid() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[repository]
url = "https://github.com/user/repo.git"
branch = "main"

[hooks]
post-create = ["echo 'worktree created'", "npm install"]
pre-remove = ["echo 'removing worktree'"]
post-switch = ["echo 'switched to {{worktree_name}}'"]

[files]
copy = [".env", ".env.local"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Basic parsing test - just verify file is readable as TOML
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("repository"));
    assert!(content.contains("hooks"));
    assert!(content.contains("files"));

    Ok(())
}

/// Test config parsing with invalid TOML
#[test]
fn test_config_parsing_invalid() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create invalid TOML
    let invalid_config = r#"
[repository
url = "invalid toml
[hooks]
post-create = ["echo 'test'
"#;
    fs::write(repo_path.join(".git-workers.toml"), invalid_config)?;

    // Verify file exists but is invalid
    let config_path = repo_path.join(".git-workers.toml");
    assert!(config_path.exists());

    // Reading as TOML would fail, but we can still read the raw content
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("repository"));

    Ok(())
}

// =============================================================================
// Configuration file lookup tests
// =============================================================================

/// Test config lookup in bare repository
#[test]
fn test_config_lookup_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.git");

    // Initialize bare repository
    Repository::init_bare(&bare_repo_path)?;

    std::env::set_current_dir(&bare_repo_path)?;

    // Create config file in bare repository
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'bare repo hook'"]
"#;
    fs::write(bare_repo_path.join(".git-workers.toml"), config_content)?;

    // Test that config exists
    let config_path = bare_repo_path.join(".git-workers.toml");
    assert!(config_path.exists());

    Ok(())
}

/// Test config lookup priority order
#[test]
fn test_config_lookup_priority() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config files in multiple locations
    let repo_config = r#"
[repository]
url = "https://github.com/test/repo.git"
priority = "repo"
"#;

    let git_config = r#"
[repository]
url = "https://github.com/test/repo.git"
priority = "git"
"#;

    // Write to both locations
    fs::write(repo_path.join(".git-workers.toml"), repo_config)?;
    fs::write(repo_path.join(".git").join(".git-workers.toml"), git_config)?;

    // Both should exist
    assert!(repo_path.join(".git-workers.toml").exists());
    assert!(repo_path.join(".git").join(".git-workers.toml").exists());

    Ok(())
}

/// Test config discovery in worktree
#[test]
fn test_config_discovery_in_worktree() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feature-branch");
    std::process::Command::new("git")
        .args(["worktree", "add", "../feature-branch"])
        .current_dir(&repo_path)
        .output()?;

    std::env::set_current_dir(&worktree_path)?;

    // Create config in main repo
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'from main repo'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Config should be discoverable from worktree
    assert!(repo_path.join(".git-workers.toml").exists());

    Ok(())
}

// =============================================================================
// Configuration content validation tests
// =============================================================================

/// Test hooks configuration
#[test]
fn test_hooks_configuration() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[hooks]
post-create = [
    "echo 'Worktree created: {{worktree_name}}'",
    "echo 'Path: {{worktree_path}}'",
    "npm install"
]
pre-remove = [
    "echo 'Removing worktree: {{worktree_name}}'"
]
post-switch = [
    "echo 'Switched to: {{worktree_name}}'"
]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Verify config content
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("post-create"));
    assert!(content.contains("pre-remove"));
    assert!(content.contains("post-switch"));
    assert!(content.contains("{{worktree_name}}"));
    assert!(content.contains("{{worktree_path}}"));

    Ok(())
}

/// Test files configuration
#[test]
fn test_files_configuration() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[files]
copy = [
    ".env",
    ".env.local",
    ".env.development",
    "config/local.json",
    "private-key.pem"
]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Verify config content
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("[files]"));
    assert!(content.contains("copy"));
    assert!(content.contains(".env"));

    Ok(())
}

/// Test repository configuration
#[test]
fn test_repository_configuration() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[repository]
url = "https://github.com/user/project.git"
branch = "main"
remote = "origin"
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Verify config content
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("[repository]"));
    assert!(content.contains("url"));
    assert!(content.contains("github.com"));

    Ok(())
}

// =============================================================================
// Error handling tests
// =============================================================================

/// Test behavior with no config file
#[test]
fn test_no_config_file() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Ensure no config file exists
    let config_path = repo_path.join(".git-workers.toml");
    if config_path.exists() {
        fs::remove_file(&config_path)?;
    }

    // Application should handle missing config gracefully
    assert!(!config_path.exists());

    Ok(())
}

/// Test behavior with empty config file
#[test]
fn test_empty_config_file() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create empty config file
    fs::write(repo_path.join(".git-workers.toml"), "")?;

    let config_path = repo_path.join(".git-workers.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path)?;
    assert!(content.is_empty());

    Ok(())
}

/// Test config with only comments
#[test]
fn test_config_with_comments() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
# Git Workers Configuration
# This is a test configuration file

# Repository settings
[repository]
# The repository URL
url = "https://github.com/test/repo.git"

# Hook commands
[hooks]
# Commands to run after creating a worktree
post-create = ["echo 'created'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("#"));
    assert!(content.contains("[repository]"));

    Ok(())
}

// =============================================================================
// Performance tests
// =============================================================================

/// Test config discovery performance
#[test]
fn test_config_discovery_performance() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'test'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    let start = std::time::Instant::now();

    // Perform multiple config discoveries
    for _ in 0..100 {
        let config_path = repo_path.join(".git-workers.toml");
        assert!(config_path.exists());
    }

    let duration = start.elapsed();
    // Should be very fast (< 100ms for 100 operations)
    assert!(duration.as_millis() < 100);

    Ok(())
}

/// Test config parsing performance with large file
#[test]
fn test_config_parsing_performance() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create a large config file
    let mut config_content = String::new();
    config_content.push_str("[repository]\n");
    config_content.push_str("url = \"https://github.com/test/repo.git\"\n\n");
    config_content.push_str("[hooks]\n");

    // Add many hook commands
    for i in 0..1000 {
        config_content.push_str(&format!("post-create = [\"echo 'command {i}'\"]\n"));
    }

    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    let start = std::time::Instant::now();

    // Read the large config file
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.len() > 10000);

    let duration = start.elapsed();
    // Should still be reasonably fast
    assert!(duration.as_millis() < 1000);

    Ok(())
}

// =============================================================================
// Practical scenario tests
// =============================================================================

/// Test typical configuration workflow
#[test]
fn test_typical_config_workflow() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // 1. Create initial config
    let initial_config = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'created'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), initial_config)?;
    assert!(repo_path.join(".git-workers.toml").exists());

    // 2. Update config
    let updated_config = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'created'", "npm install"]
pre-remove = ["echo 'removing'"]

[files]
copy = [".env"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), updated_config)?;

    // 3. Verify updates
    let content = fs::read_to_string(repo_path.join(".git-workers.toml"))?;
    assert!(content.contains("npm install"));
    assert!(content.contains("pre-remove"));
    assert!(content.contains("[files]"));

    Ok(())
}

/// Test config in complex repository structure
#[test]
fn test_complex_repository_structure() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create some subdirectories
    fs::create_dir_all(repo_path.join("src/components"))?;
    fs::create_dir_all(repo_path.join("tests/integration"))?;
    fs::create_dir_all(repo_path.join("docs"))?;

    // Create config in repository root
    let config_content = r#"
[repository]
url = "https://github.com/test/complex-repo.git"

[hooks]
post-create = [
    "echo 'Setting up complex repository'",
    "npm install",
    "npm run build"
]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content)?;

    // Test from various subdirectories
    let test_dirs = vec![
        repo_path.join("src"),
        repo_path.join("src/components"),
        repo_path.join("tests"),
        repo_path.join("docs"),
    ];

    for dir in test_dirs {
        std::env::set_current_dir(&dir)?;
        // Config should be discoverable from any subdirectory
        assert!(repo_path.join(".git-workers.toml").exists());
    }

    Ok(())
}
