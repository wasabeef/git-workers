use anyhow::Result;
use git2::{Repository, Signature};
use git_workers::repository_info::get_repository_info;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test get_repository_info with worktrees - comprehensive scenarios
/// This test consolidates and expands on the worktree-specific functionality
/// from both repository_info_public_test.rs and repository_info_comprehensive_test.rs
#[test]
fn test_get_repository_info_with_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("main-repo");

    // Initialize main repository
    Command::new("git")
        .args(["init", "main-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&main_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&main_repo)
        .output()?;

    // Create initial commit
    fs::write(main_repo.join("README.md"), "# Test")?;
    Command::new("git")
        .args(["add", "."])
        .current_dir(&main_repo)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&main_repo)
        .output()?;

    // Test main repository before worktree creation
    std::env::set_current_dir(&main_repo)?;
    let info_before = get_repository_info();
    // Should show just the repository name initially
    assert_eq!(info_before, "main-repo");

    // Create a worktree with branch
    let worktree_path = temp_dir.path().join("feature-branch");
    Command::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .current_dir(&main_repo)
        .output()?;

    // Test from main repository (should now show it has worktrees)
    std::env::set_current_dir(&main_repo)?;
    let info_after = get_repository_info();
    assert_eq!(info_after, "main-repo (main)");

    // Test from worktree
    std::env::set_current_dir(&worktree_path)?;
    let worktree_info = get_repository_info();
    // Worktree info should contain the branch name or directory name
    assert!(
        worktree_info.contains("feature") || worktree_info == "feature-branch",
        "Expected worktree info to contain 'feature' or equal 'feature-branch', got: {worktree_info}"
    );

    Ok(())
}

/// Test get_repository_info with multiple worktrees
#[test]
fn test_get_repository_info_multiple_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("multi-repo");

    // Initialize with git2 for better control
    let repo = Repository::init(&main_repo)?;
    create_initial_commit(&repo)?;

    // Create multiple worktrees
    let worktree1_path = temp_dir.path().join("feature1");
    let worktree2_path = temp_dir.path().join("feature2");

    Command::new("git")
        .current_dir(&main_repo)
        .args([
            "worktree",
            "add",
            worktree1_path.to_str().unwrap(),
            "-b",
            "feature1",
        ])
        .output()?;

    Command::new("git")
        .current_dir(&main_repo)
        .args([
            "worktree",
            "add",
            worktree2_path.to_str().unwrap(),
            "-b",
            "feature2",
        ])
        .output()?;

    // Test from main repository
    std::env::set_current_dir(&main_repo)?;
    let main_info = get_repository_info();
    assert_eq!(main_info, "multi-repo (main)");

    // Test from first worktree
    std::env::set_current_dir(&worktree1_path)?;
    let worktree1_info = get_repository_info();
    assert!(worktree1_info.contains("feature1"));

    // Test from second worktree
    std::env::set_current_dir(&worktree2_path)?;
    let worktree2_info = get_repository_info();
    assert!(worktree2_info.contains("feature2"));

    Ok(())
}

/// Test get_repository_info with worktree in subdirectory
#[test]
fn test_get_repository_info_worktree_subdirectory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("parent-repo");
    let worktrees_dir = temp_dir.path().join("worktrees");

    let repo = Repository::init(&main_repo)?;
    create_initial_commit(&repo)?;

    // Create worktrees directory
    fs::create_dir(&worktrees_dir)?;

    // Create worktree in subdirectory
    let worktree_path = worktrees_dir.join("dev-branch");
    Command::new("git")
        .current_dir(&main_repo)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "dev",
        ])
        .output()?;

    // Test from worktree subdirectory
    std::env::set_current_dir(&worktree_path)?;
    let worktree_info = get_repository_info();
    assert!(worktree_info.contains("dev") || worktree_info.contains("dev-branch"));

    // Test from deeper subdirectory within worktree
    let deep_dir = worktree_path.join("src").join("components");
    fs::create_dir_all(&deep_dir)?;
    std::env::set_current_dir(&deep_dir)?;
    let deep_info = get_repository_info();
    assert_eq!(deep_info, "components");

    Ok(())
}

/// Test get_repository_info with worktree using existing branch
#[test]
fn test_get_repository_info_worktree_existing_branch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("existing-branch-repo");

    let repo = Repository::init(&main_repo)?;
    create_initial_commit(&repo)?;

    // Create a branch and switch back to main
    Command::new("git")
        .current_dir(&main_repo)
        .args(["checkout", "-b", "existing"])
        .output()?;

    Command::new("git")
        .current_dir(&main_repo)
        .args(["checkout", "main"])
        .output()?;

    // Create worktree from existing branch
    let worktree_path = temp_dir.path().join("existing-worktree");
    Command::new("git")
        .current_dir(&main_repo)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "existing",
        ])
        .output()?;

    // Test from worktree
    std::env::set_current_dir(&worktree_path)?;
    let worktree_info = get_repository_info();
    assert!(
        worktree_info.contains("existing") || worktree_info == "existing-worktree",
        "Expected info to contain 'existing' or equal 'existing-worktree', got: {worktree_info}"
    );

    Ok(())
}

