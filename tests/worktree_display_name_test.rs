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

/// Helper function to create a bare test repository with initial commit
fn setup_bare_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
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
    Ok((temp_dir, manager))
}

#[test]
#[serial]
fn test_worktree_display_name_vs_git_name() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let worktree_name = format!("test-worktree-{timestamp}");
    let branch_name = format!("feature-branch-{timestamp}");
    let new_name = format!("renamed-worktree-{timestamp}");

    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, "main")?;

    // Rename the worktree
    manager.rename_worktree(&worktree_name, &new_name)?;

    // List worktrees
    let worktrees = manager.list_worktrees()?;
    let renamed_wt = worktrees
        .iter()
        .find(|w| w.path.ends_with(&new_name))
        .expect("Should find renamed worktree");

    // Verify display name vs git name
    assert_eq!(
        renamed_wt.name, new_name,
        "Display name should be the directory name"
    );
    assert_eq!(
        renamed_wt.git_name, worktree_name,
        "Git name should remain unchanged"
    );
    assert_eq!(renamed_wt.branch, branch_name, "Branch should be preserved");

    Ok(())
}

#[test]
#[serial]
fn test_delete_renamed_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create unique names with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let original_name = format!("delete-test-{timestamp}");
    let branch_name = format!("delete-branch-{timestamp}");
    let new_name = format!("renamed-delete-test-{timestamp}");

    // Create and rename a worktree
    manager.create_worktree_with_new_branch(&original_name, &branch_name, "main")?;
    manager.rename_worktree(&original_name, &new_name)?;

    // Try to delete using the renamed worktree
    let worktrees = manager.list_worktrees()?;
    let wt_to_delete = worktrees
        .iter()
        .find(|w| w.name == new_name)
        .expect("Should find renamed worktree");

    // Delete should use git_name internally
    manager.remove_worktree(&wt_to_delete.git_name)?;

    // Verify deletion
    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.git_name == original_name),
        "Worktree should be deleted"
    );

    Ok(())
}

#[test]
#[serial]
fn test_multiple_renames_tracking() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create unique names with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let original_name = format!("multi-rename-test-{timestamp}");
    let branch_name = format!("multi-branch-{timestamp}");
    let first_rename = format!("first-rename-{timestamp}");
    let second_rename = format!("second-rename-{timestamp}");

    // Create a worktree
    manager.create_worktree_with_new_branch(&original_name, &branch_name, "main")?;

    // First rename
    manager.rename_worktree(&original_name, &first_rename)?;

    // Second rename
    manager.rename_worktree(&original_name, &second_rename)?;

    // List and verify
    let worktrees = manager.list_worktrees()?;
    let final_wt = worktrees
        .iter()
        .find(|w| w.path.ends_with(&second_rename))
        .expect("Should find final renamed worktree");

    assert_eq!(
        final_wt.name, second_rename,
        "Display name should be the final directory name"
    );
    assert_eq!(
        final_wt.git_name, original_name,
        "Git name should still be the original"
    );
    assert_eq!(
        final_wt.branch, branch_name,
        "Branch should be preserved through multiple renames"
    );

    Ok(())
}

#[test]
#[serial]
fn test_worktree_with_different_name_and_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create unique names with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let worktree_name = format!("frontend-{timestamp}");
    let branch_name = format!("feature/new-ui-{timestamp}");
    let new_worktree_name = format!("ui-development-{timestamp}");

    // Create worktree with different name than branch
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, "main")?;

    // Rename worktree
    manager.rename_worktree(&worktree_name, &new_worktree_name)?;

    // List and verify
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == new_worktree_name)
        .expect("Should find renamed worktree");

    assert_eq!(wt.name, new_worktree_name);
    assert_eq!(wt.git_name, worktree_name);
    assert_eq!(wt.branch, branch_name);

    // Test operations using the correct name
    // This would fail if using display name instead of git_name
    manager.remove_worktree(&wt.git_name)?;

    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.git_name == worktree_name),
        "Worktree should be deleted"
    );

    Ok(())
}

#[test]
#[serial]
fn test_bare_repo_worktree_naming() -> Result<()> {
    let (_temp_dir, manager) = setup_bare_test_repo()?;

    // Create unique names with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let worktree_name = format!("bare-test-wt-{timestamp}");
    let branch_name = format!("bare-feature-{timestamp}");
    let new_name = format!("bare-renamed-wt-{timestamp}");

    // Create and rename worktree
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, "main")?;
    manager.rename_worktree(&worktree_name, &new_name)?;

    // Verify naming
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == new_name)
        .expect("Should find renamed worktree");

    assert_eq!(wt.name, new_name);
    assert_eq!(wt.git_name, worktree_name);
    assert_eq!(wt.branch, branch_name);

    Ok(())
}
