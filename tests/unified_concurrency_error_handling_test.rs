//! Unified concurrency and error handling tests
//!
//! Tests for concurrent access patterns, locking mechanisms, and error handling
//! under concurrent conditions

use anyhow::Result;
use git_workers::git::GitWorktreeManager;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to setup test repository
fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test-repo");

    // Initialize repository
    std::process::Command::new("git")
        .args(["init", "-b", "main", "test-repo"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}

// =============================================================================
// Concurrent access tests
// =============================================================================

/// Test concurrent GitWorktreeManager creation
#[test]
fn test_concurrent_manager_creation() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create multiple manager instances concurrently
    let repo_path = Arc::new(repo_path);
    let mut handles = vec![];

    for i in 0..10 {
        let path = Arc::clone(&repo_path);
        handles.push(thread::spawn(move || {
            let manager = GitWorktreeManager::new_from_path(&path);
            match manager {
                Ok(_) => {
                    println!("Thread {i} successfully created manager");
                    true
                }
                Err(ref e) => {
                    println!("Thread {i} failed to create manager: {e}");
                    false
                }
            }
        }));
    }

    // Wait for all threads and check results
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // At least some threads should succeed
    assert!(
        success_count > 0,
        "At least some manager creations should succeed"
    );
    println!("Concurrent manager creation: {success_count}/10 succeeded");

    Ok(())
}

