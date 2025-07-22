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

    // Create a worktree with a new branch using unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("feature-test-{timestamp}");
    let branch_name = format!("feature-branch-{timestamp}");
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // List worktrees before rename
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");
    assert_eq!(wt_before.branch, branch_name);

    // Rename the worktree
    let new_name = format!("renamed-feature-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_name)?;

    // List worktrees after rename
    let worktrees_after = manager.list_worktrees()?;

    // The worktree should now be tracked by its new display name
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == new_name)
        .expect("Worktree should be tracked by new display name");

    // But the branch should still be correctly identified
    assert_eq!(
        wt_after.branch, branch_name,
        "Branch should not become 'unknown'"
    );

    // Verify the path has been updated
    assert!(wt_after.path.ends_with(&new_name));
    assert!(wt_after.path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_preserves_branch_in_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare.git");
    fs::create_dir(&bare_repo_path)?;

    // Initialize bare repository
    git2::Repository::init_bare(&bare_repo_path)?;

    // Create initial commit using temporary clone
    let temp_clone_dir = TempDir::new()?;
    let clone_path = temp_clone_dir.path();
    let clone = git2::Repository::clone(bare_repo_path.to_str().unwrap(), clone_path)?;

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

        // Push to bare repository
        let mut remote = clone.find_remote("origin")?;
        remote.push(&[&format!("refs/heads/{}", config::MAIN_BRANCH)], None)?;
    }

    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    // Create a worktree with a new branch using unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("feature-test-{timestamp}");
    let branch_name = format!("feature-branch-{timestamp}");
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // List worktrees before rename
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");
    assert_eq!(wt_before.branch, branch_name);

    // Rename the worktree
    let new_name = format!("renamed-feature-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_name)?;

    // List worktrees after rename
    let worktrees_after = manager.list_worktrees()?;

    // The worktree should now be tracked by its new display name
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == new_name)
        .expect("Worktree should be tracked by new display name");

    // But the branch should still be correctly identified
    assert_eq!(
        wt_after.branch, branch_name,
        "Branch should not become 'unknown'"
    );

    // Verify the path has been updated
    assert!(wt_after.path.ends_with(&new_name));
    assert!(wt_after.path.exists());

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_with_spaces_in_path() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("feature-test-{timestamp}");
    let branch_name = format!("feature-branch-{timestamp}");
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Try to rename with spaces (should fail)
    let new_name = "renamed feature";
    let result = manager.rename_worktree(&worktree_name, new_name);
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

    // Create a worktree with unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("test-worktree-{timestamp}");
    let branch_name = format!("test-branch-{timestamp}");
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Get original path
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .unwrap();
    let old_path = wt_before.path.clone();

    // Rename
    let new_name = format!("renamed-worktree-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_name)?;

    // Check new path
    let worktrees_after = manager.list_worktrees()?;
    let wt_after = worktrees_after.iter().find(|w| w.name == new_name).unwrap();

    // Old path should not exist
    assert!(!old_path.exists());

    // New path should exist
    assert!(wt_after.path.exists());
    assert!(wt_after.path.ends_with(&new_name));

    // Should be able to open repository at new path
    let repo = git2::Repository::open(&wt_after.path)?;
    let head = repo.head()?;
    assert_eq!(head.shorthand(), Some(&branch_name[..]));

    Ok(())
}

#[test]
#[serial]
fn test_rename_worktree_with_branch_rename() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("feature-xyz-{timestamp}");
    let branch_name = format!("feature-xyz-{timestamp}");
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Rename the worktree
    let new_worktree_name = format!("feature-abc-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_worktree_name)?;

    // The branch should still be the original branch name (not automatically renamed)
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == new_worktree_name)
        .expect("Renamed worktree should exist");

    assert_eq!(wt.branch, branch_name);
    assert!(wt.path.ends_with(&new_worktree_name));

    Ok(())
}

#[test]
#[serial]
fn test_cannot_rename_current_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Try to rename the main worktree (should fail)
    let result = manager.rename_worktree("main", "renamed-main");

    // This should fail - we cannot rename the current/main worktree
    assert!(result.is_err());
    // Note: The specific error message may vary depending on implementation

    Ok(())
}
