use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
#[ignore = "Debug test for manual filesystem operations - not suitable for CI"]
fn test_debug_filesystem_operations() -> Result<()> {
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
    let worktree_name = "fs-debug-test";
    let branch_name = "fs-debug-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    // Manually test the exact same operations that rename_worktree should do
    println!("=== Manual filesystem operations test ===");

    let old_path = repo_path.join(worktree_name);
    let new_name = "fs-debug-renamed";
    let new_path = repo_path.join(new_name);

    // Step 1: Move directory (this is done by rename_worktree)
    println!("\n1. Moving directory...");
    fs::rename(&old_path, &new_path)?;
    println!("   ✓ Directory moved");

    // Step 2: Check if worktree git dir exists
    let worktree_git_dir = repo_path.join(".git/worktrees").join(worktree_name);
    println!("\n2. Checking worktree git dir:");
    println!("   Path: {worktree_git_dir:?}");
    println!("   Exists: {}", worktree_git_dir.exists());

    // Step 3: Check gitdir file
    let gitdir_file = worktree_git_dir.join("gitdir");
    println!("\n3. Checking gitdir file:");
    println!("   Path: {gitdir_file:?}");
    println!("   Exists: {}", gitdir_file.exists());

    if gitdir_file.exists() {
        let content = fs::read_to_string(&gitdir_file)?;
        println!("   Current content: {}", content.trim());

        // Try to write new content
        println!("\n4. Writing new content to gitdir file...");
        let new_content = format!("{}/.git\n", new_path.display());
        println!("   New content: {}", new_content.trim());

        fs::write(&gitdir_file, &new_content)?;
        println!("   ✓ Write successful");

        // Verify write
        let verify_content = fs::read_to_string(&gitdir_file)?;
        println!("   Verified content: {}", verify_content.trim());

        if verify_content.trim() == new_content.trim() {
            println!("   ✓ Content updated correctly");
        } else {
            println!("   ✗ Content NOT updated correctly");
        }
    }

    // Step 5: Update .git file in worktree
    println!("\n5. Updating .git file in worktree...");
    let worktree_git_file = new_path.join(".git");
    let git_file_content = format!(
        "gitdir: {}/.git/worktrees/{worktree_name}\n",
        repo_path.display()
    );
    fs::write(&worktree_git_file, &git_file_content)?;
    println!("   ✓ .git file updated");

    // Now test with actual rename_worktree
    println!("\n=== Testing actual rename_worktree ===");

    // Create another worktree
    let test2_name = "fs-test2";
    let test2_branch = "fs-branch2";
    manager.create_worktree_with_new_branch(test2_name, test2_branch, config::MAIN_BRANCH)?;

    // Check before
    let gitdir2_path = repo_path
        .join(".git/worktrees")
        .join(test2_name)
        .join("gitdir");
    let before_content = fs::read_to_string(&gitdir2_path)?;
    println!("\nBefore rename_worktree:");
    println!("  gitdir content: {}", before_content.trim());

    // Rename
    let new_test2_name = "fs-test2-renamed";
    manager.rename_worktree(test2_name, new_test2_name)?;

    // Check after
    let after_content = fs::read_to_string(&gitdir2_path)?;
    println!("\nAfter rename_worktree:");
    println!("  gitdir content: {}", after_content.trim());

    if before_content == after_content {
        println!("  ✗ Content was NOT changed by rename_worktree");
    } else {
        println!("  ✓ Content was changed by rename_worktree");
    }

    Ok(())
}
