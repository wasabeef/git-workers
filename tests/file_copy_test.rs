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
