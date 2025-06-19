# Changelog

All notable changes to Git Workers will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

For detailed release notes and binary downloads, see [GitHub Releases](https://github.com/wasabeef/git-workers/releases).

## [Unreleased]

### Changed

- **BREAKING**: Removed command-line argument options (--list, --create, etc.) in favor of interactive menu-only interface
- Simplified main.rs to focus solely on interactive menu operations
- Improved worktree rename functionality with `git worktree repair` integration
- Enhanced configuration lookup strategy:
  - Now checks current directory first (useful for bare repo worktrees)
  - Then checks parent directory's main/master worktree
  - Finally falls back to repository root
- Improved path handling for worktree creation:
  - Paths are now canonicalized to eliminate "../" in display
  - "In subdirectory" option now correctly creates worktrees in subdirectories

### Added

- Edit hooks menu option (`λ`) for managing lifecycle hooks through the interface
- Comprehensive Rustdoc documentation for all modules and functions
- Current directory configuration lookup priority for .git-workers.toml
- Parent directory configuration lookup for .git-workers.toml
- Better error handling with mutex poison recovery in tests
- Branch deletion functionality in batch delete operations
- Orphaned branch detection when deleting worktrees
- Repository URL validation in configuration files
- New test files for batch delete and edit hooks functionality

### Fixed

- All clippy warnings resolved:
  - manual_div_ceil replaced with div_ceil() method
  - manual_unwrap_or patterns simplified
  - needless_borrows in format! macros removed
  - useless_vec replaced with arrays
  - manual_flatten replaced with .flatten() method
- Test failures related to parent directory configuration search
- ESC cancellation pattern tests updated for new code style
- Worktree rename test expectations aligned with Git limitations
- "In subdirectory" option now correctly creates worktrees in worktrees/ folder
- Path display now shows clean canonical paths without "../"
- Batch delete now properly deletes orphaned branches
- Edit hooks no longer incorrectly identifies regular repos as bare

### Documentation

- Updated README.md with current features and usage:
  - Added configuration file lookup priority documentation
  - Updated worktree pattern examples
  - Added custom path creation examples
  - Added repository URL configuration example
  - Clarified batch delete branch deletion functionality
- Enhanced CLAUDE.md with architectural details and development commands
- Added detailed inline documentation for all public APIs
- Updated all Rustdoc comments to reflect recent changes

## [0.1.0] - 2024-12-17

### Added

- Initial release of Git Workers
- Interactive menu-driven interface for Git worktree management
- List worktrees with detailed status information (branch, changes, ahead/behind)
- Fuzzy search through worktrees with real-time filtering
- Create new worktrees from branches or HEAD
- Delete single or multiple worktrees with safety checks
- Switch worktrees with automatic directory change via shell integration
- Rename worktrees and optionally their branches
- Cleanup old worktrees by age
- Hook system for lifecycle events (post-create, pre-remove, post-switch)
- Shell integration for Bash and Zsh
- Configuration file support (.git-workers.toml)
- Template variable support in hooks ({{worktree_name}}, {{worktree_path}})
- Worktree pattern detection for organized directory structure
- ESC key support for cancelling operations
- Colored terminal output with theme support
- Progress indicators for long operations
- Homebrew installation support
