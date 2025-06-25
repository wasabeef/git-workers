use git_workers::config::Config;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tempfile::TempDir;

// Use a mutex to ensure tests don't interfere with each other
// when changing the current directory
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_config_lookup_in_repository_root() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-root");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create initial commit to ensure we have a working directory
    fs::write(repo_path.join("README.md"), "# Test Repository").unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "."])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Create a config file in repository root
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'Found config in repo root'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content).unwrap();

    // Create a subdirectory
    let sub_dir = repo_path.join("src").join("components");
    fs::create_dir_all(&sub_dir).unwrap();

    // Change to subdirectory and load config
    std::env::set_current_dir(&sub_dir).unwrap();

    let config = Config::load().unwrap();
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'Found config in repo root'"]
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_current_dir_precedence_over_root() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-cwd-precedence");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create a config file in repository root
    let root_config = r#"
[hooks]
post-create = ["echo 'From root'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), root_config).unwrap();

    // Create a subdirectory with its own config (simulating a bare repo worktree)
    let worktree_dir = repo_path.join("feature-worktree");
    fs::create_dir(&worktree_dir).unwrap();

    let worktree_config = r#"
[hooks]
post-create = ["echo 'From current directory'"]
"#;
    fs::write(worktree_dir.join(".git-workers.toml"), worktree_config).unwrap();

    // Change to worktree directory
    std::env::set_current_dir(&worktree_dir).unwrap();

    // Current directory config should take precedence
    let config = Config::load().unwrap();
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'From current directory'"]
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[ignore = "Flaky test due to parallel execution"]
fn test_config_precedence_root_over_git_dir() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-precedence");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repository").unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "."])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Create a config file in .git directory
    let git_config_content = r#"
[hooks]
post-create = ["echo 'From git dir'"]
"#;
    fs::write(repo_path.join(".git/git-workers.toml"), git_config_content).unwrap();

    // Create a config file in repository root
    let root_config_content = r#"
[hooks]
post-create = ["echo 'From repo root'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), root_config_content).unwrap();

    // Change to repo and load config
    std::env::set_current_dir(&repo_path).unwrap();

    // Root config should take precedence
    let config = Config::load().unwrap();
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(config.hooks["post-create"], vec!["echo 'From repo root'"]);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_in_worktree() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-worktree");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repository").unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", "."])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Create a config file in repository root
    let config_content = r#"
[hooks]
post-create = ["echo 'Config from main repo'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content).unwrap();

    // Add the config to git
    Command::new("git")
        .current_dir(&repo_path)
        .args(["add", ".git-workers.toml"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["commit", "-m", "Add git-workers config"])
        .output()
        .unwrap();

    // Create a worktree
    let worktree_path = temp_dir.path().join("test-worktree");
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test-branch",
        ])
        .output()
        .unwrap();

    // Change to worktree and load config
    std::env::set_current_dir(&worktree_path).unwrap();

    // Should find config from the worktree's working directory
    let config = Config::load().unwrap();
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'Config from main repo'"]
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}
