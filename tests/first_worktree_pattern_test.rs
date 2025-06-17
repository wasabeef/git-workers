use anyhow::Result;
use git2::Repository;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_first_worktree_with_pattern() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Test creating worktree with pattern
    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree with pattern "branch/feature"
    let worktree_path = manager.create_worktree("branch/feature", Some("feature"))?;

    // Verify the structure
    assert!(worktree_path.exists());
    assert_eq!(worktree_path.file_name().unwrap(), "feature");

    // The parent should be "branch" directory
    let parent = worktree_path.parent().unwrap();
    assert_eq!(parent.file_name().unwrap(), "branch");

    // List worktrees to verify
    let worktrees = manager.list_worktrees()?;
    assert_eq!(worktrees.len(), 1);

    // Debug: print what we actually get
    let wt = &worktrees[0];
    eprintln!("Worktree name: {}", wt.name);
    eprintln!("Worktree path: {}", wt.path.display());

    // Git worktree name is the last component only
    assert_eq!(wt.name, "feature");

    Ok(())
}

#[test]
fn test_subsequent_worktree_pattern_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create first worktree with pattern
    Command::new("git")
        .current_dir(&repo_path)
        .args(["worktree", "add", "../branch/first", "-b", "first"])
        .output()?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create second worktree - should detect the pattern
    let worktree_path = manager.create_worktree("second", Some("second"))?;

    // Should be created in branch directory
    assert!(worktree_path.exists());
    let parent = worktree_path.parent().unwrap();
    assert_eq!(parent.file_name().unwrap(), "branch");

    Ok(())
}

#[test]
fn test_worktree_without_subdirectory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree without subdirectory pattern
    let worktree_path = manager.create_worktree("feature", Some("feature"))?;

    // Should be created at same level as main repo
    assert!(worktree_path.exists());
    assert_eq!(worktree_path.file_name().unwrap(), "feature");

    // The parent should be the same as repo's parent
    let worktree_parent = worktree_path.parent().unwrap();
    let repo_parent = repo_path.parent().unwrap();

    // On macOS, paths might have /private prefix
    assert!(
        worktree_parent == repo_parent
            || worktree_parent.to_string_lossy().replace("/private", "")
                == repo_parent.to_string_lossy().replace("/private", "")
    );

    Ok(())
}

#[test]
fn test_first_worktree_same_level_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    use git_workers::git::GitWorktreeManager;
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree with pattern "{name}" (same level)
    let worktree_path = manager.create_worktree("feature", Some("feature"))?;

    // Should be created at same level as main repo
    assert!(worktree_path.exists());
    assert_eq!(worktree_path.file_name().unwrap(), "feature");

    // The parent should be the same as repo's parent
    let worktree_parent = worktree_path.parent().unwrap();
    let repo_parent = repo_path.parent().unwrap();

    // On macOS, paths might have /private prefix
    assert!(
        worktree_parent == repo_parent
            || worktree_parent.to_string_lossy().replace("/private", "")
                == repo_parent.to_string_lossy().replace("/private", "")
    );

    Ok(())
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
