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
fn test_verify_rename_fix() -> Result<()> {
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

    // Drop repo to ensure clean state
    drop(repo);

    let manager = GitWorktreeManager::new_from_path(repo_path)?;

    // Create a worktree
    let worktree_name = "verify-test";
    let branch_name = "verify-branch";
    manager.create_worktree_with_new_branch(worktree_name, branch_name, config::MAIN_BRANCH)?;

    println!("=== Verification of rename fix ===");

    // Check git worktree list before
    println!("\n1. Git worktree list BEFORE rename:");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Rename
    let new_name = "verify-renamed";
    println!("\n2. Performing rename...");
    let new_path = manager.rename_worktree(worktree_name, new_name)?;
    println!("   New path: {new_path:?}");

    // Check git worktree list after
    println!("\n3. Git worktree list AFTER rename:");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    let git_output = String::from_utf8_lossy(&output.stdout);
    println!("{git_output}");

    // Verify git shows the new path
    if git_output.contains(new_name) {
        println!("   ✓ Git shows the renamed path");
    } else {
        println!("   ✗ Git still shows old path");
    }

    // Check with porcelain for details
    println!("\n4. Git worktree list --porcelain:");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Check metadata directory
    println!("\n5. Checking metadata directory:");
    let git_worktrees_dir = repo_path.join(".git/worktrees");
    for entry in fs::read_dir(&git_worktrees_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        println!("   Directory: {}", name.to_string_lossy());

        // Check gitdir content
        let gitdir_path = entry.path().join("gitdir");
        if gitdir_path.exists() {
            let content = fs::read_to_string(&gitdir_path)?;
            println!("   gitdir points to: {}", content.trim());
        }
    }

    // Test with GitWorktreeManager
    println!("\n6. Testing with GitWorktreeManager::list_worktrees():");
    let worktrees = manager.list_worktrees()?;
    for wt in &worktrees {
        if wt.name == worktree_name {
            println!("   Name: {}", wt.name);
            println!("   Branch: {}", wt.branch);
            println!("   Path: {:?}", wt.path);

            if wt.branch == "unknown" {
                println!("   ✗ Branch is 'unknown' - fix not working");
            } else if wt.branch == branch_name {
                println!("   ✓ Branch is correct - fix is working!");
            }

            if wt.path.ends_with(new_name) {
                println!("   ✓ Path is updated");
            } else {
                println!("   ✗ Path is not updated");
            }
        }
    }

    // Final verification: Can we access the worktree?
    println!("\n7. Final verification - accessing worktree:");
    if new_path.exists() {
        println!("   ✓ New path exists");

        let branch_output = Command::new("git")
            .current_dir(&new_path)
            .args(["branch", "--show-current"])
            .output()?;
        let current_branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
        println!("   Current branch: {current_branch}");

        if current_branch == branch_name {
            println!("   ✓ Branch is accessible and correct");
        }
    } else {
        println!("   ✗ New path does not exist");
    }

    Ok(())
}
