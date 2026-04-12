#[cfg(not(test))]
use crate::constants::MAIN_SUFFIX;
use crate::constants::UNKNOWN_VALUE;
use std::env;
#[cfg(not(test))]
use std::path::Path;
use std::process::Command;

#[cfg(not(test))]
fn paths_equal(lhs: &std::path::Path, rhs: &std::path::Path) -> bool {
    match (lhs.canonicalize(), rhs.canonicalize()) {
        (Ok(lhs), Ok(rhs)) => lhs == rhs,
        _ => lhs == rhs,
    }
}

#[cfg(not(test))]
fn get_repo_name_from_git_at_path(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        let git_dir = String::from_utf8(output.stdout).ok()?;
        let git_path = std::path::PathBuf::from(git_dir.trim());

        if git_path.file_name().and_then(|name| name.to_str()) == Some(".git") {
            let toplevel_output = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .current_dir(path)
                .output()
                .ok()?;

            if toplevel_output.status.success() {
                let toplevel = String::from_utf8(toplevel_output.stdout).ok()?;
                let path = std::path::PathBuf::from(toplevel.trim());
                return path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string());
            }
        }

        if git_path.as_os_str() == "." {
            if let Some(current_dir_name) = path.file_name().and_then(|name| name.to_str()) {
                if let Some(stripped) = current_dir_name.strip_suffix(".bare") {
                    return Some(stripped.to_string());
                } else {
                    return Some(current_dir_name.to_string());
                }
            }
        }

        if let Some(parent) = git_path.parent() {
            if parent.file_name().and_then(|name| name.to_str()) == Some("worktrees") {
                if let Some(repo_dir) = parent.parent() {
                    if let Some(repo_name) = repo_dir.file_name().and_then(|name| name.to_str()) {
                        if let Some(stripped) = repo_name.strip_suffix(".bare") {
                            return Some(stripped.to_string());
                        } else if repo_name == ".git" {
                            if let Some(actual_repo_dir) = repo_dir.parent() {
                                if let Some(actual_repo_name) =
                                    actual_repo_dir.file_name().and_then(|name| name.to_str())
                                {
                                    return Some(actual_repo_name.to_string());
                                }
                            }
                        } else {
                            return Some(repo_name.to_string());
                        }
                    }
                }
            }
        }

        if let Some(git_dir_name) = git_path.file_name().and_then(|name| name.to_str()) {
            if let Some(stripped) = git_dir_name.strip_suffix(".bare") {
                return Some(stripped.to_string());
            }
        }

        None
    } else {
        None
    }
}

pub fn get_repository_info() -> String {
    get_repository_info_at_path(&env::current_dir().unwrap_or_else(|_| UNKNOWN_VALUE.into()))
}

#[cfg(test)]
pub fn get_repository_info_at_path(path: &std::path::Path) -> String {
    use std::process::Stdio;

    let git_dir_output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    if let Ok(output) = git_dir_output {
        if output.status.success() {
            if let Ok(git_dir) = String::from_utf8(output.stdout) {
                let git_path = std::path::PathBuf::from(git_dir.trim());

                if git_path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                    let toplevel_output = Command::new("git")
                        .args(["rev-parse", "--show-toplevel"])
                        .current_dir(path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .output();

                    if let Ok(toplevel_out) = toplevel_output {
                        if toplevel_out.status.success() {
                            if let Ok(toplevel) = String::from_utf8(toplevel_out.stdout) {
                                let toplevel_path = std::path::PathBuf::from(toplevel.trim());
                                if let Some(repo_name) =
                                    toplevel_path.file_name().and_then(|name| name.to_str())
                                {
                                    return repo_name.to_string();
                                }
                            }
                        }
                    }
                }

                if git_path.as_os_str() == "." {
                    if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {
                        if let Some(stripped) = dir_name.strip_suffix(".bare") {
                            return stripped.to_string();
                        } else {
                            return dir_name.to_string();
                        }
                    }
                }

                if let Some(parent) = git_path.parent() {
                    if parent.file_name().and_then(|name| name.to_str()) == Some("worktrees") {
                        if let Some(repo_dir) = parent.parent() {
                            if let Some(repo_name) =
                                repo_dir.file_name().and_then(|name| name.to_str())
                            {
                                let base_repo_name =
                                    if let Some(stripped) = repo_name.strip_suffix(".bare") {
                                        stripped.to_string()
                                    } else if repo_name == ".git" {
                                        if let Some(actual_repo_dir) = repo_dir.parent() {
                                            if let Some(actual_repo_name) = actual_repo_dir
                                                .file_name()
                                                .and_then(|name| name.to_str())
                                            {
                                                actual_repo_name.to_string()
                                            } else {
                                                UNKNOWN_VALUE.to_string()
                                            }
                                        } else {
                                            UNKNOWN_VALUE.to_string()
                                        }
                                    } else {
                                        repo_name.to_string()
                                    };

                                if let Some(worktree_name) =
                                    path.file_name().and_then(|name| name.to_str())
                                {
                                    return format!("{base_repo_name} ({worktree_name})");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(UNKNOWN_VALUE)
        .to_string()
}

#[cfg(not(test))]
pub fn get_repository_info_at_path(path: &std::path::Path) -> String {
    if let Ok(repo) = git2::Repository::discover(path) {
        let current_dir = path.to_path_buf();

        let repo_name = get_repo_name_from_git_at_path(path).unwrap_or_else(|| {
            if repo.is_bare() {
                repo.path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.strip_suffix(".bare").unwrap_or(name).to_string())
                    .unwrap_or_else(|| UNKNOWN_VALUE.to_string())
            } else if let Some(workdir) = repo.workdir() {
                workdir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(UNKNOWN_VALUE)
                    .to_string()
            } else {
                current_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(UNKNOWN_VALUE)
                    .to_string()
            }
        });

        if repo.is_bare() {
            repo_name
        } else {
            let worktree_name = current_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(UNKNOWN_VALUE);

            let dot_git_path = current_dir.join(".git");
            if dot_git_path.is_file() {
                format!("{repo_name} ({worktree_name})")
            } else if repo
                .path()
                .join(crate::constants::WORKTREES_SUBDIR)
                .exists()
            {
                if let Some(workdir) = repo.workdir() {
                    if paths_equal(workdir, &current_dir) {
                        repo_name
                    } else {
                        format!("{repo_name}{MAIN_SUFFIX}")
                    }
                } else {
                    format!("{repo_name}{MAIN_SUFFIX}")
                }
            } else {
                repo_name
            }
        }
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(UNKNOWN_VALUE)
            .to_string()
    }
}
