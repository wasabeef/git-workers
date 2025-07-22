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
fn test_rename_worktree_debug_repair() -> Result<()> {
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

    // Create a worktree with unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let worktree_name = format!("debug-{timestamp}");
    let branch_name = format!("branch-{timestamp}");

    manager.create_worktree_with_new_branch(&worktree_name, &branch_name, config::MAIN_BRANCH)?;

    // Check git worktree list before rename
    println!("=== Before rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!(
        "git worktree list:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Check metadata directory
    let git_dir = repo_path.join(".git/worktrees");
    println!("Metadata directories:");
    for entry in fs::read_dir(&git_dir)? {
        let entry = entry?;
        println!("  - {}", entry.file_name().to_string_lossy());
    }

    // Rename
    println!("\n=== Performing rename ===");
    let new_name = format!("renamed-{timestamp}");
    manager.rename_worktree(&worktree_name, &new_name)?;

    // Check git worktree list after rename
    println!("\n=== After rename ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list"])
        .output()?;
    println!(
        "git worktree list:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Check metadata directory again
    println!("Metadata directories:");
    for entry in fs::read_dir(&git_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        println!("  - {}", name.to_string_lossy());

        // Check gitdir content
        let gitdir_path = entry.path().join("gitdir");
        if gitdir_path.exists() {
            let content = fs::read_to_string(&gitdir_path)?;
            println!("    gitdir: {}", content.trim());
        }
    }

    // Check with porcelain
    println!("\n=== Porcelain output ===");
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
