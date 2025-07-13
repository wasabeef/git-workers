use super::*;
use anyhow::{anyhow, Context};
use git2::{BranchType, Repository};
use std::collections::HashMap;
use std::process::Command;

/// Implementation of GitInterface using real git operations
pub struct RealGitInterface {
    repo: Repository,
    repo_path: PathBuf,
}

impl RealGitInterface {
    /// Create a new RealGitInterface from the current directory
    pub fn new() -> Result<Self> {
        let repo = Repository::open_from_env().context("Failed to open git repository")?;
        let repo_path = repo
            .path()
            .parent()
            .ok_or_else(|| anyhow!("Failed to get repository path"))?
            .to_path_buf();

        Ok(Self { repo, repo_path })
    }

    /// Create from a specific path
    pub fn from_path(path: &Path) -> Result<Self> {
        let repo = Repository::open(path).context("Failed to open git repository")?;
        let repo_path = repo
            .path()
            .parent()
            .ok_or_else(|| anyhow!("Failed to get repository path"))?
            .to_path_buf();

        Ok(Self { repo, repo_path })
    }

    /// Execute git command and return output
    fn execute_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse worktree list output
    fn parse_worktree_list(&self, output: &str) -> Result<Vec<WorktreeInfo>> {
        let mut worktrees = Vec::new();
        let mut current_worktree = None;
        let mut path: Option<PathBuf> = None;
        let mut branch = None;
        let mut commit = None;
        let mut is_bare = false;

        for line in output.lines() {
            if let Some(stripped) = line.strip_prefix("worktree ") {
                if let Some(_wt_path) = current_worktree.take() {
                    if let (Some(p), Some(c)) = (path.take(), commit.take()) {
                        let name = p
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        worktrees.push(WorktreeInfo {
                            name: name.clone(),
                            path: p,
                            branch: branch.take(),
                            commit: c,
                            is_bare,
                            is_main: name == "main" || name == "master",
                        });
                    }
                }
                current_worktree = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("HEAD ") {
                commit = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("branch ") {
                branch = Some(stripped.to_string());
            } else if line == "bare" {
                is_bare = true;
            }

            if current_worktree.is_some() && path.is_none() {
                path = current_worktree.as_ref().map(PathBuf::from);
            }
        }

        // Handle the last worktree
        if current_worktree.is_some() {
            if let (Some(p), Some(c)) = (path, commit) {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                worktrees.push(WorktreeInfo {
                    name: name.clone(),
                    path: p,
                    branch,
                    commit: c,
                    is_bare,
                    is_main: name == "main" || name == "master",
                });
            }
        }

        Ok(worktrees)
    }
}

impl GitInterface for RealGitInterface {
    fn get_repository_info(&self) -> Result<RepositoryInfo> {
        let is_bare = self.repo.is_bare();
        let current_branch = self.get_current_branch()?;

        let remote_url = self
            .repo
            .find_remote("origin")
            .ok()
            .and_then(|remote| remote.url().map(|u| u.to_string()));

        Ok(RepositoryInfo {
            path: self.repo_path.clone(),
            is_bare,
            current_branch,
            remote_url,
        })
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let output = self.execute_git_command(&["worktree", "list", "--porcelain"])?;
        self.parse_worktree_list(&output)
    }

    fn create_worktree(&self, config: &WorktreeConfig) -> Result<WorktreeInfo> {
        let mut args = vec!["worktree", "add"];

        args.push(
            config
                .path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path"))?,
        );

        if config.create_branch {
            args.push("-b");
            args.push(
                config
                    .branch
                    .as_ref()
                    .ok_or_else(|| anyhow!("Branch name required for new branch"))?,
            );

            if let Some(base) = &config.base_branch {
                args.push(base);
            }
        } else if let Some(branch) = &config.branch {
            args.push(branch);
        }

        self.execute_git_command(&args)?;

        // Get the created worktree info
        let worktrees = self.list_worktrees()?;
        worktrees
            .into_iter()
            .find(|w| w.name == config.name)
            .ok_or_else(|| anyhow!("Failed to find created worktree"))
    }

    fn remove_worktree(&self, name: &str) -> Result<()> {
        // Find the worktree path
        let worktrees = self.list_worktrees()?;
        let worktree = worktrees
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| anyhow!("Worktree not found: {}", name))?;

        self.execute_git_command(&["worktree", "remove", worktree.path.to_str().unwrap()])?;
        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();

        // List local branches
        for branch in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch?;
            let name = branch.name()?.unwrap_or("unknown").to_string();
            let commit = branch
                .get()
                .target()
                .ok_or_else(|| anyhow!("No commit for branch"))?
                .to_string();

            branches.push(BranchInfo {
                name: name.clone(),
                is_remote: false,
                upstream: {
                    let upstream_branch = branch.upstream().ok();
                    upstream_branch.and_then(|u| u.name().ok().flatten().map(|s| s.to_string()))
                },
                commit,
            });
        }

        // List remote branches
        for branch in self.repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch?;
            let full_name = branch.name()?.unwrap_or("unknown");
            // Remove "origin/" prefix
            let name = full_name
                .strip_prefix("origin/")
                .unwrap_or(full_name)
                .to_string();
            let commit = branch
                .get()
                .target()
                .ok_or_else(|| anyhow!("No commit for branch"))?
                .to_string();

            branches.push(BranchInfo {
                name,
                is_remote: true,
                upstream: None,
                commit,
            });
        }

