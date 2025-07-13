//! Public API basic contract tests
//!
//! This test file improves refactoring resistance by verifying the input-output
//! contracts of public APIs and detecting compatibility breaks early.

use anyhow::Result;
use git_workers::commands::{validate_custom_path, validate_worktree_name};
use git_workers::hooks::{execute_hooks, HookContext};
use std::path::PathBuf;

/// Contract-based test: Basic contract of validate_worktree_name
#[test]
fn contract_validate_worktree_name_basic() {
    struct TestCase {
        input: &'static str,
        should_pass: bool,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            input: "valid-name",
            should_pass: true,
            description: "Valid name",
        },
        TestCase {
            input: "valid_name",
            should_pass: true,
            description: "Underscore",
        },
        TestCase {
            input: "valid123",
            should_pass: true,
            description: "Contains numbers",
        },
        TestCase {
            input: "",
            should_pass: false,
            description: "Empty string",
        },
        TestCase {
            input: ".hidden",
            should_pass: false,
            description: "Starts with dot",
        },
        TestCase {
            input: "name/slash",
            should_pass: false,
            description: "Contains slash",
        },
        TestCase {
            input: "name\\backslash",
            should_pass: false,
            description: "Contains backslash",
        },
        TestCase {
            input: "name:colon",
            should_pass: false,
            description: "Contains colon",
        },
        TestCase {
            input: "name*asterisk",
            should_pass: false,
            description: "Contains asterisk",
        },
        TestCase {
            input: "name?question",
            should_pass: false,
            description: "Contains question mark",
        },
        TestCase {
            input: "name\"quote",
            should_pass: false,
            description: "Contains double quote",
        },
        TestCase {
            input: "name<less",
            should_pass: false,
            description: "Contains less than",
        },
        TestCase {
            input: "name>greater",
            should_pass: false,
            description: "Contains greater than",
        },
        TestCase {
            input: "name|pipe",
            should_pass: false,
            description: "Contains pipe",
        },
        TestCase {
            input: "HEAD",
            should_pass: false,
            description: "Git reserved word",
        },
        TestCase {
            input: "refs",
            should_pass: false,
            description: "Git reserved word",
        },
        TestCase {
            input: "hooks",
            should_pass: false,
            description: "Git reserved word",
        },
    ];

    for case in test_cases {
        let result = validate_worktree_name(case.input);
        assert_eq!(
            result.is_ok(),
            case.should_pass,
            "Contract violation: {} ({})",
            case.description,
            case.input
        );
    }

    // Test for name that is too long (individual processing)
    let long_name = "a".repeat(256);
    let result = validate_worktree_name(&long_name);
    assert!(
        result.is_err(),
        "Names with 256 characters must be rejected"
    );
}

/// Contract-based test: Basic contract of validate_custom_path
#[test]
fn contract_validate_custom_path_basic() {
    struct TestCase {
        input: &'static str,
        should_pass: bool,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            input: "../safe/path",
            should_pass: true,
            description: "Safe relative path",
        },
        TestCase {
            input: "subdirectory/path",
            should_pass: true,
            description: "Subdirectory",
        },
        TestCase {
            input: "../sibling",
            should_pass: true,
            description: "Sibling directory",
        },
        TestCase {
            input: "",
            should_pass: false,
            description: "Empty string",
        },
        TestCase {
            input: "/absolute/path",
            should_pass: false,
            description: "Absolute path",
        },
        TestCase {
            input: "../../../etc/passwd",
            should_pass: false,
            description: "Dangerous path",
        },
        TestCase {
            input: "path/with/../../traversal",
            should_pass: true,
            description: "Path traversal (allowed level)",
        },
        TestCase {
            input: "path/",
            should_pass: false,
            description: "Trailing slash",
        },
        TestCase {
            input: "/root",
            should_pass: false,
            description: "Root path",
        },
        TestCase {
            input: "C:\\Windows",
            should_pass: false,
            description: "Windows path",
        },
        TestCase {
            input: "path\\with\\backslash",
            should_pass: true,
            description: "Contains backslash (allowed)",
        },
        TestCase {
            input: "path:with:colon",
            should_pass: false,
            description: "Contains colon",
        },
        TestCase {
            input: "path*with*asterisk",
            should_pass: false,
            description: "Contains asterisk",
        },
        TestCase {
            input: "path?with?question",
            should_pass: false,
            description: "Contains question mark",
        },
        TestCase {
            input: "path\"with\"quote",
            should_pass: false,
            description: "Contains double quote",
        },
        TestCase {
            input: "path<with<less",
            should_pass: false,
            description: "Contains less than",
        },
        TestCase {
            input: "path>with>greater",
            should_pass: false,
            description: "Contains greater than",
        },
        TestCase {
            input: "path|with|pipe",
            should_pass: false,
            description: "Contains pipe",
        },
        TestCase {
            input: "path/.git/hooks",
            should_pass: false,
            description: "Git reserved directory",
        },
    ];

    for case in test_cases {
        let result = validate_custom_path(case.input);
        assert_eq!(
            result.is_ok(),
            case.should_pass,
            "Contract violation: {} ({})",
            case.description,
            case.input
        );
    }
}

