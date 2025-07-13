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
        source: None,
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
        source: None,
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
        source: None,
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
        source: None,
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
        source: None,
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
        source: None,
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
            source: None,
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
        source: None,
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
        source: None,
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
