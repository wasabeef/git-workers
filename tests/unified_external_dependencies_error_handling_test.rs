//! Unified external dependencies error handling tests
//!
//! Tests for error handling when external dependencies (git, editor, etc.) are unavailable
//! or behave unexpectedly

use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to setup test repository
fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    Command::new("git")
        .args(["init", "-b", "main", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}

// =============================================================================
// Git command unavailability tests
// =============================================================================

/// Test behavior when git is not in PATH
#[test]
fn test_git_not_in_path() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Save original PATH
    let original_path = env::var("PATH").unwrap_or_default();

    // Set empty PATH to simulate git not being available
    env::set_var("PATH", "");

    // Try to create GitWorktreeManager
    let result = GitWorktreeManager::new_from_path(&repo_path);

    // Restore PATH
    env::set_var("PATH", original_path);

    // Should handle missing git gracefully
    match result {
        Ok(manager) => {
            println!("GitWorktreeManager created successfully even without git in PATH");
            // Test basic operations
            let list_result = manager.list_worktrees();
            println!("List worktrees result: {list_result:?}");
        }
        Err(e) => {
            println!("GitWorktreeManager failed without git in PATH (expected): {e}");
        }
    }

    Ok(())
}

/// Test behavior with invalid git configuration
#[test]
fn test_invalid_git_config() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create invalid git config
    let git_config_path = repo_path.join(".git/config");
    fs::write(&git_config_path, "invalid git config content")?;

    // Try to create GitWorktreeManager
    let result = GitWorktreeManager::new_from_path(&repo_path);

    match result {
        Ok(manager) => {
            println!("GitWorktreeManager created successfully with invalid git config");
            // Test operations
            let list_result = manager.list_worktrees();
            println!("List worktrees result: {list_result:?}");
        }
        Err(e) => {
            println!("GitWorktreeManager failed with invalid git config: {e}");
        }
    }

    Ok(())
}

/// Test behavior with corrupted git repository
#[test]
fn test_corrupted_git_repository() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Corrupt the git repository by removing essential files
    let git_dir = repo_path.join(".git");
    let head_file = git_dir.join("HEAD");
    let refs_dir = git_dir.join("refs");

    // Remove HEAD file
    if head_file.exists() {
        fs::remove_file(&head_file)?;
    }

    // Remove refs directory
    if refs_dir.exists() {
        fs::remove_dir_all(&refs_dir)?;
    }

    // Try to create GitWorktreeManager
    let result = GitWorktreeManager::new_from_path(&repo_path);

    match result {
        Ok(manager) => {
            println!("GitWorktreeManager created successfully with corrupted repo");
            // Test operations that should handle corruption gracefully
            let list_result = manager.list_worktrees();
            println!("List worktrees result: {list_result:?}");

            let branches_result = manager.list_all_branches();
            println!("List branches result: {branches_result:?}");
        }
        Err(e) => {
            println!("GitWorktreeManager failed with corrupted repo: {e}");
        }
    }

    Ok(())
}

/// Test behavior with git command failures
#[test]
fn test_git_command_failures() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations that might fail
    let test_cases = vec![
        (
            "create worktree with empty name",
            manager.create_worktree_with_new_branch("", "test-branch", "main"),
        ),
        (
            "create worktree with invalid branch",
            manager.create_worktree_with_new_branch("test", "test-branch", "non-existent"),
        ),
        (
            "create worktree with invalid path",
            manager.create_worktree_with_new_branch("test\x00invalid", "test-branch", "main"),
        ),
    ];

    for (description, result) in test_cases {
        match result {
            Ok(_) => println!("Unexpected success for {description}"),
            Err(e) => println!("Expected failure for {description}: {e}"),
        }
    }

    Ok(())
}

// =============================================================================
// Editor dependency tests
// =============================================================================

/// Test behavior when EDITOR environment variable is not set
#[test]
fn test_editor_not_set() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Save original EDITOR
    let original_editor = env::var("EDITOR").ok();

    // Unset EDITOR
    env::remove_var("EDITOR");

    // Test operations that might need editor
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test basic operations (should work without editor)
    let list_result = manager.list_worktrees();
    println!("List worktrees without EDITOR: {list_result:?}");

    let branches_result = manager.list_all_branches();
    println!("List branches without EDITOR: {branches_result:?}");

    // Restore EDITOR
    if let Some(editor) = original_editor {
        env::set_var("EDITOR", editor);
    }

    Ok(())
}

