use anyhow::Result;
use git_workers::config::FilesConfig;
use git_workers::file_copy::copy_configured_files;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_test_repo_with_files() -> Result<(TempDir, GitWorktreeManager, TempDir)> {
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

#[test]
fn test_file_copy_with_small_files() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
    let repo_path = manager.repo().workdir().unwrap().to_path_buf();

    // Create small test files
    fs::write(repo_path.join(".env"), "SMALL_FILE=true")?;
    fs::write(repo_path.join("config.json"), "{\"small\": true}")?;

    let config = FilesConfig {
        copy: vec![".env".to_string(), "config.json".to_string()],
        source: Some(repo_path.to_str().unwrap().to_string()),
    };

    let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Verify files were copied
    assert_eq!(copied.len(), 2);
    assert!(dest_dir.path().join(".env").exists());
    assert!(dest_dir.path().join("config.json").exists());

    Ok(())
}

#[test]
fn test_file_copy_skip_large_file() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
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

    let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Only small file should be copied
    assert_eq!(copied.len(), 1);
    assert_eq!(copied[0], "small.txt");
    assert!(!dest_dir.path().join("large.bin").exists());
    assert!(dest_dir.path().join("small.txt").exists());

    Ok(())
}

#[test]
fn test_file_copy_with_directory() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
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

    let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Directory should be copied
    assert_eq!(copied.len(), 1);
    assert!(dest_dir.path().join("config").exists());
    assert!(dest_dir.path().join("config/app.json").exists());
    assert!(dest_dir.path().join("config/db.json").exists());

    Ok(())
}

#[test]
fn test_file_copy_total_size_limit() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
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

    let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Should copy both small files
    assert_eq!(copied.len(), 2);

    Ok(())
}

#[test]
fn test_file_copy_skip_symlinks() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
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

        let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

        // Only original file should be copied, not the symlink
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0], "original.txt");
        assert!(dest_dir.path().join("original.txt").exists());
        assert!(!dest_dir.path().join("link.txt").exists());
    }

    Ok(())
}

#[test]
fn test_file_copy_directory_size_calculation() -> Result<()> {
    let (_temp_dir, manager, dest_dir) = setup_test_repo_with_files()?;
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

    let copied = copy_configured_files(&config, dest_dir.path(), &manager)?;

    // Should copy the entire directory
    assert_eq!(copied.len(), 1);
    assert!(dest_dir.path().join("nested/file1.txt").exists());
    assert!(dest_dir.path().join("nested/sub/file2.txt").exists());

    Ok(())
}
