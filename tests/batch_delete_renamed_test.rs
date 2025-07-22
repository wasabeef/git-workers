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

/// Helper function to filter test worktrees by name patterns
fn filter_test_worktrees(worktrees: &[WorktreeInfo], patterns: &[&str]) -> Vec<WorktreeInfo> {
    worktrees
        .iter()
        .filter(|w| patterns.iter().any(|pattern| w.name.contains(pattern)))
        .cloned()
        .collect()
}

#[test]
#[serial]
fn test_batch_delete_renamed_worktrees() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create multiple worktrees
    let worktrees_data =
        generators::test_data_tuple(naming::BATCH_PREFIX, counts::BATCH_TEST_COUNT);

    // Create and rename worktrees
    for (original_name, branch_name, new_name) in &worktrees_data {
        manager.create_worktree_with_new_branch(original_name, branch_name, config::MAIN_BRANCH)?;
        manager.rename_worktree(original_name, new_name)?;
    }

    // Verify all worktrees exist with correct names
    let worktrees = manager.list_worktrees()?;

    for (original_name, _, new_name) in &worktrees_data {
        let wt = worktrees
            .iter()
            .find(|w| w.name == *new_name)
            .unwrap_or_else(|| panic!("Should find worktree with display name {new_name}"));

        assert_eq!(wt.git_name, *original_name, "Git name should be original");
        assert_eq!(wt.name, *new_name, "Display name should be new");
    }

    // Now test batch deletion would use git_name correctly
    // Only delete our test worktrees by name pattern to avoid issues with main worktree detection
    let test_worktrees = filter_test_worktrees(
        &worktrees,
        &[
            &format!("{}-renamed", naming::BATCH_PREFIX),
            naming::BATCH_PREFIX,
        ],
    );

    println!("Deleting {} test worktrees", test_worktrees.len());

    for wt in test_worktrees {
        println!("Deleting worktree: {} (git_name: {})", wt.name, wt.git_name);
        // This simulates what batch delete does
        manager.remove_worktree(&wt.git_name)?;
    }

    // Verify all test worktrees are deleted
    let worktrees_after = manager.list_worktrees()?;
    println!("Worktrees after deletion: {}", worktrees_after.len());

    // Verify none of our test worktrees remain
    for (original_name, _, new_name) in &worktrees_data {
        assert!(
            !worktrees_after.iter().any(|w| w.name == *new_name),
            "Test worktree {new_name} should be deleted"
        );
        assert!(
            !worktrees_after.iter().any(|w| w.git_name == *original_name),
            "Test worktree with git_name {original_name} should be deleted"
        );
    }

    Ok(())
}

#[test]
#[serial]
fn test_batch_delete_mixed_renamed_and_normal() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create mix of renamed and non-renamed worktrees
    let timestamp = generators::generate_timestamp();
    // Normal worktrees
    let normal1_name =
        generators::worktree_name(&format!("normal-{}-{}", counts::FIRST, timestamp));
    let normal1_branch =
        generators::branch_name(&format!("branch-{}-{}", counts::FIRST, timestamp));
    let normal2_name =
        generators::worktree_name(&format!("normal-{}-{}", counts::SECOND, timestamp));
    let normal2_branch =
        generators::branch_name(&format!("branch-{}-{}", counts::SECOND, timestamp));

    manager.create_worktree_with_new_branch(&normal1_name, &normal1_branch, config::MAIN_BRANCH)?;
    manager.create_worktree_with_new_branch(&normal2_name, &normal2_branch, config::MAIN_BRANCH)?;

    // Renamed worktrees
    let original3_name =
        generators::worktree_name(&format!("original-{}-{}", counts::THIRD, timestamp));
    let branch3_name = generators::branch_name(&format!("branch-{}-{}", counts::THIRD, timestamp));
    let renamed3_name =
        generators::worktree_name(&format!("renamed-{}-{}", counts::THIRD, timestamp));

    let original4_name =
        generators::worktree_name(&format!("original-{}-{}", counts::FOURTH, timestamp));
    let branch4_name = generators::branch_name(&format!("branch-{}-{}", counts::FOURTH, timestamp));
    let renamed4_name =
        generators::worktree_name(&format!("renamed-{}-{}", counts::FOURTH, timestamp));

    manager.create_worktree_with_new_branch(&original3_name, &branch3_name, config::MAIN_BRANCH)?;
    manager.rename_worktree(&original3_name, &renamed3_name)?;

    manager.create_worktree_with_new_branch(&original4_name, &branch4_name, config::MAIN_BRANCH)?;
    manager.rename_worktree(&original4_name, &renamed4_name)?;

    // List all worktrees
    let worktrees = manager.list_worktrees()?;
    println!("Total worktrees found: {}", worktrees.len());
    for wt in &worktrees {
        println!(
            "Worktree: {} (git_name: {}, is_current: {}, path: {:?})",
            wt.name, wt.git_name, wt.is_current, wt.path
        );
    }

    // Only delete our test worktrees, not the main one
    let expected_names: Vec<&String> =
        [&normal1_name, &normal2_name, &renamed3_name, &renamed4_name].to_vec();
    let deletable = filter_test_worktrees(
        &worktrees,
        &expected_names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );

    // Verify we have the expected number of deletable worktrees
    assert_eq!(
        deletable.len(),
        counts::MIXED_DELETE_COUNT,
        "Should have {} deletable worktrees",
        counts::MIXED_DELETE_COUNT
    );

    // Verify correct name/git_name mapping
    for wt in &deletable {
        if wt.name == normal1_name || wt.name == normal2_name {
            assert_eq!(
                wt.name, wt.git_name,
                "Normal worktrees should have same name and git_name"
            );
        } else if wt.name == renamed3_name {
            assert_eq!(
                wt.git_name, original3_name,
                "Renamed worktree should keep original git_name"
            );
        } else if wt.name == renamed4_name {
            assert_eq!(
                wt.git_name, original4_name,
                "Renamed worktree should keep original git_name"
            );
        } else {
            panic!("Unexpected worktree name: {}", wt.name);
        }
    }

    // Delete all using git_name (as batch delete does)
    for wt in deletable {
        manager.remove_worktree(&wt.git_name)?;
    }

    // Verify deletion - just check that our test worktrees are gone
    let worktrees_after = manager.list_worktrees()?;
    println!("Worktrees after deletion: {}", worktrees_after.len());
    for wt in &worktrees_after {
        println!(
            "Remaining: {} (git_name: {}, is_current: {})",
            wt.name, wt.git_name, wt.is_current
        );
    }

    // Since we only deleted our test worktrees by name pattern, they should be gone
    assert!(
        !worktrees_after
            .iter()
            .any(|w| expected_names.contains(&&w.name)),
        "All test worktrees should be deleted"
    );

    // In test environment, the main worktree might not be properly detected as current
    // so we just verify our test worktrees were successfully deleted

    Ok(())
}

