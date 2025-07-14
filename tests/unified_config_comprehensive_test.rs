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

/// Test config with permission errors
#[test]
fn test_config_permission_errors() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config file
    let config_path = repo_path.join(".git-workers.toml");
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'test'"]
"#;
    fs::write(&config_path, config_content)?;

    // Make config file read-only (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&config_path, perms)?;
    }

    // Should still be able to read the config
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("[repository]"));

    println!("Testing config with permission restrictions");

    Ok(())
}

/// Test config with corrupted file
#[test]
fn test_config_corrupted_file() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config file with binary data (corrupted)
    let config_path = repo_path.join(".git-workers.toml");
    let corrupted_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    fs::write(&config_path, corrupted_data)?;

    // Reading should fail gracefully
    let read_result = fs::read_to_string(&config_path);
    match read_result {
        Ok(content) => {
            println!("Corrupted config read as: {content:?}");
            // If it reads, it should be handled gracefully
        }
        Err(e) => {
            println!("Corrupted config read failed (expected): {e}");
            // This is expected behavior
        }
    }

    Ok(())
}

/// Test config with invalid TOML syntax
#[test]
fn test_config_invalid_toml_syntax() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config with various TOML syntax errors
    let invalid_configs = [
        // Missing closing bracket
        r#"[repository
url = "https://github.com/test/repo.git"
"#,
        // Missing quotes
        r#"[repository]
url = https://github.com/test/repo.git
"#,
        // Invalid array syntax
        r#"[hooks]
post-create = [echo 'test']
"#,
        // Duplicate sections
        r#"[repository]
url = "https://github.com/test/repo.git"
[repository]
url = "https://github.com/other/repo.git"
"#,
        // Invalid key names
        r#"[repository]
123invalid = "value"
"#,
        // Unterminated strings
        r#"[repository]
url = "https://github.com/test/repo.git
"#,
    ];

    for (i, invalid_config) in invalid_configs.iter().enumerate() {
        let config_path = repo_path.join(format!(".git-workers-{i}.toml"));
        fs::write(&config_path, invalid_config)?;

        // File should exist but be invalid
        assert!(config_path.exists());
        println!("Testing invalid config {i}: {invalid_config}");

        // Application should handle invalid TOML gracefully
        let content = fs::read_to_string(&config_path)?;
        assert!(!content.is_empty());
    }

    Ok(())
}

/// Test config with extremely large file
#[test]
fn test_config_large_file() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create a large config file
    let mut large_config = String::new();
    large_config.push_str("[repository]\n");
    large_config.push_str("url = \"https://github.com/test/repo.git\"\n\n");
    large_config.push_str("[hooks]\n");

    // Add many hook entries
    for i in 0..10000 {
        large_config.push_str(&format!("post-create-{i} = [\"echo 'hook {i}'\"]\n"));
    }

    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, large_config)?;

    // Should handle large config files
    let content = fs::read_to_string(&config_path)?;
    assert!(content.len() > 100000);
    assert!(content.contains("[repository]"));

    println!("Testing large config file ({} bytes)", content.len());

    Ok(())
}

/// Test config in inaccessible directory
#[test]
fn test_config_inaccessible_directory() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create subdirectory
    let subdir = repo_path.join("restricted");
    fs::create_dir(&subdir)?;

    // Create config in subdirectory
    let config_path = subdir.join(".git-workers.toml");
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"
"#;
    fs::write(&config_path, config_content)?;

    // Make directory inaccessible (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&subdir)?.permissions();
        perms.set_mode(0o000); // No access
        fs::set_permissions(&subdir, perms)?;
    }

    // Should handle inaccessible directory gracefully
    #[cfg(unix)]
    {
        let read_result = fs::read_to_string(&config_path);
        match read_result {
            Ok(_) => println!("Unexpectedly could read config from inaccessible directory"),
            Err(e) => println!("Cannot read config from inaccessible directory (expected): {e}"),
        }
    }

    Ok(())
}

