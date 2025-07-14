//! Tests for filesystem abstraction layer
//!
//! This module tests the filesystem abstraction implementation
//! to ensure proper separation of filesystem operations from business logic.

use anyhow::Result;
use git_workers::filesystem::{mock::MockFileSystem, FileSystem, RealFileSystem};
use std::path::PathBuf;

/// Test basic MockFileSystem operations
#[test]
fn test_mock_filesystem_basic_operations() -> Result<()> {
    let fs = MockFileSystem::new()
        .with_file("/test/file.txt", "test content")
        .with_directory("/test/dir")
        .with_file("/another/file.md", "markdown content");

    // Test file existence and reading
    assert!(fs.exists(&PathBuf::from("/test/file.txt")));
    assert!(fs.is_file(&PathBuf::from("/test/file.txt")));
    assert!(!fs.is_dir(&PathBuf::from("/test/file.txt")));

    let content = fs.read_to_string(&PathBuf::from("/test/file.txt"))?;
    assert_eq!(content, "test content");

    // Test directory existence
    assert!(fs.exists(&PathBuf::from("/test/dir")));
    assert!(fs.is_dir(&PathBuf::from("/test/dir")));
    assert!(!fs.is_file(&PathBuf::from("/test/dir")));

    // Test non-existent paths
    assert!(!fs.exists(&PathBuf::from("/nonexistent")));
    assert!(!fs.is_file(&PathBuf::from("/nonexistent")));
    assert!(!fs.is_dir(&PathBuf::from("/nonexistent")));

    Ok(())
}

/// Test MockFileSystem write operations
#[test]
fn test_mock_filesystem_write_operations() -> Result<()> {
    let fs = MockFileSystem::new();

    // Write a new file
    fs.write(&PathBuf::from("/new/file.txt"), "new content")?;

    // Verify it exists and has correct content
    assert!(fs.exists(&PathBuf::from("/new/file.txt")));
    assert!(fs.is_file(&PathBuf::from("/new/file.txt")));

    let content = fs.read_to_string(&PathBuf::from("/new/file.txt"))?;
    assert_eq!(content, "new content");

    // Overwrite the file
    fs.write(&PathBuf::from("/new/file.txt"), "updated content")?;
    let updated_content = fs.read_to_string(&PathBuf::from("/new/file.txt"))?;
    assert_eq!(updated_content, "updated content");

    Ok(())
}

/// Test MockFileSystem copy operations
#[test]
fn test_mock_filesystem_copy_operations() -> Result<()> {
    let fs = MockFileSystem::new().with_file("/source/file.txt", "source content");

    // Copy file
    let bytes_copied = fs.copy(
        &PathBuf::from("/source/file.txt"),
        &PathBuf::from("/dest/file.txt"),
    )?;
    assert_eq!(bytes_copied, 14); // "source content".len()

    // Verify destination exists and has correct content
    assert!(fs.exists(&PathBuf::from("/dest/file.txt")));
    assert!(fs.is_file(&PathBuf::from("/dest/file.txt")));

    let dest_content = fs.read_to_string(&PathBuf::from("/dest/file.txt"))?;
    assert_eq!(dest_content, "source content");

    // Original should still exist
    assert!(fs.exists(&PathBuf::from("/source/file.txt")));

    Ok(())
}

/// Test MockFileSystem directory operations
#[test]
fn test_mock_filesystem_directory_operations() -> Result<()> {
    let fs = MockFileSystem::new();

    // Create directory
    fs.create_dir_all(&PathBuf::from("/test/nested/dirs"))?;
    assert!(fs.exists(&PathBuf::from("/test/nested/dirs")));
    assert!(fs.is_dir(&PathBuf::from("/test/nested/dirs")));

    // Remove directory
    fs.remove_dir_all(&PathBuf::from("/test/nested"))?;
    assert!(!fs.exists(&PathBuf::from("/test/nested/dirs")));
    assert!(!fs.exists(&PathBuf::from("/test/nested")));

    Ok(())
}

/// Test MockFileSystem remove operations
#[test]
fn test_mock_filesystem_remove_operations() -> Result<()> {
    let fs = MockFileSystem::new()
        .with_file("/remove/file.txt", "content")
        .with_directory("/remove/dir");

    // Remove file
    assert!(fs.exists(&PathBuf::from("/remove/file.txt")));
    fs.remove_file(&PathBuf::from("/remove/file.txt"))?;
    assert!(!fs.exists(&PathBuf::from("/remove/file.txt")));

    // Remove directory
    assert!(fs.exists(&PathBuf::from("/remove/dir")));
    fs.remove_dir_all(&PathBuf::from("/remove/dir"))?;
    assert!(!fs.exists(&PathBuf::from("/remove/dir")));

    Ok(())
}

/// Test MockFileSystem rename operations
#[test]
fn test_mock_filesystem_rename_operations() -> Result<()> {
    let fs = MockFileSystem::new()
        .with_file("/old/file.txt", "content")
        .with_directory("/old/dir");

    // Rename file
    fs.rename(
        &PathBuf::from("/old/file.txt"),
        &PathBuf::from("/new/file.txt"),
    )?;
    assert!(!fs.exists(&PathBuf::from("/old/file.txt")));
    assert!(fs.exists(&PathBuf::from("/new/file.txt")));

    let content = fs.read_to_string(&PathBuf::from("/new/file.txt"))?;
    assert_eq!(content, "content");

    // Rename directory
    fs.rename(&PathBuf::from("/old/dir"), &PathBuf::from("/new/dir"))?;
    assert!(!fs.exists(&PathBuf::from("/old/dir")));
    assert!(fs.exists(&PathBuf::from("/new/dir")));
    assert!(fs.is_dir(&PathBuf::from("/new/dir")));

    Ok(())
}

