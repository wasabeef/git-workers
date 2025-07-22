use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
fn test_rename_worktree_preserves_branch_fixed() -> Result<()> {
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

    // Create a worktree with unique name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let worktree_name = format!("fix-test-{timestamp}");
    let branch_name = format!("fix-branch-{timestamp}");

    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Check initial state
    let worktrees_before = manager.list_worktrees()?;
    let wt_before = worktrees_before
        .iter()
        .find(|w| w.name == worktree_name)
        .expect("Worktree should exist");

    println!("Before rename:");
    println!("  Name: {}", wt_before.name);
    println!("  Branch: {}", wt_before.branch);
    println!("  Path: {:?}", wt_before.path);

    assert_eq!(wt_before.branch, branch_name, "Initial branch should match");

    // Rename the worktree
    let new_name = format!("renamed-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_name)?;

    // Check state after rename
    let worktrees_after = manager.list_worktrees()?;

    println!("\nAfter rename:");
    for wt in &worktrees_after {
        if wt.name == new_name || wt.path.ends_with(&new_name) {
            println!("  Name: {}", wt.name);
            println!("  Branch: {}", wt.branch);
            println!("  Path: {:?}", wt.path);
        }
    }

    // Find the worktree (it should now be tracked by new display name)
    let wt_after = worktrees_after
        .iter()
        .find(|w| w.name == new_name)
        .expect("Worktree should be tracked by new display name");

    // Key assertions
    assert_eq!(
        wt_after.branch, branch_name,
        "Branch should be preserved after rename"
    );
    assert_ne!(
        wt_after.branch, "unknown",
        "Branch should not become 'unknown'"
    );
    assert!(wt_after.path.ends_with(&new_name), "Path should be updated");
    assert!(wt_after.path.exists(), "New path should exist");

    // Verify we can actually access the worktree
    let wt_repo = git2::Repository::open(&wt_after.path)?;
    let head = wt_repo.head()?;
    assert_eq!(head.shorthand(), Some(&branch_name[..]));

    Ok(())
}
