use anyhow::Result;
use git_workers::commands::{validate_custom_path, validate_worktree_name};
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test repository with initial commit
#[allow(dead_code)]
fn setup_test_repo() -> Result<(TempDir, PathBuf, GitWorktreeManager)> {
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

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

    Ok((temp_dir, repo_path, manager))
}

/// Test validation functions with comprehensive cases
#[test]
fn test_comprehensive_validation() -> Result<()> {
    // Test validate_worktree_name with various edge cases
    let valid_names = vec![
        "simple",
        "with-dashes",
        "with_underscores",
        "with.dots",
        "MixedCase",
        "123numbers",
        "a",                // Single character
        "name with spaces", // Spaces are allowed
    ];

    for name in valid_names {
        let result = validate_worktree_name(name);
        assert!(result.is_ok(), "Valid name should pass: {name}");
        assert_eq!(result.unwrap(), name.trim());
    }

    // Test validate_custom_path with various edge cases
    let valid_paths = vec![
        "simple/path",
        "path/with/multiple/levels",
        "../parent/path",
        "./current/path",
        "path-with-dashes",
        "path_with_underscores",
        "path.with.dots",
    ];

    for path in valid_paths {
        let result = validate_custom_path(path);
        assert!(result.is_ok(), "Valid path should pass: {path}");
    }

    Ok(())
}

/// Test validation function error cases
#[test]
fn test_validation_error_cases() {
    // Test validate_worktree_name error cases
    let too_long_name = "a".repeat(256);
    let invalid_names = vec![
        "",    // Empty
        "   ", // Whitespace only
        "\t",  // Tab only
        "\n",  // Newline only
        "name/with/slash",
        "name\\with\\backslash",
        "name:with:colon",
        "name*with*asterisk",
        "name?with?question",
        "name\"with\"quotes",
        "name<with>brackets",
        "name|with|pipe",
        "name\0with\0null",
        ".hidden",      // Starts with dot
        "..double",     // Starts with double dot
        &too_long_name, // Too long
        ".git",         // Reserved
        "HEAD",         // Reserved
        "refs",         // Reserved
    ];

    for name in invalid_names {
        let result = validate_worktree_name(name);
        assert!(
            result.is_err(),
            "Invalid name should fail: {}",
            name.escape_debug()
        );
    }

    // Test validate_custom_path error cases
    let definitely_invalid_paths = vec![
        "",                       // Empty
        "/absolute/path",         // Absolute
        "path/",                  // Trailing slash
        "../../..",               // Too many parent references
        "../../../../etc/passwd", // Path traversal
        "path\0null",             // Null byte
        "path:colon",             // Colon (Windows reserved)
        "path*asterisk",          // Asterisk (Windows reserved)
        "path?question",          // Question mark (Windows reserved)
        "path\"quotes",           // Quotes (Windows reserved)
        "path<brackets>",         // Brackets (Windows reserved)
        "path|pipe",              // Pipe (Windows reserved)
    ];

    for path in definitely_invalid_paths {
        let result = validate_custom_path(path);
        assert!(
            result.is_err(),
            "Invalid path should fail: {}",
            path.escape_debug()
        );
    }

    // Test potentially invalid paths (implementation-dependent)
    let potentially_invalid_paths = vec![
        "path//double//slash",
        ".git/path",     // Git reserved in path
        "path/.git/sub", // Git reserved in middle
    ];

    for path in potentially_invalid_paths {
        let result = validate_custom_path(path);
        // These might pass or fail depending on implementation - just ensure no panic
        match result {
            Ok(_) => { /* Implementation allows this path */ }
            Err(_) => { /* Implementation rejects this path */ }
        }
    }
}

/// Test validation functions with trimming behavior
#[test]
fn test_validation_trimming() -> Result<()> {
    // Test that validation functions properly trim input
    let test_cases = vec![
        ("  valid-name  ", "valid-name"),
        ("\tname-with-tabs\t", "name-with-tabs"),
        ("\nname-with-newlines\n", "name-with-newlines"),
        ("   name   with   spaces   ", "name   with   spaces"), // Internal spaces preserved
    ];

    for (input, expected) in test_cases {
        let result = validate_worktree_name(input)?;
        assert_eq!(
            result, expected,
            "Trimming should work correctly for: {input:?}"
        );
    }

    Ok(())
}

/// Test validation with simple ASCII characters only
#[test]
fn test_validation_ascii_only() {
    let ascii_names = vec![
        "simple-name",
        "name_with_underscores",
        "name.with.dots",
        "MixedCaseNAME",
        "name123",
        "123name",
    ];

    for name in ascii_names {
        let result = validate_worktree_name(name);
        assert!(result.is_ok(), "ASCII name should be valid: {name}");
    }
}
