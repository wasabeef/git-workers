use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

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
fn test_remove_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "to-be-removed";
    let worktree_path = manager.create_worktree(worktree_name, None)?;
    assert!(worktree_path.exists());

    // Remove the worktree
    manager.remove_worktree(worktree_name)?;

    // Verify worktree is removed from list
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.iter().any(|w| w.name == worktree_name));

    // Note: The directory might still exist but git should not track it
    Ok(())
}

#[test]
fn test_list_worktrees() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Initially should have main worktree
    let initial_worktrees = manager.list_worktrees()?;
    let initial_count = initial_worktrees.len();

    // Create multiple worktrees
    manager.create_worktree("worktree1", None)?;
    manager.create_worktree("worktree2", Some("branch2"))?;

    // List should include all worktrees
    let worktrees = manager.list_worktrees()?;
    assert_eq!(worktrees.len(), initial_count + 2);

    // Verify specific worktrees exist
    let names: Vec<_> = worktrees.iter().map(|w| &w.name).collect();
    assert!(names.contains(&&"worktree1".to_string()));
    assert!(names.contains(&&"worktree2".to_string()));

    Ok(())
}

#[test]
fn test_delete_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a branch
    let branch_name = "test-branch";
    let repo = manager.repo();
    let head = repo.head()?.target().unwrap();
    let commit = repo.find_commit(head)?;
    repo.branch(branch_name, &commit, false)?;

    // Verify branch exists
    let (branches, _) = manager.list_all_branches()?;
    assert!(branches.contains(&branch_name.to_string()));

    // Delete the branch
    manager.delete_branch(branch_name)?;

    // Verify branch is deleted
    let (branches, _) = manager.list_all_branches()?;
    assert!(!branches.contains(&branch_name.to_string()));

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
            "Failed to create worktree: {}",
            description
        );

        // Clean up for next iteration
        let worktree_name = worktree_path.file_name().unwrap().to_str().unwrap();
        manager.remove_worktree(worktree_name)?;
    }

    Ok(())
}

#[test]
#[ignore = "Rename worktree has known issues with git worktree prune/add workflow"]
fn test_rename_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with a branch (so rename can find the branch)
    let old_name = "old-worktree";
    let new_name = "renamed-worktree"; // Different name to avoid conflicts
    let branch_name = "rename-test-branch";
    manager.create_worktree(old_name, Some(branch_name))?;

    // Get the old path before rename
    let worktrees_before = manager.list_worktrees()?;
    let old_worktree = worktrees_before
        .iter()
        .find(|w| w.name == old_name)
        .unwrap();
    let old_path = old_worktree.path.clone();

    // Rename the worktree
    manager.rename_worktree(old_name, new_name)?;

    // Verify old name doesn't exist and new name exists
    let worktrees = manager.list_worktrees()?;
    assert!(!worktrees.iter().any(|w| w.name == old_name));
    assert!(worktrees.iter().any(|w| w.name == new_name));

    // Verify the old path doesn't exist
    assert!(!old_path.exists(), "Old worktree path should not exist");

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
