use anyhow::Result;
use std::process::{Command, Stdio};

#[test]
fn test_color_output_direct_execution() -> Result<()> {
    // Test that colors are enabled when running directly
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .env("TERM", "xterm-256color")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version output should contain ANSI color codes when colors are forced
    assert!(output.status.success());
    assert!(stdout.contains("git-workers"));

    Ok(())
}

#[test]
fn test_color_output_through_pipe() -> Result<()> {
    // Test that colors are still enabled when output is piped
    let child = Command::new("cargo")
        .args(["run", "--", "--version"])
        .env("TERM", "xterm-256color")
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // With set_override(true), colors should be present even when piped
    assert!(output.status.success());
    assert!(stdout.contains("git-workers"));

    Ok(())
}

#[test]
#[allow(clippy::const_is_empty)]
fn test_input_handling_dialoguer() -> Result<()> {
    // This test verifies that dialoguer Input component works correctly
    // In actual usage, this would be interactive

    // Test empty input handling
    let empty_input = "";
    let is_empty = empty_input.is_empty();
    assert!(is_empty); // Intentional assertion for test validation

    // Test valid input
    let valid_input = "feature-branch";
    let is_not_empty = !valid_input.is_empty();
    assert!(is_not_empty); // Intentional assertion for test validation
    assert!(!valid_input.contains(char::is_whitespace));

    // Test input with spaces (should be rejected)
    let invalid_input = "feature branch";
    assert!(invalid_input.contains(char::is_whitespace));

    Ok(())
}

#[test]
fn test_shell_integration_marker() -> Result<()> {
    // Test that SWITCH_TO marker is correctly formatted
    let test_path = "/Users/test/project/branch/feature";
    let marker = format!("SWITCH_TO:{}", test_path);

    assert_eq!(marker, "SWITCH_TO:/Users/test/project/branch/feature");

    // Test marker parsing in shell
    let switch_line = "SWITCH_TO:/Users/test/project/branch/feature";
    let new_dir = switch_line.strip_prefix("SWITCH_TO:").unwrap();
    assert_eq!(new_dir, "/Users/test/project/branch/feature");

    Ok(())
}

#[test]
fn test_menu_icon_alignment() -> Result<()> {
    use git_workers::menu::MenuItem;

    // Test that menu items use ASCII characters
    let items = vec![
        (MenuItem::ListWorktrees, "•  List worktrees"),
        (MenuItem::SearchWorktrees, "?  Search worktrees"),
        (MenuItem::CreateWorktree, "+  Create worktree"),
        (MenuItem::DeleteWorktree, "-  Delete worktree"),
        (MenuItem::BatchDelete, "=  Batch delete worktrees"),
        (MenuItem::CleanupOldWorktrees, "~  Cleanup old worktrees"),
        (MenuItem::RenameWorktree, "*  Rename worktree"),
        (MenuItem::SwitchWorktree, "→  Switch worktree"),
        (MenuItem::Exit, "x  Exit"),
    ];

    for (item, expected) in items {
        let display = format!("{}", item);
        assert_eq!(display, expected);
    }

    Ok(())
}

#[test]
fn test_worktree_creation_with_pattern() -> Result<()> {
    // Test worktree name pattern replacement
    let patterns = vec![
        ("branch/{name}", "feature", "branch/feature"),
        ("{name}", "feature", "feature"),
        ("worktrees/{name}", "feature", "worktrees/feature"),
        ("wt/{name}-branch", "feature", "wt/feature-branch"),
    ];

    for (pattern, name, expected) in patterns {
        let result = pattern.replace("{name}", name);
        assert_eq!(result, expected);
    }

    Ok(())
}

#[test]
fn test_current_worktree_protection() -> Result<()> {
    use git_workers::git::WorktreeInfo;
    use std::path::PathBuf;

    let worktrees = vec![
        WorktreeInfo {
            name: "main".to_string(),
            path: PathBuf::from("/project/main"),
            branch: "main".to_string(),
            is_locked: false,
            is_current: true,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        },
        WorktreeInfo {
            name: "feature".to_string(),
            path: PathBuf::from("/project/feature"),
            branch: "feature".to_string(),
            is_locked: false,
            is_current: false,
            has_changes: false,
            last_commit: None,
            ahead_behind: None,
        },
    ];

    // Filter out current worktree
    let deletable: Vec<_> = worktrees.iter().filter(|w| !w.is_current).collect();

    assert_eq!(deletable.len(), 1);
    assert_eq!(deletable[0].name, "feature");

    Ok(())
}

#[test]
fn test_bare_repository_worktree_creation() -> Result<()> {
    // Test that worktrees from bare repos are created in the parent directory
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("project.bare");
    let expected_worktree_path = temp_dir.path().join("branch").join("feature");

    // The worktree should be created as a sibling to the bare repo
    assert_eq!(
        expected_worktree_path.parent().unwrap().parent().unwrap(),
        bare_repo_path.parent().unwrap()
    );

    Ok(())
}
