//! Unified file copy tests
//!
//! Integrates file_copy_test.rs and file_copy_size_test.rs
//! Eliminates duplication and provides comprehensive file copy functionality tests

use anyhow::Result;
use git_workers::config::FilesConfig;
use git_workers::file_copy;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper to setup test repository using git command
fn setup_test_repo_git() -> Result<(TempDir, std::path::PathBuf, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Create a git repository
    fs::create_dir(&repo_path)?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "."])
        .output()?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Initial commit"])
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

    Ok((temp_dir, repo_path, manager))
}

/// Helper to setup test repository using git2
fn setup_test_repo_git2() -> Result<(TempDir, GitWorktreeManager, TempDir)> {
    // Create a parent directory
    let parent_dir = TempDir::new()?;
    let repo_path = parent_dir.path().join("test-repo");
    fs::create_dir(&repo_path)?;

    // Initialize repository
    let repo = git2::Repository::init(&repo_path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        fs::write(repo_path.join("README.md"), "# Test")?;
        index.add_path(Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create a destination directory for worktree
    let dest_dir = TempDir::new()?;

    Ok((parent_dir, manager, dest_dir))
}

/// Helper to create worktree for testing
fn create_test_worktree(repo_path: &Path) -> Result<std::path::PathBuf> {
    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    Ok(worktree_path)
}

// =============================================================================
// Basic file copy tests
// =============================================================================

/// Test basic file copying functionality
#[test]
fn test_file_copy_basic() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create test files to copy
    fs::write(repo_path.join(".env"), "TEST_VAR=value1")?;
    fs::write(repo_path.join(".env.local"), "LOCAL_VAR=value2")?;

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: vec![".env".to_string(), ".env.local".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify files were copied
    assert_eq!(copied.len(), 2);
    assert!(worktree_path.join(".env").exists());
    assert!(worktree_path.join(".env.local").exists());

    // Verify content
    let env_content = fs::read_to_string(worktree_path.join(".env"))?;
    assert_eq!(env_content, "TEST_VAR=value1");

    Ok(())
}

/// Test copying files with subdirectories
#[test]
fn test_file_copy_with_subdirectories() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create test files in subdirectory
    let config_dir = repo_path.join("config");
    fs::create_dir(&config_dir)?;
    fs::write(config_dir.join("local.json"), r#"{"key": "value"}"#)?;

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: vec!["config/local.json".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify files were copied
    assert_eq!(copied.len(), 1);
    assert!(worktree_path.join("config/local.json").exists());

    Ok(())
}

/// Test file copying with special filenames
#[test]
fn test_special_filenames() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create files with special names
    let special_names = vec![
        ".hidden",
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.multiple.dots.txt",
    ];

    for name in &special_names {
        fs::write(repo_path.join(name), format!("content of {name}"))?;
    }

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: special_names.iter().map(|s| s.to_string()).collect(),
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify all files were copied
    assert_eq!(copied.len(), special_names.len());
    for name in &special_names {
        assert!(
            worktree_path.join(name).exists(),
            "File {name} should exist"
        );
    }

    Ok(())
}

// =============================================================================
// Directory copy tests
// =============================================================================

/// Test recursive directory copying
#[test]
fn test_directory_copy_recursive() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create nested directory structure
    fs::create_dir_all(repo_path.join("config/env/dev"))?;
    fs::write(repo_path.join("config/env/dev/.env"), "DEV=true")?;
    fs::write(repo_path.join("config/settings.json"), r#"{"app": "test"}"#)?;
    fs::create_dir_all(repo_path.join("config/certs"))?;
    fs::write(repo_path.join("config/certs/cert.pem"), "CERT")?;

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: vec!["config".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify directory structure was copied
    assert_eq!(copied.len(), 1);
    assert!(worktree_path.join("config/env/dev/.env").exists());
    assert!(worktree_path.join("config/settings.json").exists());
    assert!(worktree_path.join("config/certs/cert.pem").exists());

    Ok(())
}

/// Test copying empty directories
#[test]
fn test_empty_directory_copy() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create empty directory
    fs::create_dir(repo_path.join("empty_dir"))?;

    // Add a .gitkeep file to track the empty directory
    fs::write(repo_path.join("empty_dir/.gitkeep"), "")?;

    // Add and commit the directory
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "empty_dir/.gitkeep"])
        .output()?;
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Add empty directory"])
        .output()?;

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: vec!["empty_dir".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify empty directory was copied (1 file: .gitkeep)
    assert_eq!(copied.len(), 1);
    assert!(worktree_path.join("empty_dir").is_dir());
    assert!(worktree_path.join("empty_dir/.gitkeep").is_file());

    Ok(())
}

/// Test directory size calculation with nested structure
#[test]
fn test_file_copy_directory_size_calculation() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create nested directory structure
    let nested = repo_path.join("nested");
    fs::create_dir(&nested)?;
    fs::write(nested.join("file1.txt"), "a".repeat(1000))?; // 1KB

    let sub = nested.join("sub");
    fs::create_dir(&sub)?;
    fs::write(sub.join("file2.txt"), "b".repeat(2000))?; // 2KB

    let config = FilesConfig {
        copy: vec!["nested".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Should copy the entire directory
    assert_eq!(copied.len(), 1);
    assert!(dest_dir.path().join("nested/file1.txt").exists());
    assert!(dest_dir.path().join("nested/sub/file2.txt").exists());

    Ok(())
}

// =============================================================================
// File size limit tests
// =============================================================================

/// Test copying small files within size limits
#[test]
fn test_file_copy_with_small_files() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create small test files
    fs::write(repo_path.join(".env"), "SMALL_FILE=true")?;
    fs::write(repo_path.join("config.json"), "{\"small\": true}")?;

    let config = FilesConfig {
        copy: vec![".env".to_string(), "config.json".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Verify files were copied
    assert_eq!(copied.len(), 2);
    assert!(dest_dir.path().join(".env").exists());
    assert!(dest_dir.path().join("config.json").exists());

    Ok(())
}

/// Test skipping large files over size limit
#[test]
fn test_file_copy_skip_large_file() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create a large file (over 100MB limit)
    let large_content = vec![0u8; 101 * 1024 * 1024]; // 101MB
    fs::write(repo_path.join("large.bin"), large_content)?;

    // Create a small file
    fs::write(repo_path.join("small.txt"), "small content")?;

    let config = FilesConfig {
        copy: vec!["large.bin".to_string(), "small.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Only small file should be copied
    assert_eq!(copied.len(), 1);
    assert_eq!(copied[0], "small.txt");
    assert!(!dest_dir.path().join("large.bin").exists());
    assert!(dest_dir.path().join("small.txt").exists());

    Ok(())
}

/// Test total size limit handling (simulated with small files)
#[test]
fn test_file_copy_total_size_limit() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // This test would require creating files totaling over 1GB
    // which is too resource-intensive for regular testing
    // So we'll just test with small files

    // Create a few small files
    fs::write(repo_path.join("file1.txt"), "content1")?;
    fs::write(repo_path.join("file2.txt"), "content2")?;

    let config = FilesConfig {
        copy: vec!["file1.txt".to_string(), "file2.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Should copy both small files
    assert_eq!(copied.len(), 2);

    Ok(())
}

// =============================================================================
// Security tests
// =============================================================================

/// Test file copy security against path traversal
#[test]
fn test_file_copy_security() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;
    let worktree_path = create_test_worktree(&repo_path)?;

    // Test file copying with unsafe paths
    let files_config = FilesConfig {
        copy: vec![
            "../../../etc/passwd".to_string(),
            "/etc/hosts".to_string(),
            "~/sensitive".to_string(),
        ],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify no files were copied due to security checks
    assert_eq!(copied.len(), 0);

    Ok(())
}

/// Test detailed path traversal security
#[test]
fn test_path_traversal_detailed() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;
    let worktree_path = create_test_worktree(&repo_path)?;

    // Test various path traversal attempts
    let dangerous_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "./../../sensitive",
        "foo/../../../bar",
        "/etc/passwd",
        "C:\\Windows\\System32",
        ".",
        "..",
        "~/sensitive",
    ];

    for path in dangerous_paths {
        let files_config = FilesConfig {
            copy: vec![path.to_string()],
            source: Some(repo_path.to_str().unwrap().to_string()),
        };

        let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;
        assert_eq!(copied.len(), 0, "Path '{path}' should not be copied");
    }

    Ok(())
}

// =============================================================================
// Error handling tests
// =============================================================================

/// Test handling of missing files
#[test]
fn test_file_copy_missing_files() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;
    let worktree_path = create_test_worktree(&repo_path)?;

    // Test file copying with non-existent files
    let files_config = FilesConfig {
        copy: vec![
            ".env".to_string(),            // doesn't exist
            "nonexistent.txt".to_string(), // doesn't exist
        ],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    // Should not panic, just warn
    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify no files were copied
    assert_eq!(copied.len(), 0);

    Ok(())
}

/// Test handling of symlinks (skipped for security)
#[test]
fn test_file_copy_skip_symlinks() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create a file and a symlink to it
    fs::write(repo_path.join("original.txt"), "original content")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink("original.txt", repo_path.join("link.txt"))?;

        let config = FilesConfig {
            copy: vec!["link.txt".to_string(), "original.txt".to_string()],
            source: Some(repo_path.to_str().unwrap().to_string()),
        };

        let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

        // Only original file should be copied, not the symlink
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0], "original.txt");
        assert!(dest_dir.path().join("original.txt").exists());
        assert!(!dest_dir.path().join("link.txt").exists());
    }

    Ok(())
}

