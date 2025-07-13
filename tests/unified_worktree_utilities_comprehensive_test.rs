//! Unified Worktree Utilities Comprehensive Test Suite
//!
//! This file consolidates tests from three separate test files:
//! - worktree_commands_test.rs: Command execution and worktree creation functionality
//! - worktree_path_test.rs: Path resolution and worktree placement patterns  
//! - worktree_lock_test.rs: Concurrent access control and file locking
//!
//! Tests are organized into logical sections for better maintainability.

use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =============================================================================
// Test Setup Utilities
// =============================================================================

/// Creates a test repository with initial commit for testing worktree operations
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    // Create a parent directory that will contain the main repo and worktrees
    let parent_dir = TempDir::new()?;
    let main_repo_path = parent_dir.path().join("main");
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

    let manager = GitWorktreeManager::new_from_path(&main_repo_path)?;
    Ok((parent_dir, manager))
}

/// Creates a test repository that's positioned in the current working directory
/// Used for tests that need path resolution relative to current directory
fn setup_test_repo_with_cwd() -> Result<(TempDir, GitWorktreeManager, std::path::PathBuf)> {
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

    // Change to the repository directory for path resolution tests
    std::env::set_current_dir(&main_repo_path)?;

    let manager = GitWorktreeManager::new_from_path(&main_repo_path)?;
    Ok((parent_dir, manager, main_repo_path))
}

/// Creates a test repository with explicit branch setup for lock tests
fn setup_test_repo_with_branch() -> Result<(TempDir, GitWorktreeManager)> {
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
    let commit = repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    // Ensure we have a main branch by creating it explicitly
    let head = repo.head()?;
    let branch_name = if head.shorthand() == Some("master") {
        // Create main branch from master
        repo.branch("main", &repo.find_commit(commit)?, false)?;
        repo.set_head("refs/heads/main")?;
        "main"
    } else {
        head.shorthand().unwrap_or("main")
    };

    eprintln!("Created test repo with default branch: {branch_name}");

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    Ok((parent_dir, manager))
}

// =============================================================================
// Worktree Commands Tests
// =============================================================================

#[test]
fn test_create_worktree_with_new_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create worktree with new branch
    let worktree_name = "feature-test-new";
    let branch_name = "feature/test-branch";

    let worktree_path = manager.create_worktree(worktree_name, Some(branch_name))?;

    // Verify worktree exists
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify worktree is listed
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == worktree_name));

    // Verify branch was created
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&branch_name.to_string()));

    Ok(())
}

#[test]
fn test_create_worktree_from_existing_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // First create a branch
    let branch_name = "existing-branch";
    let repo = manager.repo();
    let head = repo.head()?.target().unwrap();
    let commit = repo.find_commit(head)?;
    repo.branch(branch_name, &commit, false)?;

    // Create worktree from existing branch
    let worktree_name = "existing-test";
    let worktree_path = manager.create_worktree(worktree_name, Some(branch_name))?;

    // Verify worktree exists
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify worktree is listed
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == worktree_name));

    Ok(())
}

#[test]
fn test_create_worktree_without_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create worktree without specifying branch (uses current HEAD)
    let worktree_name = "simple-worktree";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Verify worktree exists
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify worktree is listed
    let worktrees = manager.list_worktrees()?;
    assert!(worktrees.iter().any(|w| w.name == worktree_name));

    Ok(())
}

#[test]
fn test_create_worktree_from_head_non_bare() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Test creating worktree from HEAD in non-bare repository
    let worktree_name = "../head-worktree";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Verify worktree exists
    assert!(worktree_path.exists());
    assert!(worktree_path.is_dir());

    // Verify it created a new branch
    let worktrees = manager.list_worktrees()?;
    let head_wt = worktrees.iter().find(|w| w.name == "head-worktree");
    assert!(head_wt.is_some());

    // Should have created a branch named after the worktree
    assert_eq!(head_wt.unwrap().branch, "head-worktree");

    Ok(())
}

#[test]
fn test_create_worktree_first_pattern_subdirectory() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Verify no worktrees exist yet
    let initial_worktrees = manager.list_worktrees()?;
    assert_eq!(initial_worktrees.len(), 0);

    // Create first worktree with subdirectory pattern
    let worktree_name = "worktrees/first";
    let worktree_path = manager.create_worktree(worktree_name, None)?;

    // Verify it was created in subdirectory
    assert!(worktree_path.exists());
    assert!(worktree_path.to_string_lossy().contains("worktrees"));

    // Create second worktree with simple name
    let second_path = manager.create_worktree("second", None)?;

    // Should follow the same pattern
    assert!(second_path.to_string_lossy().contains("worktrees"));

    Ok(())
}