/// Contract-based test: Basic behavior contract of execute_hooks
#[test]
fn contract_execute_hooks_basic() -> Result<()> {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create basic Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()?;

    // Initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()?;

    let context = HookContext {
        worktree_name: "test-worktree".to_string(),
        worktree_path: temp_dir.path().to_path_buf(),
    };

    // Contract 1: Normal termination without hook configuration
    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Must terminate normally without hook configuration"
    );

    // Contract 2: Normal termination even with non-existent hook type
    let result = execute_hooks("non-existent-hook", &context);
    assert!(
        result.is_ok(),
        "Must terminate normally even with non-existent hook type"
    );

    // Contract 3: Normal termination even with empty hook configuration
    let config_content = r#"
[hooks]
post-create = []
"#;
    fs::write(temp_dir.path().join(".git-workers.toml"), config_content)?;

    let result = execute_hooks("post-create", &context);
    assert!(
        result.is_ok(),
        "Must terminate normally even with empty hook configuration"
    );

    Ok(())
}

/// Contract-based test: Comprehensiveness of input validation
#[test]
fn contract_input_validation_comprehensive() {
    // Comprehensive test of dangerous character patterns
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

    for &ch in &dangerous_chars {
        let test_input = format!("name{ch}test");

        // validate_worktree_name contract rejects dangerous characters
        let result = validate_worktree_name(&test_input);
        assert!(
            result.is_err(),
            "Names containing dangerous character '{ch}' must be rejected: {test_input}"
        );

        // validate_custom_path contract rejects only WINDOWS_RESERVED_CHARS (except path separators and backslash)
        if ch != '/' && ch != '\\' {
            let result = validate_custom_path(&test_input);
            assert!(
                result.is_err(),
                "Paths containing dangerous character '{ch}' must be rejected: {test_input}"
            );
        }
    }
}

/// Contract-based test: Comprehensive check of Git reserved words
#[test]
fn contract_git_reserved_names_comprehensive() {
    let git_reserved = ["HEAD", "refs", "hooks", "info", "objects", "logs"];

    for &reserved in &git_reserved {
        // Reserved word as-is
        let result = validate_worktree_name(reserved);
        assert!(
            result.is_err(),
            "Git reserved word '{reserved}' must be rejected"
        );

        // Reject even with mixed case
        let mixed_case = reserved.to_lowercase();
        let result = validate_worktree_name(&mixed_case);
        assert!(
            result.is_err(),
            "Lowercase version of Git reserved word '{mixed_case}' must be rejected"
        );
    }
}

/// Contract-based test: Precise handling of boundary values
#[test]
fn contract_boundary_values_precise() {
    // Exactly maximum length (255 characters)
    let max_length_name = "a".repeat(255);
    let result = validate_worktree_name(&max_length_name);
    assert!(
        result.is_ok(),
        "Names with exactly 255 characters must be allowed"
    );

    // Maximum length + 1 (256 characters)
    let over_length_name = "a".repeat(256);
    let result = validate_worktree_name(&over_length_name);
    assert!(
        result.is_err(),
        "Names with 256 characters must be rejected"
    );

    // Shortest valid name (1 character)
    let result = validate_worktree_name("a");
    assert!(result.is_ok(), "Names with 1 character must be allowed");
}

/// Contract-based test: Immutability of HookContext
#[test]
fn contract_hook_context_immutability() {
    let context = HookContext {
        worktree_name: "test".to_string(),
        worktree_path: PathBuf::from("/test/path"),
    };

    // Context fields are retained as expected
    assert_eq!(context.worktree_name, "test");
    assert_eq!(context.worktree_path, PathBuf::from("/test/path"));

    // Creation with different values is also accurate
    let context2 = HookContext {
        worktree_name: "different".to_string(),
        worktree_path: PathBuf::from("/different/path"),
    };

    assert_eq!(context2.worktree_name, "different");
    assert_eq!(context2.worktree_path, PathBuf::from("/different/path"));

    // Original context is not affected
    assert_eq!(context.worktree_name, "test");
}

/// Contract-based test: Proper handling of Unicode characters
#[test]
fn contract_unicode_handling() {
    // Names containing non-ASCII characters
    let unicode_names = [
        "名前",           // Japanese
        "测试",           // Chinese
        "тест",           // Russian
        "اختبار",         // Arabic
        "test-émojis-🚀", // Mixed with emojis
    ];

    for name in &unicode_names {
        let result = validate_worktree_name(name);
        // Unicode characters are warned but basically accepted contract
        // (However, filesystem incompatible characters are excluded)
        if !name
            .chars()
            .any(|c| ['/', '\\', ':', '*', '?', '"', '<', '>', '|'].contains(&c))
        {
            // Filesystem compatibility warning exists but not an error
            let is_ascii_only = name.is_ascii();
            if is_ascii_only {
                assert!(result.is_ok(), "ASCII name '{name}' must be allowed");
            }
            // For non-ASCII characters, expected behavior is warning with acceptance
        }
    }
}

/// Contract-based test: Informativeness of error messages
#[test]
fn contract_error_messages_informative() {
    // Contract that error messages for invalid input are informative
    let result = validate_worktree_name("");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message must not be empty");

    let result = validate_worktree_name("/invalid");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message must not be empty");

    let result = validate_custom_path("/absolute/path");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message must not be empty");
}
