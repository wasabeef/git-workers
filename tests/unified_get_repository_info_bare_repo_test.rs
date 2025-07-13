use anyhow::Result;
use git2::Repository;
use git_workers::repository_info::get_repository_info;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test get_repository_info in bare repository (basic case with .git extension)
#[test]
fn test_get_repository_info_bare_repo_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo = temp_dir.path().join("test.git");

    // Initialize bare repository using git command
    Command::new("git")
        .args(["init", "--bare", "test.git"])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&bare_repo)?;

    let info = get_repository_info();

    // Should return bare repository name
    assert_eq!(info, "test.git");

    Ok(())
}

/// Test get_repository_info in bare repository using git2 library
#[test]
fn test_get_repository_info_bare_repo_git2() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.bare");

    // Initialize bare repository using git2
    Repository::init_bare(&bare_repo_path)?;

    std::env::set_current_dir(&bare_repo_path)?;

    let info = get_repository_info();
    // Bare repos just show the directory name without special formatting
    assert!(info.contains("test-repo.bare"));

    Ok(())
}

/// Test bare repository with various naming conventions
#[test]
fn test_get_repository_info_bare_with_various_names() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test both with and without .git extension and other naming patterns
    let bare_names = vec![
        "project.git",    // Traditional bare naming
        "project-bare",   // Alternative bare naming
        "repo.git",       // Short name with .git
        "my-project.git", // With hyphens
        "123-repo.git",   // Starting with numbers
    ];

    for bare_name in bare_names {
        let bare_path = temp_dir.path().join(bare_name);

        // Initialize bare repository
        Command::new("git")
            .args(["init", "--bare", bare_name])
            .current_dir(temp_dir.path())
            .output()?;

        std::env::set_current_dir(&bare_path)?;

        let info = get_repository_info();
        assert_eq!(info, bare_name, "Failed for bare repo: {bare_name}");
    }

    // Test uppercase separately due to filesystem case sensitivity
    let uppercase_name = "PROJECT.GIT";
    let uppercase_path = temp_dir.path().join(uppercase_name);

    Command::new("git")
        .args(["init", "--bare", uppercase_name])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&uppercase_path)?;

    let info = get_repository_info();
    // On case-insensitive filesystems, the name might be normalized to lowercase
    assert!(
        info == uppercase_name || info == uppercase_name.to_lowercase(),
        "Failed for uppercase bare repo: {uppercase_name}, got: {info}"
    );

    Ok(())
}

/// Test bare repository with special characters in name
#[test]
fn test_get_repository_info_bare_special_characters() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let special_names = vec![
        "my_project.git",        // Underscores
        "project.with.dots.git", // Multiple dots
        "project-2024.git",      // With year
    ];

    for special_name in special_names {
        let bare_path = temp_dir.path().join(special_name);

        // Initialize bare repository
        Command::new("git")
            .args(["init", "--bare", special_name])
            .current_dir(temp_dir.path())
            .output()?;

        std::env::set_current_dir(&bare_path)?;

        let info = get_repository_info();
        assert_eq!(
            info, special_name,
            "Failed for special bare repo: {special_name}"
        );
    }

    Ok(())
}

/// Test bare repository with spaces in name
#[test]
fn test_get_repository_info_bare_with_spaces() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let space_names = vec!["my project.git", "test repo.git"];

    for space_name in space_names {
        let bare_path = temp_dir.path().join(space_name);

        // Initialize bare repository
        Command::new("git")
            .args(["init", "--bare", space_name])
            .current_dir(temp_dir.path())
            .output()?;

        std::env::set_current_dir(&bare_path)?;

        let info = get_repository_info();
        assert_eq!(
            info, space_name,
            "Failed for bare repo with spaces: {space_name}"
        );
    }

    Ok(())
}