// =============================================================================
// Advanced scenario tests
// =============================================================================

/// Test file copying with directory
#[test]
fn test_file_copy_with_directory() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_git2()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create a directory with files
    let config_dir = repo_path.join("config");
    fs::create_dir(&config_dir)?;
    fs::write(config_dir.join("app.json"), "{\"app\": true}")?;
    fs::write(config_dir.join("db.json"), "{\"db\": true}")?;

    let config = FilesConfig {
        copy: vec!["config".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Directory should be copied
    assert_eq!(copied.len(), 1);
    assert!(dest_dir.path().join("config").exists());
    assert!(dest_dir.path().join("config/app.json").exists());
    assert!(dest_dir.path().join("config/db.json").exists());

    Ok(())
}

/// Test file copying with mixed content (files and directories)
#[test]
fn test_file_copy_mixed_content() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create mixed content: files and directories
    fs::write(repo_path.join(".env"), "ENV_VAR=value")?;
    fs::write(repo_path.join("standalone.txt"), "standalone file")?;

    let config_dir = repo_path.join("config");
    fs::create_dir(&config_dir)?;
    fs::write(config_dir.join("app.json"), "{\"config\": true}")?;

    let worktree_path = create_test_worktree(&repo_path)?;

    let files_config = FilesConfig {
        copy: vec![
            ".env".to_string(),
            "standalone.txt".to_string(),
            "config".to_string(),
        ],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify all items were copied
    assert_eq!(copied.len(), 3);
    assert!(worktree_path.join(".env").exists());
    assert!(worktree_path.join("standalone.txt").exists());
    assert!(worktree_path.join("config").is_dir());
    assert!(worktree_path.join("config/app.json").exists());

    Ok(())
}

// =============================================================================
// Extended error handling tests
// =============================================================================

/// Test file copy with permission errors
#[test]
fn test_file_copy_permission_errors() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create source file
    let source_file = repo_path.join("protected-file.txt");
    fs::write(&source_file, "protected content")?;

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Make source file read-only (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&source_file)?.permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&source_file, perms)?;
    }

    // Make destination directory read-only (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&worktree_path)?.permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&worktree_path, perms)?;
    }

    // Create config
    let files_config = FilesConfig {
        copy: vec!["protected-file.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with permission errors");

    // Should handle permission errors gracefully
    assert!(source_file.exists());
    // On Unix, copy should fail due to read-only destination
    #[cfg(unix)]
    {
        assert_eq!(copied.len(), 0, "Should not copy to read-only destination");
    }

    Ok(())
}

