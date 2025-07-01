use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use tempfile::TempDir;

fn setup_test_environment() -> Result<(TempDir, GitWorktreeManager)> {
    // Create a parent directory for our test
    let parent_dir = TempDir::new()?;
    let repo_path = parent_dir.path().join("test-repo");
    fs::create_dir(&repo_path)?;

    // Initialize repository
    let repo = git2::Repository::init(&repo_path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        fs::write(repo_path.join("README.md"), "# Test Repo")?;
        index.add_path(std::path::Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    // Change to repo directory
    std::env::set_current_dir(&repo_path)?;

    let manager = GitWorktreeManager::new()?;
    Ok((parent_dir, manager))
}

#[test]
#[ignore = "Requires user input - for manual testing only"]
fn test_commands_create_worktree_integration() -> Result<()> {
    let (_temp_dir, _manager) = setup_test_environment()?;

    // This test would require mocking user input
    // Skipping for automated tests

    Ok(())
}

#[test]
fn test_create_worktree_internal_with_first_pattern() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Test first worktree creation with "../" pattern
    let worktree_path = manager.create_worktree("../first-worktree", None)?;

    // Verify worktree was created at correct location
    assert!(worktree_path.exists());
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "first-worktree"
    );

    // Verify it's at the same level as the repository
    // The worktree should be a sibling to the test-repo directory
    let current_dir = std::env::current_dir()?;
    let repo_parent = current_dir.parent().unwrap();

    // Both should have the same parent directory
    // Use canonicalize to resolve any symlinks for comparison
    let worktree_parent = worktree_path
        .canonicalize()?
        .parent()
        .unwrap()
        .to_path_buf();
    let expected_parent = repo_parent.canonicalize()?;

    assert_eq!(
        worktree_parent, expected_parent,
        "Worktree should be at the same level as the repository"
    );

    Ok(())
}

#[test]
fn test_create_worktree_bare_repository() -> Result<()> {
    // Create a bare repository
    let parent_dir = TempDir::new()?;
    let bare_repo_path = parent_dir.path().join("test.git");

    let bare_repo = git2::Repository::init_bare(&bare_repo_path)?;

    // Create initial commit using plumbing commands
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut builder = bare_repo.treebuilder(None)?;
        let blob_id = bare_repo.blob(b"# Test Content")?;
        builder.insert("README.md", blob_id, 0o100644)?;
        builder.write()?
    };

    let tree = bare_repo.find_tree(tree_id)?;
    bare_repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    std::env::set_current_dir(&bare_repo_path)?;
    let manager = GitWorktreeManager::new()?;

    // Create worktree from bare repository with unique name
    let unique_name = format!("../bare-worktree-{}", std::process::id());
    let worktree_path = manager.create_worktree(&unique_name, None)?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    Ok(())
}

#[test]
fn test_create_worktree_with_special_characters() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Test worktree name with hyphens and numbers
    let special_name = "../feature-123-test";
    let worktree_path = manager.create_worktree(special_name, None)?;

    assert!(worktree_path.exists());
    assert_eq!(
        worktree_path.file_name().unwrap().to_str().unwrap(),
        "feature-123-test"
    );

    Ok(())
}

#[test]
fn test_create_worktree_pattern_detection() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Create first worktree to establish pattern
    let first = manager.create_worktree("worktrees/first", None)?;
    assert!(first.to_string_lossy().contains("worktrees"));

    // Create second with simple name - should follow pattern
    let second = manager.create_worktree("second", None)?;

    // Second should also be in worktrees subdirectory
    assert!(second.to_string_lossy().contains("worktrees"));

    // Both should have "worktrees" as their parent directory name
    assert_eq!(
        first.parent().unwrap().file_name().unwrap(),
        second.parent().unwrap().file_name().unwrap()
    );

    Ok(())
}

#[test]
fn test_create_worktree_custom_path() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Test custom relative path
    let custom_worktree = manager.create_worktree("../custom-location/my-worktree", None)?;

    // Verify worktree was created at the specified custom location
    assert!(custom_worktree.exists());
    assert_eq!(
        custom_worktree.file_name().unwrap().to_str().unwrap(),
        "my-worktree"
    );

    // Verify it's in the custom directory structure
    assert!(custom_worktree
        .to_string_lossy()
        .contains("custom-location"));

    Ok(())
}

#[test]
fn test_create_worktree_custom_subdirectory() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Test custom subdirectory path
    let custom_worktree = manager.create_worktree("temp/experiments/test-feature", None)?;

    // Verify worktree was created at the specified location
    assert!(custom_worktree.exists());
    assert_eq!(
        custom_worktree.file_name().unwrap().to_str().unwrap(),
        "test-feature"
    );

    // Verify it's in the correct subdirectory structure
    assert!(custom_worktree.to_string_lossy().contains("temp"));
    assert!(custom_worktree.to_string_lossy().contains("experiments"));

    Ok(())
}