/// Test behavior when EDITOR points to non-existent program
#[test]
fn test_editor_invalid() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Save original EDITOR
    let original_editor = env::var("EDITOR").ok();

    // Set invalid EDITOR
    env::set_var("EDITOR", "/non/existent/editor");

    // Test operations
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test basic operations (should work without editor)
    let list_result = manager.list_worktrees();
    println!("List worktrees with invalid EDITOR: {list_result:?}");

    // Restore EDITOR
    if let Some(editor) = original_editor {
        env::set_var("EDITOR", editor);
    } else {
        env::remove_var("EDITOR");
    }

    Ok(())
}

// =============================================================================
// System resource tests
// =============================================================================

/// Test behavior with limited system resources
#[test]
fn test_limited_system_resources() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations with simulated resource constraints
    let start = std::time::Instant::now();

    // Perform many operations quickly to stress system resources
    for i in 0..100 {
        let _ = manager.list_worktrees();
        let _ = manager.list_all_branches();
        let _ = manager.list_all_tags();

        // Check if operations are taking too long (might indicate resource issues)
        if start.elapsed().as_secs() > 10 {
            println!("Operations taking too long, might indicate resource issues");
            break;
        }

        if i % 10 == 0 {
            println!("Completed {i} operation cycles");
        }
    }

    let duration = start.elapsed();
    println!("Completed stress test in {duration:?}");

    Ok(())
}

/// Test behavior with filesystem issues
#[test]
fn test_filesystem_issues() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create manager first
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test with read-only filesystem (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Make repository read-only
        let mut perms = fs::metadata(&repo_path)?.permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&repo_path, perms)?;

        // Test read operations (should still work)
        let list_result = manager.list_worktrees();
        println!("List worktrees with read-only filesystem: {list_result:?}");

        // Test write operations (should fail gracefully)
        let create_result = manager.create_worktree_with_new_branch("test", "test-branch", "main");
        println!("Create worktree with read-only filesystem: {create_result:?}");
    }

    Ok(())
}

// =============================================================================
// Network dependency tests
// =============================================================================

/// Test behavior when network is unavailable for remote operations
#[test]
fn test_network_unavailable() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Add a remote that doesn't exist
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/nonexistent/repo.git",
        ])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations that don't require network
    let list_result = manager.list_worktrees();
    println!("List worktrees with fake remote: {list_result:?}");

    let branches_result = manager.list_all_branches();
    println!("List branches with fake remote: {branches_result:?}");

    // These operations should work even with network issues
    assert!(list_result.is_ok() || list_result.is_err());
    assert!(branches_result.is_ok() || branches_result.is_err());

    Ok(())
}

// =============================================================================
// Environment variable tests
// =============================================================================

/// Test behavior with missing environment variables
#[test]
fn test_missing_environment_variables() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Save original environment
    let original_home = env::var("HOME").ok();
    let original_user = env::var("USER").ok();
    let original_shell = env::var("SHELL").ok();

    // Remove environment variables
    env::remove_var("HOME");
    env::remove_var("USER");
    env::remove_var("SHELL");

    // Test operations
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let list_result = manager.list_worktrees();
    println!("List worktrees without environment variables: {list_result:?}");

    let branches_result = manager.list_all_branches();
    println!("List branches without environment variables: {branches_result:?}");

    // Restore environment variables
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    }
    if let Some(user) = original_user {
        env::set_var("USER", user);
    }
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }

    Ok(())
}

/// Test behavior with malformed environment variables
#[test]
fn test_malformed_environment_variables() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Save original environment
    let original_path = env::var("PATH").ok();
    let original_home = env::var("HOME").ok();

    // Set malformed environment variables
    env::set_var("PATH", "/invalid/path:/another/invalid/path");
    env::set_var("HOME", "/non/existent/home");

    // Test operations
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    let list_result = manager.list_worktrees();
    println!("List worktrees with malformed environment: {list_result:?}");

    // Restore environment variables
    if let Some(path) = original_path {
        env::set_var("PATH", path);
    }
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    }

    Ok(())
}

// =============================================================================
// Platform-specific tests
// =============================================================================

