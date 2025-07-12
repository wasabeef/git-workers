use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager, std::path::PathBuf)> {
    // Create a parent directory that will contain the main repo and worktrees
    let parent_dir = TempDir::new()?;
    let main_repo_path = parent_dir.path().join("test-repo");
    fs::create_dir(&main_repo_path)?;

    // Initialize a new git repository
    let repo = git2::Repository::init(&main_repo_path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        let readme_path = main_repo_path.join("README.md");
        fs::write(&readme_path, "# Test Repository")?;
        index.add_path(Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    // Change to the repository directory
    std::env::set_current_dir(&main_repo_path)?;

    let manager = GitWorktreeManager::new_from_path(&main_repo_path)?;
    Ok((parent_dir, manager, main_repo_path))
}

#[test]
fn test_create_worktree_from_head_with_relative_path() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo()?;

    // Test relative path at same level as repository
    let worktree_path = manager.create_worktree("../feature-relative", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify it's at the correct location (sibling to main repo)
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "feature-relative"
    );

    // Verify worktree is listed
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == "feature-relative"));

    Ok(())
}

#[test]
fn test_create_worktree_from_head_with_subdirectory_pattern() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo()?;

    // Test subdirectory pattern
    let worktree_path = manager.create_worktree("worktrees/feature-sub", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify it's in the correct subdirectory
    assert!(worktree_path.to_string_lossy().contains("worktrees"));
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "feature-sub"
    );

    Ok(())
}

#[test]
fn test_create_worktree_from_head_with_simple_name() -> Result<()> {
    let (_temp_dir, manager, repo_path) = setup_test_repo()?;

    // Create first worktree to establish pattern
    manager.create_worktree("../first", None)?;

    // Test simple name (should follow established pattern)
    let worktree_path = manager.create_worktree("second", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Should be at same level as first worktree
    let parent = repo_path.parent().unwrap();
    assert_eq!(
        worktree_path.parent().unwrap().canonicalize()?,
        parent.canonicalize()?
    );

    Ok(())
}

#[test]
fn test_create_worktree_with_absolute_path() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo()?;
    let temp_worktree_dir = TempDir::new()?;
    let absolute_path = temp_worktree_dir.path().join("absolute-worktree");

    // Test absolute path
    let worktree_path = manager.create_worktree(absolute_path.to_str().unwrap(), None)?;

    // Verify worktree was created at absolute path
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());
    // Compare canonical paths to handle symlinks on macOS
    assert_eq!(worktree_path.canonicalize()?, absolute_path.canonicalize()?);

    Ok(())
}

#[test]
fn test_create_worktree_with_complex_relative_path() -> Result<()> {
    let (temp_dir, manager, _repo_path) = setup_test_repo()?;

    // Create a subdirectory structure
    let sibling_dir = temp_dir.path().join("sibling").join("nested");
    fs::create_dir_all(&sibling_dir)?;

    // Test complex relative path
    let worktree_path = manager.create_worktree("../sibling/nested/feature", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify it's at the correct nested location
    assert!(worktree_path.to_string_lossy().contains("sibling/nested"));
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "feature"
    );

    Ok(())
}

#[test]
fn test_create_worktree_path_normalization() -> Result<()> {
    let (_temp_dir, manager, repo_path) = setup_test_repo()?;

    // Test path with ".." components
    let worktree_path = manager.create_worktree("worktrees/../feature-norm", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Should be normalized to repository directory level
    assert_eq!(
        worktree_path.parent().unwrap().canonicalize()?,
        repo_path.canonicalize()?
    );

    Ok(())
}

#[test]
fn test_create_worktree_with_trailing_slash() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo()?;

    // Test path with trailing slash
    let worktree_path = manager.create_worktree("../feature-trail/", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Name should not include trailing slash
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "feature-trail"
    );

    Ok(())
}

#[test]
fn test_create_worktree_error_on_existing_worktree() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo()?;

    // Create first worktree successfully
    manager.create_worktree("../existing-worktree", None)?;

    // Try to create another worktree with the same name
    let result = manager.create_worktree("../existing-worktree", None);

    // Should fail with appropriate error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("already exists")
            || error_msg.contains("File exists")
            || error_msg.contains("is not an empty directory")
            || error_msg.contains("already registered"),
        "Expected error about existing path, got: {error_msg}"
    );

    Ok(())
}

#[test]
fn test_create_worktree_from_head_detached_state() -> Result<()> {
    let (_temp_dir, manager, repo_path) = setup_test_repo()?;

    // Get current commit hash
    let repo = git2::Repository::open(&repo_path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let commit_id = commit.id();

    // Checkout commit directly to create detached HEAD
    repo.set_head_detached(commit_id)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    // Create worktree from detached HEAD
    let worktree_path = manager.create_worktree("../detached-worktree", None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify new branch was created for the worktree
    let worktrees = manager.list_worktrees()?;
    let detached_wt = worktrees.iter().find(|w| w.name == "detached-worktree");
    assert!(detached_wt.is_some());

    // The worktree should have its own branch
    assert!(!detached_wt.unwrap().branch.is_empty());

    Ok(())
}

#[test]
fn test_first_worktree_pattern_selection() -> Result<()> {
    let (_temp_dir, manager, repo_path) = setup_test_repo()?;

    // Verify no worktrees exist yet
    let worktrees = manager.list_worktrees()?;
    assert_eq!(worktrees.len(), 0);

    // Create first worktree with same-level pattern
    let worktree_path = manager.create_worktree("../first-pattern", None)?;

    // Verify it was created at the correct level
    assert!(worktree_path.exists());
    let expected_parent = repo_path.parent().unwrap();
    assert_eq!(
        worktree_path.parent().unwrap().canonicalize()?,
        expected_parent.canonicalize()?
    );

    // Create second worktree with simple name
    let second_path = manager.create_worktree("second-pattern", None)?;

    // Should follow the established pattern (same level)
    assert_eq!(
        second_path.parent().unwrap().canonicalize()?,
        expected_parent.canonicalize()?
    );

    Ok(())
}
