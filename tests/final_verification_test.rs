use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
fn test_rename_worktree_final_verification() -> Result<()> {
    // Use a unique temporary directory for each test run
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_base = format!("/tmp/gw-test-{timestamp}");
    fs::create_dir_all(&test_base)?;

    let repo_path = PathBuf::from(&test_base).join("repo");
    fs::create_dir_all(&repo_path)?;

    // Initialize repository
    git2::Repository::init(&repo_path)?;

    // Create initial commit
    let repo = git2::Repository::open(&repo_path)?;
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

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree with subdirectory pattern to keep it organized
    let worktree_name = format!("{test_base}/worktrees/test-wt");
    let branch_name = "test-branch";

    println!("=== Creating worktree ===");
    println!("Worktree path: {worktree_name}");

    let worktree_path = manager.create_worktree_with_new_branch(
        &worktree_name,
        branch_name,
        config::MAIN_BRANCH,
    )?;

    println!("Created at: {worktree_path:?}");

    // Get the actual worktree name (last component)
    let actual_worktree_name = worktree_path.file_name().unwrap().to_str().unwrap();

    println!("\n=== Before rename ===");

    // Check gitdir content
    let gitdir_path = repo_path
        .join(".git/worktrees")
        .join(actual_worktree_name)
        .join("gitdir");
    let gitdir_content_before = fs::read_to_string(&gitdir_path)?;
    println!("gitdir content: {}", gitdir_content_before.trim());

    // List worktrees
    let worktrees_before = manager.list_worktrees()?;
    for wt in &worktrees_before {
        if wt.name == actual_worktree_name {
            println!("Worktree '{}': branch = {}", wt.name, wt.branch);
        }
    }

    // Rename
    println!("\n=== Performing rename ===");
    let new_name = "renamed-wt";
    let new_path = manager.rename_worktree(actual_worktree_name, new_name)?;
    println!("New path: {new_path:?}");

    // Check gitdir content after rename
    println!("\n=== After rename ===");
    let gitdir_content_after = fs::read_to_string(&gitdir_path)?;
    println!("gitdir content: {}", gitdir_content_after.trim());

    if gitdir_content_before != gitdir_content_after {
        println!("✓ gitdir was updated");
        let expected = format!("{}/.git", new_path.display());
        if gitdir_content_after.trim() == expected {
            println!("✓ gitdir points to correct new path");
        } else {
            println!("✗ gitdir doesn't point to expected path");
            println!("  Expected: {expected}");
        }
    } else {
        println!("✗ gitdir was NOT updated - this is the bug");
    }

    // List worktrees after rename
    let worktrees_after = manager.list_worktrees()?;
    for wt in &worktrees_after {
        if wt.name == actual_worktree_name {
            println!("\nWorktree '{}' after rename:", wt.name);
            println!("  Branch: {}", wt.branch);
            println!("  Path: {:?}", wt.path);

            if wt.branch == "unknown" {
                println!("  ✗ Branch is 'unknown' - bug confirmed");
            } else if wt.branch == branch_name {
                println!("  ✓ Branch is preserved - fix is working!");
            }
        }
    }

    // Cleanup
    fs::remove_dir_all(&test_base)?;

    Ok(())
}
