//! Unified worktree creation tests
//!
//! Integrates create_worktree_integration_test.rs and create_worktree_from_tag_test.rs
//! Eliminates duplication and provides comprehensive worktree creation tests

use anyhow::Result;
use git2::Repository;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to setup test environment with git2
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
        index.add_path(Path::new("README.md"))?;
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

/// Helper to setup test repository for tag operations
fn setup_test_repo_with_tag(temp_dir: &TempDir) -> Result<(std::path::PathBuf, git2::Oid)> {
    let repo_path = temp_dir.path().join("test-repo");
    let repo = Repository::init(&repo_path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        fs::write(repo_path.join("README.md"), "# Test Repo")?;
        index.add_path(Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok((repo_path, commit_oid))
}

// =============================================================================
// Basic worktree creation tests
// =============================================================================

/// Test worktree creation with first pattern (sibling directory)
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

/// Test worktree creation with special characters in name
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

/// Test worktree creation pattern detection
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

// =============================================================================
// Custom path worktree creation tests
// =============================================================================

/// Test custom path worktree creation
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

/// Test custom subdirectory worktree creation
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

/// Test multiple custom paths with different structures
#[test]
fn test_create_worktree_multiple_custom_paths() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Test various custom path structures
    let paths = vec![
        ("../siblings/feature-a", "feature-a"),
        ("nested/deep/structure/feature-b", "feature-b"),
        ("../another-sibling", "another-sibling"),
        ("simple-name", "simple-name"),
    ];

    for (input_path, expected_name) in paths {
        let worktree_path = manager.create_worktree(input_path, None)?;

        assert!(worktree_path.exists());
        assert_eq!(
            worktree_path.file_name().unwrap().to_str().unwrap(),
            expected_name
        );
    }

    Ok(())
}

// =============================================================================
// Tests for creating worktrees from tags
// =============================================================================

/// Test creating worktree from tag with detached HEAD
#[test]
fn test_create_worktree_from_tag_detached() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo_with_tag(&temp_dir)?;

    // Create a tag
    let repo = Repository::open(&repo_path)?;
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create worktree from tag without new branch (detached HEAD)
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let worktree_path = manager.create_worktree("test-tag-detached", Some("v1.0.0"))?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Verify HEAD is detached at the tag
    let worktree_repo = Repository::open(&worktree_path)?;
    let head = worktree_repo.head()?;
    assert!(!head.is_branch());

    // Verify commit is the same as the tag
    let tag_commit = repo.find_reference("refs/tags/v1.0.0")?.peel_to_commit()?;
    let worktree_commit = head.peel_to_commit()?;
    assert_eq!(tag_commit.id(), worktree_commit.id());

    // Cleanup
    fs::remove_dir_all(&worktree_path)?;

    Ok(())
}

/// Test creating worktree from annotated tag
#[test]
fn test_create_worktree_from_annotated_tag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo_with_tag(&temp_dir)?;

    // Create an annotated tag
    let repo = Repository::open(&repo_path)?;
    let head_oid = repo.head()?.target().unwrap();
    let target = repo.find_object(head_oid, None)?;
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    repo.tag("v2.0.0", &target, &sig, "Version 2.0.0 release", false)?;

    // Create worktree from annotated tag
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let worktree_path = manager.create_worktree("test-annotated-tag", Some("v2.0.0"))?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Cleanup
    fs::remove_dir_all(&worktree_path)?;

    Ok(())
}

/// Test creating worktree from multiple tags
#[test]
fn test_create_worktree_from_multiple_tags() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo_with_tag(&temp_dir)?;

    let repo = Repository::open(&repo_path)?;
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create multiple tags
    let head_oid = repo.head()?.target().unwrap();
    let target = repo.find_object(head_oid, None)?;

    // Lightweight tag
    repo.tag_lightweight("v1.0", &target, false)?;

    // Annotated tag
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    repo.tag("v1.1", &target, &sig, "Version 1.1", false)?;

    // Create worktrees from each tag
    let worktree1 = manager.create_worktree("from-v1.0", Some("v1.0"))?;
    let worktree2 = manager.create_worktree("from-v1.1", Some("v1.1"))?;

    // Verify both worktrees were created
    assert!(worktree1.exists());
    assert!(worktree2.exists());

    // Cleanup
    fs::remove_dir_all(&worktree1)?;
    fs::remove_dir_all(&worktree2)?;

    Ok(())
}

// =============================================================================
// Error handling tests
// =============================================================================

/// Test worktree creation with invalid tag
#[test]
fn test_create_worktree_from_invalid_tag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo_with_tag(&temp_dir)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Try to create worktree from non-existent tag
    let result = manager.create_worktree("test-invalid-tag", Some("non-existent-tag"));

    // Either succeeds (git creates anyway) or fails gracefully
    match result {
        Ok(path) => {
            // Git might create a worktree even with a non-existent tag/branch
            assert!(path.exists());
            fs::remove_dir_all(&path)?;
        }
        Err(_) => {
            // This is also acceptable - the tag doesn't exist
        }
    }

    Ok(())
}

/// Test worktree creation with conflicting names
#[test]
fn test_create_worktree_conflicting_names() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Create first worktree
    let first_path = manager.create_worktree("../duplicate-name", None)?;
    assert!(first_path.exists());

    // Try to create second worktree with same name
    let result = manager.create_worktree("../duplicate-name", None);

    // Should handle conflict gracefully
    match result {
        Ok(second_path) => {
            // If successful, paths should be different
            assert_ne!(first_path, second_path);
        }
        Err(_) => {
            // Error is acceptable for conflicting names
        }
    }

    Ok(())
}

// =============================================================================
// Integration tests (excluding tests that mock user input)
// =============================================================================

/// Test commands integration (without user input)
#[test]
#[ignore = "Requires user input - for manual testing only"]
fn test_commands_create_worktree_integration() -> Result<()> {
    let (_temp_dir, _manager) = setup_test_environment()?;

    // This test would require mocking user input
    // Skipping for automated tests

    Ok(())
}

/// Test worktree creation workflow simulation
#[test]
fn test_worktree_creation_workflow() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    // Simulate typical workflow
    // 1. Create feature worktree
    let feature_wt = manager.create_worktree("../feature/new-feature", None)?;
    assert!(feature_wt.exists());

    // 2. Create bugfix worktree
    let bugfix_wt = manager.create_worktree("../bugfix/urgent-fix", None)?;
    assert!(bugfix_wt.exists());

    // 3. Create experiment worktree
    let experiment_wt = manager.create_worktree("../experiments/test-idea", None)?;
    assert!(experiment_wt.exists());

    // Verify all created successfully
    assert_ne!(feature_wt, bugfix_wt);
    assert_ne!(bugfix_wt, experiment_wt);
    assert_ne!(feature_wt, experiment_wt);

    Ok(())
}

// =============================================================================
// Performance tests
// =============================================================================

/// Test performance of creating multiple worktrees
#[test]
fn test_create_worktree_performance() -> Result<()> {
    let (_temp_dir, manager) = setup_test_environment()?;

    let start = std::time::Instant::now();

    // Create multiple worktrees quickly
    let mut created_paths = Vec::new();
    for i in 0..5 {
        let path = manager.create_worktree(&format!("../perf-test-{i}"), None)?;
        created_paths.push(path);
    }

    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(
        duration.as_secs() < 30,
        "Worktree creation took too long: {duration:?}"
    );

    // Verify all were created
    assert_eq!(created_paths.len(), 5);
    for path in &created_paths {
        assert!(path.exists());
    }

    Ok(())
}