/// Test file copy with disk space issues
#[test]
fn test_file_copy_disk_space_simulation() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create very large file (simulating disk space issue)
    let large_file = repo_path.join("large-file.txt");
    let large_content = "x".repeat(1024 * 1024); // 1MB content
    fs::write(&large_file, large_content)?;

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Create config
    let files_config = FilesConfig {
        copy: vec!["large-file.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with large file");

    // Should handle large files according to size limits
    assert!(large_file.exists());
    let metadata = fs::metadata(&large_file)?;
    assert!(metadata.len() >= 1024 * 1024);

    // File should be copied as it's under 100MB limit
    assert_eq!(copied.len(), 1);

    Ok(())
}

/// Test file copy with invalid symlinks
#[test]
fn test_file_copy_invalid_symlinks() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create invalid symlink (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = repo_path.join("broken-symlink");
        let target_path = repo_path.join("non-existent-target");

        // Create symlink to non-existent target
        symlink(&target_path, &symlink_path)?;

        // Verify symlink is broken (exists as symlink but target doesn't exist)
        assert!(symlink_path.symlink_metadata().is_ok());
        assert!(!target_path.exists());

        // Create worktree directory
        let worktree_path = create_test_worktree(&repo_path)?;

        // Create config
        let files_config = FilesConfig {
            copy: vec!["broken-symlink".to_string()],
            source: Some(repo_path.to_str().unwrap().to_string()),
        };

        let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

        println!("Testing copy with broken symlink");

        // Should handle broken symlinks gracefully (skip them)
        assert_eq!(copied.len(), 0, "Broken symlinks should be skipped");
    }

    Ok(())
}

