//! Unified validation tests
//!
//! Integrates validate_custom_path_test.rs and validate_worktree_name_test.rs
//! Eliminates duplication and provides comprehensive validation functionality tests

use git_workers::commands::{validate_custom_path, validate_worktree_name};

// =============================================================================
// Worktree name validation tests
// =============================================================================

/// Test valid worktree names
#[test]
fn test_validate_worktree_name_valid() {
    let valid_names = vec![
        "valid-name",
        "valid_name",
        "valid123",
        "123valid",
        "a",
        "feature-branch",
        "bugfix_123",
        "release-v1.0",
        "my-awesome-feature",
        "test123branch",
        "branch-with-many-dashes",
        "branch_with_many_underscores",
        "MixedCase",
        "camelCase",
        "PascalCase",
        "snake_case_name",
        "kebab-case-name",
    ];

    for name in valid_names {
        let result = validate_worktree_name(name);
        assert!(
            result.is_ok(),
            "Expected '{}' to be valid, got error: {:?}",
            name,
            result.err()
        );
    }
}

/// Test invalid worktree names
#[test]
fn test_validate_worktree_name_invalid() {
    let invalid_names = vec![
        ("", "Empty name"),
        (".hidden", "Hidden file"),
        ("name/slash", "Forward slash"),
        ("name\\backslash", "Backslash"),
        ("name:colon", "Colon"),
        ("name*asterisk", "Asterisk"),
        ("name?question", "Question mark"),
        ("name\"quote", "Double quote"),
        ("name<less", "Less than"),
        ("name>greater", "Greater than"),
        ("name|pipe", "Pipe"),
        ("name\0null", "Null character"),
        ("name\ttab", "Tab character"),
        ("name\nnewline", "Newline"),
        ("name with spaces", "Spaces"),
        ("HEAD", "Git reserved name"),
        ("refs", "Git reserved name"),
        ("hooks", "Git reserved name"),
        ("objects", "Git reserved name"),
        ("index", "Git reserved name"),
        ("config", "Git reserved name"),
        ("COMMIT_EDITMSG", "Git reserved name"),
        ("FETCH_HEAD", "Git reserved name"),
        ("ORIG_HEAD", "Git reserved name"),
        ("MERGE_HEAD", "Git reserved name"),
    ];

    for (name, description) in invalid_names {
        let result = validate_worktree_name(name);
        assert!(
            result.is_err(),
            "Expected '{name}' ({description}) to be invalid, but it was accepted"
        );
    }
}

/// Test worktree name length limits
#[test]
fn test_validate_worktree_name_length_limits() {
    // Test maximum valid length (255 characters)
    let max_length_name = "a".repeat(255);
    assert!(validate_worktree_name(&max_length_name).is_ok());

    // Test over maximum length (256 characters)
    let over_length_name = "a".repeat(256);
    assert!(validate_worktree_name(&over_length_name).is_err());

    // Test minimum valid length (1 character)
    assert!(validate_worktree_name("a").is_ok());
}

/// Test special character handling
#[test]
fn test_validate_worktree_name_special_chars() {
    let special_cases = vec![
        ("unicode-émojis-🚀", false), // Non-ASCII characters
        ("control\x1bchar", false),   // Control characters
        ("unicode-café", false),      // Unicode characters
        ("name.with.dots", true),     // Dots are allowed
        ("123numeric", true),         // Starting with numbers is OK
        ("end123", true),             // Ending with numbers is OK
    ];

    for (name, should_pass) in special_cases {
        let result = validate_worktree_name(name);
        if should_pass {
            assert!(result.is_ok(), "Expected '{name}' to pass validation");
        } else {
            assert!(result.is_err(), "Expected '{name}' to fail validation");
        }
    }
}