/// Test get_repository_info with bare repository and worktrees
#[test]
fn test_get_repository_info_bare_repo_with_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo = temp_dir.path().join("bare-repo.git");

    // Initialize bare repository
    Repository::init_bare(&bare_repo)?;

    // Clone to create initial content
    let initial_clone = temp_dir.path().join("initial");
    Command::new("git")
        .current_dir(temp_dir.path())
        .args(["clone", bare_repo.to_str().unwrap(), "initial"])
        .output()?;

    // Configure and create initial commit
    Command::new("git")
        .current_dir(&initial_clone)
        .args(["config", "user.email", "test@example.com"])
        .output()?;

    Command::new("git")
        .current_dir(&initial_clone)
        .args(["config", "user.name", "Test User"])
        .output()?;

    fs::write(initial_clone.join("README.md"), "# Bare repo test")?;
    Command::new("git")
        .current_dir(&initial_clone)
        .args(["add", "."])
        .output()?;

    Command::new("git")
        .current_dir(&initial_clone)
        .args(["commit", "-m", "Initial commit"])
        .output()?;

    Command::new("git")
        .current_dir(&initial_clone)
        .args(["push", "origin", "main"])
        .output()?;

    // Create worktree from bare repository
    let worktree_path = temp_dir.path().join("bare-worktree");
    Command::new("git")
        .current_dir(&bare_repo)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .output()?;

    // Test from worktree created from bare repo
    std::env::set_current_dir(&worktree_path)?;
    let worktree_info = get_repository_info();
    // The worktree should show parent repo name with worktree name
    assert!(
        worktree_info.ends_with(" (bare-worktree)"),
        "Expected info to end with ' (bare-worktree)', got: {worktree_info}"
    );

    Ok(())
}

/// Test get_repository_info with worktree special characters in names
#[test]
fn test_get_repository_info_worktree_special_names() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("special-chars-repo");

    let repo = Repository::init(&main_repo)?;
    create_initial_commit(&repo)?;

    // Test with various special characters in worktree names
    let special_names = vec![
        ("feature-123", "feature-branch-123"),
        ("bug_fix", "bug-fix-worktree"),
        ("release.v1.0", "release-worktree"),
    ];

    for (branch_name, worktree_dir) in special_names {
        let worktree_path = temp_dir.path().join(worktree_dir);

        // Create worktree with special character branch name
        Command::new("git")
            .current_dir(&main_repo)
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                "-b",
                branch_name,
            ])
            .output()?;

        // Test from worktree
        std::env::set_current_dir(&worktree_path)?;
        let worktree_info = get_repository_info();
        // The worktree should show parent repo name with worktree name
        assert!(
            worktree_info.ends_with(&format!(" ({worktree_dir})")),
            "Failed for branch {branch_name}, worktree {worktree_dir}, expected to end with ' ({worktree_dir})', got: {worktree_info}"
        );

        // Clean up worktree for next iteration
        Command::new("git")
            .current_dir(&main_repo)
            .args(["worktree", "remove", worktree_path.to_str().unwrap()])
            .output()?;
    }

    Ok(())
}

/// Test get_repository_info worktree performance with many worktrees
#[test]
fn test_get_repository_info_many_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo = temp_dir.path().join("many-worktrees-repo");

    let repo = Repository::init(&main_repo)?;
    create_initial_commit(&repo)?;

    // Create multiple worktrees (limited number for test performance)
    let mut worktree_paths = Vec::new();
    for i in 0..5 {
        let worktree_path = temp_dir.path().join(format!("worktree-{i}"));
        Command::new("git")
            .current_dir(&main_repo)
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                "-b",
                &format!("branch-{i}"),
            ])
            .output()?;
        worktree_paths.push(worktree_path);
    }

    // Test from main repository
    std::env::set_current_dir(&main_repo)?;
    let main_info = get_repository_info();
    assert_eq!(main_info, "many-worktrees-repo (main)");

    // Test from each worktree
    for (i, worktree_path) in worktree_paths.iter().enumerate() {
        std::env::set_current_dir(worktree_path)?;
        let worktree_info = get_repository_info();
        assert!(
            worktree_info.contains(&format!("branch-{i}"))
                || worktree_info.contains(&format!("worktree-{i}")),
            "Failed for worktree {i}, got: {worktree_info}"
        );
    }

    Ok(())
}

// Helper function to create initial commit using git2
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}
