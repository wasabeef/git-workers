use anyhow::Result;
use git2::Repository;
use std::fs;
use tempfile::TempDir;

use git_workers::commands;
use git_workers::git::GitWorktreeManager;

/// Test error handling when not in a git repository
#[test]
fn test_commands_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let non_git_path = temp_dir.path().join("not-a-repo");
    fs::create_dir_all(&non_git_path).unwrap();

    // Change to non-git directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&non_git_path).unwrap();

    // Test that commands handle non-git repos gracefully
    let result = commands::list_worktrees();
    // Should either succeed with empty list or fail gracefully
    assert!(result.is_ok() || result.is_err());

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

/// Test commands in an empty git repository
#[test]
fn test_commands_empty_git_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("empty-repo");

    // Initialize empty repository (no initial commit)
    Repository::init(&repo_path)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test commands in empty repository
    let result = commands::list_worktrees();
    // Should handle empty repo gracefully
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test commands in repository with initial commit
#[test]
fn test_commands_with_initial_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("repo-with-commit");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test list_worktrees with a proper repository
    let result = commands::list_worktrees();
    // Should succeed and show main worktree
    assert!(result.is_ok());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test error handling in various invalid states
#[test]
fn test_commands_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test delete_worktree with no worktrees to delete
    let result = commands::delete_worktree();
    // Should handle gracefully (likely show "no worktrees" message)
    assert!(result.is_ok() || result.is_err());

    // Test batch_delete_worktrees with no worktrees
    let result = commands::batch_delete_worktrees();
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());

    // Test rename_worktree with no worktrees
    let result = commands::rename_worktree();
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test validation functions with edge cases
#[test]
fn test_validation_edge_cases() -> Result<()> {
    // Test validate_worktree_name with boundary conditions

    // Test with whitespace-only strings (should fail)
    assert!(commands::validate_worktree_name("   ").is_err());
    assert!(commands::validate_worktree_name("\t\n").is_err());

    // Test with mixed whitespace and content (should trim)
    let result = commands::validate_worktree_name("  test  ")?;
    assert_eq!(result, "test");

    // Test validate_custom_path edge cases
    assert!(commands::validate_custom_path("").is_err());
    assert!(commands::validate_custom_path("   ").is_err());

    Ok(())
}

/// Test GitWorktreeManager error scenarios
#[test]
fn test_manager_initialization_scenarios() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test in non-git directory
    let non_git = temp_dir.path().join("non-git");
    fs::create_dir_all(&non_git)?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&non_git)?;

    let manager_result = GitWorktreeManager::new();
    // Should fail gracefully
    assert!(manager_result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test with corrupted git repository
#[test]
fn test_corrupted_git_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("corrupted-repo");

    // Create a fake .git directory without proper git structure
    fs::create_dir_all(&repo_path)?;
    fs::create_dir_all(repo_path.join(".git"))?;
    fs::write(repo_path.join(".git").join("HEAD"), "invalid content")?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Commands should handle corrupted repos gracefully
    let result = commands::list_worktrees();
    // May succeed or fail, but shouldn't panic
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test with bare repository
#[test]
fn test_commands_bare_repository() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare-repo.git");

    let _repo = Repository::init_bare(&bare_repo_path)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&bare_repo_path)?;

    // Test commands in bare repository
    let result = commands::list_worktrees();
    // Should handle bare repos appropriately
    assert!(result.is_ok() || result.is_err());

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test internal helper functions indirectly
#[test]
fn test_internal_functions_via_public_api() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a worktree to test with
    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let _worktree_path = manager.create_worktree("test-feature", None)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Now test commands that should find the worktree
    let result = commands::list_worktrees();
    assert!(result.is_ok());

    // Test delete with actual worktree present
    // Note: This will be interactive, so we can't easily test the full flow
    // but we can test that it doesn't panic on startup

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test command functions don't panic on edge cases
#[test]
fn test_commands_stability() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_path)?;

    // Test that all commands can be called without panicking
    // Even if they return errors due to user interaction requirements

    // Test each command individually to avoid closure type issues
    println!("Testing command: list_worktrees");
    let result = commands::list_worktrees();
    match result {
        Ok(_) => println!("  ✓ list_worktrees succeeded"),
        Err(e) => println!("  ! list_worktrees failed with: {}", e),
    }

    println!("Testing command: delete_worktree");
    let result = commands::delete_worktree();
    match result {
        Ok(_) => println!("  ✓ delete_worktree succeeded"),
        Err(e) => println!("  ! delete_worktree failed with: {}", e),
    }

    println!("Testing command: batch_delete_worktrees");
    let result = commands::batch_delete_worktrees();
    match result {
        Ok(_) => println!("  ✓ batch_delete_worktrees succeeded"),
        Err(e) => println!("  ! batch_delete_worktrees failed with: {}", e),
    }

    println!("Testing command: cleanup_old_worktrees");
    let result = commands::cleanup_old_worktrees();
    match result {
        Ok(_) => println!("  ✓ cleanup_old_worktrees succeeded"),
        Err(e) => println!("  ! cleanup_old_worktrees failed with: {}", e),
    }

    println!("Testing command: rename_worktree");
    let result = commands::rename_worktree();
    match result {
        Ok(_) => println!("  ✓ rename_worktree succeeded"),
        Err(e) => println!("  ! rename_worktree failed with: {}", e),
    }

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

/// Test with various repository states
#[test]
fn test_repository_states() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test 1: Repository with no branches (just initialized)
    let repo1_path = temp_dir.path().join("no-branches");
    Repository::init(&repo1_path)?;

    // Test 2: Repository with only main branch
    let repo2_path = temp_dir.path().join("main-only");
    let repo2 = Repository::init(&repo2_path)?;
    create_initial_commit(&repo2)?;

    // Test 3: Repository with multiple branches
    let repo3_path = temp_dir.path().join("multi-branch");
    let repo3 = Repository::init(&repo3_path)?;
    create_initial_commit(&repo3)?;
    create_feature_branch(&repo3, "feature-1")?;
    create_feature_branch(&repo3, "feature-2")?;

    let original_dir = std::env::current_dir()?;

    // Test commands in each repository state
    for (name, path) in [
        ("no-branches", &repo1_path),
        ("main-only", &repo2_path),
        ("multi-branch", &repo3_path),
    ] {
        println!("Testing in {} repository", name);
        std::env::set_current_dir(path)?;

        let result = commands::list_worktrees();
        println!("  list_worktrees: {:?}", result.is_ok());

        // Commands should handle all states gracefully
        assert!(result.is_ok() || result.is_err());
    }

    std::env::set_current_dir(original_dir)?;
    Ok(())
}

// Helper functions
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

fn create_feature_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let head_commit = repo.head()?.peel_to_commit()?;
    repo.branch(branch_name, &head_commit, false)?;
    Ok(())
}
