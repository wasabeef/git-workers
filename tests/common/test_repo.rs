//! Test repository setup utilities

use anyhow::Result;
use git2::{Repository, Signature};
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A test repository with helper methods
pub struct TestRepo {
    path: PathBuf,
    repo: Repository,
}

impl TestRepo {
    /// Create a new test repository
    pub fn new(temp_dir: &TempDir) -> Result<Self> {
        let path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&path)?;

        let repo = Repository::init(&path)?;

        // Create initial commit
        let sig = Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write()?;
            index.write_tree()?
        };

        let tree = repo.find_tree(tree_id)?;
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

        // Drop tree before moving repo
        drop(tree);

        Ok(Self { path, repo })
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a GitWorktreeManager for this test repository
    pub fn manager(&self) -> Result<GitWorktreeManager> {
        GitWorktreeManager::new_from_path(&self.path)
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head = self.repo.head()?.target().unwrap();
        let commit = self.repo.find_commit(head)?;
        self.repo.branch(name, &commit, false)?;
        Ok(())
    }

    /// Create a new commit
    #[allow(dead_code)]
    pub fn create_commit(&self, message: &str) -> Result<()> {
        let sig = Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = self.repo.index()?;
            index.write()?;
            index.write_tree()?
        };

        let tree = self.repo.find_tree(tree_id)?;
        let parent = self.repo.head()?.target().unwrap();
        let parent_commit = self.repo.find_commit(parent)?;

        self.repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit])?;

        Ok(())
    }
}
