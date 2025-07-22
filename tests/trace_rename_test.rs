use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
fn test_trace_rename_execution() -> Result<()> {
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
    drop(repo);

    let manager = GitWorktreeManager::new_from_path(repo_path)?;

    // Create worktree
    let worktree_name = "trace-test";
    let branch_name = "trace-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Check initial gitdir content
    let gitdir_path = repo_path
        .join(".git/worktrees")
        .join(worktree_name)
        .join("gitdir");
    println!("Before rename:");
    println!("  gitdir path: {gitdir_path:?}");
    let gitdir_content = fs::read_to_string(&gitdir_path)?;
    println!("  gitdir content: {}", gitdir_content.trim());

    // Rename
    println!("\nPerforming rename...");
    let new_name = "trace-renamed";
    let new_path = manager.rename_worktree(worktree_name, new_name)?;
    println!("  Returned new path: {new_path:?}");

    // Check gitdir content after rename
    println!("\nAfter rename:");
    let gitdir_content_after = fs::read_to_string(&gitdir_path)?;
    println!("  gitdir content: {}", gitdir_content_after.trim());

    // Expected vs actual
    let expected_content = format!("{}/.git", new_path.display());
    println!("\nComparison:");
    println!("  Expected: {expected_content}");
    println!("  Actual:   {}", gitdir_content_after.trim());

    if gitdir_content_after.trim() == expected_content {
        println!("  ✓ gitdir was updated correctly");
    } else {
        println!("  ✗ gitdir was NOT updated");
    }

    // Check .git file in worktree
    let worktree_git_file = new_path.join(".git");
    if worktree_git_file.exists() {
        let git_file_content = fs::read_to_string(&worktree_git_file)?;
        println!("\nWorktree .git file:");
        println!("  Content: {}", git_file_content.trim());
    } else {
        println!("\nWorktree .git file does not exist at {worktree_git_file:?}");
    }

    Ok(())
}
