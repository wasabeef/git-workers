//! Git operations abstraction layer
//!
//! This module provides an abstraction over Git operations,
//! allowing for testable code by separating business logic from Git dependencies.

#![allow(dead_code)]
#![allow(clippy::wrong_self_convention)]

use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::git::WorktreeInfo;

/// Branch information
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is a remote branch
    pub is_remote: bool,
}

/// Tag information
#[derive(Debug, Clone)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Optional tag message (for annotated tags)
    pub message: Option<String>,
}

/// Trait for read-only Git operations
///
/// This trait abstracts Git read operations, making the code testable
/// by allowing mock implementations for testing and real implementations for production.
pub trait GitReadOperations {
    /// List all worktrees in the repository
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// List all branches (local and remote)
    fn list_branches(&self) -> Result<Vec<BranchInfo>>;

    /// List all tags with optional messages
    fn list_tags(&self) -> Result<Vec<TagInfo>>;

    /// Get the current branch name
    fn get_current_branch(&self) -> Result<String>;

    /// Get repository information
    fn get_repository_info(&self) -> Result<String>;

    /// Check if the repository is bare
    fn is_bare_repository(&self) -> Result<bool>;

    /// Get the repository root path
    fn get_repository_root(&self) -> Result<PathBuf>;

    /// Check if a worktree exists
    fn worktree_exists(&self, name: &str) -> Result<bool>;

    /// Get branch to worktree mapping
    fn get_branch_worktree_map(&self) -> Result<std::collections::HashMap<String, String>>;
}

/// Production implementation using GitWorktreeManager
pub struct RealGitOperations {
    manager: crate::git::GitWorktreeManager,
}

impl RealGitOperations {
    /// Create a new RealGitOperations instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: crate::git::GitWorktreeManager::new()?,
        })
    }
}

impl GitReadOperations for RealGitOperations {
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        self.manager.list_worktrees()
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let (local_branches, remote_branches) = self.manager.list_all_branches()?;
        let mut branches = Vec::new();

        // Add local branches
        for name in local_branches {
            branches.push(BranchInfo {
                name,
                is_remote: false,
            });
        }

        // Add remote branches
        for name in remote_branches {
            branches.push(BranchInfo {
                name,
                is_remote: true,
            });
        }

        Ok(branches)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let tags = self.manager.list_all_tags()?;
        Ok(tags
            .into_iter()
            .map(|(name, message)| TagInfo { name, message })
            .collect())
    }

    fn get_current_branch(&self) -> Result<String> {
        let head = self.manager.repo.head()?;
        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    fn get_repository_info(&self) -> Result<String> {
        Ok(crate::repository_info::get_repository_info())
    }

    fn is_bare_repository(&self) -> Result<bool> {
        Ok(self.manager.repo.is_bare())
    }

    fn get_repository_root(&self) -> Result<PathBuf> {
        if self.manager.repo.is_bare() {
            Ok(self.manager.repo.path().to_path_buf())
        } else {
            Ok(self
                .manager
                .repo
                .workdir()
                .ok_or_else(|| anyhow!("Repository has no working directory"))?
                .to_path_buf())
        }
    }

    fn worktree_exists(&self, name: &str) -> Result<bool> {
        let worktrees = self.list_worktrees()?;
        Ok(worktrees.iter().any(|w| w.name == name))
    }

    fn get_branch_worktree_map(&self) -> Result<std::collections::HashMap<String, String>> {
        self.manager.get_branch_worktree_map()
    }
}

