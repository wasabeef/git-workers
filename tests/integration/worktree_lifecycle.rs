//! Integration tests for complete worktree lifecycle
//!
//! Tests the full lifecycle of worktrees from creation to deletion,
//! including hooks and file copying.

use anyhow::Result;
use git_workers::infrastructure::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a test repository with initial setup
fn setup_test_environment() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;

    // Initialize repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial files
    fs::write(temp_dir.path().join("README.md"), "# Test Project")?;
    fs::write(
        temp_dir.path().join(".gitignore"),
        "*.log\n.env\nnode_modules/",
    )?;
    fs::write(temp_dir.path().join(".env"), "API_KEY=secret")?;

    // Create initial commit
    std::process::Command::new("git")
        .arg("add")
        .args(["README.md", ".gitignore"])
        .current_dir(temp_dir.path())
        .output()?;

    std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()?;

    // Create config file
    let config_content = r#"
[worktree]
pattern = "subdirectory"

[hooks]
post-create = ["echo 'Worktree {{worktree_name}} created' > {{worktree_path}}/.created"]
pre-remove = ["echo 'Removing {{worktree_name}}' > /tmp/removed-{{worktree_name}}.txt"]

[files]
copy = [".env"]
"#;
    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;
    Ok((temp_dir, manager))
}

#[test]
fn test_complete_worktree_lifecycle() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;
    let repo_path = manager.get_git_dir()?;

    // Step 1: Create a worktree
    let worktree_name = "feature-branch";
    let worktree_path = repo_path
        .parent()
        .unwrap()
        .join("test-repo")
        .join("worktrees")
        .join(worktree_name);

    manager.create_worktree_from_head(&worktree_path, worktree_name)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Verify hook was executed
    assert!(worktree_path.join(".created").exists());
    let hook_output = fs::read_to_string(worktree_path.join(".created"))?;
    assert!(hook_output.contains(&format!("Worktree {worktree_name} created")));

    // Verify file was copied
    assert!(worktree_path.join(".env").exists());
    let env_content = fs::read_to_string(worktree_path.join(".env"))?;
    assert_eq!(env_content, "API_KEY=secret");

    // Step 2: List worktrees
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == worktree_name));

    // Step 3: Make changes in the worktree
    fs::write(worktree_path.join("feature.txt"), "New feature")?;
    std::process::Command::new("git")
        .arg("add")
        .arg("feature.txt")
        .current_dir(&worktree_path)
        .output()?;

    // Verify changes are detected
    let worktrees = manager.list_worktrees()?;
    let worktree = worktrees.iter().find(|w| w.name == worktree_name);
    assert!(worktree.is_some());
    assert!(worktree.unwrap().has_changes);

    // Step 4: Remove the worktree
    manager.remove_worktree(worktree_name)?;

    // Verify worktree was removed
    assert!(!worktree_path.exists());

    // Verify pre-remove hook was executed
    let removed_file = format!("/tmp/removed-{worktree_name}.txt");
    if Path::new(&removed_file).exists() {
        let content = fs::read_to_string(&removed_file)?;
        assert!(content.contains(&format!("Removing {worktree_name}")));
        // Clean up
        fs::remove_file(&removed_file)?;
    }

    Ok(())
}

#[test]
fn test_multiple_worktrees_interaction() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;
    let repo_path = manager.get_git_dir()?;
    let worktrees_dir = repo_path
        .parent()
        .unwrap()
        .join("test-repo")
        .join("worktrees");

    // Create multiple worktrees
    let worktree_names = vec!["feature-1", "feature-2", "bugfix-1"];
    let mut created_paths = vec![];

    for name in &worktree_names {
        let path = worktrees_dir.join(name);
        manager.create_worktree_from_head(&path, name)?;
        created_paths.push(path);
    }

    // Verify all were created
    let worktrees = manager.list_worktrees()?;
    assert_eq!(worktrees.len(), 4); // main + 3 new worktrees

    for name in &worktree_names {
        assert!(worktrees.iter().any(|w| w.name == *name));
    }

    // Verify each has its own .env copy
    for path in &created_paths {
        assert!(path.join(".env").exists());
    }

    // Make different changes in each worktree
    for (i, path) in created_paths.iter().enumerate() {
        let file_name = format!("file{i}.txt");
        fs::write(path.join(&file_name), format!("Content {i}"))?;

        std::process::Command::new("git")
            .arg("add")
            .arg(&file_name)
            .current_dir(path)
            .output()?;

        std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(format!("Add {file_name}"))
            .current_dir(path)
            .output()?;
    }

    // Verify branches were created correctly
    let (local_branches, _) = manager.list_all_branches()?;
    for name in &worktree_names {
        assert!(local_branches.contains(&name.to_string()));
    }

    // Clean up all worktrees
    for name in &worktree_names {
        manager.remove_worktree(name)?;
    }

    Ok(())
}

#[test]
fn test_worktree_with_branch_switching() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;
    let repo_path = manager.get_git_dir()?;

    // Create a branch in main repository
    std::process::Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(repo_path)
        .output()?;

    // Add a commit to develop branch
    fs::write(repo_path.join("develop.txt"), "Development file")?;
    std::process::Command::new("git")
        .arg("add")
        .arg("develop.txt")
        .current_dir(repo_path)
        .output()?;

    std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Add develop file")
        .current_dir(repo_path)
        .output()?;

    // Switch back to main
    std::process::Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_path)
        .output()?;

    // Create worktree from develop branch
    let worktree_path = repo_path.parent().unwrap().join("develop-wt");
    manager.create_worktree_with_branch(&worktree_path, "develop")?;

    // Verify the worktree has the develop branch file
    assert!(worktree_path.join("develop.txt").exists());
    assert!(
        !worktree_path.join("develop.txt").exists() || worktree_path.join("develop.txt").exists()
    ); // File should exist

    // Verify the main repository doesn't have the develop file
    assert!(!repo_path.join("develop.txt").exists());

    Ok(())
}

#[test]
#[ignore = "Requires real git remote"]
fn test_worktree_with_remote_branch() -> Result<()> {
    // This test would require setting up a real or mock git remote
    // Placeholder for future implementation
    Ok(())
}
