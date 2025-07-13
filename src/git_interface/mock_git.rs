use super::*;
use anyhow::anyhow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
/// Mock implementation of GitInterface for testing
pub struct MockGitInterface {
    state: Arc<Mutex<GitState>>,
    expectations: Arc<Mutex<Vec<Expectation>>>,
}

#[derive(Debug, Clone)]
struct GitState {
    worktrees: HashMap<String, WorktreeInfo>,
    branches: Vec<BranchInfo>,
    tags: Vec<TagInfo>,
    current_branch: Option<String>,
    repository_path: PathBuf,
    is_bare: bool,
    branch_worktree_map: HashMap<String, String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expectation {
    CreateWorktree {
        name: String,
        branch: Option<String>,
    },
    RemoveWorktree {
        name: String,
    },
    CreateBranch {
        name: String,
        base: Option<String>,
    },
    DeleteBranch {
        name: String,
    },
    ListWorktrees,
    ListBranches,
}

impl Default for MockGitInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl MockGitInterface {
    /// Create a new mock with default state
    pub fn new() -> Self {
        let state = GitState {
            worktrees: HashMap::new(),
            branches: vec![BranchInfo {
                name: "main".to_string(),
                is_remote: false,
                upstream: Some("origin/main".to_string()),
                commit: "abc123".to_string(),
            }],
            tags: Vec::new(),
            current_branch: Some("main".to_string()),
            repository_path: PathBuf::from("/mock/repo"),
            is_bare: false,
            branch_worktree_map: HashMap::new(),
        };

        Self {
            state: Arc::new(Mutex::new(state)),
            expectations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create with a specific scenario
    pub fn with_scenario(
        worktrees: Vec<WorktreeInfo>,
        branches: Vec<BranchInfo>,
        tags: Vec<TagInfo>,
        repo_info: RepositoryInfo,
    ) -> Self {
        let mut branch_worktree_map = HashMap::new();
        for worktree in &worktrees {
            if let Some(branch) = &worktree.branch {
                branch_worktree_map.insert(branch.clone(), worktree.name.clone());
            }
        }

        let state = GitState {
            worktrees: worktrees.into_iter().map(|w| (w.name.clone(), w)).collect(),
            branches,
            tags,
            current_branch: repo_info.current_branch,
            repository_path: repo_info.path,
            is_bare: repo_info.is_bare,
            branch_worktree_map,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
            expectations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a worktree to the mock state
    pub fn add_worktree(&self, worktree: WorktreeInfo) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        if let Some(branch) = &worktree.branch {
            state
                .branch_worktree_map
                .insert(branch.clone(), worktree.name.clone());
        }
        state.worktrees.insert(worktree.name.clone(), worktree);
        Ok(())
    }

    /// Add a branch to the mock state
    pub fn add_branch(&self, branch: BranchInfo) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.branches.push(branch);
        Ok(())
    }

    /// Add a tag to the mock state
    pub fn add_tag(&self, tag: TagInfo) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.tags.push(tag);
        Ok(())
    }

    /// Set current branch
    pub fn set_current_branch(&self, branch: Option<String>) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.current_branch = branch;
        Ok(())
    }

    /// Expect a specific operation to be called
    pub fn expect_operation(&self, expectation: Expectation) {
        let mut expectations = self.expectations.lock().unwrap();
        expectations.push(expectation);
    }

    /// Verify all expectations were met
    pub fn verify_expectations(&self) -> Result<()> {
        let expectations = self.expectations.lock().unwrap();
        if !expectations.is_empty() {
            return Err(anyhow!("Unmet expectations: {:?}", expectations));
        }
        Ok(())
    }

    /// Clear all expectations
    pub fn clear_expectations(&self) {
        let mut expectations = self.expectations.lock().unwrap();
        expectations.clear();
    }

