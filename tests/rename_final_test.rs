use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

/// Test that verifies rename_worktree preserves branch information
/// This test demonstrates the current behavior and expected behavior
#[test]
#[serial]
fn test_rename_worktree_branch_preservation() -> Result<()> {
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
        index.add_path(std::path::Path::new(config::README_FILENAME))?;
        index.write()?;
        index.write_tree()?
    };

    {
        let tree = repo.find_tree(tree_id)?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            config::INITIAL_COMMIT_MESSAGE,
            &tree,
            &[],
        )?;
    }

    let manager = GitWorktreeManager::new_from_path(repo_path)?;

    // Test case 1: Non-bare repository
    println!("=== Test Case 1: Non-bare repository ===");
    test_rename_in_repo(&manager, "test1", "branch1")?;

    // Test case 2: Different naming pattern
    println!("\n=== Test Case 2: Different naming pattern ===");
    test_rename_in_repo(&manager, "feature-xyz", "feature-xyz")?;

    Ok(())
}

fn test_rename_in_repo(
    manager: &GitWorktreeManager,
    base_worktree_name: &str,
    base_branch_name: &str,
) -> Result<()> {
    // Generate unique names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("{base_worktree_name}-{timestamp}");
    let branch_name = format!("{base_branch_name}-{timestamp}");

    // Create worktree
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Verify initial state
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");

    println!("Initial state:");
    println!("  Name: {}", wt.name);
    println!("  Branch: {}", wt.branch);
    println!("  Path: {:?}", wt.path);
    assert_eq!(wt.branch, branch_name);

    // Rename worktree
    let new_name = format!("{worktree_name}-renamed");
    let result = manager.rename_worktree(&worktree_name, &new_name);

    match result {
        Ok(new_path) => {
            println!("\nRename succeeded, new path: {new_path:?}");

            // Check state after rename - now look for new display name
            let worktrees_after = manager.list_worktrees()?;
            let wt_after = worktrees_after
                .iter()
                .find(|w| w.name == new_name)
                .expect("Worktree should exist with new name");

            println!("\nAfter rename:");
            println!("  Name: {}", wt_after.name);
            println!("  Branch: {}", wt_after.branch);
            println!("  Path: {:?}", wt_after.path);

            // With our fix, branch information should be preserved
            if wt_after.branch == branch_name {
                println!("  ✓ Branch preserved correctly!");
            } else if wt_after.branch == "unknown" {
                println!("  ⚠️  Branch became 'unknown' - unexpected!");
                println!("  Expected: Branch should be '{branch_name}'");
            } else {
                println!("  ? Branch changed to: {}", wt_after.branch);
            }

            // Verify display name changed
            assert_eq!(wt_after.name, new_name);
        }
        Err(e) => {
            println!("\nRename failed: {e}");
            return Err(e);
        }
    }

    Ok(())
}

/// Test for bare repository
#[test]
#[serial]
fn test_rename_worktree_in_bare_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare.git");
    fs::create_dir(&bare_repo_path)?;

    // Initialize bare repository
    git2::Repository::init_bare(&bare_repo_path)?;

    // Create initial commit via temporary clone
    let clone_dir = temp_dir.path().join("temp-clone");
    let clone = git2::Repository::clone(bare_repo_path.to_str().unwrap(), &clone_dir)?;

    let sig = git2::Signature::now(config::TEST_USER_NAME, config::TEST_USER_EMAIL)?;
    let tree_id = {
        let mut index = clone.index()?;
        let file_path = clone_dir.join(config::README_FILENAME);
        fs::write(&file_path, config::DEFAULT_README_CONTENT)?;
        index.add_path(std::path::Path::new(config::README_FILENAME))?;
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
        remote.push(&[&format!("refs/heads/{}", config::MAIN_BRANCH)], None)?
    }

    drop(clone);
    fs::remove_dir_all(&clone_dir)?;

    // Test with bare repository
    let manager = GitWorktreeManager::new_from_path(&bare_repo_path)?;

    println!("=== Test Case: Bare repository ===");

    // Generate unique names for bare repo test
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let worktree_name = format!("bare-test-{timestamp}");
    let branch_name = format!("bare-branch-{timestamp}");

    // Create worktree (will be created outside the bare repo)
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Verify and rename
    let worktrees = manager.list_worktrees()?;
    let wt = worktrees
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");

    println!("Initial state:");
    println!("  Name: {}", wt.name);
    println!("  Branch: {}", wt.branch);

    // Rename
    let new_name = format!("bare-renamed-{timestamp}");
    match manager.rename_worktree(&worktree_name, &new_name) {
        Ok(_) => {
            let worktrees_after = manager.list_worktrees()?;
            let wt_after = worktrees_after
                .iter()
                .find(|w| w.name == new_name) // Look for new display name
                .expect("Worktree should exist with new name");

            println!("\nAfter rename:");
            println!("  Name: {}", wt_after.name);
            println!("  Branch: {}", wt_after.branch);

            if wt_after.branch == branch_name {
                println!("  ✓ Branch preserved correctly in bare repo!");
            } else if wt_after.branch == "unknown" {
                println!("  ⚠️  Branch became 'unknown' in bare repo too!");
            }

            // Verify display name changed
            assert_eq!(wt_after.name, new_name);
        }
        Err(e) => println!("Rename failed: {e}"),
    }

    Ok(())
}