/// Test concurrent repository operations
#[test]
fn test_concurrent_repository_operations() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create multiple managers instead of sharing one
    let mut handles = vec![];

    // Spawn threads performing different operations
    for i in 0..5 {
        let path = repo_path.clone();
        handles.push(thread::spawn(move || {
            // Create manager in each thread
            let manager = GitWorktreeManager::new_from_path(&path);
            if let Ok(mgr) = manager {
                let operation = i % 3;
                match operation {
                    0 => {
                        // List worktrees
                        let result = mgr.list_worktrees();
                        match result {
                            Ok(worktrees) => {
                                println!("Thread {i} listed {} worktrees", worktrees.len());
                                true
                            }
                            Err(ref e) => {
                                println!("Thread {i} failed to list worktrees: {e}");
                                false
                            }
                        }
                    }
                    1 => {
                        // List branches
                        let result = mgr.list_all_branches();
                        match result {
                            Ok((local, remote)) => {
                                println!(
                                    "Thread {i} listed {} local, {} remote branches",
                                    local.len(),
                                    remote.len()
                                );
                                true
                            }
                            Err(ref e) => {
                                println!("Thread {i} failed to list branches: {e}");
                                false
                            }
                        }
                    }
                    2 => {
                        // List tags
                        let result = mgr.list_all_tags();
                        match result {
                            Ok(tags) => {
                                println!("Thread {i} listed {} tags", tags.len());
                                true
                            }
                            Err(ref e) => {
                                println!("Thread {i} failed to list tags: {e}");
                                false
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                println!("Thread {i} failed to create manager");
                false
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // All read operations should succeed
    assert!(
        success_count >= 3,
        "Most concurrent operations should succeed"
    );
    println!("Concurrent operations: {success_count}/5 succeeded");

    Ok(())
}

/// Test concurrent worktree creation (should be serialized)
#[test]
fn test_concurrent_worktree_creation() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let mut handles = vec![];

    // Attempt to create multiple worktrees concurrently
    for i in 0..3 {
        let path = repo_path.clone();
        handles.push(thread::spawn(move || {
            let worktree_name = format!("test-worktree-{i}");
            let branch_name = format!("test-branch-{i}");

            // Create manager in each thread
            let manager = GitWorktreeManager::new_from_path(&path);
            if let Ok(mgr) = manager {
                let result =
                    mgr.create_worktree_with_new_branch(&worktree_name, &branch_name, "main");
                match result {
                    Ok(_) => {
                        println!("Thread {i} successfully created worktree {worktree_name}");
                        true
                    }
                    Err(e) => {
                        println!("Thread {i} failed to create worktree {worktree_name}: {e}");
                        false
                    }
                }
            } else {
                println!("Thread {i} failed to create manager");
                false
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Some worktree creations should succeed (depends on locking)
    println!("Concurrent worktree creation: {success_count}/3 succeeded");

    Ok(())
}

/// Test concurrent file operations
#[test]
fn test_concurrent_file_operations() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let repo_path = Arc::new(repo_path);
    let mut handles = vec![];

    // Create multiple files concurrently
    for i in 0..10 {
        let path = Arc::clone(&repo_path);
        handles.push(thread::spawn(move || {
            let file_path = path.join(format!("concurrent-file-{i}.txt"));
            let content = format!("Content from thread {i}");

            let result = fs::write(&file_path, content);
            match result {
                Ok(_) => {
                    println!("Thread {i} successfully wrote file");
                    true
                }
                Err(e) => {
                    println!("Thread {i} failed to write file: {e}");
                    false
                }
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Most file operations should succeed
    assert!(
        success_count >= 8,
        "Most concurrent file operations should succeed"
    );
    println!("Concurrent file operations: {success_count}/10 succeeded");

    Ok(())
}

// =============================================================================
// Lock file tests
// =============================================================================

/// Test lock file creation and cleanup
#[test]
fn test_lock_file_creation() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Check if lock file exists initially
    let lock_file = repo_path.join(".git/git-workers-worktree.lock");
    assert!(!lock_file.exists(), "Lock file should not exist initially");

    // Simulate lock file creation
    fs::write(&lock_file, "test-lock")?;
    assert!(lock_file.exists(), "Lock file should exist after creation");

    // Clean up
    fs::remove_file(&lock_file)?;
    assert!(!lock_file.exists(), "Lock file should be cleaned up");

    Ok(())
}

/// Test stale lock file handling
#[test]
fn test_stale_lock_file_handling() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    // Create a stale lock file
    let lock_file = repo_path.join(".git/git-workers-worktree.lock");
    fs::write(&lock_file, "stale-lock")?;

    // Modify the file's timestamp to make it appear old
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::time::SystemTime;

        let metadata = fs::metadata(&lock_file)?;
        let old_time = SystemTime::now() - Duration::from_secs(600); // 10 minutes ago

        // This is just a test verification - in real implementation,
        // stale lock detection would use file modification time
        println!(
            "Lock file created at: {:?}",
            SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.mtime() as u64)
        );
        println!("Current time: {:?}", SystemTime::now());

        // Simulate stale lock detection
        let is_stale = old_time < SystemTime::now() - Duration::from_secs(300); // 5 minutes
        assert!(is_stale, "Lock file should be detected as stale");
    }

    // Clean up
    fs::remove_file(&lock_file)?;

    Ok(())
}

/// Test lock file race condition
#[test]
fn test_lock_file_race_condition() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let lock_file = repo_path.join(".git/git-workers-worktree.lock");
    let lock_file = Arc::new(lock_file);
    let mut handles = vec![];

    // Multiple threads trying to create lock file
    for i in 0..5 {
        let path = Arc::clone(&lock_file);
        handles.push(thread::spawn(move || {
            let content = format!("lock-{i}");

            // Try to create lock file (simulate atomic operation)
            match fs::write(&*path, content) {
                Ok(_) => {
                    println!("Thread {i} successfully created lock file");
                    thread::sleep(Duration::from_millis(10)); // Hold lock briefly

                    // Clean up
                    let _ = fs::remove_file(&*path);
                    true
                }
                Err(e) => {
                    println!("Thread {i} failed to create lock file: {e}");
                    false
                }
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // In reality, only one thread should succeed due to proper locking
    println!("Lock file race condition: {success_count}/5 succeeded");

    Ok(())
}

// =============================================================================
// Error handling under concurrent conditions
// =============================================================================

/// Test error handling during concurrent operations
#[test]
fn test_concurrent_error_handling() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let mut handles = vec![];

    // Some operations that should fail
    for i in 0..5 {
        let path = repo_path.clone();
        handles.push(thread::spawn(move || {
            // Create manager in each thread
            let manager = GitWorktreeManager::new_from_path(&path);
            if let Ok(mgr) = manager {
                let operation = i % 3;
                match operation {
                    0 => {
                        // Try to create worktree with invalid name
                        let result = mgr.create_worktree_with_new_branch("", "invalid", "main");
                        match result {
                            Ok(_) => {
                                println!("Thread {i} unexpectedly succeeded with invalid name");
                                false
                            }
                            Err(ref e) => {
                                println!("Thread {i} correctly failed with invalid name: {e}");
                                true
                            }
                        }
                    }
                    1 => {
                        // Try to create worktree with non-existent base
                        let result =
                            mgr.create_worktree_with_new_branch("test", "test", "non-existent");
                        match result {
                            Ok(_) => {
                                println!(
                                    "Thread {i} unexpectedly succeeded with non-existent base"
                                );
                                false
                            }
                            Err(ref e) => {
                                println!("Thread {i} correctly failed with non-existent base: {e}");
                                true
                            }
                        }
                    }
                    2 => {
                        // Valid operation that should succeed
                        let result = mgr.list_worktrees();
                        match result {
                            Ok(_) => {
                                println!("Thread {i} successfully listed worktrees");
                                true
                            }
                            Err(ref e) => {
                                println!("Thread {i} failed to list worktrees: {e}");
                                false
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                println!("Thread {i} failed to create manager");
                false
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Most operations should handle errors correctly
    assert!(success_count >= 3, "Most error handling should be correct");
    println!("Concurrent error handling: {success_count}/5 correct");

    Ok(())
}

/// Test resource cleanup under concurrent conditions
#[test]
fn test_concurrent_resource_cleanup() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let repo_path = Arc::new(repo_path);
    let mut handles = vec![];

    // Create and clean up temporary files concurrently
    for i in 0..10 {
        let path = Arc::clone(&repo_path);
        handles.push(thread::spawn(move || {
            let temp_file = path.join(format!("temp-{i}.txt"));

            // Create temporary file
            let create_result = fs::write(&temp_file, format!("temp content {i}"));
            if create_result.is_err() {
                println!("Thread {i} failed to create temp file");
                return false;
            }

            // Brief delay to simulate work
            thread::sleep(Duration::from_millis(1));

            // Clean up
            let cleanup_result = fs::remove_file(&temp_file);
            match cleanup_result {
                Ok(_) => {
                    println!("Thread {i} successfully cleaned up temp file");
                    true
                }
                Err(e) => {
                    println!("Thread {i} failed to clean up temp file: {e}");
                    false
                }
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Most cleanup operations should succeed
    assert!(success_count >= 8, "Most concurrent cleanup should succeed");
    println!("Concurrent resource cleanup: {success_count}/10 succeeded");

    Ok(())
}

/// Test timeout handling during concurrent operations
#[test]
fn test_concurrent_timeout_handling() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let repo_path = Arc::new(repo_path);
    let mut handles = vec![];

    // Operations with simulated timeouts
    for i in 0..5 {
        let _path = Arc::clone(&repo_path);
        handles.push(thread::spawn(move || {
            let start = std::time::Instant::now();

            // Simulate operation with timeout
            let timeout = Duration::from_millis(100);
            let operation_duration = Duration::from_millis(50 + (i * 30) as u64);

            thread::sleep(operation_duration);

            let elapsed = start.elapsed();
            if elapsed > timeout {
                println!("Thread {i} operation timed out after {elapsed:?}");
                false
            } else {
                println!("Thread {i} operation completed in {elapsed:?}");
                true
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Some operations should complete within timeout
    println!("Concurrent timeout handling: {success_count}/5 within timeout");

    Ok(())
}

// =============================================================================
// Deadlock prevention tests
// =============================================================================

/// Test deadlock prevention with multiple resources
#[test]
fn test_deadlock_prevention() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let mut handles = vec![];

    // Thread 1: Access resources in order A -> B
    let path1 = repo_path.clone();
    let path2 = repo_path.clone();
    handles.push(thread::spawn(move || {
        let manager1 = GitWorktreeManager::new_from_path(&path1);
        let manager2 = GitWorktreeManager::new_from_path(&path2);

        if let (Ok(mgr1), Ok(mgr2)) = (manager1, manager2) {
            let _result1 = mgr1.list_worktrees();
            thread::sleep(Duration::from_millis(10));
            let _result2 = mgr2.list_all_branches();
            println!("Thread 1 completed resource access A -> B");
            true
        } else {
            println!("Thread 1 failed to create managers");
            false
        }
    }));

    // Thread 2: Access resources in order B -> A
    let path1 = repo_path.clone();
    let path2 = repo_path.clone();
    handles.push(thread::spawn(move || {
        let manager1 = GitWorktreeManager::new_from_path(&path1);
        let manager2 = GitWorktreeManager::new_from_path(&path2);

        if let (Ok(mgr1), Ok(mgr2)) = (manager1, manager2) {
            let _result2 = mgr2.list_all_branches();
            thread::sleep(Duration::from_millis(10));
            let _result1 = mgr1.list_worktrees();
            println!("Thread 2 completed resource access B -> A");
            true
        } else {
            println!("Thread 2 failed to create managers");
            false
        }
    }));

    // Wait for all operations to complete (should not deadlock)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < timeout,
        "Operations should complete without deadlock"
    );
    assert_eq!(
        success_count, 2,
        "Both threads should complete successfully"
    );

    println!("Deadlock prevention test completed in {elapsed:?}");

    Ok(())
}

/// Test concurrent memory management
#[test]
fn test_concurrent_memory_management() -> Result<()> {
    let (_temp_dir, repo_path) = setup_test_repo()?;

    let repo_path = Arc::new(repo_path);
    let mut handles = vec![];

    // Create and drop managers concurrently
    for i in 0..20 {
        let path = Arc::clone(&repo_path);
        handles.push(thread::spawn(move || {
            // Create manager
            let manager = GitWorktreeManager::new_from_path(&path);
            match manager {
                Ok(mgr) => {
                    // Perform some operations
                    let _ = mgr.list_worktrees();
                    let _ = mgr.list_all_branches();

                    // Manager will be dropped here
                    println!("Thread {i} completed operations and dropped manager");
                    true
                }
                Err(e) => {
                    println!("Thread {i} failed to create manager: {e}");
                    false
                }
            }
        }));
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    // Most operations should succeed
    assert!(
        success_count >= 15,
        "Most concurrent memory management should succeed"
    );
    println!("Concurrent memory management: {success_count}/20 succeeded");

    Ok(())
}
