use anyhow::Result;
use git_workers::git_interface::{
    test_helpers::GitScenarioBuilder, BranchInfo, GitInterface, MockGitInterface, TagInfo,
    WorktreeConfig, WorktreeInfo,
};
use std::path::PathBuf;

#[test]
fn test_mock_basic_operations() -> Result<()> {
    let mock = MockGitInterface::new();

    // Test initial state
    let repo_info = mock.get_repository_info()?;
    assert_eq!(repo_info.current_branch, Some("main".to_string()));
    assert!(!repo_info.is_bare);

    // Test list worktrees (should be empty initially)
    let worktrees = mock.list_worktrees()?;
    assert!(worktrees.is_empty());

    // Test list branches (should have main)
    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");

    Ok(())
}

#[test]
fn test_mock_create_worktree_with_new_branch() -> Result<()> {
    let mock = MockGitInterface::new();

    let config = WorktreeConfig {
        name: "feature".to_string(),
        path: PathBuf::from("/mock/repo/feature"),
        branch: Some("feature-branch".to_string()),
        create_branch: true,
        base_branch: None,
    };

    let worktree = mock.create_worktree(&config)?;
    assert_eq!(worktree.name, "feature");
    assert_eq!(worktree.path, PathBuf::from("/mock/repo/feature"));
    assert_eq!(worktree.branch, Some("feature-branch".to_string()));

    // Verify worktree was added
    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 1);
    assert_eq!(worktrees[0].name, "feature");

    // Verify branch was created
    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 2);
    assert!(branches.iter().any(|b| b.name == "feature-branch"));

    // Verify branch-worktree mapping
    let map = mock.get_branch_worktree_map()?;
    assert_eq!(map.get("feature-branch"), Some(&"feature".to_string()));

    Ok(())
}

#[test]
fn test_mock_create_worktree_existing_branch() -> Result<()> {
    let mock = MockGitInterface::new();

    // Add a branch first
    mock.add_branch(BranchInfo {
        name: "develop".to_string(),
        is_remote: false,
        upstream: None,
        commit: "dev123".to_string(),
    })?;

    let config = WorktreeConfig {
        name: "dev-work".to_string(),
        path: PathBuf::from("/mock/repo/dev-work"),
        branch: Some("develop".to_string()),
        create_branch: false,
        base_branch: None,
    };

    let worktree = mock.create_worktree(&config)?;
    assert_eq!(worktree.branch, Some("develop".to_string()));

    // Branch count should not increase
    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 2); // main + develop

    Ok(())
}