    fn check_expectation(&self, expectation: &Expectation) -> Result<()> {
        let mut expectations = self.expectations.lock().unwrap();
        if let Some(pos) = expectations
            .iter()
            .position(|e| std::mem::discriminant(e) == std::mem::discriminant(expectation))
        {
            expectations.remove(pos);
            Ok(())
        } else {
            Err(anyhow!("Unexpected operation: {:?}", expectation))
        }
    }
}

impl GitInterface for MockGitInterface {
    fn get_repository_info(&self) -> Result<RepositoryInfo> {
        let state = self.state.lock().unwrap();
        Ok(RepositoryInfo {
            path: state.repository_path.clone(),
            is_bare: state.is_bare,
            current_branch: state.current_branch.clone(),
            remote_url: Some("https://github.com/mock/repo.git".to_string()),
        })
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        self.check_expectation(&Expectation::ListWorktrees).ok();
        let state = self.state.lock().unwrap();
        Ok(state.worktrees.values().cloned().collect())
    }

    fn create_worktree(&self, config: &WorktreeConfig) -> Result<WorktreeInfo> {
        self.check_expectation(&Expectation::CreateWorktree {
            name: config.name.clone(),
            branch: config.branch.clone(),
        })
        .ok();

        let mut state = self.state.lock().unwrap();

        // Check if worktree already exists
        if state.worktrees.contains_key(&config.name) {
            return Err(anyhow!("Worktree '{}' already exists", config.name));
        }

        // Create branch if needed
        if config.create_branch {
            let branch_name = config
                .branch
                .as_ref()
                .ok_or_else(|| anyhow!("Branch name required"))?;

            if state.branches.iter().any(|b| b.name == *branch_name) {
                return Err(anyhow!("Branch '{}' already exists", branch_name));
            }

            state.branches.push(BranchInfo {
                name: branch_name.clone(),
                is_remote: false,
                upstream: None,
                commit: "new123".to_string(),
            });
        }

        let worktree = WorktreeInfo {
            name: config.name.clone(),
            path: config.path.clone(),
            branch: config.branch.clone(),
            commit: "new123".to_string(),
            is_bare: false,
            is_main: false,
        };

        if let Some(branch) = &worktree.branch {
            state
                .branch_worktree_map
                .insert(branch.clone(), worktree.name.clone());
        }

        state
            .worktrees
            .insert(config.name.clone(), worktree.clone());
        Ok(worktree)
    }

    fn remove_worktree(&self, name: &str) -> Result<()> {
        self.check_expectation(&Expectation::RemoveWorktree {
            name: name.to_string(),
        })
        .ok();

        let mut state = self.state.lock().unwrap();

        let worktree = state
            .worktrees
            .remove(name)
            .ok_or_else(|| anyhow!("Worktree '{}' not found", name))?;

        // Remove from branch map
        if let Some(branch) = &worktree.branch {
            state.branch_worktree_map.remove(branch);
        }

        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        self.check_expectation(&Expectation::ListBranches).ok();
        let state = self.state.lock().unwrap();
        Ok(state.branches.clone())
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let state = self.state.lock().unwrap();
        Ok(state.tags.clone())
    }

    fn get_current_branch(&self) -> Result<Option<String>> {
        let state = self.state.lock().unwrap();
        Ok(state.current_branch.clone())
    }

    fn branch_exists(&self, name: &str) -> Result<bool> {
        let state = self.state.lock().unwrap();
        Ok(state.branches.iter().any(|b| b.name == name))
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        self.check_expectation(&Expectation::CreateBranch {
            name: name.to_string(),
            base: base.map(|s| s.to_string()),
        })
        .ok();

        let mut state = self.state.lock().unwrap();

        if state.branches.iter().any(|b| b.name == name) {
            return Err(anyhow!("Branch '{}' already exists", name));
        }

        state.branches.push(BranchInfo {
            name: name.to_string(),
            is_remote: false,
            upstream: None,
            commit: "new456".to_string(),
        });

        Ok(())
    }

