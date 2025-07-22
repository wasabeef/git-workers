use anyhow::Result;
use git_workers::git::{GitWorktreeManager, WorktreeInfo};
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod test_constants;
use test_constants::{config, counts, generators, naming};

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

/// Helper function to create and rename a worktree
fn create_and_rename_worktree(
    manager: &GitWorktreeManager,
    original_name: &str,
    branch_name: &str,
    new_name: &str,
) -> Result<WorktreeInfo> {
    manager.create_worktree_with_new_branch(original_name, branch_name, config::MAIN_BRANCH)?;
    manager.rename_worktree(original_name, new_name)?;

    let worktrees = manager.list_worktrees()?;
    let renamed_wt = worktrees
        .iter()
        .find(|w| w.name == new_name)
        .expect("Should find renamed worktree")
        .clone();

    Ok(renamed_wt)
}

/// Helper function to verify worktree properties
fn verify_worktree_properties(
    manager: &GitWorktreeManager,
    expected_name: &str,
    expected_git_name: &str,
    expected_branch: &str,
) -> Result<()> {
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == expected_name)
        .expect("Should find worktree");

    assert_eq!(wt.name, expected_name, "Display name should match");
    assert_eq!(wt.git_name, expected_git_name, "Git name should match");
    assert_eq!(wt.branch, expected_branch, "Branch should match");

    Ok(())
}

#[test]
#[serial]
fn test_switch_to_renamed_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create and rename a worktree with unique names
    let original_name = generators::worktree_name(&format!("{}-test", naming::SWITCH_PREFIX));
    let branch_name = generators::branch_name(&format!("{}-branch", naming::SWITCH_PREFIX));
    let new_name = generators::worktree_name(&format!("{}-renamed", naming::SWITCH_PREFIX));

    let renamed_wt = create_and_rename_worktree(&manager, &original_name, &branch_name, &new_name)?;

    // Verify the worktree uses display name in UI but correct path
    assert_eq!(renamed_wt.name, new_name, "Display name should be new name");
    assert_eq!(
        renamed_wt.git_name, original_name,
        "Git name should be original"
    );
    assert!(
        renamed_wt.path.ends_with(&new_name),
        "Path should reflect new name"
    );

    // The switch operation uses the path, so it should work regardless of naming
    assert!(renamed_wt.path.exists(), "Worktree path should exist");

    Ok(())
}

#[test]
#[serial]
fn test_delete_renamed_worktree_with_ui() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create and rename a worktree with unique names
    let original_name = generators::worktree_name(&format!("{}-ui-test", naming::DELETE_PREFIX));
    let branch_name = generators::branch_name(&format!("{}-ui-branch", naming::DELETE_PREFIX));
    let new_name = generators::worktree_name(&format!("{}-ui-renamed", naming::DELETE_PREFIX));

    let wt_to_delete =
        create_and_rename_worktree(&manager, &original_name, &branch_name, &new_name)?;

    // The delete command should use git_name internally
    manager.remove_worktree(&wt_to_delete.git_name)?;

    // Verify deletion
    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.git_name == original_name),
        "Worktree should be deleted by git_name"
    );
    assert!(
        !worktrees_after.iter().any(|w| w.name == new_name),
        "Worktree should not appear with display name either"
    );

    Ok(())
}

#[test]
#[serial]
fn test_rename_already_renamed_worktree() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with unique names
    let original_name = generators::worktree_name(&format!("{}-rename", naming::MULTI_PREFIX));
    let branch_name = generators::branch_name(&format!("{}-branch", naming::MULTI_PREFIX));
    manager.create_worktree_with_new_branch(&original_name, &branch_name, config::MAIN_BRANCH)?;

    // First rename with unique name
    let first_new_name = generators::worktree_name(&format!("first-{}", naming::RENAMED_PREFIX));
    manager.rename_worktree(&original_name, &first_new_name)?;

    // Verify first rename
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == first_new_name)
        .expect("Should find first renamed worktree");

    assert_eq!(
        wt.git_name, original_name,
        "Git name should still be original"
    );

    // Second rename with unique name - should use git_name not display name
    let second_new_name = generators::worktree_name(&format!("second-{}", naming::RENAMED_PREFIX));
    manager.rename_worktree(&wt.git_name, &second_new_name)?;

    // Verify second rename
    let worktrees_after = manager.list_worktrees()?;
    let final_wt = worktrees_after
        .iter()
        .find(|w| w.name == second_new_name)
        .expect("Should find second renamed worktree");

    assert_eq!(
        final_wt.git_name, original_name,
        "Git name should still be original"
    );
    assert_eq!(
        final_wt.name, second_new_name,
        "Display name should be latest"
    );
    assert!(
        final_wt.path.ends_with(&second_new_name),
        "Path should reflect latest name"
    );

    Ok(())
}

