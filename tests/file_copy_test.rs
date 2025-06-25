use anyhow::Result;
use git_workers::config::FilesConfig;
use git_workers::file_copy;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_file_copy_basic() -> Result<()> {
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

    // Create test files to copy
    fs::write(repo_path.join(".env"), "TEST_VAR=value1")?;
    fs::write(repo_path.join(".env.local"), "LOCAL_VAR=value2")?;

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    // Test file copying
    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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

#[test]
fn test_file_copy_with_subdirectories() -> Result<()> {
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

    // Create test files in subdirectory
    let config_dir = repo_path.join("config");
    fs::create_dir(&config_dir)?;
    fs::write(config_dir.join("local.json"), r#"{"key": "value"}"#)?;

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    // Test file copying
    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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

#[test]
fn test_file_copy_security() -> Result<()> {
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

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    // Test file copying with unsafe paths
    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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

#[test]
fn test_file_copy_missing_files() -> Result<()> {
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

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    // Test file copying with non-existent files
    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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

#[test]
fn test_path_traversal_detailed() -> Result<()> {
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

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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
        assert_eq!(copied.len(), 0, "Path '{}' should not be copied", path);
    }

    Ok(())
}

#[test]
fn test_directory_copy_recursive() -> Result<()> {
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

    // Create nested directory structure
    fs::create_dir_all(repo_path.join("config/env/dev"))?;
    fs::write(repo_path.join("config/env/dev/.env"), "DEV=true")?;
    fs::write(repo_path.join("config/settings.json"), r#"{"app": "test"}"#)?;
    fs::create_dir_all(repo_path.join("config/certs"))?;
    fs::write(repo_path.join("config/certs/cert.pem"), "CERT")?;

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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

#[test]
fn test_empty_directory_copy() -> Result<()> {
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

    // Create empty directory
    fs::create_dir(repo_path.join("empty_dir"))?;

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

    let files_config = FilesConfig {
        copy: vec!["empty_dir".to_string()],
        source: None,
    };

    let copied = file_copy::copy_configured_files(&files_config, &worktree_path, &manager)?;

    // Verify empty directory was created (empty directories report 0 files copied)
    assert_eq!(copied.len(), 0);
    assert!(worktree_path.join("empty_dir").is_dir());

    Ok(())
}

#[test]
fn test_special_filenames() -> Result<()> {
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

    // Create files with special names
    let special_names = vec![
        ".hidden",
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.multiple.dots.txt",
    ];

    for name in &special_names {
        fs::write(repo_path.join(name), format!("content of {}", name))?;
    }

    // Create worktrees directory
    let worktrees_dir = repo_path.join("worktrees");
    fs::create_dir(&worktrees_dir)?;

    // Create a new worktree
    let worktree_path = worktrees_dir.join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()?;

    std::env::set_current_dir(&repo_path)?;
    let manager = GitWorktreeManager::new()?;

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
            "File {} should exist",
            name
        );
    }

    Ok(())
}
