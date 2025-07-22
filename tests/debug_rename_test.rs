use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
#[ignore = "Debug test for manual rename operations - not suitable for CI"]
fn test_rename_worktree_debug() -> Result<()> {
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

    // Create a worktree
    println!("Creating worktree...");
    let worktree_name = "debug-test-wt";
    let branch_name = "debug-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Check git worktree list
    println!("\n=== Git worktree list BEFORE rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Check git2 behavior
    println!("\n=== git2 library BEFORE rename ===");
    let repo = git2::Repository::open(repo_path)?;
    let worktrees = repo.worktrees()?;
    for name in worktrees.iter().flatten() {
        if let Ok(wt) = repo.find_worktree(name) {
            println!("git2 worktree '{}': path = {:?}", name, wt.path());
        }
    }

    // Rename the worktree
    println!("\n=== Performing rename ===");
    let new_name = "renamed-debug-wt";
    manager.rename_worktree(worktree_name, new_name)?;

    // Check git worktree list after rename
    println!("\n=== Git worktree list AFTER rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Need to re-open repository to refresh git2's cache
    println!("\n=== git2 library AFTER rename (without re-opening) ===");
    let worktrees = repo.worktrees()?;
    for name in worktrees.iter().flatten() {
        if let Ok(wt) = repo.find_worktree(name) {
            println!("git2 worktree '{}': path = {:?}", name, wt.path());
        }
    }

    // Re-open repository to see if it refreshes
    println!("\n=== git2 library AFTER re-opening repository ===");
    drop(repo);
    let fresh_repo = git2::Repository::open(repo_path)?;
    let fresh_worktrees = fresh_repo.worktrees()?;
    for name in fresh_worktrees.iter().flatten() {
        if let Ok(wt) = fresh_repo.find_worktree(name) {
            println!("git2 worktree '{}': path = {:?}", name, wt.path());
        }
    }

    // Check with fresh manager
    println!("\n=== Creating fresh GitWorktreeManager ===");
    let fresh_manager = GitWorktreeManager::new_from_path(repo_path)?;
    let worktrees_fresh = fresh_manager.list_worktrees()?;
    for wt in &worktrees_fresh {
        println!(
            "Fresh manager - Name: {}, Branch: {}, Path: {:?}",
            wt.name, wt.branch, wt.path
        );
    }

    Ok(())
}