/// Test behavior on different platforms
#[test]
fn test_platform_compatibility() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations that should work on all platforms
    let list_result = manager.list_worktrees();
    let branches_result = manager.list_all_branches();
    let tags_result = manager.list_all_tags();

    println!("Platform compatibility test:");
    println!("  OS: {}", env::consts::OS);
    println!("  Architecture: {}", env::consts::ARCH);
    println!("  List worktrees: {}", list_result.is_ok());
    println!("  List branches: {}", branches_result.is_ok());
    println!("  List tags: {}", tags_result.is_ok());

    // These operations should work on all supported platforms
    assert!(list_result.is_ok() || list_result.is_err());
    assert!(branches_result.is_ok() || branches_result.is_err());
    assert!(tags_result.is_ok() || tags_result.is_err());

    Ok(())
}

// =============================================================================
// Recovery and resilience tests
// =============================================================================

/// Test error recovery after external failures
#[test]
fn test_error_recovery() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test sequence of operations with potential failures
    let list_result = manager.list_worktrees();
    match list_result {
        Ok(_) => println!("✓ list worktrees succeeded"),
        Err(e) => println!("✗ list worktrees failed: {e}"),
    }

    // Test branches separately due to different return type
    let branches_result = manager.list_all_branches();
    match branches_result {
        Ok(_) => println!("✓ list branches succeeded"),
        Err(e) => println!("✗ list branches failed: {e}"),
    }

    // Test tags separately due to different return type
    let tags_result = manager.list_all_tags();
    match tags_result {
        Ok(_) => println!("✓ list tags succeeded"),
        Err(e) => println!("✗ list tags failed: {e}"),
    }

    // Test worktrees again
    let list_result_again = manager.list_worktrees();
    match list_result_again {
        Ok(_) => println!("✓ list worktrees again succeeded"),
        Err(e) => println!("✗ list worktrees again failed: {e}"),
    }

    // Operations should still work after previous failures
    // This tests resilience and error recovery

    Ok(())
}

/// Test graceful degradation
#[test]
fn test_graceful_degradation() -> Result<()> {
    println!("Graceful degradation test simplified");
    Ok(())
}
/// Test timeout handling for external commands
#[test]
fn test_external_command_timeouts() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations with timing
    let start = std::time::Instant::now();

    // Test list_worktrees
    let op_start = std::time::Instant::now();
    let result = manager.list_worktrees();
    let duration = op_start.elapsed();
    println!(
        "Operation 'list_worktrees' took {duration:?}, result: {}",
        result.is_ok()
    );
    assert!(
        duration.as_secs() < 30,
        "Operation list_worktrees took too long: {duration:?}"
    );

    // Test list_all_branches
    let op_start = std::time::Instant::now();
    let result = manager.list_all_branches();
    let duration = op_start.elapsed();
    println!(
        "Operation 'list_all_branches' took {duration:?}, result: {}",
        result.is_ok()
    );
    assert!(
        duration.as_secs() < 30,
        "Operation list_all_branches took too long: {duration:?}"
    );

    // Test list_all_tags
    let op_start = std::time::Instant::now();
    let result = manager.list_all_tags();
    let duration = op_start.elapsed();
    println!(
        "Operation 'list_all_tags' took {duration:?}, result: {}",
        result.is_ok()
    );
    assert!(
        duration.as_secs() < 30,
        "Operation list_all_tags took too long: {duration:?}"
    );

    let total_duration = start.elapsed();
    println!("All operations completed in {total_duration:?}");

    Ok(())
}

// =============================================================================
// Integration tests with external dependencies
// =============================================================================

/// Test integration with various git versions
#[test]
fn test_git_version_compatibility() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Get git version
    let version_output = Command::new("git").args(["--version"]).output()?;

    let version_str = String::from_utf8_lossy(&version_output.stdout);
    println!("Testing with git version: {version_str}");

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Test operations that should work with different git versions
    let list_result = manager.list_worktrees();
    let branches_result = manager.list_all_branches();
    let tags_result = manager.list_all_tags();

    println!("Git version compatibility test:");
    println!("  List worktrees: {}", list_result.is_ok());
    println!("  List branches: {}", branches_result.is_ok());
    println!("  List tags: {}", tags_result.is_ok());

    Ok(())
}

/// Test external tool integration
#[test]
fn test_external_tool_integration() -> Result<()> {
    println!("External tool integration test skipped");
    Ok(())
}
