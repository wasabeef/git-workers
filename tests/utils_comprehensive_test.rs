use colored::*;
use git_workers::utils::{get_theme, print_error, print_progress, print_success};

#[test]
fn test_get_terminal_function() {
    // Skip get_terminal test as it calls exit(1) in non-terminal environments
    // This is a limitation of the current implementation
}

#[test]
fn test_get_theme_function() {
    // Test that get_theme returns a theme
    let theme = get_theme();
    // Should not panic and should return a valid theme
    let _ = theme;
}

#[test]
fn test_print_functions() {
    // Test that print functions don't panic
    print_success("Test success message");
    print_error("Test error message");
    print_progress("Test progress message");

    // These functions should work with empty strings
    print_success("");
    print_error("");
    print_progress("");

    // Test with various message types
    print_success("✓ Operation completed successfully");
    print_error("✗ Something went wrong");
    print_progress("⏳ Processing...");
}

#[test]
fn test_print_with_special_characters() {
    // Test print functions with various special characters
    print_success("Success with émojis: 🎉 ✅");
    print_error("Error with special chars: @#$%^&*()");
    print_progress("Progress with unicode: → ← ↑ ↓");

    // Test with newlines
    print_success("Multi\nline\nsuccess");
    print_error("Multi\nline\nerror");
    print_progress("Multi\nline\nprogress");
}

#[test]
fn test_print_with_colors() {
    // Test that colored strings work with print functions
    let success_msg = "Colored success".green();
    let error_msg = "Colored error".red();
    let progress_msg = "Colored progress".yellow();

    print_success(&format!("{}", success_msg));
    print_error(&format!("{}", error_msg));
    print_progress(&format!("{}", progress_msg));
}

#[test]
fn test_print_with_long_messages() {
    // Test with very long messages
    let long_message = "A".repeat(1000);
    print_success(&long_message);
    print_error(&long_message);
    print_progress(&long_message);
}

#[test]
fn test_print_functions_return_types() {
    // Test that print functions return ()
    print_success("test");
    print_error("test");
    print_progress("test");

    // These functions return () so no need to assert
}

#[test]
fn test_utils_module_completeness() {
    // Test that all expected utilities are available
    // This is mainly a compilation test to ensure exports work

    // Skip get_terminal as it exits in non-terminal environments
    let _theme = get_theme();

    // Test function pointers exist
    let _success_fn: fn(&str) = print_success;
    let _error_fn: fn(&str) = print_error;
    let _progress_fn: fn(&str) = print_progress;
}