#[test]
fn test_mock_remove_worktree() -> Result<()> {
    let mock = MockGitInterface::new();

    // Create a worktree first
    mock.add_worktree(WorktreeInfo {
        name: "temp".to_string(),
        path: PathBuf::from("/mock/repo/temp"),
        branch: Some("temp-branch".to_string()),
        commit: "temp123".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    // Verify it exists
    assert_eq!(mock.list_worktrees()?.len(), 1);

    // Remove it
    mock.remove_worktree("temp")?;

    // Verify it's gone
    assert_eq!(mock.list_worktrees()?.len(), 0);

    // Verify branch mapping is cleaned up
    let map = mock.get_branch_worktree_map()?;
    assert!(!map.contains_key("temp-branch"));

    Ok(())
}

#[test]
fn test_mock_branch_operations() -> Result<()> {
    let mock = MockGitInterface::new();

    // Create a new branch
    mock.create_branch("feature", Some("main"))?;

    // Verify it exists
    assert!(mock.branch_exists("feature")?);
    let branches = mock.list_branches()?;
    assert!(branches.iter().any(|b| b.name == "feature"));

    // Delete the branch
    mock.delete_branch("feature", false)?;

    // Verify it's gone
    assert!(!mock.branch_exists("feature")?);

    Ok(())
}

#[test]
fn test_mock_with_scenario_builder() -> Result<()> {
    let (worktrees, branches, tags, repo_info) = GitScenarioBuilder::new()
        .with_worktree("main", "/repo", Some("main"))
        .with_worktree("feature", "/repo/feature", Some("feature-branch"))
        .with_worktree("hotfix", "/repo/hotfix", Some("hotfix/v1"))
        .with_branch("main", false)
        .with_branch("feature-branch", false)
        .with_branch("develop", false)
        .with_branch("hotfix/v1", false)
        .with_tag("v1.0.0", Some("Release 1.0.0"))
        .with_tag("v1.0.1", None)
        .with_current_branch("feature-branch")
        .build();

    let mock = MockGitInterface::with_scenario(worktrees, branches, tags, repo_info);

    // Verify scenario setup
    let worktrees = mock.list_worktrees()?;
    assert_eq!(worktrees.len(), 3);

    let branches = mock.list_branches()?;
    assert_eq!(branches.len(), 4);

    let tags = mock.list_tags()?;
    assert_eq!(tags.len(), 2);
    assert!(tags.iter().any(|t| t.name == "v1.0.0" && t.is_annotated));
    assert!(tags.iter().any(|t| t.name == "v1.0.1" && !t.is_annotated));

    let current_branch = mock.get_current_branch()?;
    assert_eq!(current_branch, Some("feature-branch".to_string()));

    Ok(())
}

#[test]
fn test_mock_rename_worktree() -> Result<()> {
    let mock = MockGitInterface::new();

    // Add a worktree
    mock.add_worktree(WorktreeInfo {
        name: "old-name".to_string(),
        path: PathBuf::from("/mock/repo/old-name"),
        branch: Some("feature".to_string()),
        commit: "abc123".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    // Rename it
    mock.rename_worktree("old-name", "new-name")?;

    // Verify old name is gone
    assert!(mock.get_worktree("old-name")?.is_none());

    // Verify new name exists
    let worktree = mock.get_worktree("new-name")?;
    assert!(worktree.is_some());
    assert_eq!(worktree.unwrap().name, "new-name");

    // Verify branch mapping is updated
    let map = mock.get_branch_worktree_map()?;
    assert_eq!(map.get("feature"), Some(&"new-name".to_string()));

    Ok(())
}

#[test]
fn test_mock_error_conditions() -> Result<()> {
    let mock = MockGitInterface::new();

    // Try to create duplicate worktree
    mock.add_worktree(WorktreeInfo {
        name: "existing".to_string(),
        path: PathBuf::from("/mock/repo/existing"),
        branch: None,
        commit: "abc123".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    let config = WorktreeConfig {
        name: "existing".to_string(),
        path: PathBuf::from("/mock/repo/existing2"),
        branch: None,
        create_branch: false,
        base_branch: None,
    };

    assert!(mock.create_worktree(&config).is_err());

    // Try to remove non-existent worktree
    assert!(mock.remove_worktree("non-existent").is_err());

    // Try to create duplicate branch
    assert!(mock.create_branch("main", None).is_err());

    Ok(())
}

#[test]
fn test_mock_tags_operations() -> Result<()> {
    let mock = MockGitInterface::new();

    // Add some tags
    mock.add_tag(TagInfo {
        name: "v1.0.0".to_string(),
        commit: "tag123".to_string(),
        message: Some("First release".to_string()),
        is_annotated: true,
    })?;

    mock.add_tag(TagInfo {
        name: "v1.0.1".to_string(),
        commit: "tag456".to_string(),
        message: None,
        is_annotated: false,
    })?;

    let tags = mock.list_tags()?;
    assert_eq!(tags.len(), 2);

    // Verify tag properties
    let v1_0_0 = tags.iter().find(|t| t.name == "v1.0.0").unwrap();
    assert!(v1_0_0.is_annotated);
    assert_eq!(v1_0_0.message, Some("First release".to_string()));

    let v1_0_1 = tags.iter().find(|t| t.name == "v1.0.1").unwrap();
    assert!(!v1_0_1.is_annotated);
    assert_eq!(v1_0_1.message, None);

    Ok(())
}

#[test]
fn test_mock_main_worktree() -> Result<()> {
    let mock = MockGitInterface::new();

    // Initially no main worktree
    assert!(mock.get_main_worktree()?.is_none());

    // Add main worktree
    mock.add_worktree(WorktreeInfo {
        name: "main".to_string(),
        path: PathBuf::from("/mock/repo"),
        branch: Some("main".to_string()),
        commit: "main123".to_string(),
        is_bare: false,
        is_main: true,
    })?;

    // Add another worktree
    mock.add_worktree(WorktreeInfo {
        name: "feature".to_string(),
        path: PathBuf::from("/mock/repo/feature"),
        branch: Some("feature".to_string()),
        commit: "feat123".to_string(),
        is_bare: false,
        is_main: false,
    })?;

    // Get main worktree
    let main = mock.get_main_worktree()?;
    assert!(main.is_some());
    assert_eq!(main.unwrap().name, "main");

    // Has worktrees should be true
    assert!(mock.has_worktrees()?);

    Ok(())
}

#[test]
fn test_mock_bare_repository() -> Result<()> {
    let (worktrees, branches, tags, mut repo_info) =
        GitScenarioBuilder::new().with_bare_repository(true).build();

    repo_info.is_bare = true;
    let mock = MockGitInterface::with_scenario(worktrees, branches, tags, repo_info);

    let info = mock.get_repository_info()?;
    assert!(info.is_bare);

    Ok(())
}

#[test]
fn test_mock_detached_head() -> Result<()> {
    let mock = MockGitInterface::new();

    // Set current branch to None (detached HEAD)
    mock.set_current_branch(None)?;

    let current = mock.get_current_branch()?;
    assert_eq!(current, None);

    Ok(())
}
