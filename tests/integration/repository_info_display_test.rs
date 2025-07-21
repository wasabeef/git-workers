//! Integration tests for repository info display patterns
//!
//! These tests ensure that repository names are displayed correctly
//! in all scenarios: bare repositories, worktrees, and regular repositories.

use anyhow::Result;
use git2::Repository;
use git_workers::repository_info::get_repository_info_at_path;
use std::{env, fs};
use tempfile::TempDir;

/// Helper to create initial commit for repository
fn create_initial_commit(repo: &Repository) -> Result<()> {
    let signature = git2::Signature::now("Test User", "test@example.com")?;

    // Create a file for non-bare repos
    if let Some(workdir) = repo.workdir() {
        fs::write(workdir.join("README.md"), "# Test Repository")?;

        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("README.md"))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
    } else {
        // For bare repositories, create an empty tree
        let tree_id = repo.treebuilder(None)?.write()?;
        let tree = repo.find_tree(tree_id)?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
    }

    Ok(())
}

/// Helper to initialize a completely isolated repository
fn init_isolated_repo(path: &std::path::Path, bare: bool) -> Result<Repository> {
    use std::process::Command;

    // Use git command to ensure complete isolation from parent repository
    let mut cmd = Command::new("git");
    cmd.arg("init");
    if bare {
        cmd.arg("--bare");
    }
    cmd.arg(path);
    // Clear only git-related environment variables, not all
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_CEILING_DIRECTORIES");

    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to initialize repository: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Open with git2
    Repository::open(path).map_err(Into::into)
}

/// Execute test in isolated environment
fn run_isolated_test<F>(test_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    // Create a temporary directory that's completely isolated
    let isolated_temp = TempDir::new()?;
    let isolated_path = isolated_temp.path().join("isolated_test");
    fs::create_dir(&isolated_path)?;

    // Save original directory and environment
    let original_dir = env::current_dir()?;
    let original_git_dir = env::var("GIT_DIR").ok();
    let original_git_work_tree = env::var("GIT_WORK_TREE").ok();

    // Clear git environment variables to ensure isolation
    env::remove_var("GIT_DIR");
    env::remove_var("GIT_WORK_TREE");
    env::remove_var("GIT_COMMON_DIR");

    // Change to isolated directory first
    env::set_current_dir(&isolated_path)?;

    // Run the test
    let result = test_fn();

    // Always restore original directory and environment
    env::set_current_dir(original_dir)?;
    if let Some(git_dir) = original_git_dir {
        env::set_var("GIT_DIR", git_dir);
    }
    if let Some(git_work_tree) = original_git_work_tree {
        env::set_var("GIT_WORK_TREE", git_work_tree);
    }

    result
}

/// Test repository info display for regular repositories
#[test]
fn test_regular_repository_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = init_isolated_repo(&repo_path, false)?;
        create_initial_commit(&repo)?;

        env::set_current_dir(&repo_path)?;

        let info = get_repository_info_at_path(&repo_path);
        assert_eq!(
            info, "test-repo",
            "Regular repository should show just repo name"
        );

        Ok(())
    })
}

/// Test repository info display for bare repository root
#[test]
fn test_bare_repository_root_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;
        let bare_repo_path = temp_dir.path().join("test-repo.bare");

        let repo = init_isolated_repo(&bare_repo_path, true)?;
        create_initial_commit(&repo)?;

        env::set_current_dir(&bare_repo_path)?;

        let info = get_repository_info_at_path(&bare_repo_path);
        assert_eq!(
            info, "test-repo",
            "Bare repository root should show just repo name without .bare suffix"
        );

        Ok(())
    })
}

/// Test repository info display for bare repository subdirectory
#[test]
fn test_bare_repository_subdirectory_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;
        let bare_repo_path = temp_dir.path().join("test-repo.bare");

        let repo = init_isolated_repo(&bare_repo_path, true)?;
        create_initial_commit(&repo)?;

        // Create subdirectory
        let subdir = bare_repo_path.join("branch");
        fs::create_dir(&subdir)?;

        env::set_current_dir(&subdir)?;

        let info = get_repository_info_at_path(&subdir);
        assert_eq!(
            info, "test-repo",
            "Bare repository subdirectory should show just repo name"
        );

        Ok(())
    })
}