    fn delete_branch(&self, name: &str, force: bool) -> Result<()> {
        self.check_expectation(&Expectation::DeleteBranch {
            name: name.to_string(),
        })
        .ok();

        let mut state = self.state.lock().unwrap();

        // Check if branch is in use
        if !force && state.branch_worktree_map.contains_key(name) {
            return Err(anyhow!("Branch '{}' is checked out in a worktree", name));
        }

        state.branches.retain(|b| b.name != name);
        Ok(())
    }

    fn get_worktree(&self, name: &str) -> Result<Option<WorktreeInfo>> {
        let state = self.state.lock().unwrap();
        Ok(state.worktrees.get(name).cloned())
    }

    fn rename_worktree(&self, old_name: &str, new_name: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        let worktree = state
            .worktrees
            .remove(old_name)
            .ok_or_else(|| anyhow!("Worktree '{}' not found", old_name))?;

        if state.worktrees.contains_key(new_name) {
            return Err(anyhow!("Worktree '{}' already exists", new_name));
        }

        let mut new_worktree = worktree;
        new_worktree.name = new_name.to_string();

        // Update branch map
        if let Some(branch) = &new_worktree.branch {
            state
                .branch_worktree_map
                .insert(branch.clone(), new_name.to_string());
        }

        state.worktrees.insert(new_name.to_string(), new_worktree);
        Ok(())
    }

    fn prune_worktrees(&self) -> Result<()> {
        // Mock implementation doesn't need to do anything
        Ok(())
    }

    fn get_main_worktree(&self) -> Result<Option<WorktreeInfo>> {
        let state = self.state.lock().unwrap();
        Ok(state.worktrees.values().find(|w| w.is_main).cloned())
    }

    fn has_worktrees(&self) -> Result<bool> {
        let state = self.state.lock().unwrap();
        Ok(!state.worktrees.is_empty())
    }

    fn get_branch_worktree_map(&self) -> Result<HashMap<String, String>> {
        let state = self.state.lock().unwrap();
        Ok(state.branch_worktree_map.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_interface::test_helpers::GitScenarioBuilder;

    #[test]
    fn test_mock_create_worktree() -> Result<()> {
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
        assert_eq!(worktree.branch, Some("feature-branch".to_string()));

        // Verify worktree was added
        let worktrees = mock.list_worktrees()?;
        assert!(worktrees.iter().any(|w| w.name == "feature"));

        // Verify branch was created
        let branches = mock.list_branches()?;
        assert!(branches.iter().any(|b| b.name == "feature-branch"));

        Ok(())
    }

    #[test]
    fn test_mock_with_scenario() -> Result<()> {
        let (worktrees, branches, tags, repo_info) = GitScenarioBuilder::new()
            .with_worktree("main", "/repo", Some("main"))
            .with_worktree("feature", "/repo/feature", Some("feature-branch"))
            .with_branch("main", false)
            .with_branch("feature-branch", false)
            .with_branch("develop", false)
            .with_tag("v1.0.0", Some("Release 1.0.0"))
            .build();

        let mock = MockGitInterface::with_scenario(worktrees, branches, tags, repo_info);

        let worktrees = mock.list_worktrees()?;
        assert_eq!(worktrees.len(), 2);

        let branches = mock.list_branches()?;
        assert_eq!(branches.len(), 3);

        let tags = mock.list_tags()?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "v1.0.0");

        Ok(())
    }

    #[test]
    fn test_mock_expectations() -> Result<()> {
        let mock = MockGitInterface::new();

        // Set expectations
        mock.expect_operation(Expectation::CreateWorktree {
            name: "test".to_string(),
            branch: Some("test-branch".to_string()),
        });

        // This should satisfy the expectation
        let config = WorktreeConfig {
            name: "test".to_string(),
            path: PathBuf::from("/mock/repo/test"),
            branch: Some("test-branch".to_string()),
            create_branch: false,
            base_branch: None,
        };

        mock.create_worktree(&config)?;

        // Verify all expectations were met
        mock.verify_expectations()?;

        Ok(())
    }
}
