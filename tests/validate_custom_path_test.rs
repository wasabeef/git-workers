use anyhow::Result;
use git_workers::commands::validate_custom_path;

#[test]
fn test_validate_custom_path_valid_paths() -> Result<()> {
    // Valid relative paths
    let test_cases = vec![
        "../custom-worktree",
        "temp/worktrees/feature",
        "../projects/feature-branch",
        "worktrees/test-feature",
        "custom_dir/my-worktree",
    ];

    for case in test_cases {
        println!("Testing: '{}'", case);
        match validate_custom_path(case) {
            Ok(_) => println!("  ✓ Valid"),
            Err(e) => {
                println!("  ✗ Error: {}", e);
                panic!("Expected '{}' to be valid", case);
            }
        }
    }

    Ok(())
}

#[test]
fn test_validate_custom_path_invalid_empty() -> Result<()> {
    let result = validate_custom_path("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));

    let result = validate_custom_path("   ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));

    Ok(())
}

#[test]
fn test_validate_custom_path_invalid_absolute() -> Result<()> {
    let result = validate_custom_path("/absolute/path");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be relative"));

    // Test different absolute path formats
    let result = validate_custom_path("/usr/local/bin");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_validate_custom_path_invalid_characters() -> Result<()> {
    // Test Windows reserved characters
    let invalid_chars = vec!['<', '>', ':', '"', '|', '?', '*'];

    for char in invalid_chars {
        let path = format!("test{}path", char);
        let result = validate_custom_path(&path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("reserved characters"));
    }

    // Test null byte
    let result = validate_custom_path("test\0path");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));

    Ok(())
}

#[test]
fn test_validate_custom_path_invalid_slashes() -> Result<()> {
    // Test consecutive slashes
    let result = validate_custom_path("test//path");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("consecutive slashes"));

    // Test starting with slash (this is caught by absolute path check)
    let result = validate_custom_path("/test/path");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be relative"));

    // Test ending with slash
    let result = validate_custom_path("test/path/");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("start or end with slash"));

    Ok(())
}

#[test]
fn test_validate_custom_path_path_traversal() -> Result<()> {
    // Test going too far above project root (more than one level up)
    let result = validate_custom_path("../../above-root");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("above project root"));

    // Test multiple levels up
    let result = validate_custom_path("../../../way-above");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("above project root"));

    // Test mixed paths that go too far up - this one actually ends up as ../above which is allowed
    // So we test a worse case: some/path/../../../../above (which goes 2 levels above project)
    let result = validate_custom_path("some/path/../../../../above");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("above project root"));

    Ok(())
}

#[test]
fn test_validate_custom_path_reserved_names() -> Result<()> {
    let reserved_names = vec![".git", "HEAD", "refs", "hooks", "info", "objects", "logs"];

    for name in reserved_names {
        // Test reserved name as path component
        let path = format!("test/{}/worktree", name);
        let result = validate_custom_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved by git"));

        // Test case insensitive
        let path = format!("test/{}/worktree", name.to_uppercase());
        let result = validate_custom_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved by git"));
    }

    Ok(())
}

#[test]
fn test_validate_custom_path_edge_cases() -> Result<()> {
    // Test current directory reference (should be valid but not useful)
    assert!(validate_custom_path("./test-worktree").is_ok());

    // Test complex but valid path
    assert!(validate_custom_path("../projects/client-work/feature-branch").is_ok());

    // Test path that goes up then down (should be valid)
    assert!(validate_custom_path("../sibling/worktrees/feature").is_ok());

    Ok(())
}

#[test]
fn test_validate_custom_path_boundary_conditions() -> Result<()> {
    // Test exactly at project root level (should be valid)
    assert!(validate_custom_path("../same-level-worktree").is_ok());

    // Test one level down from current (should be valid)
    assert!(validate_custom_path("subdirectory/worktree").is_ok());

    // Test path that goes up then comes back to same level
    assert!(validate_custom_path("../project/back-to-same-level").is_ok());

    Ok(())
}
