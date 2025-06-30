use anyhow::Result;
use git_workers::commands;

#[test]
fn test_validate_worktree_name_with_ascii() -> Result<()> {
    // Normal ASCII names should pass
    assert_eq!(
        commands::validate_worktree_name("feature-123")?,
        "feature-123"
    );
    assert_eq!(commands::validate_worktree_name("my_branch")?, "my_branch");
    assert_eq!(
        commands::validate_worktree_name("test-branch")?,
        "test-branch"
    );
    assert_eq!(commands::validate_worktree_name("UPPERCASE")?, "UPPERCASE");
    assert_eq!(
        commands::validate_worktree_name("123-numbers")?,
        "123-numbers"
    );

    Ok(())
}

#[test]
fn test_validate_worktree_name_invalid_chars() -> Result<()> {
    // Names with invalid characters should fail
    assert!(commands::validate_worktree_name("feature/slash").is_err());
    assert!(commands::validate_worktree_name("back\\slash").is_err());
    assert!(commands::validate_worktree_name("colon:name").is_err());
    assert!(commands::validate_worktree_name("star*name").is_err());
    assert!(commands::validate_worktree_name("question?name").is_err());
    assert!(commands::validate_worktree_name("\"quoted\"").is_err());
    assert!(commands::validate_worktree_name("<angle>").is_err());
    assert!(commands::validate_worktree_name("pipe|name").is_err());
    assert!(commands::validate_worktree_name("null\0char").is_err());

    Ok(())
}

#[test]
fn test_validate_worktree_name_empty() -> Result<()> {
    // Empty name should fail
    assert!(commands::validate_worktree_name("").is_err());
    assert!(commands::validate_worktree_name("   ").is_err()); // Only spaces

    Ok(())
}

#[test]
fn test_validate_worktree_name_reserved() -> Result<()> {
    // Reserved git names should fail
    assert!(commands::validate_worktree_name("HEAD").is_err());
    assert!(commands::validate_worktree_name("head").is_err());

    Ok(())
}

#[test]
fn test_validate_worktree_name_length() -> Result<()> {
    // Very long names should fail
    let long_name = "a".repeat(256);
    assert!(commands::validate_worktree_name(&long_name).is_err());

    // Max length (255) should pass
    let max_name = "a".repeat(255);
    assert_eq!(commands::validate_worktree_name(&max_name)?, max_name);

    Ok(())
}

#[test]
#[ignore = "Requires user interaction - for manual testing only"]
fn test_validate_worktree_name_non_ascii_interactive() -> Result<()> {
    // This test requires user interaction to accept/reject non-ASCII names
    // Run manually with: cargo test test_validate_worktree_name_non_ascii_interactive -- --ignored --nocapture

    // Japanese characters
    let result = commands::validate_worktree_name("日本語-ブランチ");
    println!("Japanese name result: {:?}", result);

    // Chinese characters
    let result = commands::validate_worktree_name("中文-分支");
    println!("Chinese name result: {:?}", result);

    // Emoji
    let result = commands::validate_worktree_name("feature-🚀-rocket");
    println!("Emoji name result: {:?}", result);

    // Mixed ASCII and non-ASCII
    let result = commands::validate_worktree_name("feature-テスト-123");
    println!("Mixed name result: {:?}", result);

    Ok(())
}

#[test]
fn test_validate_worktree_name_trimming() -> Result<()> {
    // Names should be trimmed
    assert_eq!(commands::validate_worktree_name("  feature  ")?, "feature");
    assert_eq!(commands::validate_worktree_name("\tfeature\t")?, "feature");
    assert_eq!(commands::validate_worktree_name("\nfeature\n")?, "feature");

    Ok(())
}