#[test]
#[serial]
fn test_operations_preserve_git_name() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create multiple worktrees with unique names
    let timestamp = generators::generate_timestamp();
    let worktrees_to_create = vec![
        (
            generators::worktree_name(&format!("{}-{}", naming::PRESERVE_PREFIX, counts::FIRST)),
            generators::branch_name(&format!("feat-{}-{}", counts::FIRST, timestamp)),
        ),
        (
            generators::worktree_name(&format!("{}-{}", naming::PRESERVE_PREFIX, counts::SECOND)),
            generators::branch_name(&format!("feat-{}-{}", counts::SECOND, timestamp)),
        ),
        (
            generators::worktree_name(&format!("{}-{}", naming::PRESERVE_PREFIX, counts::THIRD)),
            generators::branch_name(&format!("feat-{}-{}", counts::THIRD, timestamp)),
        ),
    ];

    for (name, branch) in &worktrees_to_create {
        manager.create_worktree_with_new_branch(name, branch, config::MAIN_BRANCH)?;
    }

    // Rename all of them with unique names
    let new_names: Vec<(String, String)> = worktrees_to_create
        .iter()
        .enumerate()
        .map(|(i, (original, _))| {
            (
                original.clone(),
                generators::worktree_name(&format!(
                    "new-{}-{}",
                    naming::PRESERVE_PREFIX,
                    i + counts::FIRST
                )),
            )
        })
        .collect();

    for (old, new) in &new_names {
        manager.rename_worktree(old, new)?;
    }

    // Verify all have correct git_name preserved
    let worktrees = manager.list_worktrees()?;

    for (original, new) in &new_names {
        let wt = worktrees
            .iter()
            .find(|w| w.name == *new)
            .unwrap_or_else(|| panic!("Should find worktree {new}"));

        assert_eq!(
            wt.git_name, *original,
            "Git name should be preserved as {original}"
        );
        assert_eq!(wt.name, *new, "Display name should be {new}");
    }

    // Now delete one using the correct git_name
    let to_delete = worktrees
        .iter()
        .find(|w| {
            w.name.contains(&format!(
                "new-{}-{}",
                naming::PRESERVE_PREFIX,
                counts::SECOND
            ))
        })
        .expect("Should find worktree to delete");

    manager.remove_worktree(&to_delete.git_name)?;

    // Verify correct one was deleted
    let worktrees_after = manager.list_worktrees()?;
    let deleted_git_name = &to_delete.git_name;

    assert!(
        !worktrees_after
            .iter()
            .any(|w| w.git_name == *deleted_git_name),
        "Deleted worktree should be removed"
    );

    // Verify others still exist
    let remaining_count = worktrees_after
        .iter()
        .filter(|w| {
            w.git_name
                .contains(&format!("{}-", naming::PRESERVE_PREFIX))
        })
        .count();
    assert_eq!(
        remaining_count,
        counts::REMAINING_PRESERVE_COUNT,
        "Should have {} preserve worktrees remaining",
        counts::REMAINING_PRESERVE_COUNT
    );

    Ok(())
}

#[test]
#[serial]
fn test_edge_case_same_name_as_git_name() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create a worktree with unique names
    let name = generators::worktree_name(naming::EDGE_CASE_PREFIX);
    let branch = generators::branch_name(naming::EDGE_CASE_PREFIX);
    // Create, rename to temp name, then rename back to original
    let temp_name = generators::worktree_name(naming::TEMP_PREFIX);
    create_and_rename_worktree(&manager, &name, &branch, &temp_name)?;
    manager.rename_worktree(&name, &name)?;

    // Verify state using helper
    verify_worktree_properties(&manager, &name, &name, &branch)?;

    // Get worktree for deletion
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == name)
        .expect("Should find worktree");

    assert_eq!(wt.name, name, "Display name should be original name");
    assert_eq!(wt.git_name, name, "Git name should still be original name");

    // Operations should still work correctly
    manager.remove_worktree(&wt.git_name)?;

    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.git_name == name),
        "Worktree should be deleted"
    );

    Ok(())
}