        Ok(branches)
    }

    fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let mut tags = Vec::new();

        self.repo.tag_foreach(|oid, name| {
            if let Some(tag_name) = name.strip_prefix(b"refs/tags/") {
                let tag_name = String::from_utf8_lossy(tag_name).to_string();
                let commit = oid.to_string();

                // Check if it's an annotated tag
                let (message, is_annotated) = if let Ok(tag_obj) = self.repo.find_tag(oid) {
                    (tag_obj.message().map(|s| s.to_string()), true)
                } else {
                    (None, false)
                };

                tags.push(TagInfo {
                    name: tag_name,
                    commit,
                    message,
                    is_annotated,
                });
            }
            true
        })?;

        Ok(tags)
    }

    fn get_current_branch(&self) -> Result<Option<String>> {
        if self.repo.head_detached()? {
            return Ok(None);
        }

        let head = self.repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(Some(name.to_string()))
        } else {
            Ok(None)
        }
    }

    fn branch_exists(&self, name: &str) -> Result<bool> {
        Ok(self.repo.find_branch(name, BranchType::Local).is_ok()
            || self
                .repo
                .find_branch(&format!("origin/{name}"), BranchType::Remote)
                .is_ok())
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        let target = if let Some(base_name) = base {
            let base_ref = self
                .repo
                .find_reference(&format!("refs/heads/{base_name}"))
                .or_else(|_| {
                    self.repo
                        .find_reference(&format!("refs/remotes/origin/{base_name}"))
                })
                .context("Base branch not found")?;
            self.repo.find_commit(base_ref.target().unwrap())?
        } else {
            self.repo.head()?.peel_to_commit()?
        };

        self.repo.branch(name, &target, false)?;
        Ok(())
    }

    fn delete_branch(&self, name: &str, _force: bool) -> Result<()> {
        let mut branch = self.repo.find_branch(name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    fn get_worktree(&self, name: &str) -> Result<Option<WorktreeInfo>> {
        let worktrees = self.list_worktrees()?;
        Ok(worktrees.into_iter().find(|w| w.name == name))
    }

    fn rename_worktree(&self, _old_name: &str, _new_name: &str) -> Result<()> {
        // Git doesn't have a native rename command, so we need to implement it manually
        // This would involve moving directories and updating git metadata
        Err(anyhow!(
            "Worktree rename not supported in real git interface yet"
        ))
    }

    fn prune_worktrees(&self) -> Result<()> {
        self.execute_git_command(&["worktree", "prune"])?;
        Ok(())
    }

    fn get_main_worktree(&self) -> Result<Option<WorktreeInfo>> {
        let worktrees = self.list_worktrees()?;
        Ok(worktrees.into_iter().find(|w| w.is_main))
    }

    fn has_worktrees(&self) -> Result<bool> {
        Ok(!self.list_worktrees()?.is_empty())
    }

    fn get_branch_worktree_map(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        for worktree in self.list_worktrees()? {
            if let Some(branch) = worktree.branch {
                map.insert(branch, worktree.name);
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, RealGitInterface)> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        // Create initial commit
        std::fs::write(repo_path.join("README.md"), "# Test Repo")?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()?;
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()?;

        let git_interface = RealGitInterface::from_path(repo_path)?;
        Ok((temp_dir, git_interface))
    }

    #[test]
    fn test_get_repository_info() -> Result<()> {
        let (_temp_dir, git) = setup_test_repo()?;
        let info = git.get_repository_info()?;

        assert!(!info.is_bare);
        assert!(info.current_branch.is_some());
        Ok(())
    }

    #[test]
    fn test_list_branches() -> Result<()> {
        let (_temp_dir, git) = setup_test_repo()?;
        let branches = git.list_branches()?;

        assert!(!branches.is_empty());
        assert!(branches.iter().any(|b| !b.is_remote));
        Ok(())
    }
}