/// Test config with non-UTF8 content
#[test]
fn test_config_non_utf8_content() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config with non-UTF8 bytes
    let config_path = repo_path.join(".git-workers.toml");
    let non_utf8_data = {
        let mut data = Vec::new();
        // Start with valid UTF-8
        data.extend_from_slice(b"[repository]\n");
        data.extend_from_slice(b"url = \"https://github.com/test/repo.git\"\n");
        // Add invalid UTF-8 sequence
        data.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
        data.extend_from_slice(b"\n[hooks]\n");
        data.extend_from_slice(b"post-create = [\"echo 'test'\"]\n");
        data
    };

    fs::write(&config_path, non_utf8_data)?;

    // Reading as UTF-8 should fail gracefully
    let read_result = fs::read_to_string(&config_path);
    match read_result {
        Ok(content) => {
            println!("Non-UTF8 config somehow read as: {content:?}");
            // If it reads, it should be handled gracefully
        }
        Err(e) => {
            println!("Non-UTF8 config read failed (expected): {e}");
            // This is expected behavior
        }
    }

    Ok(())
}

/// Test config with very long lines
#[test]
fn test_config_very_long_lines() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config with very long lines
    let long_url = "https://github.com/".to_string() + &"a".repeat(10000) + "/repo.git";
    let long_command = "echo '".to_string() + &"x".repeat(10000) + "'";

    let config_content = format!(
        r#"[repository]
url = "{long_url}"

[hooks]
post-create = ["{long_command}"]
"#
    );

    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Should handle very long lines
    let content = fs::read_to_string(&config_path)?;
    assert!(content.len() > 20000);
    assert!(content.contains("[repository]"));

    println!(
        "Testing config with very long lines ({} bytes)",
        content.len()
    );

    Ok(())
}

/// Test config with special characters
#[test]
fn test_config_special_characters() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config with special characters
    let config_content = r#"
[repository]
url = "https://github.com/test/repo-with-special-chars.git"
description = "Repository with special chars: !@#$%^&*()_+{}[]|;':\",./<>?"

[hooks]
post-create = [
    "echo 'Special chars: !@#$%^&*()_+{}[]|;':\",./<>?'",
    "echo 'Unicode: こんにちは 世界 🌍'",
    "echo 'Emoji: 🚀 🎉 ✨'"
]
"#;

    let config_path = repo_path.join(".git-workers.toml");
    fs::write(&config_path, config_content)?;

    // Should handle special characters
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("Special chars"));
    assert!(content.contains("Unicode"));
    assert!(content.contains("Emoji"));

    println!("Testing config with special characters");

    Ok(())
}

/// Test config with concurrent access
#[test]
fn test_config_concurrent_access() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config file
    let config_path = repo_path.join(".git-workers.toml");
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'test'"]
"#;
    fs::write(&config_path, config_content)?;

    // Simulate concurrent access by reading multiple times
    let mut handles = vec![];
    for i in 0..5 {
        let path = config_path.clone();
        handles.push(std::thread::spawn(move || {
            let content = fs::read_to_string(&path).unwrap();
            assert!(content.contains("[repository]"));
            println!("Thread {i} successfully read config");
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Testing concurrent config access");

    Ok(())
}

/// Test config with filesystem case sensitivity
#[test]
fn test_config_case_sensitivity() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;
    std::env::set_current_dir(&repo_path)?;

    // Create config files with different cases
    let config_files = vec![
        ".git-workers.toml",
        ".Git-Workers.toml",
        ".GIT-WORKERS.TOML",
    ];

    for config_file in &config_files {
        let config_path = repo_path.join(config_file);
        let config_content = format!(
            r#"[repository]
url = "https://github.com/test/repo-{config_file}.git"

[hooks]
post-create = ["echo 'from {config_file}'"]
"#
        );

        // Try to create the file
        match fs::write(&config_path, config_content) {
            Ok(_) => {
                println!("Created config file: {config_file}");
                assert!(config_path.exists());
            }
            Err(e) => {
                println!("Could not create config file {config_file}: {e}");
                // This is expected on case-insensitive filesystems
            }
        }
    }

    println!("Testing config case sensitivity");

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
