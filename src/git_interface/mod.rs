use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod mock_git;
pub mod real_git;

pub use mock_git::MockGitInterface;
pub use real_git::RealGitInterface;

/// Represents a git worktree with its associated metadata
#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_bare: bool,
    pub is_main: bool,
}

/// Configuration for creating a new worktree
#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub create_branch: bool,
    pub base_branch: Option<String>,
}

/// Information about a git branch
#[derive(Debug, Clone, PartialEq)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub commit: String,
}

/// Information about a git tag
#[derive(Debug, Clone, PartialEq)]
pub struct TagInfo {
    pub name: String,
    pub commit: String,
    pub message: Option<String>,
    pub is_annotated: bool,
}

/// Repository information
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub is_bare: bool,
    pub current_branch: Option<String>,
    pub remote_url: Option<String>,
}

/// Main trait for abstracting git operations
pub trait GitInterface: Send {
    /// Get repository information
    fn get_repository_info(&self) -> Result<RepositoryInfo>;

    /// List all worktrees in the repository
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// Create a new worktree with the given configuration
    fn create_worktree(&self, config: &WorktreeConfig) -> Result<WorktreeInfo>;

    /// Remove a worktree by name
    fn remove_worktree(&self, name: &str) -> Result<()>;

    /// List all branches in the repository
    fn list_branches(&self) -> Result<Vec<BranchInfo>>;

    /// List all tags in the repository
    fn list_tags(&self) -> Result<Vec<TagInfo>>;

    /// Get the current branch name
    fn get_current_branch(&self) -> Result<Option<String>>;

    /// Check if a branch exists
    fn branch_exists(&self, name: &str) -> Result<bool>;

    /// Create a new branch
    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()>;

    /// Delete a branch
    fn delete_branch(&self, name: &str, force: bool) -> Result<()>;

    /// Get worktree by name
    fn get_worktree(&self, name: &str) -> Result<Option<WorktreeInfo>>;

    /// Rename a worktree
    fn rename_worktree(&self, old_name: &str, new_name: &str) -> Result<()>;

    /// Prune worktrees (clean up stale entries)
    fn prune_worktrees(&self) -> Result<()>;

    /// Get the main worktree
    fn get_main_worktree(&self) -> Result<Option<WorktreeInfo>>;

    /// Check if repository has any worktrees
    fn has_worktrees(&self) -> Result<bool>;

    /// Get branch-to-worktree mapping
    fn get_branch_worktree_map(&self) -> Result<std::collections::HashMap<String, String>>;
}

/// Builder pattern for creating mock scenarios
pub mod test_helpers {
    use super::*;

    /// Builder for creating test scenarios
    pub struct GitScenarioBuilder {
        worktrees: Vec<WorktreeInfo>,
        branches: Vec<BranchInfo>,
        tags: Vec<TagInfo>,
        current_branch: Option<String>,
        repository_path: PathBuf,
        is_bare: bool,
    }

    impl Default for GitScenarioBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GitScenarioBuilder {
        pub fn new() -> Self {
            Self {
                worktrees: Vec::new(),
                branches: Vec::new(),
                tags: Vec::new(),
                current_branch: Some("main".to_string()),
                repository_path: PathBuf::from("/test/repo"),
                is_bare: false,
            }
        }

        pub fn with_worktree(mut self, name: &str, path: &str, branch: Option<&str>) -> Self {
            self.worktrees.push(WorktreeInfo {
                name: name.to_string(),
                path: PathBuf::from(path),
                branch: branch.map(|s| s.to_string()),
                commit: "abc123".to_string(),
                is_bare: false,
                is_main: name == "main",
            });
            self
        }

        pub fn with_branch(mut self, name: &str, is_remote: bool) -> Self {
            self.branches.push(BranchInfo {
                name: name.to_string(),
                is_remote,
                upstream: if is_remote {
                    None
                } else {
                    Some(format!("origin/{name}"))
                },
                commit: "def456".to_string(),
            });
            self
        }

        pub fn with_tag(mut self, name: &str, message: Option<&str>) -> Self {
            self.tags.push(TagInfo {
                name: name.to_string(),
                commit: "tag789".to_string(),
                message: message.map(|s| s.to_string()),
                is_annotated: message.is_some(),
            });
            self
        }

        pub fn with_current_branch(mut self, branch: &str) -> Self {
            self.current_branch = Some(branch.to_string());
            self
        }

        pub fn with_bare_repository(mut self, is_bare: bool) -> Self {
            self.is_bare = is_bare;
            self
        }

        pub fn build(
            self,
        ) -> (
            Vec<WorktreeInfo>,
            Vec<BranchInfo>,
            Vec<TagInfo>,
            RepositoryInfo,
        ) {
            let repo_info = RepositoryInfo {
                path: self.repository_path,
                is_bare: self.is_bare,
                current_branch: self.current_branch,
                remote_url: Some("https://github.com/test/repo.git".to_string()),
            };

            (self.worktrees, self.branches, self.tags, repo_info)
        }
    }
}
