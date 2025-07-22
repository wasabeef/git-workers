use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

/// Helper function to create a test repository with initial commit
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Initialize repository
    git2::Repository::init(repo_path)?;

    // Create initial commit
    let repo = git2::Repository::open(repo_path)?;
    let sig = git2::Signature::now(config::TEST_USER_NAME, config::TEST_USER_EMAIL)?;
    let tree_id = {
        let mut index = repo.index()?;
        let file_path = repo_path.join(config::README_FILENAME);
        fs::write(&file_path, config::DEFAULT_README_CONTENT)?;
        index.add_path(Path::new(config::README_FILENAME))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        config::INITIAL_COMMIT_MESSAGE,
        &tree,
        &[],
    )?;

    let manager = GitWorktreeManager::new_from_path(repo_path)?;
    Ok((temp_dir, manager))
}

#[test]
#[serial]
fn test_rename_worktree_preserves_branch_in_non_bare_repo() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with a new branch
    let worktree_name = "feature-test";
    let branch_name = "feature-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // List worktrees before rename
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");
    assert_eq!(wt_before.branch, branch_name);

    // Rename the worktree
    let new_name = "renamed-feature";
    manager.rename_worktree(worktree_name, new_name)?;

    // List worktrees after rename
    let worktrees_after = manager.list_worktrees()?;

    // The worktree should still be tracked by its original name internally
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should still be tracked by original name");

    // But the branch should still be correctly identified
    assert_eq!(
        wt_after.branch, branch_name,
        "Branch should not become 'unknown'"
    );

    // Verify the path has been updated
    assert!(wt_after.path.ends_with(new_name));
    assert!(wt_after.path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_preserves_branch_in_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Initialize bare repository
    git2::Repository::init_bare(repo_path)?;

    // Create initial commit using temporary clone
    let temp_clone_dir = TempDir::new()?;
    let clone_path = temp_clone_dir.path();
    let clone = git2::Repository::clone(repo_path.to_str().unwrap(), clone_path)?;

    let sig = git2::Signature::now(config::TEST_USER_NAME, config::TEST_USER_EMAIL)?;
    let tree_id = {
        let mut index = clone.index()?;
        let file_path = clone_path.join(config::README_FILENAME);
        fs::write(&file_path, config::DEFAULT_README_CONTENT)?;
        index.add_path(Path::new(config::README_FILENAME))?;
        index.write()?;
        index.write_tree()?
    };

    {
        let tree = clone.find_tree(tree_id)?;
        clone.commit(
            Some("HEAD"),
            &sig,
            &sig,
            config::INITIAL_COMMIT_MESSAGE,
            &tree,
            &[],
        )?;

        // Push to bare repo
        let mut remote = clone.find_remote("origin")?;
        remote.push(&["refs/heads/main"], None)?;
    }
    drop(clone);
    drop(temp_clone_dir);

    // Now work with the bare repository
    let manager = GitWorktreeManager::new_from_path(repo_path)?;

    // Create a worktree with a new branch
    let worktree_name = "feature-test";
    let branch_name = "feature-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // List worktrees before rename
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");
    assert_eq!(wt_before.branch, branch_name);

    // Rename the worktree
    let new_name = "renamed-feature";
    manager.rename_worktree(worktree_name, new_name)?;

    // List worktrees after rename
    let worktrees_after = manager.list_worktrees()?;

    // The worktree should still be tracked by its original name internally
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should still be tracked by original name");

    // But the branch should still be correctly identified
    assert_eq!(
        wt_after.branch, branch_name,
        "Branch should not become 'unknown'"
    );

    // Verify the path has been updated
    assert!(wt_after.path.ends_with(new_name));
    assert!(wt_after.path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_with_spaces_in_path() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "feature-test";
    let branch_name = "feature-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Try to rename with spaces (should fail)
    let new_name = "renamed feature";
    let result = manager.rename_worktree(worktree_name, new_name);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("spaces"));

    // Verify original worktree still works
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Original worktree should still exist");
    assert_eq!(wt.branch, branch_name);

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_updates_path_correctly() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "test-worktree";
    let branch_name = "test-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Get original path
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .unwrap();
    let old_path = wt_before.path.clone();

    // Rename
    let new_name = "renamed-worktree";
    manager.rename_worktree(worktree_name, new_name)?;

    // Check new path
    let worktrees_after = manager.list_worktrees()?;
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == worktree_name)
        .unwrap();

    // Old path should not exist
    assert!(!old_path.exists());

    // New path should exist
    assert!(wt_after.path.exists());
    assert!(wt_after.path.ends_with(new_name));

    // Should be able to open repository at new path
    let repo = git2::Repository::open(&wt_after.path)?;
    let head = repo.head()?;
    assert_eq!(head.shorthand(), Some(branch_name));

    Ok(())
}

#[test]
#[serial]
fn test_cannot_rename_current_worktree() -> Result<()> {
    let (temp_dir, _) = setup_test_repo()?;

    // Create manager from the main worktree
    let manager = GitWorktreeManager::new_from_path(temp_dir.path())?;

    // Try to rename main worktree (should fail)
    let result = manager.rename_worktree("main", "renamed-main");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("current worktree"));

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_with_branch_rename() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree
    let worktree_name = "feature-xyz";
    let branch_name = "feature-xyz"; // Same as worktree name
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Rename worktree
    let new_name = "renamed-feature";
    manager.rename_worktree(worktree_name, new_name)?;

    // Also rename the branch
    manager.rename_branch(branch_name, new_name)?;

    // Verify the worktree still shows the correct (new) branch name
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");

    assert_eq!(wt.branch, new_name, "Branch name should be updated");

    Ok(())
}