/// Test MockFileSystem error simulation
#[test]
fn test_mock_filesystem_error_simulation() {
    let fs = MockFileSystem::new().with_failure("/fail/path", "Simulated I/O error");

    // All operations should fail for the configured path
    assert!(fs.create_dir_all(&PathBuf::from("/fail/path")).is_err());
    assert!(fs.write(&PathBuf::from("/fail/path"), "content").is_err());
    assert!(fs.read_to_string(&PathBuf::from("/fail/path")).is_err());
    assert!(fs
        .copy(&PathBuf::from("/other"), &PathBuf::from("/fail/path"))
        .is_err());
    assert!(fs
        .rename(&PathBuf::from("/fail/path"), &PathBuf::from("/other"))
        .is_err());
    assert!(fs.remove_file(&PathBuf::from("/fail/path")).is_err());
    assert!(fs.remove_dir_all(&PathBuf::from("/fail/path")).is_err());

    // Other paths should work normally
    assert!(fs.create_dir_all(&PathBuf::from("/normal/path")).is_ok());
}

/// Test error handling for non-existent files
#[test]
fn test_mock_filesystem_error_handling() {
    let fs = MockFileSystem::new();

    // Reading non-existent file should fail
    assert!(fs.read_to_string(&PathBuf::from("/nonexistent")).is_err());

    // Removing non-existent file should fail
    assert!(fs.remove_file(&PathBuf::from("/nonexistent")).is_err());

    // Copying from non-existent source should fail
    assert!(fs
        .copy(&PathBuf::from("/nonexistent"), &PathBuf::from("/dest"))
        .is_err());

    // Renaming non-existent path should fail
    assert!(fs
        .rename(&PathBuf::from("/nonexistent"), &PathBuf::from("/dest"))
        .is_err());
}

/// Test RealFileSystem instantiation
#[test]
fn test_real_filesystem_creation() {
    let fs = RealFileSystem::new();
    // Just verify we can create instances
    // Real filesystem operations are tested through integration tests
    let _ = fs;

    let fs_default = RealFileSystem::new();
    let _ = fs_default;
}

/// Test MockFileSystem builder pattern
#[test]
fn test_mock_filesystem_builder_pattern() -> Result<()> {
    let fs = MockFileSystem::new()
        .with_file("/project/README.md", "# Project")
        .with_file("/project/src/main.rs", "fn main() {}")
        .with_directory("/project/src")
        .with_directory("/project/target")
        .with_directory_contents("/project/tests", vec!["test1.rs", "test2.rs"]);

    // Verify all files and directories were created
    assert!(fs.exists(&PathBuf::from("/project/README.md")));
    assert!(fs.exists(&PathBuf::from("/project/src/main.rs")));
    assert!(fs.exists(&PathBuf::from("/project/src")));
    assert!(fs.exists(&PathBuf::from("/project/target")));
    assert!(fs.exists(&PathBuf::from("/project/tests")));

    // Verify file contents
    let readme = fs.read_to_string(&PathBuf::from("/project/README.md"))?;
    assert_eq!(readme, "# Project");

    let main_rs = fs.read_to_string(&PathBuf::from("/project/src/main.rs"))?;
    assert_eq!(main_rs, "fn main() {}");

    Ok(())
}

/// Test filesystem abstraction with complex directory structures
#[test]
fn test_mock_filesystem_complex_structure() -> Result<()> {
    let fs = MockFileSystem::new()
        .with_directory("/project")
        .with_directory("/project/src")
        .with_directory("/project/src/bin")
        .with_file("/project/Cargo.toml", "[package]\nname = \"test\"")
        .with_file("/project/src/lib.rs", "pub mod utils;")
        .with_file("/project/src/bin/main.rs", "fn main() {}")
        .with_file("/project/.gitignore", "target/\n.env");

    // Test deep file operations
    let cargo_content = fs.read_to_string(&PathBuf::from("/project/Cargo.toml"))?;
    assert!(cargo_content.contains("name = \"test\""));

    // Test directory tree exists
    assert!(fs.is_dir(&PathBuf::from("/project")));
    assert!(fs.is_dir(&PathBuf::from("/project/src")));
    assert!(fs.is_dir(&PathBuf::from("/project/src/bin")));

    // Test file operations in nested directories
    fs.write(
        &PathBuf::from("/project/src/utils.rs"),
        "pub fn helper() {}",
    )?;
    assert!(fs.exists(&PathBuf::from("/project/src/utils.rs")));

    // Test copy operations across directories
    fs.copy(
        &PathBuf::from("/project/src/lib.rs"),
        &PathBuf::from("/project/backup.rs"),
    )?;
    let backup_content = fs.read_to_string(&PathBuf::from("/project/backup.rs"))?;
    assert_eq!(backup_content, "pub mod utils;");

    Ok(())
}
