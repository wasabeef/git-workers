use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

mod test_constants;
use test_constants::config;

#[test]
#[serial]
fn test_manual_rename_approach() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Initialize repository
    Command::new("git")
        .current_dir(repo_path)
        .args(["init"])
        .output()?;

    Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.email", config::TEST_USER_EMAIL])
        .output()?;

    Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.name", config::TEST_USER_NAME])
        .output()?;

    // Create initial commit
    fs::write(
        repo_path.join(config::README_FILENAME),
        config::DEFAULT_README_CONTENT,
    )?;
    Command::new("git")
        .current_dir(repo_path)
        .args(["add", "."])
        .output()?;
    Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", config::INITIAL_COMMIT_MESSAGE])
        .output()?;

    // Create worktree
    let worktree_name = "manual-test";
    let branch_name = "manual-branch";
    Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "add", worktree_name, "-b", branch_name])
        .output()?;

    println!("=== Before manual rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Manual rename process
    let old_path = repo_path.join(worktree_name);
    let new_name = "manual-renamed";
    let new_path = repo_path.join(new_name);

    println!("\n=== Performing manual rename ===");

    // Step 1: Move directory
    println!("1. Moving directory from {old_path:?} to {new_path:?}");
    fs::rename(&old_path, &new_path)?;

    // Step 2: Update gitdir file (DO NOT rename metadata directory)
    let gitdir_file = repo_path
        .join(".git/worktrees")
        .join(worktree_name)
        .join("gitdir");
    println!("2. Updating gitdir file at {gitdir_file:?}");
    let new_gitdir_content = format!("{}/.git\n", new_path.display());
    println!("   New content: {}", new_gitdir_content.trim());
    fs::write(&gitdir_file, new_gitdir_content)?;

    // Step 3: Update .git file in worktree
    let worktree_git_file = new_path.join(".git");
    println!("3. Updating .git file at {worktree_git_file:?}");
    let new_git_content = format!(
        "gitdir: {}/.git/worktrees/{worktree_name}\n",
        repo_path.display()
    );
    println!("   New content: {}", new_git_content.trim());
    fs::write(&worktree_git_file, new_git_content)?;

    // Step 4: Run repair
    println!("4. Running git worktree repair");
    Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "repair"])
        .output()?;

    println!("\n=== After manual rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Test branch access
    println!("\n=== Testing branch access ===");
    let branch_output = Command::new("git")
        .current_dir(&new_path)
        .args(["branch", "--show-current"])
        .output()?;
    println!(
        "Branch: {}",
        String::from_utf8_lossy(&branch_output.stdout).trim()
    );

    // Test with git2
    println!("\n=== Testing with git2 ===");
    let repo = git2::Repository::open(repo_path)?;
    let worktrees = repo.worktrees()?;
    for name in worktrees.iter().flatten() {
        if let Ok(wt) = repo.find_worktree(name) {
            let path = wt.path();
            println!("Worktree '{name}': path = {path:?}");

            if let Ok(wt_repo) = git2::Repository::open(path) {
                if let Ok(head) = wt_repo.head() {
                    println!("  Branch: {:?}", head.shorthand());
                } else {
                    println!("  Branch: unknown (no HEAD)");
                }
            } else {
                println!("  Branch: unknown (cannot open)");
            }
        }
    }

    Ok(())
}