/// Test repository info display for worktree simulation
/// This simulates the file structure of a worktree without using git commands
#[test]
fn test_worktree_simulation_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;

        // Simulate bare repository worktree structure
        let bare_repo_path = temp_dir.path().join("test-repo.bare");
        let bare_repo = init_isolated_repo(&bare_repo_path, true)?;
        create_initial_commit(&bare_repo)?;

        // Create worktree directory
        let worktree_path = temp_dir.path().join("branch").join("feature-x");
        fs::create_dir_all(&worktree_path)?;

        // Create .git file that points to worktree git directory
        let git_file_content = format!("gitdir: {}/worktrees/feature-x", bare_repo_path.display());
        fs::write(worktree_path.join(".git"), git_file_content)?;

        // Create worktree git directory structure
        let worktree_git_dir = bare_repo_path.join("worktrees").join("feature-x");
        fs::create_dir_all(&worktree_git_dir)?;

        // Create commondir file that points back to bare repo
        fs::write(worktree_git_dir.join("commondir"), "../..")?;

        // Create HEAD file
        fs::write(worktree_git_dir.join("HEAD"), "ref: refs/heads/feature-x")?;

        // Create gitdir file
        fs::write(
            worktree_git_dir.join("gitdir"),
            worktree_path.join(".git").display().to_string(),
        )?;

        env::set_current_dir(&worktree_path)?;

        let info = get_repository_info_at_path(&worktree_path);
        assert_eq!(
            info, "test-repo (feature-x)",
            "Simulated worktree should show repo name with worktree name"
        );

        Ok(())
    })
}

/// Test repository info display for non-bare repository main
#[test]
fn test_non_bare_repository_main_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = init_isolated_repo(&repo_path, false)?;
        create_initial_commit(&repo)?;

        env::set_current_dir(&repo_path)?;

        let info = get_repository_info_at_path(&repo_path);
        assert_eq!(
            info, "test-repo",
            "Non-bare repository main should show just repo name"
        );

        Ok(())
    })
}

/// Test repository info display for non-bare repository worktree simulation
#[test]
fn test_non_bare_worktree_simulation_display() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        // Create main repository
        let repo = init_isolated_repo(&repo_path, false)?;
        create_initial_commit(&repo)?;

        // Create .git/worktrees directory to simulate having worktrees
        let worktrees_dir = repo_path.join(".git").join("worktrees");
        fs::create_dir_all(&worktrees_dir)?;

        // Test from main repository with worktrees
        env::set_current_dir(&repo_path)?;
        let info = get_repository_info_at_path(&repo_path);
        assert_eq!(
            info, "test-repo",
            "Main repository with worktrees should show just repo name"
        );

        // Simulate worktree structure
        let worktree_path = temp_dir.path().join("feature-branch");
        fs::create_dir_all(&worktree_path)?;

        // Create .git file that points to worktree git directory
        let git_file_content = format!(
            "gitdir: {}/.git/worktrees/feature-branch",
            repo_path.display()
        );
        fs::write(worktree_path.join(".git"), git_file_content)?;

        // Create worktree git directory
        let worktree_git_dir = worktrees_dir.join("feature-branch");
        fs::create_dir_all(&worktree_git_dir)?;

        // Create commondir file
        fs::write(worktree_git_dir.join("commondir"), "../..")?;

        // Create HEAD file
        fs::write(
            worktree_git_dir.join("HEAD"),
            "ref: refs/heads/feature-branch",
        )?;

        // Create gitdir file
        fs::write(
            worktree_git_dir.join("gitdir"),
            worktree_path.join(".git").display().to_string(),
        )?;

        // Test from worktree
        env::set_current_dir(&worktree_path)?;
        let info = get_repository_info_at_path(&worktree_path);
        assert_eq!(
            info, "test-repo (feature-branch)",
            "Worktree should show repo name with worktree name"
        );

        Ok(())
    })
}

