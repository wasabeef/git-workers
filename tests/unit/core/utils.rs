//! Unit tests for utils module
//!
//! Tests for utility functions like error display, progress indicators, etc.

use anyhow::Result;
use git_workers::utils::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_error_formatting() {
    let error = anyhow::anyhow!("Test error message");
    let formatted = format!("{error}");
    assert!(!formatted.is_empty());
    assert!(formatted.contains("Test error message"));
}

#[test]
fn test_error_with_context() {
    let error = anyhow::anyhow!("Root cause")
        .context("Middle layer")
        .context("Top layer");
    let formatted = format!("{error}");
    assert!(!formatted.is_empty());
    assert!(formatted.contains("Top layer"));
}

#[test]
fn test_print_functions() {
    // These functions print to stdout/stderr, so we just ensure they don't panic
    print_error("This is an error");
    print_warning("This is a warning");
    print_success("This is a success message");
    print_progress("This is a progress message");
}

#[test]
fn test_write_switch_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_path = temp_dir.path().join("test_worktree");

    // Test with GW_SWITCH_FILE environment variable
    let switch_file = temp_dir.path().join("switch_file");
    std::env::set_var("GW_SWITCH_FILE", switch_file.to_str().unwrap());

    write_switch_path(&test_path);

    // Verify the file was created with the correct content
    let content = fs::read_to_string(&switch_file)?;
    assert_eq!(content.trim(), test_path.to_str().unwrap());

    // Clean up
    std::env::remove_var("GW_SWITCH_FILE");

    Ok(())
}

#[test]
fn test_path_operations() {
    // Test basic path operations that are available
    let test_path = PathBuf::from("/tmp/test");
    assert!(test_path.is_absolute());

    let relative_path = PathBuf::from("test");
    assert!(!relative_path.is_absolute());
}

#[test]
fn test_confirm_action() {
    // This function requires user input, so we can't test it directly
    // We would need to mock the dialoguer library for proper testing
}

#[test]
fn test_press_any_key_to_continue() {
    // This function waits for user input, so we can't test it directly
    // We would need to mock stdin for proper testing
}

#[test]
fn test_terminal_operations() {
    // Test that terminal functions exist and can be created
    // Note: get_terminal() now returns Term::stderr() without validation
    let terminal = get_terminal();
    // Just ensure it doesn't panic
    drop(terminal);

    // Test is_term check separately
    let term = console::Term::stderr();
    // This will be false in CI but true in local terminal
    let _is_terminal = term.is_term();
}