/// Test bare repository with long names
#[test]
fn test_get_repository_info_bare_long_name() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a long bare repository name
    let long_name = format!("very-long-bare-repository-name-{}.git", "x".repeat(30));
    let bare_path = temp_dir.path().join(&long_name);

    // Initialize bare repository
    Command::new("git")
        .args(["init", "--bare", &long_name])
        .current_dir(temp_dir.path())
        .output()?;

    std::env::set_current_dir(&bare_path)?;

    let info = get_repository_info();
    assert_eq!(info, long_name);

    Ok(())
}

/// Test bare repository in nested directory structure
#[test]
fn test_get_repository_info_bare_nested() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let nested_dir = temp_dir.path().join("repos").join("bare");
    fs::create_dir_all(&nested_dir)?;

    let bare_repo_path = nested_dir.join("nested-repo.git");

    // Initialize bare repository using git2 in nested directory
    Repository::init_bare(&bare_repo_path)?;

    std::env::set_current_dir(&bare_repo_path)?;

    let info = get_repository_info();
    assert!(info.contains("nested-repo.git"));

    Ok(())
}

/// Test bare repository with worktrees created from it
#[test]
fn test_get_repository_info_bare_with_worktrees() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare-main.git");

    // Initialize bare repository
    Repository::init_bare(&bare_repo_path)?;

    // Create an initial commit in the bare repo
    let repo = Repository::open(&bare_repo_path)?;
    create_initial_commit_bare(&repo)?;

    // Test from bare repository
    std::env::set_current_dir(&bare_repo_path)?;
    let info = get_repository_info();
    assert_eq!(info, "bare-main.git");

    // Create a worktree from the bare repository
    let worktree_path = temp_dir.path().join("worktree-from-bare");
    Command::new("git")
        .current_dir(&bare_repo_path)
        .args(["worktree", "add", worktree_path.to_str().unwrap()])
        .output()?;

    // Test from worktree created from bare repo
    std::env::set_current_dir(&worktree_path)?;
    let worktree_info = get_repository_info();
    // The worktree should show its own directory name
    assert_eq!(worktree_info, "worktree-from-bare");

    Ok(())
}

/// Test comparison between bare and non-bare repositories
#[test]
fn test_get_repository_info_bare_vs_normal() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create normal repository
    let normal_repo_path = temp_dir.path().join("normal-repo");
    let normal_repo = Repository::init(&normal_repo_path)?;
    create_initial_commit_normal(&normal_repo)?;

    // Create bare repository
    let bare_repo_path = temp_dir.path().join("bare-repo.git");
    Repository::init_bare(&bare_repo_path)?;

    // Test normal repository
    std::env::set_current_dir(&normal_repo_path)?;
    let normal_info = get_repository_info();
    assert!(
        normal_info.contains("normal-repo"),
        "Expected normal repo info to contain 'normal-repo', got: {normal_info}"
    );
    assert!(
        !normal_info.contains(".git"),
        "Normal repos should not show .git in name, got: {normal_info}"
    ); // Normal repos don't show .git in name

    // Test bare repository
    std::env::set_current_dir(&bare_repo_path)?;
    let bare_info = get_repository_info();
    assert_eq!(bare_info, "bare-repo.git"); // Bare repos show full directory name

    Ok(())
}

/// Test edge case: bare repository without .git extension
#[test]
fn test_get_repository_info_bare_no_extension() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare-repo-no-ext");

    // Initialize bare repository without .git extension
    Repository::init_bare(&bare_repo_path)?;

    std::env::set_current_dir(&bare_repo_path)?;

    let info = get_repository_info();
    assert_eq!(info, "bare-repo-no-ext");

    Ok(())
}

// Helper function to create initial commit in bare repository
fn create_initial_commit_bare(repo: &Repository) -> Result<()> {
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    // Create initial commit with empty tree
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok(())
}

// Helper function to create initial commit in normal repository
fn create_initial_commit_normal(repo: &Repository) -> Result<()> {
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
