use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::{config, generators, naming};

#[test]
#[serial]
fn test_rename_worktree_basic() -> Result<()> {
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

    // Create a worktree (will be created relative to repo)
    println!("Creating worktree...");
    let worktree_name = generators::worktree_name(&format!("{}-wt", naming::SIMPLE_PREFIX));
    let branch_name = generators::branch_name(naming::FEATURE_BRANCH_PREFIX);
    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Check initial state
    println!("Checking initial state...");
    let worktrees_before = manager.list_worktrees()?;
    for wt in &worktrees_before {
        println!(
            "  Before - Name: {}, Branch: {}, Path: {path:?}",
            wt.name,
            wt.branch,
            path = wt.path
        );
    }

    // Rename the worktree
    println!("Renaming worktree...");
    let new_name = generators::worktree_name(&format!("{}-test-wt", naming::RENAMED_PREFIX));
    manager.rename_worktree(&worktree_name, &new_name)?;

    // Check state after rename
    println!("Checking state after rename...");
    let worktrees_after = manager.list_worktrees()?;
    for wt in &worktrees_after {
        println!(
            "  After - Name: {}, Branch: {}, Path: {path:?}",
            wt.name,
            wt.branch,
            path = wt.path
        );
    }

    // Find the renamed worktree
    let renamed_wt = worktrees_after.iter().find(|w| w.path.ends_with(&new_name));

    if let Some(wt) = renamed_wt {
        println!("Found renamed worktree:");
        println!("  - Internal name: {}", wt.name);
        println!("  - Branch: {}", wt.branch);
        println!("  - Path: {:?}", wt.path);

        // The key assertion: branch should not be "unknown"
        assert_ne!(
            wt.branch, "unknown",
            "Branch should not become 'unknown' after rename"
        );
        assert_eq!(wt.branch, branch_name, "Branch name should be preserved");
    } else {
        panic!("Could not find renamed worktree");
    }

    Ok(())
}
