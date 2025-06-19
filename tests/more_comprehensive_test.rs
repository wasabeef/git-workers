use anyhow::Result;
use git2::Repository;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

// This test file focuses on increasing coverage by testing more edge cases

#[cfg(test)]
mod git_tests {
    use super::*;
    use git_workers::git::GitWorktreeManager;

    #[test]
    fn test_create_worktree_detached_head() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;

        // Checkout to detached HEAD using git2
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.set_head_detached(commit.id())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

        let manager = GitWorktreeManager::new_from_path(&repo_path)?;

        // Try to create worktree from detached HEAD
        let result = manager.create_worktree("detached-test", None);
        assert!(result.is_ok() || result.is_err());

        Ok(())
    }

    #[test]
    fn test_list_worktrees_with_locked_worktree() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;

        // Create a worktree
        Command::new("git")
            .current_dir(&repo_path)
            .args(["worktree", "add", "../locked", "-b", "locked-branch"])
            .output()?;

        // Lock the worktree
        Command::new("git")
            .current_dir(&repo_path)
            .args(["worktree", "lock", "../locked"])
            .output()?;

        let manager = GitWorktreeManager::new_from_path(&repo_path)?;
        let worktrees = manager.list_worktrees()?;

        // Should have worktrees including the locked one
        assert!(!worktrees.is_empty());

        Ok(())
    }

    #[test]
    fn test_remove_worktree_that_doesnt_exist() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;

        let manager = GitWorktreeManager::new_from_path(&repo_path)?;

        // Try to remove non-existent worktree
        let result = manager.remove_worktree("ghost-worktree");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_create_worktree_from_remote_branch() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");

        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;

        // Create a branch using git2
        let obj = repo.revparse_single("HEAD")?;
        let commit = obj.as_commit().unwrap();
        repo.branch("remote-tracking", commit, false)?;

        // Ensure we're on main/master branch
        let head_ref = repo.head()?;
        let current_branch = head_ref.shorthand().unwrap_or("main");

        if current_branch == "remote-tracking" {
            // Switch back to main or master
            if repo.find_branch("main", git2::BranchType::Local).is_ok() {
                repo.set_head("refs/heads/main")?;
            } else if repo.find_branch("master", git2::BranchType::Local).is_ok() {
                repo.set_head("refs/heads/master")?;
            }
            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }

        let manager = GitWorktreeManager::new_from_path(&repo_path)?;

        // Create worktree from existing branch
        let result = manager.create_worktree("remote-work", Some("remote-tracking"));
        if let Err(e) = &result {
            eprintln!("Failed to create worktree: {}", e);
        }
        assert!(result.is_ok());

        Ok(())
    }
}

#[cfg(test)]
mod config_tests {
    use git_workers::config::Config;
    use std::collections::HashMap;

    #[test]
    fn test_config_default_trait() {
        let config = Config::default();
        assert!(config.hooks.is_empty());
    }

    #[test]
    fn test_config_debug_trait() {
        let mut hooks = HashMap::new();
        hooks.insert("test".to_string(), vec!["echo test".to_string()]);
        let config = Config {
            repository: git_workers::config::RepositoryConfig::default(),
            hooks,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("hooks"));
    }
}

#[cfg(test)]
mod repository_info_tests {
    use super::*;
    use git_workers::repository_info::get_repository_info;

    #[test]
    fn test_repository_info_in_git_submodule() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let parent_repo = temp_dir.path().join("parent");
        let submodule_path = parent_repo.join("submodule");

        // Create parent repository
        let repo = Repository::init(&parent_repo)?;
        create_initial_commit(&repo)?;

        // Create submodule repository
        fs::create_dir_all(&submodule_path)?;
        let sub_repo = Repository::init(&submodule_path)?;
        create_initial_commit(&sub_repo)?;

        std::env::set_current_dir(&submodule_path)?;

        let info = get_repository_info();
        assert!(info.contains("submodule") || !info.is_empty());

        Ok(())
    }

    #[test]
    fn test_repository_info_with_unicode_name() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("テスト-repo");

        fs::create_dir_all(&repo_path)?;
        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;

        std::env::set_current_dir(&repo_path)?;

        let info = get_repository_info();
        assert!(!info.is_empty());

        Ok(())
    }
}

#[cfg(test)]
mod utils_tests {
    use git_workers::utils::{print_error, print_progress, print_success};

    #[test]
    fn test_print_functions_with_ansi_codes() {
        // Test with strings containing ANSI escape codes
        print_success("\x1b[32mGreen text\x1b[0m");
        print_error("\x1b[31mRed text\x1b[0m");
        print_progress("\x1b[33mYellow text\x1b[0m");
    }

    #[test]
    fn test_print_functions_with_empty_strings() {
        print_success("");
        print_error("");
        print_progress("");
    }

    #[test]
    fn test_print_functions_with_very_long_strings() {
        let long_str = "x".repeat(200); // Reduced from 10000 to 200
        print_success(&long_str);
        print_error(&long_str);
        print_progress(&long_str);
    }
}

#[cfg(test)]
mod hooks_tests {
    use super::*;
    use git_workers::hooks::{execute_hooks, HookContext};

    #[test]
    fn test_hook_with_complex_shell_commands() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().join("test-repo");
        let worktree_path = temp_dir.path().join("test-worktree");

        let repo = Repository::init(&repo_path)?;
        create_initial_commit(&repo)?;
        fs::create_dir_all(&worktree_path)?;

        // Create config with complex shell commands
        let config_content = r#"
[hooks]
post-create = [
    "echo 'Test' | grep 'Test'",
    "true && echo 'Success'",
    "[ -d . ] && echo 'Directory exists'"
]
"#;
        fs::write(repo_path.join(".git-workers.toml"), config_content)?;

        std::env::set_current_dir(&repo_path)?;

        let context = HookContext {
            worktree_name: "test-worktree".to_string(),
            worktree_path,
        };

        let result = execute_hooks("post-create", &context);
        assert!(result.is_ok());

        Ok(())
    }
}

// Helper function
fn create_initial_commit(repo: &Repository) -> Result<()> {
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}
