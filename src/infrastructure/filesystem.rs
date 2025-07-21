//! Filesystem operations abstraction layer
//!
//! This module provides an abstraction over filesystem operations,
//! allowing for testable code by separating business logic from filesystem dependencies.

#![allow(dead_code)]

use anyhow::Result;
use std::fs::{DirEntry, File, OpenOptions};
use std::path::{Path, PathBuf};

/// Trait for filesystem operations
///
/// This trait abstracts filesystem operations, making the code testable
/// by allowing mock implementations for testing and real implementations for production.
pub trait FileSystem {
    /// Create a directory and all its parent directories
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove a file
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Remove a directory and all its contents
    fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// Read the entire contents of a file into a string
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Write a string to a file, creating the file if it doesn't exist
    fn write(&self, path: &Path, contents: &str) -> Result<()>;

    /// Copy a file from source to destination
    fn copy(&self, from: &Path, to: &Path) -> Result<u64>;

    /// Rename/move a file or directory
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Read directory entries
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Check if a path is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Check if a path is a directory
    fn is_dir(&self, path: &Path) -> bool;

    /// Open a file with options
    fn open_with_options(&self, path: &Path, options: &OpenOptions) -> Result<File>;

    /// Get file metadata
    fn metadata(&self, path: &Path) -> Result<std::fs::Metadata>;

    /// Get symlink metadata (doesn't follow symlinks)
    fn symlink_metadata(&self, path: &Path) -> Result<std::fs::Metadata>;
}

/// Production implementation using std::fs
pub struct RealFileSystem;

impl RealFileSystem {
    /// Create a new RealFileSystem instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RealFileSystem {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<()> {
        std::fs::write(path, contents)?;
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
        let bytes = std::fs::copy(from, to)?;
        Ok(bytes)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to)?;
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let entries = std::fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
        Ok(entries)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn open_with_options(&self, path: &Path, options: &OpenOptions) -> Result<File> {
        let file = options.open(path)?;
        Ok(file)
    }

    fn metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata)
    }

    fn symlink_metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
        let metadata = std::fs::symlink_metadata(path)?;
        Ok(metadata)
    }
}

