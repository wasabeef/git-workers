use anyhow::Result;
use git_workers::git_interface::{
    test_helpers::GitScenarioBuilder, GitInterface, MockGitInterface, WorktreeConfig, WorktreeInfo,
};
use std::path::PathBuf;

/// Example of migrating a test from real git operations to mock interface
/// Original test: test_create_worktree_with_new_branch from unified_git_comprehensive_test.rs
#[test]
fn test_create_worktree_with_new_branch_mocked() -> Result<()> {
    // Build scenario
    let (worktrees, branches, tags, repo_info) = GitScenarioBuilder::new()
        .with_branch("main", false)
        .with_current_branch("main")
        .build();

    let mock = MockGitInterface::with_scenario(worktrees, branches, tags, repo_info);

    // Create worktree with new branch
    let config = WorktreeConfig {
        name: "feature".to_string(),
        path: PathBuf::from("/repo/worktrees/feature"),
        branch: Some("feature-branch".to_string()),
        create_branch: true,
        base_branch: Some("main".to_string()),
    };

    let worktree = mock.create_worktree(&config)?;

    // Verify worktree was created
    assert_eq!(worktree.name, "feature");
    assert_eq!(worktree.branch, Some("feature-branch".to_string()));

    // Verify branch was created
    let branches = mock.list_branches()?;
    assert!(branches.iter().any(|b| b.name == "feature-branch"));

    // Verify worktree is listed
    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 1);
    assert_eq!(worktrees[0].name, "feature");

    Ok(())
}

/// Example of migrating branch operations test
#[test]
fn test_branch_operations_mocked() -> Result<()> {
    let mock = MockGitInterface::new();

    // Initial state: only main branch exists
    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");

    // Create new branch
    mock.create_branch("develop", Some("main"))?;

    // Verify branch was created
    assert!(mock.branch_exists("develop")?);
    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 2);

    // Create worktree with existing branch
    let config = WorktreeConfig {
        name: "dev-work".to_string(),
        path: PathBuf::from("/repo/worktrees/dev-work"),
        branch: Some("develop".to_string()),
        create_branch: false,
        base_branch: None,
    };

    mock.create_worktree(&config)?;

    // Verify branch-worktree mapping
    let map = mock.get_branch_worktree_map()?;
    assert_eq!(map.get("develop"), Some(&"dev-work".to_string()));

    // Try to delete branch that's in use (should fail if force=false)
    assert!(mock.delete_branch("develop", false).is_err());

    // Remove worktree first
    mock.remove_worktree("dev-work")?;

    // Now delete branch should succeed
    mock.delete_branch("develop", false)?;
    assert!(!mock.branch_exists("develop")?);

    Ok(())
}

/// Example of testing complex worktree scenarios
#[test]
fn test_complex_worktree_scenario_mocked() -> Result<()> {
    // Setup a complex scenario with multiple worktrees and branches
    let (worktrees, branches, tags, repo_info) = GitScenarioBuilder::new()
        .with_worktree("main", "/repo", Some("main"))
        .with_worktree(
            "feature-1",
            "/repo/worktrees/feature-1",
            Some("feature/auth"),
        )
        .with_worktree("hotfix", "/repo/worktrees/hotfix", Some("hotfix/v1.0.1"))
        .with_branch("main", false)
        .with_branch("feature/auth", false)
        .with_branch("hotfix/v1.0.1", false)
        .with_branch("develop", false)
        .with_tag("v1.0.0", Some("Initial release"))
        .with_tag("v1.0.1", None)
        .with_current_branch("feature/auth")
        .build();

    let mock = MockGitInterface::with_scenario(worktrees, branches, tags, repo_info);

    // Verify initial state
    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 3);

    // Test get main worktree
    let main = mock.get_main_worktree()?;
    assert!(main.is_some());
    assert_eq!(main.unwrap().name, "main");

    // Test branch-worktree mapping
    let map = mock.get_branch_worktree_map()?;
    assert_eq!(map.len(), 3);
    assert_eq!(map.get("main"), Some(&"main".to_string()));
    assert_eq!(map.get("feature/auth"), Some(&"feature-1".to_string()));
    assert_eq!(map.get("hotfix/v1.0.1"), Some(&"hotfix".to_string()));

    // Test current branch
    assert_eq!(mock.get_current_branch()?, Some("feature/auth".to_string()));

    // Test tags
    let tags = mock.list_tags()?;
    assert_eq!(tags.len(), 2);
    assert!(tags.iter().any(|t| t.name == "v1.0.0" && t.is_annotated));

    // Remove a worktree
    mock.remove_worktree("hotfix")?;
    assert_eq!(mock.list_worktrees()?.len(), 2);
    assert!(!mock
        .get_branch_worktree_map()?
        .contains_key("hotfix/v1.0.1"));

    Ok(())
}

