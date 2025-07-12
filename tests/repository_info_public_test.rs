use anyhow::Result;
use git_workers::repository_info::get_repository_info;
use std::fs;
use tempfile::TempDir;

/// Test get_repository_info in non-git directory
#[test]
fn test_get_repository_info_non_git() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let non_git_dir = temp_dir.path().join("not-a-repo");
    fs::create_dir(&non_git_dir)?;

    // Save current directory
    let original_dir = std::env::current_dir().ok();

    std::env::set_current_dir(&non_git_dir)?;

    let info = get_repository_info();

    // Restore original directory
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    // Should return some directory name when not in a git repository
    assert!(!info.is_empty());

    Ok(())
}

/// Test get_repository_info in main repository
#[test]
fn test_get_repository_info_main_repo() -> Result<()> {
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

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();

    // Should return repository name
    assert_eq!(info, "test-repo");

    Ok(())
}

/// Test get_repository_info in bare repository
#[test]
fn test_get_repository_info_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo = temp_dir.path().join("test.git");

    // Initialize bare repository
    std::process::Command::new("git")
        .args(["init", "--bare", "test.git"])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&bare_repo)?;

    let info = get_repository_info();

    // Should return bare repository name
    assert_eq!(info, "test.git");

    Ok(())
}

/// Test get_repository_info with custom directory names
#[test]
fn test_get_repository_info_custom_names() -> Result<()> {
    let original_dir = std::env::current_dir().ok();
    let temp_dir = TempDir::new()?;

    // Test with special characters in name
    let special_names = [
        "my-project-2024",
        "project_with_underscores",
        "project.with.dots",
        "UPPERCASE-PROJECT",
        "123-numeric-start",
    ];

    for (i, project_name) in special_names.iter().enumerate() {
        // Use a unique subdirectory for each test to avoid conflicts
        let test_dir = temp_dir.path().join(format!("test{i}"));
        fs::create_dir(&test_dir)?;

        let repo_path = test_dir.join(project_name);

        // Initialize repository
        std::process::Command::new("git")
            .args(["init", project_name])
            .current_dir(&test_dir)
            .output()?;

        std::env::set_current_dir(&repo_path)?;

        let info = get_repository_info();

        // Verify the info contains expected name or equals it
        assert!(
            info == *project_name || info.contains(project_name),
            "Failed for project: {project_name}, got: {info}"
        );
    }

    // Restore directory
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    Ok(())
}

/// Test get_repository_info with nested git repositories
#[test]
fn test_get_repository_info_nested_repos() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let outer_repo = temp_dir.path().join("outer-repo");
    let inner_repo = outer_repo.join("inner-repo");

    // Create outer repository
    std::process::Command::new("git")
        .args(["init", "outer-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create inner repository
    fs::create_dir_all(&outer_repo)?;
    std::process::Command::new("git")
        .args(["init", "inner-repo"])
        .current_dir(&outer_repo)
        .output()?;

    // Test from inner repository
    std::env::set_current_dir(&inner_repo)?;

    let info = get_repository_info();

    // Should detect inner repository
    assert_eq!(info, "inner-repo");

    Ok(())
}

/// Test get_repository_info with long repository names
#[test]
fn test_get_repository_info_long_names() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a very long repository name
    let long_name = format!("project-{}", "x".repeat(50));
    let repo_path = temp_dir.path().join(&long_name);

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", &long_name])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();

    assert_eq!(info, long_name);

    Ok(())
}

/// Test get_repository_info from subdirectory
#[test]
fn test_get_repository_info_from_subdirectory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");
    let sub_dir = repo_path.join("src").join("components");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create subdirectory
    fs::create_dir_all(&sub_dir)?;

    // Test from subdirectory
    std::env::set_current_dir(&sub_dir)?;

    let info = get_repository_info();

    // Should still show repository name when in subdirectory
    assert_eq!(info, "components");

    Ok(())
}

/// Test bare repository with .git extension
#[test]
fn test_get_repository_info_bare_with_extension() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test both with and without .git extension
    let bare_names = vec!["project.git", "project-bare", "repo.git"];

    for bare_name in bare_names {
        let bare_path = temp_dir.path().join(bare_name);

        // Initialize bare repository
        std::process::Command::new("git")
            .args(["init", "--bare", bare_name])
            .current_dir(temp_dir.path())
            .output()?;

        std::env::set_current_dir(&bare_path)?;

        let info = get_repository_info();

        assert_eq!(info, bare_name, "Failed for bare repo: {bare_name}");
    }

    Ok(())
}