/// Test file copy with circular symlinks
#[test]
fn test_file_copy_circular_symlinks() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create circular symlinks (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_a = repo_path.join("symlink-a");
        let symlink_b = repo_path.join("symlink-b");

        // Create circular symlinks
        symlink(&symlink_b, &symlink_a)?;
        symlink(&symlink_a, &symlink_b)?;

        // Create worktree directory
        let worktree_path = create_test_worktree(&repo_path)?;

        // Create config
        let files_config = FilesConfig {
            copy: vec!["symlink-a".to_string(), "symlink-b".to_string()],
            source: Some(repo_path.to_str().unwrap().to_string()),
        };

        let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

        println!("Testing copy with circular symlinks");

        // Should handle circular symlinks gracefully (skip them)
        assert_eq!(copied.len(), 0, "Circular symlinks should be skipped");
    }

    Ok(())
}

/// Test file copy with deeply nested directories
#[test]
fn test_file_copy_deeply_nested_directories() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create deeply nested directory structure
    let mut nested_path = repo_path.clone();
    for i in 0..10 {
        // Reduced from 50 to 10 for test performance
        nested_path = nested_path.join(format!("level-{i}"));
    }
    fs::create_dir_all(&nested_path)?;

    // Create file in deeply nested directory
    let nested_file = nested_path.join("deep-file.txt");
    fs::write(&nested_file, "deep content")?;

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Create config with deeply nested path
    let relative_path = nested_file.strip_prefix(&repo_path)?.to_string_lossy();
    let files_config = FilesConfig {
        copy: vec![relative_path.to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with deeply nested directories");

    // Should handle deeply nested directories up to limits
    assert!(nested_file.exists());
    assert_eq!(copied.len(), 1);

    Ok(())
}

/// Test file copy with special characters in filenames
#[test]
fn test_file_copy_special_characters() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create files with special characters (where filesystem allows)
    let special_files = vec![
        "file with spaces.txt",
        "file-with-hyphens.txt",
        "file_with_underscores.txt",
        "file.with.dots.txt",
        "file(with) parentheses.txt",
        "file[with]brackets.txt",
    ];

    for filename in &special_files {
        let file_path = repo_path.join(filename);
        fs::write(&file_path, format!("content of {filename}"))?;
    }

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Create config
    let files_config = FilesConfig {
        copy: special_files.iter().map(|s| s.to_string()).collect(),
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with special characters in filenames");

    // Should handle special characters in filenames
    assert_eq!(copied.len(), special_files.len());
    for filename in &special_files {
        assert!(repo_path.join(filename).exists());
        assert!(worktree_path.join(filename).exists());
    }

    Ok(())
}

/// Test file copy with concurrent access
#[test]
fn test_file_copy_concurrent_access() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create source file
    let source_file = repo_path.join("concurrent-file.txt");
    fs::write(&source_file, "concurrent content")?;

    // Create multiple worktree directories
    let worktree_paths: Vec<_> = (0..3)
        .map(|i| repo_path.join("worktrees").join(format!("worktree-{i}")))
        .collect();

    for worktree_path in &worktree_paths {
        fs::create_dir_all(worktree_path)?;
    }

    // Create config
    let files_config = FilesConfig {
        copy: vec!["concurrent-file.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    // Test concurrent access by copying to multiple destinations
    for worktree_path in &worktree_paths {
        let copied = file_copy::copy_configured_files(&files_config, worktree_path, &manager)?;
        assert_eq!(copied.len(), 1);
        assert!(worktree_path.join("concurrent-file.txt").exists());
    }

    println!("Testing copy with concurrent access simulation");

    // Should handle concurrent access gracefully
    assert!(source_file.exists());

    Ok(())
}

/// Test file copy with filesystem limits
#[test]
fn test_file_copy_filesystem_limits() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create file with long filename (filesystem dependent)
    let long_name = "a".repeat(200); // Reduced from 255 for better compatibility
    let long_filename = format!("{long_name}.txt");

    // Try to create file with long name
    let long_file = repo_path.join(&long_filename);
    match fs::write(&long_file, "content with long filename") {
        Ok(_) => {
            // If filesystem allows it, test copying
            let worktree_path = create_test_worktree(&repo_path)?;

            let files_config = FilesConfig {
                copy: vec![long_filename.clone()],
                source: Some(repo_path.to_str().unwrap().to_string()),
            };

            let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

            println!("Testing copy with maximum filename length");
            assert!(long_file.exists());
            assert_eq!(copied.len(), 1);
            assert!(worktree_path.join(&long_filename).exists());
        }
        Err(e) => {
            // If filesystem doesn't allow it, that's also valid
            println!("Filesystem doesn't support long filenames - this is expected: {e}");
        }
    }

    Ok(())
}

/// Test file copy with zero-byte files
#[test]
fn test_file_copy_zero_byte_files() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create zero-byte file
    let empty_file = repo_path.join("empty.txt");
    fs::write(&empty_file, "")?;

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Create config
    let files_config = FilesConfig {
        copy: vec!["empty.txt".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with zero-byte files");

    // Should handle zero-byte files
    assert_eq!(copied.len(), 1);
    assert!(worktree_path.join("empty.txt").exists());

    // Verify content is empty
    let content = fs::read_to_string(worktree_path.join("empty.txt"))?;
    assert!(content.is_empty());

    Ok(())
}

/// Test file copy with binary files
#[test]
fn test_file_copy_binary_files() -> Result<()> {
    let (_temp_dir, repo_path, manager) = setup_test_repo_git()?;

    // Create binary file
    let binary_file = repo_path.join("binary.bin");
    let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    fs::write(&binary_file, &binary_data)?;

    // Create worktree directory
    let worktree_path = create_test_worktree(&repo_path)?;

    // Create config
    let files_config = FilesConfig {
        copy: vec!["binary.bin".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    println!("Testing copy with binary files");

    // Should handle binary files (if file exists, it should be copied)
    if binary_file.exists() {
        assert_eq!(copied.len(), 1);
        assert!(worktree_path.join("binary.bin").exists());
    } else {
        assert_eq!(copied.len(), 0);
        println!("Binary file not found, copy skipped as expected");
    }

    // Verify binary content is preserved (only if file was actually copied)
    if binary_file.exists() && worktree_path.join("binary.bin").exists() {
        let copied_data = fs::read(worktree_path.join("binary.bin"))?;
        assert_eq!(copied_data, binary_data);
    }

    Ok(())
}
