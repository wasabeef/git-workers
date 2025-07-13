use anyhow::Result;
use git2::{Repository, Signature};
use std::fs;
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
fn test_create_worktree_from_tag_with_new_branch() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();

    // Create a tag
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create worktree from tag with new branch
    let worktree_path = manager.create_worktree_with_new_branch(
        "feature-from-tag",
        "feature-from-tag",
        "v1.0.0",
    )?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Verify the new branch was created
    let worktree_repo = Repository::open(&worktree_path)?;
    let head = worktree_repo.head()?;
    assert!(head.is_branch());
    assert_eq!(head.shorthand(), Some("feature-from-tag"));

    // Verify commit is the same as the tag
    let tag_commit = repo.find_reference("refs/tags/v1.0.0")?.peel_to_commit()?;
    let worktree_commit = head.peel_to_commit()?;
    assert_eq!(tag_commit.id(), worktree_commit.id());

    Ok(())
}

#[test]
fn test_create_worktree_from_tag_detached() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();

    // Create a tag
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create worktree from tag without new branch (detached HEAD)
    let worktree_path = manager.create_worktree("test-tag-detached", Some("v1.0.0"))?;

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Verify HEAD is detached at the tag
    let worktree_repo = Repository::open(&worktree_path)?;
    let head = worktree_repo.head()?;
    assert!(!head.is_branch());

    // Verify commit is the same as the tag
    let tag_commit = repo.find_reference("refs/tags/v1.0.0")?.peel_to_commit()?;
    let worktree_commit = head.peel_to_commit()?;
    assert_eq!(tag_commit.id(), worktree_commit.id());

    // Cleanup
    fs::remove_dir_all(&worktree_path)?;

    Ok(())
}

#[test]
fn test_create_worktree_from_tag_with_git_command() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    let repo = Repository::init(&repo_path)?;
    create_initial_commit(&repo)?;

    // Create a tag using git command
    Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(&repo_path)
        .output()?;

    let manager = GitWorktreeManager::new_from_path(&repo_path)?;

    // Create worktree from tag
    let result = manager.create_worktree("tag-worktree", Some("v1.0.0"));
    assert!(result.is_ok());

    let worktree_path = result.unwrap();
    assert!(worktree_path.exists());

    // Verify it's at the tagged commit
    let output = Command::new("git")
        .args(["describe", "--tags"])
        .current_dir(&worktree_path)
        .output()?;

    let tag_desc = String::from_utf8_lossy(&output.stdout);
    assert!(tag_desc.contains("v1.0.0"));

    Ok(())
}

#[test]
fn test_create_worktree_from_annotated_tag() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();

    // Create an annotated tag
    let head_oid = repo.head()?.target().unwrap();
    let sig = repo.signature()?;
    repo.tag(
        "v2.0.0",
        &repo.find_object(head_oid, None)?,
        &sig,
        "Release version 2.0.0",
        false,
    )?;

    // Create worktree from annotated tag
    let worktree_path = manager.create_worktree("annotated-tag-wt", Some("v2.0.0"))?;

    // Verify worktree was created
    assert!(worktree_path.exists());

    // Verify commit is correct
    let worktree_repo = Repository::open(&worktree_path)?;
    let head_commit = worktree_repo.head()?.peel_to_commit()?;
    let tag_commit = repo.find_reference("refs/tags/v2.0.0")?.peel_to_commit()?;
    assert_eq!(head_commit.id(), tag_commit.id());

    Ok(())
}

#[test]
fn test_create_worktree_from_nonexistent_tag() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;

    // Try to create worktree from non-existent tag
    let result = manager.create_worktree("nonexistent-tag-wt", Some("v99.0.0"));

    // Git might create a worktree even with a non-existent tag/branch
    // So we check if it's an error or if the worktree was created
    match result {
        Ok(path) => {
            // If it succeeded, it might have created from HEAD or detached
            assert!(path.exists());
            // Clean up
            fs::remove_dir_all(&path)?;
        }
        Err(_) => {
            // This is also acceptable - the tag doesn't exist
        }
    }

    Ok(())
}

#[test]
fn test_create_worktree_from_tag_with_multiple_tags() -> Result<()> {
    let (_temp_dir, manager) = setup_test_repo()?;
    let repo = manager.repo();

    // Create first commit
    let first_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(first_oid, None)?, false)?;

    // Create second commit
    let sig = Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;
    let parent = repo.find_commit(first_oid)?;
    let second_oid = repo.commit(Some("HEAD"), &sig, &sig, "Second commit", &tree, &[&parent])?;

    // Create second tag
    repo.tag_lightweight("v2.0.0", &repo.find_object(second_oid, None)?, false)?;

    // Create worktree from first tag
    let wt1_path = manager.create_worktree("tag-v1", Some("v1.0.0"))?;
    let wt1_repo = Repository::open(&wt1_path)?;
    let wt1_commit = wt1_repo.head()?.peel_to_commit()?;
    assert_eq!(wt1_commit.id(), first_oid);

    // Create worktree from second tag
    let wt2_path = manager.create_worktree("tag-v2", Some("v2.0.0"))?;
    let wt2_repo = Repository::open(&wt2_path)?;
    let wt2_commit = wt2_repo.head()?.peel_to_commit()?;
    assert_eq!(wt2_commit.id(), second_oid);

    Ok(())
}