pub mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Mock implementation for testing
    pub struct MockGitOperations {
        worktrees: RefCell<Vec<WorktreeInfo>>,
        branches: RefCell<Vec<BranchInfo>>,
        tags: RefCell<Vec<TagInfo>>,
        current_branch: RefCell<String>,
        is_bare: bool,
        repository_root: PathBuf,
        branch_worktree_map: RefCell<HashMap<String, String>>,
    }

    impl Default for MockGitOperations {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockGitOperations {
        /// Create a new MockGitOperations instance
        pub fn new() -> Self {
            Self {
                worktrees: RefCell::new(Vec::new()),
                branches: RefCell::new(Vec::new()),
                tags: RefCell::new(Vec::new()),
                current_branch: RefCell::new("main".to_string()),
                is_bare: false,
                repository_root: PathBuf::from("/mock/repo"),
                branch_worktree_map: RefCell::new(HashMap::new()),
            }
        }

        /// Add a worktree to the mock
        pub fn with_worktree(self, name: &str, path: &str, branch: Option<&str>) -> Self {
            let info = WorktreeInfo {
                name: name.to_string(),
                path: PathBuf::from(path),
                branch: branch.unwrap_or("HEAD").to_string(),
                is_locked: false,
                is_current: false,
                has_changes: false,
                last_commit: None,
                ahead_behind: None,
            };
            self.worktrees.borrow_mut().push(info);
            if let Some(branch) = branch {
                self.branch_worktree_map
                    .borrow_mut()
                    .insert(branch.to_string(), name.to_string());
            }
            self
        }

        /// Add a branch to the mock
        pub fn with_branch(self, name: &str, is_remote: bool) -> Self {
            let info = BranchInfo {
                name: name.to_string(),
                is_remote,
            };
            self.branches.borrow_mut().push(info);
            self
        }

        /// Add a tag to the mock
        pub fn with_tag(self, name: &str, message: Option<&str>) -> Self {
            let info = TagInfo {
                name: name.to_string(),
                message: message.map(|m| m.to_string()),
            };
            self.tags.borrow_mut().push(info);
            self
        }

        /// Set the current branch
        pub fn with_current_branch(self, branch: &str) -> Self {
            *self.current_branch.borrow_mut() = branch.to_string();
            self
        }

        /// Set whether the repository is bare
        pub fn as_bare(mut self) -> Self {
            self.is_bare = true;
            self
        }

        /// Set the repository root
        pub fn with_repository_root(mut self, path: &str) -> Self {
            self.repository_root = PathBuf::from(path);
            self
        }

        /// Mark a worktree as current
        pub fn with_current_worktree(self, name: &str) -> Self {
            let mut worktrees = self.worktrees.borrow_mut();
            for worktree in worktrees.iter_mut() {
                worktree.is_current = worktree.name == name;
            }
            drop(worktrees);
            self
        }

        /// Mark a worktree as having changes
        pub fn with_worktree_changes(self, name: &str) -> Self {
            let mut worktrees = self.worktrees.borrow_mut();
            for worktree in worktrees.iter_mut() {
                if worktree.name == name {
                    worktree.has_changes = true;
                }
            }
            drop(worktrees);
            self
        }
    }

    impl GitReadOperations for MockGitOperations {
        fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
            Ok(self.worktrees.borrow().clone())
        }

        fn list_branches(&self) -> Result<Vec<BranchInfo>> {
            Ok(self.branches.borrow().clone())
        }

        fn list_tags(&self) -> Result<Vec<TagInfo>> {
            Ok(self.tags.borrow().clone())
        }

        fn get_current_branch(&self) -> Result<String> {
            Ok(self.current_branch.borrow().clone())
        }

        fn get_repository_info(&self) -> Result<String> {
            if self.is_bare {
                Ok(format!("{} (bare)", self.repository_root.display()))
            } else {
                Ok(format!(
                    "{} on {}",
                    self.repository_root.display(),
                    self.current_branch.borrow()
                ))
            }
        }

        fn is_bare_repository(&self) -> Result<bool> {
            Ok(self.is_bare)
        }

        fn get_repository_root(&self) -> Result<PathBuf> {
            Ok(self.repository_root.clone())
        }

        fn worktree_exists(&self, name: &str) -> Result<bool> {
            Ok(self.worktrees.borrow().iter().any(|w| w.name == name))
        }

        fn get_branch_worktree_map(&self) -> Result<HashMap<String, String>> {
            Ok(self.branch_worktree_map.borrow().clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockGitOperations;
    use super::*;

    #[test]
    fn test_mock_git_operations_creation() {
        let mock = MockGitOperations::new()
            .with_worktree("main", "/repo/main", Some("main"))
            .with_worktree("feature", "/repo/feature", Some("feature/new"))
            .with_branch("main", false)
            .with_branch("feature/new", false)
            .with_branch("origin/main", true)
            .with_tag("v1.0.0", Some("Release 1.0.0"))
            .with_current_branch("main")
            .with_current_worktree("main");

        // Test worktrees
        let worktrees = mock.list_worktrees().unwrap();
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].name, "main");
        assert!(worktrees[0].is_current);

        // Test branches
        let branches = mock.list_branches().unwrap();
        assert_eq!(branches.len(), 3);

        // Test tags
        let tags = mock.list_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "v1.0.0");

        // Test current branch
        assert_eq!(mock.get_current_branch().unwrap(), "main");

        // Test worktree exists
        assert!(mock.worktree_exists("main").unwrap());
        assert!(mock.worktree_exists("feature").unwrap());
        assert!(!mock.worktree_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_bare_repository() {
        let mock = MockGitOperations::new()
            .as_bare()
            .with_repository_root("/bare/repo");

        assert!(mock.is_bare_repository().unwrap());
        assert_eq!(
            mock.get_repository_root().unwrap(),
            PathBuf::from("/bare/repo")
        );
        assert!(mock.get_repository_info().unwrap().contains("(bare)"));
    }

    #[test]
    fn test_branch_worktree_mapping() {
        let mock = MockGitOperations::new()
            .with_worktree("main", "/repo/main", Some("main"))
            .with_worktree("feature", "/repo/feature", Some("feature/new"));

        let map = mock.get_branch_worktree_map().unwrap();
        assert_eq!(map.get("main").unwrap(), "main");
        assert_eq!(map.get("feature/new").unwrap(), "feature");
    }

    #[test]
    fn test_worktree_with_changes() {
        let mock = MockGitOperations::new()
            .with_worktree("main", "/repo/main", Some("main"))
            .with_worktree("feature", "/repo/feature", Some("feature/new"))
            .with_worktree_changes("feature");

        let worktrees = mock.list_worktrees().unwrap();
        assert!(!worktrees[0].has_changes); // main
        assert!(worktrees[1].has_changes); // feature
    }

    #[test]
    fn test_real_git_operations_creation() {
        // This test will only work in a git repository
        if std::env::var("CI").is_ok() {
            return; // Skip in CI environment
        }

        match RealGitOperations::new() {
            Ok(_) => {
                // Successfully created in a git repository
            }
            Err(_) => {
                // Not in a git repository, which is fine for this test
            }
        }
    }
}