/// Example of testing error conditions
#[test]
fn test_error_conditions_mocked() -> Result<()> {
    let mock = MockGitInterface::new();

    // Add a worktree
    mock.add_worktree(WorktreeInfo {
        name: "existing".to_string(),
        path: PathBuf::from("/repo/worktrees/existing"),
        branch: Some("existing-branch".to_string()),
        commit: "abc123".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    // Try to create duplicate worktree
    let config = WorktreeConfig {
        name: "existing".to_string(),
        path: PathBuf::from("/repo/worktrees/existing2"),
        branch: None,
        create_branch: false,
        base_branch: None,
    };
    assert!(mock.create_worktree(&config).is_err());

    // Try to remove non-existent worktree
    assert!(mock.remove_worktree("non-existent").is_err());

    // Try to create branch that already exists
    assert!(mock.create_branch("main", None).is_err());

    // Try to rename to existing worktree
    mock.add_worktree(WorktreeInfo {
        name: "another".to_string(),
        path: PathBuf::from("/repo/worktrees/another"),
        branch: None,
        commit: "def456".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    assert!(mock.rename_worktree("existing", "another").is_err());

    Ok(())
}

/// Example benchmark comparing mock vs real operations
#[test]
fn test_performance_comparison_mocked() -> Result<()> {
    use std::time::Instant;

    // Mock operations
    let start = Instant::now();
    let mock = MockGitInterface::new();

    // Create 10 worktrees with mock
    for i in 0..10 {
        let config = WorktreeConfig {
            name: format!("feature-{i}"),
            path: PathBuf::from(format!("/repo/worktrees/feature-{i}")),
            branch: Some(format!("feature-{i}")),
            create_branch: true,
            base_branch: Some("main".to_string()),
        };
        mock.create_worktree(&config)?;
    }

    // List worktrees 100 times
    for _ in 0..100 {
        let _ = mock.list_worktrees()?;
    }

    let mock_duration = start.elapsed();

    // Clean up
    for i in 0..10 {
        mock.remove_worktree(&format!("feature-{i}"))?;
    }

    println!("Mock operations completed in: {mock_duration:?}");

    // Mock operations should be very fast (< 10ms typically)
    assert!(mock_duration.as_millis() < 100);

    Ok(())
}

/// Example of testing with expectations
#[test]
fn test_with_expectations_mocked() -> Result<()> {
    use git_workers::git_interface::mock_git::Expectation;

    let mock = MockGitInterface::new();

    // Set expectations
    mock.expect_operation(Expectation::ListWorktrees);
    mock.expect_operation(Expectation::CreateWorktree {
        name: "test".to_string(),
        branch: Some("test-branch".to_string()),
    });
    mock.expect_operation(Expectation::ListBranches);

    // Execute operations in expected order
    let _ = mock.list_worktrees()?;

    let config = WorktreeConfig {
        name: "test".to_string(),
        path: PathBuf::from("/repo/test"),
        branch: Some("test-branch".to_string()),
        create_branch: false,
        base_branch: None,
    };
    mock.create_worktree(&config)?;

    let _ = mock.list_branches()?;

    // Verify all expectations were met
    mock.verify_expectations()?;

    Ok(())
}