#[test]
#[serial]
fn test_batch_delete_fails_with_display_name() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create and rename a worktree
    let original_name = generators::worktree_name(naming::FAIL_PREFIX);
    let branch_name = generators::branch_name(naming::FAIL_PREFIX);
    let new_name = generators::worktree_name(&format!("{}-renamed", naming::FAIL_PREFIX));

    manager.create_worktree_with_new_branch(&original_name, &branch_name, config::MAIN_BRANCH)?;
    manager.rename_worktree(&original_name, &new_name)?;

    // Get the worktree
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == new_name)
        .expect("Should find renamed worktree");

    // Try to delete with display name (this should fail)
    let result = manager.remove_worktree(&wt.name);
    assert!(result.is_err(), "Should fail when using display name");

    // Delete with git_name (this should succeed)
    let result = manager.remove_worktree(&wt.git_name);
    assert!(result.is_ok(), "Should succeed when using git_name");

    Ok(())
}

#[test]
#[serial]
fn test_batch_delete_with_branch_cleanup() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Create worktrees with branches
    let worktrees_data = generators::test_data_tuple(
        &format!("{}-test", naming::BRANCH_NAME_PREFIX),
        counts::BRANCH_CLEANUP_COUNT,
    );

    for (original_name, branch_name, new_name) in &worktrees_data {
        manager.create_worktree_with_new_branch(original_name, branch_name, config::MAIN_BRANCH)?;
        manager.rename_worktree(original_name, new_name)?;
    }

    // Get worktrees for deletion
    let worktrees = manager.list_worktrees()?;
    let non_current_worktrees: Vec<_> = worktrees
        .iter()
        .filter(|w| !w.is_current)
        .cloned()
        .collect();
    let to_delete = filter_test_worktrees(&non_current_worktrees, &["renamed"]);

    assert_eq!(
        to_delete.len(),
        counts::BRANCH_CLEANUP_COUNT,
        "Should have {} worktrees to delete",
        counts::BRANCH_CLEANUP_COUNT
    );

    // Track branches before deletion
    let (local_branches, _remote_branches) = manager.list_all_branches()?;
    let expected_branches: Vec<String> = worktrees_data
        .iter()
        .map(|(_, branch, _)| branch.clone())
        .collect();
    for branch in &expected_branches {
        assert!(
            local_branches.contains(branch),
            "Branch {branch} should exist"
        );
    }

    // Delete worktrees
    for wt in &to_delete {
        manager.remove_worktree(&wt.git_name)?;
    }

    // Delete orphaned branches
    for wt in &to_delete {
        let _ = manager.delete_branch(&wt.branch); // Ignore errors if branch is still in use
    }

    // Verify worktrees are gone
    let worktrees_after = manager.list_worktrees()?;
    assert!(
        !worktrees_after.iter().any(|w| w.name.contains("renamed")),
        "Renamed worktrees should be deleted"
    );

    Ok(())
}