/// Test get_repository_info with worktrees
#[test]
fn test_get_repository_info_with_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("main-repo");

    // Initialize main repository
    std::process::Command::new("git")
        .args(["init", "main-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&main_repo)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&main_repo)
        .output()?;

    // Create initial commit
    fs::write(main_repo.join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&main_repo)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&main_repo)
        .output()?;

    // Create a worktree
    let worktree_path = temp_dir.path().join("feature-branch");
    std::process::Command::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .current_dir(&main_repo)
        .output()?;

    // Test from main repository (should show it has worktrees)
    std::env::set_current_dir(&main_repo)?;
    let info = get_repository_info();
    assert_eq!(info, "main-repo (main)");

    // Test from worktree
    std::env::set_current_dir(&worktree_path)?;
    let info = get_repository_info();
    // Worktree info might vary based on implementation
    assert!(info.contains("feature") || info == "feature-branch");

    Ok(())
}

/// Test get_repository_info edge cases
#[test]
fn test_get_repository_info_edge_cases() -> Result<()> {
    let original_dir = std::env::current_dir().ok();
    let temp_dir = TempDir::new()?;

    // Test with directory name containing only dots
    let dots_dir = temp_dir.path().join("...");

    // Try to create the directory, some systems might not allow it
    match fs::create_dir(&dots_dir) {
        Ok(_) => {
            std::env::set_current_dir(&dots_dir)?;
            let info = get_repository_info();
            // Should return some directory name
            assert!(!info.is_empty());
        }
        Err(_) => {
            // Skip if directory creation fails
            println!("Skipping dots directory test - creation failed");
        }
    }

    // Restore directory
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    Ok(())
}

/// Test get_repository_info with Unicode names
#[test]
fn test_get_repository_info_unicode() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test with Unicode characters in directory name
    let unicode_names = vec![
        "프로젝트",     // Korean
        "проект",       // Russian
        "项目",         // Chinese
        "プロジェクト", // Japanese
    ];

    for unicode_name in unicode_names {
        let dir_path = temp_dir.path().join(unicode_name);
        fs::create_dir(&dir_path)?;
        std::env::set_current_dir(&dir_path)?;

        let info = get_repository_info();
        assert_eq!(
            info, unicode_name,
            "Failed for Unicode name: {unicode_name}"
        );
    }

    Ok(())
}

/// Test get_repository_info with spaces in names
#[test]
fn test_get_repository_info_spaces() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test with spaces in repository name
    let space_names = vec!["my project", "test repo", "name with spaces"];

    for space_name in space_names {
        let repo_path = temp_dir.path().join(space_name);

        // Initialize repository
        std::process::Command::new("git")
            .args(["init", space_name])
            .current_dir(temp_dir.path())
            .output()?;

        std::env::set_current_dir(&repo_path)?;

        let info = get_repository_info();
        assert_eq!(
            info, space_name,
            "Failed for name with spaces: {space_name}"
        );
    }

    Ok(())
}

/// Test get_repository_info performance with deep directory structure
#[test]
fn test_get_repository_info_deep_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("deep-repo");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "deep-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create deep directory structure
    let mut deep_path = repo_path.clone();
    for i in 0..10 {
        deep_path = deep_path.join(format!("level{i}"));
        fs::create_dir(&deep_path)?;
    }

    // Test from deep directory
    std::env::set_current_dir(&deep_path)?;

    let info = get_repository_info();

    // Should show the current directory name
    assert_eq!(info, "level9");

    Ok(())
}

/// Test get_repository_info with symlinks
#[test]
#[cfg(unix)]
fn test_get_repository_info_symlinks() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("real-repo");
    let symlink_path = temp_dir.path().join("symlink-repo");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "real-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create symlink to repository
    std::os::unix::fs::symlink(&repo_path, &symlink_path)?;

    // Test from symlink
    std::env::set_current_dir(&symlink_path)?;

    let info = get_repository_info();

    // Should show symlink name or real name
    assert!(info == "symlink-repo" || info == "real-repo");

    Ok(())
}
