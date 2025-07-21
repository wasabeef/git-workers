//! Performance benchmarks for Git Workers
//!
//! Benchmarks for measuring performance of critical operations.

use anyhow::Result;
use git_workers::config::Config;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

/// Helper to create a test repository
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize repository
    Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    Command::new("git")
        .arg("add")
        .arg("README.md")
        .current_dir(temp_dir.path())
        .output()?;
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

#[test]
fn bench_list_worktrees_small_repo() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;

    // Create a few worktrees
    for i in 1..=5 {
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join(format!("perf-test-wt-{i}"));
        manager.create_worktree_from_head(&worktree_path, &format!("wt-{i}"))?;
    }

    // Benchmark listing worktrees
    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _worktrees = manager.list_worktrees()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time to list {} worktrees: {:?}", 5, avg_time);
    assert!(avg_time.as_millis() < 100); // Should be under 100ms

    Ok(())
}

#[test]
#[ignore = "Resource intensive test"]
fn bench_list_worktrees_large_repo() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;

    // Create many worktrees
    for i in 1..=50 {
        // Reduced from 100 for test performance
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join(format!("perf-test-wt-{i}"));
        manager.create_worktree_from_head(&worktree_path, &format!("wt-{i}"))?;
    }

    // Benchmark listing worktrees
    let start = Instant::now();
    let iterations = 10;

    for _ in 0..iterations {
        let _worktrees = manager.list_worktrees()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time to list {} worktrees: {:?}", 50, avg_time);
    assert!(avg_time.as_millis() < 500); // Should be under 500ms

    Ok(())
}

#[test]
fn bench_create_worktree() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;

    // Benchmark worktree creation
    let start = Instant::now();
    let iterations = 10;

    for i in 0..iterations {
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join(format!("perf-bench-wt-{i}"));
        manager.create_worktree_from_head(&worktree_path, &format!("bench-wt-{i}"))?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time to create worktree: {avg_time:?}");
    assert!(avg_time.as_secs() < 5); // Should be under 5 seconds

    Ok(())
}

#[test]
fn bench_branch_listing() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo_root = manager.get_default_worktree_base_path()?;

    // Create many branches
    for i in 1..=20 {
        // Reduced for test performance
        Command::new("git")
            .args(["checkout", "-b", &format!("branch-{i}")])
            .current_dir(&repo_root)
            .output()?;
        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&repo_root)
            .output()?;
    }

    // Benchmark branch listing
    let start = Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        let _branches = manager.list_all_branches()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time to list branches: {avg_time:?}");
    assert!(avg_time.as_millis() < 50); // Should be under 50ms

    Ok(())
}

#[test]
fn bench_file_copy_operations() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo_root = manager.get_default_worktree_base_path()?;

    // Create test files to copy
    fs::write(repo_root.join(".env"), "API_KEY=secret")?;
    fs::write(repo_root.join("config.json"), r#"{"key": "value"}"#)?;

    // Create config
    let config_content = r#"
[files]
copy = [".env", "config.json"]
"#;
    fs::write(repo_root.join(".git-workers.toml"), config_content)?;

    // Benchmark file copying
    let start = Instant::now();
    let iterations = 20;

    for i in 0..iterations {
        let worktree_path = repo_root.join(format!("copy-wt-{i}"));
        fs::create_dir_all(&worktree_path)?;

        // Simulate file copying with FilesConfig
        let config = git_workers::config::FilesConfig {
            copy: vec![".env".to_string(), "config.json".to_string()],
            source: Some(repo_root.to_str().unwrap().to_string()),
        };

        git_workers::infrastructure::file_copy::copy_configured_files(
            &config,
            &worktree_path,
            &manager,
        )?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time to copy files: {avg_time:?}");
    assert!(avg_time.as_millis() < 100); // Should be under 100ms

    Ok(())
}

#[test]
fn bench_config_parsing() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create complex config file
    let config_content = r#"
[worktree]
pattern = "subdirectory"
default_branch = "main"

[hooks]
post-create = [
    "echo 'Created {{worktree_name}}'",
    "cp .env.example .env",
    "npm install",
    "echo 'Setup complete'"
]
pre-remove = [
    "echo 'Removing {{worktree_name}}'",
    "rm -rf node_modules",
    "echo 'Cleanup complete'"
]
post-switch = ["echo 'Switched to {{worktree_name}}'"]

[files]
copy = [
    ".env",
    ".env.local", 
    ".env.development",
    "config/local.json",
    "config/development.json",
    "keys/private.key",
    "certs/cert.pem"
]
source = "/path/to/source"
"#;
    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Benchmark config parsing
    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let _config = Config::load()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    std::env::set_current_dir(original_dir)?;

    println!("Average time to parse config: {avg_time:?}");
    assert!(avg_time.as_micros() < 1000); // Should be under 1ms

    Ok(())
}

#[test]
fn bench_git_operations() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Benchmark basic git operations
    let start = Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        let _repo_root = manager.get_default_worktree_base_path()?;
        let _git_dir = manager.get_git_dir()?;
        let _worktrees = manager.list_worktrees()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time for git operations: {avg_time:?}");
    assert!(avg_time.as_millis() < 10); // Should be under 10ms

    Ok(())
}

#[test]
fn bench_worktree_status_check() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;

    // Create worktree and add some changes
    let worktree_path = temp_dir.path().parent().unwrap().join("perf-status-wt");
    manager.create_worktree_from_head(&worktree_path, "status-wt")?;

    // Add uncommitted changes
    fs::write(worktree_path.join("test.txt"), "test content")?;

    // Benchmark status checking
    let start = Instant::now();
    let iterations = 30;

    for _ in 0..iterations {
        let _worktrees = manager.list_worktrees()?;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time for status check: {avg_time:?}");
    assert!(avg_time.as_millis() < 200); // Should be under 200ms

    Ok(())
}

#[test]
fn bench_memory_usage_simulation() -> Result<()> {
    let (temp_dir, manager) = setup_test_repo()?;

    // Create multiple worktrees to simulate memory usage
    for i in 1..=10 {
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join(format!("perf-mem-wt-{i}"));
        manager.create_worktree_from_head(&worktree_path, &format!("mem-wt-{i}"))?;
    }

    // Simulate continuous operations
    let start = Instant::now();
    let iterations = 20;

    for _ in 0..iterations {
        let worktrees = manager.list_worktrees()?;
        assert!(worktrees.len() >= 10);

        // Simulate some processing
        for worktree in &worktrees {
            let _ = worktree.name.len();
            let _ = worktree.path.exists();
        }
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average time for memory simulation: {avg_time:?}");
    assert!(avg_time.as_millis() < 100);

    Ok(())
}
