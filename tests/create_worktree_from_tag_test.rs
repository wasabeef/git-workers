use anyhow::Result;
use git2::Repository;
use std::fs;
use tempfile::TempDir;

fn setup_test_repo(temp_dir: &TempDir) -> Result<(std::path::PathBuf, git2::Oid)> {
    let repo_path = temp_dir.path().join("test-repo");
    let repo = Repository::init(&repo_path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        fs::write(repo_path.join("README.md"), "# Test Repo")?;
        index.add_path(std::path::Path::new("README.md"))?;
        index.write()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    Ok((repo_path, commit_oid))
}

#[test]
fn test_list_all_tags() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo(&temp_dir)?;

    // Create test tags
    let repo = Repository::open(&repo_path)?;

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

    // List tags
    let manager = git_workers::git::GitWorktreeManager::new_from_path(&repo_path)?;
    let tags = manager.list_all_tags()?;

    // Verify results
    assert_eq!(tags.len(), 2);

    // Tags should be sorted in reverse order (v2.0.0 first)
    assert_eq!(tags[0].0, "v2.0.0");
    assert_eq!(tags[0].1, Some("Release version 2.0.0".to_string()));

    assert_eq!(tags[1].0, "v1.0.0");
    assert_eq!(tags[1].1, None); // Lightweight tag has no message

    Ok(())
}

#[test]
fn test_create_worktree_from_tag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo(&temp_dir)?;

    // Create a tag
    let repo = Repository::open(&repo_path)?;
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create worktree from tag with new branch
    let manager = git_workers::git::GitWorktreeManager::new_from_path(&repo_path)?;
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

    // Cleanup
    fs::remove_dir_all(&worktree_path)?;

    Ok(())
}

#[test]
fn test_create_worktree_from_tag_detached() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let (repo_path, _initial_commit) = setup_test_repo(&temp_dir)?;

    // Create a tag
    let repo = Repository::open(&repo_path)?;
    let head_oid = repo.head()?.target().unwrap();
    repo.tag_lightweight("v1.0.0", &repo.find_object(head_oid, None)?, false)?;

    // Create worktree from tag without new branch (detached HEAD)
    let manager = git_workers::git::GitWorktreeManager::new_from_path(&repo_path)?;
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