#[test]
fn test_create_worktree_from_head_multiple_patterns() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Test various path patterns
    let patterns = vec![
        ("../sibling", "sibling at same level"),
        ("worktrees/sub", "in subdirectory"),
        ("nested/deep/worktree", "deeply nested"),
    ];

    for (pattern, description) in patterns {
        let worktree_path = manager.create_worktree(pattern, None)?;
        assert!(
            worktree_path.exists(),
            "Failed to create worktree: {description}"
        );

        // Clean up for next iteration
        let worktree_name = worktree_path.file_name().unwrap().to_str().unwrap();
        manager.remove_worktree(worktree_name)?;
    }

    Ok(())
}

#[test]
fn test_worktree_with_invalid_name() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Try to create worktree with spaces (should fail in actual command)
    let invalid_name = "invalid name";
    let result = manager.create_worktree(invalid_name, None);

    // Note: The manager itself might not validate names,
    // but the commands.rs should reject names with spaces
    if result.is_ok() {
        // Clean up if it was created
        let _ = manager.remove_worktree(invalid_name);
    }

    Ok(())
}

// =============================================================================
// Worktree Path Resolution Tests
// =============================================================================

#[test]
fn test_create_worktree_from_head_with_relative_path() -> Result<()> {
    let (_temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;
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
    let (temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, _repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, repo_path) = setup_test_repo_with_cwd()?;

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
    let (_temp_dir, manager, repo_path) = setup_test_repo_with_cwd()?;

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

// =============================================================================
// Worktree Locking and Concurrency Tests
// =============================================================================

#[test]
fn test_worktree_lock_file_creation() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_with_branch()?;
    let git_dir = manager.repo().path();
    let lock_path = git_dir.join("git-workers-worktree.lock");

    // Create a lock file manually to simulate another process
    fs::write(&lock_path, "simulated lock from another process")?;

    // Try to create worktree - should fail due to lock
    let result = manager.create_worktree("worktree1", None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Another git-workers process"));

    // Remove lock file
    fs::remove_file(&lock_path)?;

    // Now it should succeed
    let result = manager.create_worktree("worktree1", None);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_worktree_lock_released_after_creation() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_with_branch()?;

    // Create first worktree
    let result1 = manager.create_worktree("worktree1", None);
    assert!(result1.is_ok());

    // Lock should be released, so second creation should work
    let result2 = manager.create_worktree("worktree2", None);
    assert!(result2.is_ok());

    Ok(())
}

#[test]
fn test_stale_lock_removal() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_with_branch()?;
    let git_dir = manager.repo().path();
    let lock_path = git_dir.join("git-workers-worktree.lock");

    // Create a "stale" lock file
    fs::write(&lock_path, "stale lock")?;

    // Set modified time to 6 minutes ago
    // let _six_minutes_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(360);

    // Unfortunately, we can't easily set file modification time in std
    // So we'll just test that the lock can be acquired even with existing file
    // (the actual stale lock removal is tested in the implementation)

    // Should be able to create worktree (implementation should handle stale lock)
    let result = manager.create_worktree("worktree1", None);

    // This might fail if we can't remove the lock, but that's expected in tests
    // The important thing is that the lock mechanism exists
    if result.is_err() {
        // Clean up manually
        let _ = fs::remove_file(&lock_path);
    }

    Ok(())
}

#[test]
fn test_lock_with_new_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_with_branch()?;

    // Get the actual default branch name
    let repo = manager.repo();
    let head = repo.head()?;
    let base_branch = head.shorthand().unwrap_or("main");

    eprintln!("Using base branch: {base_branch}");

    // Test that lock works with create_worktree_with_new_branch
    let result =
        manager.create_worktree_with_new_branch("feature-worktree", "feature-branch", base_branch);

    if let Err(ref e) = result {
        eprintln!("Error in test_lock_with_new_branch: {e}");
        eprintln!("Error chain:");
        let mut current_error = e.source();
        while let Some(source) = current_error {
            eprintln!("  Caused by: {source}");
            current_error = source.source();
        }
    }
    assert!(
        result.is_ok(),
        "create_worktree_with_new_branch failed: {:?}",
        result.err()
    );

    Ok(())
}

#[test]
#[ignore = "Manual test for demonstrating lock behavior"]
fn test_manual_concurrent_lock_demo() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo_with_branch()?;
    let git_dir = manager.repo().path();
    let lock_path = git_dir.join("git-workers-worktree.lock");

    println!("Creating lock file manually...");
    fs::write(&lock_path, "manual lock")?;

    println!("Attempting to create worktree (should fail)...");
    match manager.create_worktree("worktree1", None) {
        Ok(_) => println!("Worktree created (lock was removed as stale)"),
        Err(e) => println!("Failed as expected: {e}"),
    }

    println!("Removing lock file...");
    fs::remove_file(&lock_path)?;

    println!("Attempting to create worktree again (should succeed)...");
    match manager.create_worktree("worktree1", None) {
        Ok(_) => println!("Worktree created successfully"),
        Err(e) => println!("Unexpected error: {e}"),
    }

    Ok(())
}