/// Test edge cases for repository name extraction
#[test]
fn test_repository_name_edge_cases() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;

        // Test with special characters
        let special_repo_path = temp_dir.path().join("test-repo_with-special.chars");
        let repo = init_isolated_repo(&special_repo_path, false)?;
        create_initial_commit(&repo)?;

        env::set_current_dir(&special_repo_path)?;
        let info = get_repository_info_at_path(&special_repo_path);
        assert_eq!(info, "test-repo_with-special.chars");

        // Test with numbers
        let number_repo_path = temp_dir.path().join("123-repo");
        let repo = init_isolated_repo(&number_repo_path, false)?;
        create_initial_commit(&repo)?;

        env::set_current_dir(&number_repo_path)?;
        let info = get_repository_info_at_path(&number_repo_path);
        assert_eq!(info, "123-repo");

        Ok(())
    })
}

/// Test all critical display patterns to ensure they're preserved
#[test]
fn test_critical_display_patterns() -> Result<()> {
    run_isolated_test(|| {
        // This test ensures all the display patterns we fixed are preserved
        let temp_dir = TempDir::new()?;

        // Pattern 1: Regular repository → "repo-name"
        let regular_repo = temp_dir.path().join("regular-repo");
        let repo = init_isolated_repo(&regular_repo, false)?;
        create_initial_commit(&repo)?;
        env::set_current_dir(&regular_repo)?;
        assert_eq!(get_repository_info_at_path(&regular_repo), "regular-repo");

        // Pattern 2: Bare repository root → "repo-name" (without .bare suffix)
        let bare_repo = temp_dir.path().join("bare-repo.bare");
        init_isolated_repo(&bare_repo, true)?;
        env::set_current_dir(&bare_repo)?;
        assert_eq!(get_repository_info_at_path(&bare_repo), "bare-repo");

        // Pattern 3: Bare repository subdirectory → "repo-name"
        let bare_subdir = bare_repo.join("subdir");
        fs::create_dir(&bare_subdir)?;
        env::set_current_dir(&bare_subdir)?;
        assert_eq!(get_repository_info_at_path(&bare_subdir), "bare-repo");

        // Pattern 4: Non-git directory → "directory-name"
        let non_git = temp_dir.path().join("not-a-repo");
        fs::create_dir(&non_git)?;
        env::set_current_dir(&non_git)?;
        assert_eq!(get_repository_info_at_path(&non_git), "not-a-repo");

        Ok(())
    })
}

/// Test that repository info never shows patterns we explicitly fixed
#[test]
fn test_fixed_incorrect_patterns() -> Result<()> {
    run_isolated_test(|| {
        let temp_dir = TempDir::new()?;

        // These patterns should NEVER appear:
        // - "git" alone
        // - "git (something)"
        // - "main (main)"
        // - ".git (worktree)"

        // Test bare repository doesn't show "git"
        let bare_repo = temp_dir.path().join("test.bare");
        init_isolated_repo(&bare_repo, true)?;
        env::set_current_dir(&bare_repo)?;
        let info = get_repository_info_at_path(&bare_repo);
        assert_ne!(info, "git", "Should never show 'git' alone");
        assert!(
            !info.starts_with("git ("),
            "Should never show 'git (something)'"
        );

        // Test regular repository doesn't show redundant patterns
        let regular_repo = temp_dir.path().join("main");
        init_isolated_repo(&regular_repo, false)?;
        env::set_current_dir(&regular_repo)?;
        let info = get_repository_info_at_path(&regular_repo);
        assert_ne!(
            info, "main (main)",
            "Should never show redundant 'main (main)'"
        );

        Ok(())
    })
}

/// Test specific documented patterns
#[test]
fn test_documented_patterns() -> Result<()> {
    // These are the specific patterns we've fixed in this session:

    // 1. Bare repository root: "wasabeef" (not "git (wasabeef.bare)")
    // 2. Bare repository subdirectory: "wasabeef" (not "git (branch)")
    // 3. Bare repository worktree: "wasabeef (main)"
    // 4. Non-bare repository: "yank-for-claude.nvim"
    // 5. Non-bare worktree: "yank-for-claude.nvim (ccc)"

    // All these patterns are tested in the tests above
    // This test serves as documentation of the patterns we're testing
    Ok(())
}
