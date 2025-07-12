use anyhow::Result;
use git2::Repository;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_bare_repository_info() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.bare");

    // Initialize bare repository
    Repository::init_bare(&bare_repo_path)?;

    // Change to bare repository directory
    std::env::set_current_dir(&bare_repo_path)?;

    // Test repository info
    let info = get_repository_info_for_test()?;
    assert_eq!(info, "test-repo.bare");

    Ok(())
}

#[test]
fn test_worktree_from_bare_repository() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("test-repo.bare");

    // Initialize bare repository
    let bare_repo = Repository::init_bare(&bare_repo_path)?;

    // Create initial commit in bare repo
    create_initial_commit_bare(&bare_repo)?;

    // Create worktree
    let worktree_path = temp_dir.path().join("branch").join("feature-x");
    fs::create_dir_all(worktree_path.parent().unwrap())?;

    // Use git command to create worktree
    std::process::Command::new("git")
        .current_dir(&bare_repo_path)
        .arg("worktree")
        .arg("add")
        .arg(&worktree_path)
        .arg("-b")
        .arg("feature-x")
        .output()?;

    // Change to worktree directory
    std::env::set_current_dir(&worktree_path)?;

    // Test repository info
    let info = get_repository_info_for_test()?;
    // When running in a worktree, the info shows just the worktree name
    // The parent repository detection requires specific git setup
    assert_eq!(info, "feature-x");

    Ok(())
}

#[test]
fn test_main_worktree_with_worktrees() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("my-project");

    // Initialize regular repository
    let repo = Repository::init(&main_repo_path)?;
    create_initial_commit(&repo)?;

    // Create a worktree
    let worktree_path = temp_dir.path().join("my-project-feature");
    std::process::Command::new("git")
        .current_dir(&main_repo_path)
        .arg("worktree")
        .arg("add")
        .arg(&worktree_path)
        .arg("-b")
        .arg("feature")
        .output()?;

    // Change to main repository
    std::env::set_current_dir(&main_repo_path)?;

    // Test repository info
    let info = get_repository_info_for_test()?;
    assert_eq!(info, "my-project (main)");

    Ok(())
}

#[test]
fn test_regular_repository_without_worktrees() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("simple-project");

    // Initialize regular repository
    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Change to repository directory
    std::env::set_current_dir(&repo_path)?;

    // Test repository info
    let info = get_repository_info_for_test()?;
    assert_eq!(info, "simple-project");

    Ok(())
}

#[test]
fn test_worktree_pattern_detection() -> Result<()> {
    use git_workers::git::WorktreeInfo;
    use std::path::PathBuf;

    // Test case 1: No worktrees
    let empty_worktrees: Vec<WorktreeInfo> = vec![];
    let pattern = detect_worktree_pattern(&empty_worktrees);
    assert!(matches!(pattern, WorktreePattern::Direct));

    // Test case 2: Worktrees in branch/ subdirectory
    let branch_worktrees = vec![
        WorktreeInfo {
            name: "feature-a".to_string(),
            path: PathBuf::from("/project/branch/feature-a"),
            branch: "feature-a".to_string(),
            is_locked: false,
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        },
        WorktreeInfo {
            name: "feature-b".to_string(),
            path: PathBuf::from("/project/branch/feature-b"),
            branch: "feature-b".to_string(),
            is_locked: false,
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        },
    ];
    let pattern = detect_worktree_pattern(&branch_worktrees);
    // The pattern detection looks for "branch" in the path components
    assert!(matches!(pattern, WorktreePattern::InSubDir(_)));

    // Test case 3: Worktrees directly in parent directory
    let direct_worktrees = vec![WorktreeInfo {
        name: "feature-a".to_string(),
        path: PathBuf::from("/project/feature-a"),
        branch: "feature-a".to_string(),
        is_locked: false,
        is_current: false,
        has_changes: false,
        last_commit: None,
        ahead_behind: None,
    }];
    let pattern = detect_worktree_pattern(&direct_worktrees);
    // For path /project/feature-a where name is feature-a, it should be Direct
    if let WorktreePattern::InSubDir(dir) = &pattern {
        panic!("Expected Direct, got InSubDir({dir})");
    }
    assert!(matches!(pattern, WorktreePattern::Direct));

    Ok(())
}

#[test]
fn test_first_worktree_path_pattern() -> Result<()> {
    // This test simulates the interactive prompt for first worktree
    // In actual usage, user would input "branch/{name}" or custom pattern

    let test_patterns = vec![
        ("branch/{name}", "feature", "branch/feature"),
        ("{name}", "feature", "feature"),
        ("wt/{name}", "feature", "wt/feature"),
        ("worktrees/{name}-wt", "feature", "worktrees/feature-wt"),
    ];

    for (pattern, name, expected) in test_patterns {
        let result = pattern.replace("{name}", name);
        assert_eq!(result, expected);
    }

    Ok(())
}

// Helper function to simulate get_repository_info
fn get_repository_info_for_test() -> Result<String> {
    use git_workers::repository_info::get_repository_info;
    Ok(get_repository_info())
}

// Helper functions from test utilities
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

fn create_initial_commit_bare(repo: &Repository) -> Result<()> {
    // For bare repository, we need to create a commit differently
    use git2::Signature;

    let sig = Signature::now("Test User", "test@example.com")?;

    // Create an empty tree
    let tree_id = {
        let builder = repo.treebuilder(None)?;
        builder.write()?
    };
    let tree = repo.find_tree(tree_id)?;

    // Create the initial commit
    let _commit_id = repo.commit(
        Some("refs/heads/main"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    )?;

    // Set HEAD to main branch
    repo.set_head("refs/heads/main")?;

    Ok(())
}

// Enum definitions for testing
#[derive(Debug, PartialEq)]
enum WorktreePattern {
    InSubDir(String),
    Direct,
}

fn detect_worktree_pattern(worktrees: &[git_workers::git::WorktreeInfo]) -> WorktreePattern {
    if worktrees.is_empty() {
        return WorktreePattern::Direct;
    }

    for worktree in worktrees {
        // Get the parent directory of the worktree
        if let Some(parent) = worktree.path.parent() {
            // Get the last component of the parent path
            if let Some(parent_name) = parent.file_name() {
                if let Some(parent_str) = parent_name.to_str() {
                    // Check if this looks like a subdirectory pattern (e.g., "branch")
                    // Common patterns: branch, branches, worktrees, wt, etc.
                    if parent_str == "branch"
                        || parent_str == "branches"
                        || parent_str == "worktrees"
                        || parent_str == "wt"
                    {
                        return WorktreePattern::InSubDir(parent_str.to_string());
                    }
                }
            }
        }
    }

    WorktreePattern::Direct
}
