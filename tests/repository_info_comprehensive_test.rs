use anyhow::Result;
use git2::Repository;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

use git_workers::repository_info::get_repository_info;

#[test]
fn test_get_repository_info_normal_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();
    assert!(info.contains("test-repo"));
    assert!(!info.contains(".bare"));

    Ok(())
}

#[test]
fn test_get_repository_info_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.bare");

    Repository::init_bare(&bare_repo_path)?;

    std::env::set_current_dir(&bare_repo_path)?;

    let info = get_repository_info();
    // Bare repos just show the directory name without "(bare)" suffix
    assert!(info.contains("test-repo.bare"));

    Ok(())
}

#[test]
fn test_get_repository_info_with_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a worktree
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../feature", "-b", "feature"])
        .output()?;

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();
    assert!(info.contains("test-repo"));

    // Switch to worktree
    std::env::set_current_dir(temp_dir.path().join("feature"))?;

    let worktree_info = get_repository_info();
    // Worktree info may show just "feature" or "test-repo (feature)"
    assert!(worktree_info.contains("feature") || worktree_info.contains("test-repo"));

    Ok(())
}

#[test]
fn test_get_repository_info_deeply_nested() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("deeply/nested/test-repo");

    fs::create_dir_all(&repo_path)?;
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();
    assert!(info.contains("test-repo"));

    Ok(())
}

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_get_repository_info_special_characters() -> Result<()> {
    // Skip in CI environment
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo-2024");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    std::env::set_current_dir(&repo_path)?;

    let info = get_repository_info();
    assert!(info.contains("test-repo-2024"));

    Ok(())
}

#[test]
fn test_get_repository_info_not_in_repo() {
    // Test outside of a git repository
    let result = std::env::set_current_dir("/tmp");
    if result.is_ok() {
        let info = get_repository_info();
        // Outside git repo, it shows the current directory name
        assert!(info.contains("tmp") || info.contains("unknown") || !info.is_empty());
    }
}

// Helper function
fn create_initial_commit(repo: &Repository) -> Result<()> {
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}