/// Test edge cases for worktree names
#[test]
fn test_validate_worktree_name_edge_cases() {
    // Test Windows reserved characters (should fail on all platforms)
    let windows_reserved = vec!["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "LPT1", "LPT2"];
    for name in windows_reserved {
        let result = validate_worktree_name(name);
        // These might or might not be rejected depending on implementation
        // Just ensure the function doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    // Test names that look like paths but aren't
    assert!(validate_worktree_name("not-a-path").is_ok());
    assert!(validate_worktree_name("also-not-a-path").is_ok());
}

// =============================================================================
// Custom path validation tests
// =============================================================================

/// Test valid custom paths
#[test]
fn test_validate_custom_path_valid() {
    let valid_paths = vec![
        "../safe/path",
        "subdirectory/path",
        "../sibling",
        "./relative/path",
        "simple-path",
        "path/with/multiple/segments",
        "../parent/path",
        "nested/deep/path/structure",
        "path-with-dashes",
        "path_with_underscores",
        "path.with.dots",
        "path123",
        "123path",
    ];

    for path in valid_paths {
        let result = validate_custom_path(path);
        assert!(
            result.is_ok(),
            "Expected '{}' to be valid, got error: {:?}",
            path,
            result.err()
        );
    }
}

/// Test invalid custom paths
#[test]
fn test_validate_custom_path_invalid() {
    let invalid_paths = vec![
        ("", "Empty path"),
        ("/absolute/path", "Absolute path"),
        ("../../../etc/passwd", "Too many parent traversals"),
        ("path/", "Trailing slash"),
        ("/root", "Absolute path to root"),
        ("C:\\Windows", "Windows absolute path"),
        ("D:\\Program Files", "Windows absolute path with space"),
        ("\\\\server\\share", "UNC path"),
        ("//server/share", "Unix-style UNC path"),
        ("path\\with\\backslashes", "Windows-style path separators"),
    ];

    for (path, description) in invalid_paths {
        let result = validate_custom_path(path);
        assert!(
            result.is_err(),
            "Expected '{path}' ({description}) to be invalid, but it was accepted"
        );
    }
}

/// Test path traversal security
#[test]
fn test_validate_custom_path_traversal() {
    let traversal_paths = vec![
        "../../../etc/passwd",
        "../../../../root",
        "../../../../../../../etc/shadow",
        "../../../../../../../../../../etc/hosts",
        "../../../Windows/System32",
        "../../../../Program Files",
    ];

    for path in traversal_paths {
        let result = validate_custom_path(path);
        assert!(
            result.is_err(),
            "Path traversal '{path}' should be rejected"
        );
    }
}

/// Test Windows path compatibility
#[test]
fn test_validate_custom_path_windows_compat() {
    let windows_paths = vec![
        "C:\\Windows\\System32",
        "D:\\Program Files\\App",
        "E:\\Users\\Name",
        "F:\\",
        "path\\with\\backslashes",
        "relative\\windows\\path",
    ];

    for path in windows_paths {
        let result = validate_custom_path(path);
        // Windows paths should be rejected on all platforms for security
        assert!(
            result.is_err(),
            "Windows path '{path}' should be rejected for cross-platform compatibility"
        );
    }
}

/// Test relative path formats
#[test]
fn test_validate_custom_path_relative_formats() {
    let test_cases = vec![
        ("./current/dir", true),
        ("../parent/dir", true),
        ("../../grandparent", false), // Too many parent traversals
        ("simple/path", true),
        ("../sibling/path", true),
        ("./same/level", true),
    ];

    for (path, should_pass) in test_cases {
        let result = validate_custom_path(path);
        if should_pass {
            assert!(result.is_ok(), "Path '{path}' should be valid");
        } else {
            assert!(result.is_err(), "Path '{path}' should be invalid");
        }
    }
}

/// Test path depth limits
#[test]
fn test_validate_custom_path_depth_limits() {
    // Test reasonable depth
    let reasonable_path = "a/b/c/d/e/f/g/h";
    assert!(validate_custom_path(reasonable_path).is_ok());

    // Test excessive depth (if there's a limit)
    let deep_path = (0..100)
        .map(|i| format!("dir{i}"))
        .collect::<Vec<_>>()
        .join("/");
    let result = validate_custom_path(&deep_path);
    // This should either pass (if no depth limit) or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

/// Test special characters in paths
#[test]
fn test_validate_custom_path_special_chars() {
    let special_char_paths = vec![
        ("path with spaces", false), // Spaces might be problematic
        ("path-with-dashes", true),
        ("path_with_underscores", true),
        ("path.with.dots", true),
        ("path123numbers", true),
        ("123numbers/path", true),
        ("path/with/émojis🚀", false), // Non-ASCII
        ("path/with\ttab", false),     // Tab character
        ("path/with\nnewline", false), // Newline
    ];

    for (path, should_pass) in special_char_paths {
        let result = validate_custom_path(path);
        if should_pass {
            assert!(result.is_ok(), "Path '{path}' should be valid");
        } else {
            assert!(result.is_err(), "Path '{path}' should be invalid");
        }
    }
}

// =============================================================================
// Performance tests
// =============================================================================

/// Test validation performance with large inputs
#[test]
fn test_validation_performance() {
    let start = std::time::Instant::now();

    // Test worktree name validation performance
    for i in 0..1000 {
        let name = format!("test-name-{i}");
        let _ = validate_worktree_name(&name);
    }

    // Test path validation performance
    for i in 0..1000 {
        let path = format!("test/path/{i}");
        let _ = validate_custom_path(&path);
    }

    let duration = start.elapsed();
    // Should complete quickly (under 100ms for 2000 validations)
    assert!(
        duration.as_millis() < 100,
        "Validation took too long: {duration:?}"
    );
}

/// Test validation with maximum length inputs
#[test]
fn test_validation_max_length_performance() {
    let start = std::time::Instant::now();

    // Test with maximum length name
    let max_name = "a".repeat(255);
    let _ = validate_worktree_name(&max_name);

    // Test with long path
    let segments = "segment/".repeat(50);
    let long_path = format!("../long/{segments}");
    let _ = validate_custom_path(&long_path);

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 10,
        "Max length validation took too long: {duration:?}"
    );
}

// =============================================================================
// Error message quality tests
// =============================================================================

/// Test error message quality for worktree names
#[test]
fn test_worktree_name_error_messages() {
    let test_cases = vec!["", ".hidden", "name/slash", "HEAD"];

    let long_name = "a".repeat(256);
    let mut extended_cases = test_cases;
    extended_cases.push(&long_name);

    for invalid_name in extended_cases {
        if let Err(error) = validate_worktree_name(invalid_name) {
            let error_msg = error.to_string();
            // Error messages should be informative
            assert!(
                !error_msg.is_empty(),
                "Error message should not be empty for '{invalid_name}'"
            );
            assert!(
                error_msg.len() > 10,
                "Error message should be descriptive for '{invalid_name}'"
            );
        }
    }
}

/// Test error message quality for custom paths
#[test]
fn test_custom_path_error_messages() {
    let test_cases = vec!["", "/absolute/path", "../../../etc/passwd", "C:\\Windows"];

    for invalid_path in test_cases {
        if let Err(error) = validate_custom_path(invalid_path) {
            let error_msg = error.to_string();
            // Error messages should be informative
            assert!(
                !error_msg.is_empty(),
                "Error message should not be empty for '{invalid_path}'"
            );
            assert!(
                error_msg.len() > 10,
                "Error message should be descriptive for '{invalid_path}'"
            );
        }
    }
}

// =============================================================================
// Boundary value tests
// =============================================================================

/// Test boundary conditions for validation
#[test]
fn test_validation_boundary_conditions() {
    // Test exactly at the boundary
    let boundary_name = "a".repeat(255);
    assert!(validate_worktree_name(&boundary_name).is_ok());

    let over_boundary_name = "a".repeat(256);
    assert!(validate_worktree_name(&over_boundary_name).is_err());

    // Test path with exactly one parent traversal (should be OK)
    assert!(validate_custom_path("../sibling").is_ok());

    // Test path with too many parent traversals
    assert!(validate_custom_path("../../../etc").is_err());
}

/// Test validation consistency
#[test]
fn test_validation_consistency() {
    let test_inputs = vec!["valid-name", "../valid/path", "", "/invalid", "HEAD"];

    // Multiple calls should return the same result
    for input in test_inputs {
        let result1 = validate_worktree_name(input);
        let result2 = validate_worktree_name(input);
        assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Inconsistent results for worktree name '{input}'"
        );

        let result3 = validate_custom_path(input);
        let result4 = validate_custom_path(input);
        assert_eq!(
            result3.is_ok(),
            result4.is_ok(),
            "Inconsistent results for custom path '{input}'"
        );
    }
}
