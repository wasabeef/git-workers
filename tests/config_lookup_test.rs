use git_workers::config::Config;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tempfile::TempDir;

// Use a mutex to ensure tests don't interfere with each other
// when changing the current directory
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_config_lookup_in_git_directory() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-lookup");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create a config file in the repository root
    let config_content = r#"
[repository]
url = "https://github.com/test/repo.git"

[hooks]
post-create = ["echo 'Found config in git dir'"]
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
        vec!["echo 'Found config in git dir'"]
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_current_directory_priority() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-current-dir");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Create a config file in the repository root
    let root_config = r#"
[hooks]
post-create = ["echo 'From root'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), root_config).unwrap();

    // Create a subdirectory with its own config
    let sub_dir = repo_path.join("worktrees").join("feature");
    fs::create_dir_all(&sub_dir).unwrap();

    let sub_config = r#"
[hooks]
post-create = ["echo 'From current directory'"]
"#;
    fs::write(sub_dir.join(".git-workers.toml"), sub_config).unwrap();

    // Change to subdirectory and load config
    std::env::set_current_dir(&sub_dir).unwrap();

    // Should load config from current directory, not repository root
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
fn test_config_parent_main_worktree() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("project");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Create a worktree structure: project/main and project/feature
    let main_dir = base_path.join("main");
    let feature_dir = base_path.join("feature");
    fs::create_dir_all(&main_dir).unwrap();
    fs::create_dir_all(&feature_dir).unwrap();

    // Initialize git in main
    Command::new("git")
        .current_dir(&main_dir)
        .args(["init"])
        .output()
        .unwrap();

    // Create config in main worktree
    let main_config = r#"
[hooks]
post-create = ["echo 'From main worktree'"]
"#;
    fs::write(main_dir.join(".git-workers.toml"), main_config).unwrap();

    // Initialize git in feature
    Command::new("git")
        .current_dir(&feature_dir)
        .args(["init"])
        .output()
        .unwrap();

    // Change to feature directory and load config
    std::env::set_current_dir(&feature_dir).unwrap();

    // Should find config in parent's main directory
    let config = Config::load().unwrap();
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(
        config.hooks["post-create"],
        vec!["echo 'From main worktree'"]
    );

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_repository_url_validation() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo-url");

    // Save current directory to restore later
    let original_dir = std::env::current_dir().unwrap();

    // Initialize a git repository with origin
    fs::create_dir(&repo_path).unwrap();
    Command::new("git")
        .current_dir(&repo_path)
        .args(["init"])
        .output()
        .unwrap();

    // Add a remote origin
    Command::new("git")
        .current_dir(&repo_path)
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/actual/repo.git",
        ])
        .output()
        .unwrap();

    // Create a config file with mismatched URL
    let config_content = r#"
[repository]
url = "https://github.com/wrong/repo.git"

[hooks]
post-create = ["echo 'Should not run'"]
"#;
    fs::write(repo_path.join(".git-workers.toml"), config_content).unwrap();

    // Change to repo and load config
    std::env::set_current_dir(&repo_path).unwrap();

    // Config should return default (empty) due to URL mismatch
    let config = Config::load().unwrap();
    assert!(config.hooks.is_empty());

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}
