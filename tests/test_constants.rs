//! Common constants and utilities for tests

/// Test naming constants to eliminate hardcoded values
#[allow(dead_code)]
pub mod naming {
    /// Prefix for branch names in tests
    pub const BRANCH_NAME_PREFIX: &str = "test-branch";

    /// Prefix for feature branch names
    pub const FEATURE_BRANCH_PREFIX: &str = "feature";

    /// Prefix for renamed worktrees
    pub const RENAMED_PREFIX: &str = "renamed";

    /// Prefix for batch operation tests
    pub const BATCH_PREFIX: &str = "batch";

    /// Prefix for delete operation tests
    pub const DELETE_PREFIX: &str = "delete";

    /// Prefix for simple test cases
    pub const SIMPLE_PREFIX: &str = "simple";

    /// Prefix for multi-operation tests
    pub const MULTI_PREFIX: &str = "multi";

    /// Prefix for switch operation tests
    pub const SWITCH_PREFIX: &str = "switch";

    /// Prefix for preserve operation tests  
    pub const PRESERVE_PREFIX: &str = "preserve";

    /// Prefix for edge case tests
    pub const EDGE_CASE_PREFIX: &str = "edge-case";

    /// Prefix for temporary names
    pub const TEMP_PREFIX: &str = "temp";

    /// Prefix for fail test cases
    pub const FAIL_PREFIX: &str = "fail";
}

/// Test configuration constants
#[allow(dead_code)]
pub mod config {
    /// Default test user name
    pub const TEST_USER_NAME: &str = "Test User";

    /// Default test user email
    pub const TEST_USER_EMAIL: &str = "test@example.com";

    /// Default commit message for test repositories
    pub const INITIAL_COMMIT_MESSAGE: &str = "Initial commit";

    /// Default README content for test repositories
    pub const DEFAULT_README_CONTENT: &str = "# Test Repository\n";

    /// Default README filename
    pub const README_FILENAME: &str = "README.md";

    /// Default main branch name
    pub const MAIN_BRANCH: &str = "main";
}

/// Test timing and limits constants (currently unused but kept for future use)
#[allow(dead_code)]
pub mod limits {
    /// Maximum depth for directory traversal in tests
    pub const MAX_DIRECTORY_DEPTH: u32 = 50;

    /// Maximum file size for test operations (in bytes)
    pub const MAX_TEST_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

    /// Lock timeout in minutes
    pub const LOCK_TIMEOUT_MINUTES: u64 = 5;

    /// Maximum character limit for names
    pub const MAX_NAME_LENGTH: usize = 255;
}

/// Test count constants to eliminate magic numbers
#[allow(dead_code)]
pub mod counts {
    /// Number of test worktrees to create in batch operations
    pub const BATCH_TEST_COUNT: usize = 3;

    /// Number of expected deletable worktrees in mixed tests
    pub const MIXED_DELETE_COUNT: usize = 4;

    /// Number of remaining worktrees after selective deletion
    pub const REMAINING_PRESERVE_COUNT: usize = 2;

    /// Number of worktrees for branch cleanup tests
    pub const BRANCH_CLEANUP_COUNT: usize = 2;

    /// Sequence numbers for test naming
    pub const FIRST: usize = 1;
    pub const SECOND: usize = 2;
    pub const THIRD: usize = 3;
    pub const FOURTH: usize = 4;
}

/// Helper functions for generating test names
#[allow(dead_code)]
pub mod generators {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Generate unique timestamp for test isolation
    pub fn generate_timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    /// Generate unique timestamp in seconds for backwards compatibility
    #[allow(dead_code)]
    pub fn generate_timestamp_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Generate a worktree name with timestamp
    pub fn worktree_name(prefix: &str) -> String {
        let timestamp = generate_timestamp();
        format!("{prefix}-{timestamp}")
    }

    /// Generate a branch name with timestamp
    pub fn branch_name(prefix: &str) -> String {
        let timestamp = generate_timestamp();
        format!("{prefix}-{timestamp}")
    }

    /// Generate a feature branch name with timestamp
    #[allow(dead_code)]
    pub fn feature_branch_name(feature: &str) -> String {
        format!(
            "{}/{}-{}",
            naming::FEATURE_BRANCH_PREFIX,
            feature,
            generate_timestamp()
        )
    }

    /// Generate test data tuple (worktree_name, branch_name, renamed_name)
    pub fn test_data_tuple(base: &str, count: usize) -> Vec<(String, String, String)> {
        (1..=count)
            .map(|i| {
                let timestamp = generate_timestamp();
                (
                    format!("{base}-{i}-{timestamp}"),
                    format!("{base}-branch-{i}-{timestamp}"),
                    format!("{base}-renamed-{i}-{timestamp}"),
                )
            })
            .collect()
    }
}

/// Test file patterns and validation (currently unused but kept for future use)
#[allow(dead_code)]
pub mod patterns {
    /// Pattern for identifying test worktrees
    pub const TEST_WORKTREE_PATTERN: &str = "test-";

    /// Pattern for identifying batch test worktrees
    pub const BATCH_TEST_PATTERN: &str = "batch-";

    /// Pattern for identifying renamed test worktrees
    pub const RENAMED_PATTERN: &str = "renamed-";

    /// Pattern for identifying normal test worktrees
    pub const NORMAL_PATTERN: &str = "normal-";

    /// Characters that are invalid in filesystem names
    pub const INVALID_FILESYSTEM_CHARS: &[char] =
        &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

    /// Git reserved directory names
    pub const GIT_RESERVED_NAMES: &[&str] = &[".git", "HEAD", "refs", "objects", "config", "hooks"];
}
