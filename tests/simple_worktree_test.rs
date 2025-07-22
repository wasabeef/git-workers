use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
fn test_worktree_path_detection() -> Result<()> {
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

    // Create worktree - check where it's actually created
    println!("Repository path: {repo_path:?}");

    let worktree_name = "path-test";
    let branch_name = "path-branch";
    let result =
        manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH);

    match result {
        Ok(path) => {
            println!("Worktree created at: {path:?}");

            // Check if it's in temp dir or elsewhere
            if path.starts_with(temp_dir.path()) {
                println!("✓ Worktree is in temp directory");
            } else {
                println!("✗ Worktree is NOT in temp directory");
                println!("  It's at: {path:?}");
            }
        }
        Err(e) => {
            println!("Failed to create worktree: {e}");
        }
    }

    Ok(())
}