pub mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Mock filesystem for testing
    pub struct MockFileSystem {
        files: RefCell<HashMap<PathBuf, String>>,
        directories: RefCell<HashMap<PathBuf, Vec<String>>>,
        should_fail: RefCell<HashMap<PathBuf, &'static str>>,
    }

    impl Default for MockFileSystem {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockFileSystem {
        /// Create a new MockFileSystem instance
        pub fn new() -> Self {
            Self {
                files: RefCell::new(HashMap::new()),
                directories: RefCell::new(HashMap::new()),
                should_fail: RefCell::new(HashMap::new()),
            }
        }

        /// Add a file to the mock filesystem
        pub fn with_file(self, path: &str, content: &str) -> Self {
            self.files
                .borrow_mut()
                .insert(PathBuf::from(path), content.to_string());
            self
        }

        /// Add a directory to the mock filesystem
        pub fn with_directory(self, path: &str) -> Self {
            self.directories
                .borrow_mut()
                .insert(PathBuf::from(path), Vec::new());
            self
        }

        /// Add a directory with files to the mock filesystem
        pub fn with_directory_contents(self, path: &str, files: Vec<&str>) -> Self {
            let entries = files.into_iter().map(|f| f.to_string()).collect();
            self.directories
                .borrow_mut()
                .insert(PathBuf::from(path), entries);
            self
        }

        /// Make an operation fail for a specific path
        pub fn with_failure(self, path: &str, error: &'static str) -> Self {
            self.should_fail
                .borrow_mut()
                .insert(PathBuf::from(path), error);
            self
        }

        /// Check if a path should fail
        fn check_failure(&self, path: &Path) -> Result<()> {
            if let Some(error) = self.should_fail.borrow().get(path) {
                return Err(anyhow::anyhow!("Mock filesystem error: {error}"));
            }
            Ok(())
        }
    }

    impl FileSystem for MockFileSystem {
        fn create_dir_all(&self, path: &Path) -> Result<()> {
            self.check_failure(path)?;
            self.directories
                .borrow_mut()
                .insert(path.to_path_buf(), Vec::new());
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.check_failure(path)?;
            if self.files.borrow_mut().remove(path).is_some() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("File not found: {}", path.display()))
            }
        }

        fn remove_dir_all(&self, path: &Path) -> Result<()> {
            self.check_failure(path)?;

            // Remove directory and all files/subdirectories under it
            let path_str = path.to_string_lossy();
            self.directories
                .borrow_mut()
                .retain(|p, _| !p.to_string_lossy().starts_with(&*path_str));
            self.files
                .borrow_mut()
                .retain(|p, _| !p.to_string_lossy().starts_with(&*path_str));

            Ok(())
        }

        fn read_to_string(&self, path: &Path) -> Result<String> {
            self.check_failure(path)?;
            if let Some(content) = self.files.borrow().get(path) {
                Ok(content.clone())
            } else {
                Err(anyhow::anyhow!("File not found: {}", path.display()))
            }
        }

        fn write(&self, path: &Path, contents: &str) -> Result<()> {
            self.check_failure(path)?;
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_string());
            Ok(())
        }

        fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
            self.check_failure(from)?;
            self.check_failure(to)?;

            let content = {
                let files = self.files.borrow();
                files.get(from).cloned()
            };

            if let Some(content) = content {
                let bytes = content.len() as u64;
                self.files.borrow_mut().insert(to.to_path_buf(), content);
                Ok(bytes)
            } else {
                Err(anyhow::anyhow!("Source file not found: {}", from.display()))
            }
        }

        fn rename(&self, from: &Path, to: &Path) -> Result<()> {
            self.check_failure(from)?;
            self.check_failure(to)?;

            // Handle file renaming
            let file_content = self.files.borrow_mut().remove(from);
            if let Some(content) = file_content {
                self.files.borrow_mut().insert(to.to_path_buf(), content);
                return Ok(());
            }

            // Handle directory renaming
            let dir_entries = self.directories.borrow_mut().remove(from);
            if let Some(entries) = dir_entries {
                self.directories
                    .borrow_mut()
                    .insert(to.to_path_buf(), entries);
                return Ok(());
            }

            Err(anyhow::anyhow!("Source not found: {}", from.display()))
        }

        fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
            self.check_failure(path)?;

            // MockFileSystem doesn't create real DirEntry objects
            // For testing purposes, this is a simplified implementation
            if self.directories.borrow().contains_key(path) {
                // Return empty vec for now - in real tests, this method might not be used
                // or we'd need a more complex mock implementation
                Ok(Vec::new())
            } else {
                Err(anyhow::anyhow!("Directory not found: {}", path.display()))
            }
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.borrow().contains_key(path) || self.directories.borrow().contains_key(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.files.borrow().contains_key(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.directories.borrow().contains_key(path)
        }

        fn open_with_options(&self, path: &Path, _options: &OpenOptions) -> Result<File> {
            self.check_failure(path)?;

            // For mock testing, we can't create real File objects easily
            // This would need a more sophisticated implementation for full testing
            Err(anyhow::anyhow!(
                "MockFileSystem: open_with_options not fully implemented"
            ))
        }

        fn metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
            self.check_failure(path)?;

            // Mock metadata is complex to implement
            // For testing purposes, we might not need these methods
            Err(anyhow::anyhow!("MockFileSystem: metadata not implemented"))
        }

        fn symlink_metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
            self.check_failure(path)?;

            // Mock metadata is complex to implement
            Err(anyhow::anyhow!(
                "MockFileSystem: symlink_metadata not implemented"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockFileSystem;
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mock_filesystem_basic_operations() -> Result<()> {
        let fs = MockFileSystem::new()
            .with_file("/test/file.txt", "content")
            .with_directory("/test/dir");

        // Test file operations
        assert!(fs.exists(&PathBuf::from("/test/file.txt")));
        assert!(fs.is_file(&PathBuf::from("/test/file.txt")));
        assert!(!fs.is_dir(&PathBuf::from("/test/file.txt")));

        let content = fs.read_to_string(&PathBuf::from("/test/file.txt"))?;
        assert_eq!(content, "content");

        // Test directory operations
        assert!(fs.exists(&PathBuf::from("/test/dir")));
        assert!(fs.is_dir(&PathBuf::from("/test/dir")));
        assert!(!fs.is_file(&PathBuf::from("/test/dir")));

        Ok(())
    }

    #[test]
    fn test_mock_filesystem_write_and_copy() -> Result<()> {
        let fs = MockFileSystem::new();

        // Write a file
        fs.write(&PathBuf::from("/new/file.txt"), "new content")?;
        assert!(fs.exists(&PathBuf::from("/new/file.txt")));

        let content = fs.read_to_string(&PathBuf::from("/new/file.txt"))?;
        assert_eq!(content, "new content");

        // Copy the file
        let bytes = fs.copy(
            &PathBuf::from("/new/file.txt"),
            &PathBuf::from("/copied.txt"),
        )?;
        assert_eq!(bytes, 11); // "new content".len()

        let copied_content = fs.read_to_string(&PathBuf::from("/copied.txt"))?;
        assert_eq!(copied_content, "new content");

        Ok(())
    }

    #[test]
    fn test_mock_filesystem_failures() {
        let fs = MockFileSystem::new().with_failure("/fail/path", "Simulated error");

        // Test that operations fail as expected
        assert!(fs.create_dir_all(&PathBuf::from("/fail/path")).is_err());
        assert!(fs.write(&PathBuf::from("/fail/path"), "content").is_err());
        assert!(fs.read_to_string(&PathBuf::from("/fail/path")).is_err());
    }

    #[test]
    fn test_mock_filesystem_remove_operations() -> Result<()> {
        let fs = MockFileSystem::new()
            .with_file("/remove/file.txt", "content")
            .with_directory("/remove/dir");

        // Remove file
        fs.remove_file(&PathBuf::from("/remove/file.txt"))?;
        assert!(!fs.exists(&PathBuf::from("/remove/file.txt")));

        // Remove directory
        fs.remove_dir_all(&PathBuf::from("/remove/dir"))?;
        assert!(!fs.exists(&PathBuf::from("/remove/dir")));

        Ok(())
    }

    #[test]
    fn test_real_filesystem_creation() {
        let _fs = RealFileSystem::new();
        // Just test that we can create the instance
        // Real filesystem operations are tested through integration tests
    }
}
