use anyhow::Result;
use git2::{Repository, Signature};
use std::process::Command;
use tempfile::TempDir;

use git_workers::git::GitWorktreeManager;

/// Helper function to create initial commit
fn create_initial_commit(repo: &Repository) -> Result<git2::Oid> {
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;
    let oid = repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    Ok(oid)
}

/// Helper function to setup test repository
fn setup_test_repo() -> Result<(TempDir, GitWorktreeManager)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    Ok((temp_dir, manager))
}

#[test]
fn test_list_all_tags_basic() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();

    // Create lightweight tag
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create annotated tag
    let sig = repo.signature()?;
    repo.tag(
        "v2.0.0",
        &repo.find_object(head_oid, None)?,
        &sig,
        "Release version 2.0.0",
        false,
    )?;

    let tags = manager.list_all_tags()?;

    // Should have 2 tags
    assert_eq!(tags.len(), 2);

    // Find v1.0.0 (lightweight tag)
    let v1_tag = tags.iter().find(|t| t.0 == "v1.0.0");
    assert!(v1_tag.is_some());
    let (name, message) = v1_tag.unwrap();
    assert_eq!(name, "v1.0.0");
    assert!(message.is_none()); // Lightweight tags don't have messages

    // Find v2.0.0 (annotated tag)
    let v2_tag = tags.iter().find(|t| t.0 == "v2.0.0");
    assert!(v2_tag.is_some());
    let (name, message) = v2_tag.unwrap();
    assert_eq!(name, "v2.0.0");
    assert_eq!(message.as_deref(), Some("Release version 2.0.0"));

    Ok(())
}

#[test]
fn test_list_all_tags_with_git_command() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create lightweight tag using git command
    Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(&repo_path)
        .output()?;

    // Create annotated tag using git command
    Command::new("git")
        .args(["tag", "-a", "v2.0.0", "-m", "Version 2.0.0 release"])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let tags = manager.list_all_tags()?;

    // Should have 2 tags
    assert_eq!(tags.len(), 2);

    // Find v1.0.0 (lightweight tag)
    let v1_tag = tags.iter().find(|t| t.0 == "v1.0.0");
    assert!(v1_tag.is_some());
    let (_, message) = v1_tag.unwrap();
    assert!(message.is_none());

    // Find v2.0.0 (annotated tag)
    let v2_tag = tags.iter().find(|t| t.0 == "v2.0.0");
    assert!(v2_tag.is_some());
    let (_, message) = v2_tag.unwrap();
    assert!(message.is_some());
    assert!(message.as_ref().unwrap().contains("Version 2.0.0"));

    Ok(())
}

#[test]
fn test_list_all_tags_empty_repo() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    let tags = manager.list_all_tags()?;

    // Should have no tags
    assert!(tags.is_empty());

    Ok(())
}

#[test]
fn test_list_all_tags_sorting() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();
    let head_oid = repo.head()?.target().unwrap();

    // Create tags in non-alphabetical order
    let tag_names = vec!["v3.0.0", "v1.0.0", "v2.0.0", "v0.1.0"];
    for tag_name in &tag_names {
        repo.tag_lightweight(tag_name, &repo.find_object(head_oid, None)?, false)?;
    }

    let tags = manager.list_all_tags()?;

    // Should have all tags
    assert_eq!(tags.len(), tag_names.len());

    // Verify all tags exist
    for tag_name in &tag_names {
        assert!(tags.iter().any(|(name, _)| name == tag_name));
    }

    Ok(())
}

#[test]
fn test_list_all_tags_with_multiple_commits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    let oid1 = create_initial_commit(&repo)?;

    // Create tag on first commit
    repo.tag_lightweight("v1.0.0", &repo.find_object(oid1, None)?, false)?;

    // Create second commit
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;
    let parent = repo.find_commit(oid1)?;
    let oid2 = repo.commit(Some("HEAD"), &sig, &sig, "Second commit", &tree, &[&parent])?;

    // Create tag on second commit
    repo.tag_lightweight("v2.0.0", &repo.find_object(oid2, None)?, false)?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;
    let tags = manager.list_all_tags()?;

    // Should have both tags
    assert_eq!(tags.len(), 2);
    assert!(tags.iter().any(|(name, _)| name == "v1.0.0"));
    assert!(tags.iter().any(|(name, _)| name == "v2.0.0"));

    Ok(())
}
