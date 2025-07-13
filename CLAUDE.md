# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Git Workers is an interactive CLI tool for managing Git worktrees, written in Rust. It provides a menu-driven interface for creating, deleting, switching, and renaming worktrees, with shell integration for automatic directory switching.

## Development Commands

### Build and Run

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run directly (development)
cargo run

# Run the binary
./target/debug/gw
./target/release/gw

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests single-threaded (for flaky tests)
cargo test -- --test-threads=1

# Run tests with output for debugging
cargo test test_name -- --nocapture
```

### Quality Checks

```bash
# Format check and apply
cargo fmt --check
cargo fmt

# Clippy (linter)
cargo clippy --all-features -- -D warnings

# Type check
cargo check --all-features

# Generate documentation
cargo doc --no-deps --open
```

### Installation

```bash
# Install locally from source
cargo install --path .

# Setup shell integration
./setup.sh

# Or manually add to ~/.bashrc or ~/.zshrc:
source /path/to/git-workers/shell/gw.sh
```

## Recent Changes

### v0.3.0 File Copy Feature

- Automatically copy gitignored files (like `.env`) from main worktree to new worktrees
- Configurable via `[files]` section in `.git-workers.toml`
- Security validation to prevent path traversal attacks
- Follows same discovery priority as configuration files

### Branch Option Enhancement

- Enhanced from 2 options to 3: "Create from current HEAD", "Select branch", and "Select tag"
- Branch selection automatically handles conflicts and offers appropriate actions
- Tag selection allows creating worktrees from specific versions

### Custom Path Support

- Added third option for first worktree creation: "Custom path (specify relative to project root)"
- Allows users to specify arbitrary relative paths for worktree creation
- Comprehensive path validation with security checks:
  - Prevents absolute paths
  - Validates against filesystem-incompatible characters
  - Blocks git reserved names in path components
  - Prevents excessive path traversal (max one level above project root)
  - Cross-platform compatibility checks

### Key Methods Added/Modified

- **`get_branch_worktree_map()`**: Maps branch names to worktree names, including main worktree detection
- **`list_all_branches()`**: Returns both local and remote branches (remote without "origin/" prefix)
- **`list_all_tags()`**: Returns all tags with optional messages for annotated tags
- **`create_worktree_with_new_branch()`**: Creates worktree with new branch from base branch (supports git-flow style workflows)
- **`create_worktree_with_branch()`**: Enhanced to handle tag references for creating worktrees at specific versions
- **`copy_configured_files()`**: Copies files specified in config to new worktrees
- **`create_worktree_from_head()`**: Fixed path resolution for non-bare repositories (converts relative paths to absolute)
- **`validate_custom_path()`**: Validates custom paths for security and compatibility
- **`create_worktree_internal()`**: Enhanced with custom path input option and tag selection

## Architecture

### Core Module Structure

```
src/
├── main.rs              # CLI entry point and main menu loop
├── lib.rs               # Library exports
├── commands.rs          # Command implementations for menu items
├── git.rs               # Git worktree operations (git2 + process::Command)
├── menu.rs              # MenuItem enum and icon definitions
├── config.rs            # .git-workers.toml configuration management
├── hooks.rs             # Hook system (post-create, pre-remove, etc.)
├── repository_info.rs   # Repository information display
├── input_esc_raw.rs     # Custom input handling with ESC support
├── constants.rs         # Centralized constants (strings, formatting)
├── file_copy.rs         # File copy functionality for gitignored files
└── utils.rs             # Common utilities (error display, etc.)
```

### Technology Stack

- **dialoguer + console**: Interactive CLI (Select, Confirm, Input prompts)
- **git2**: Git repository operations (branch listing, commit info)
- **std::process::Command**: Git CLI invocation (worktree add/prune)
- **colored**: Terminal output coloring
- **fuzzy-matcher**: Worktree search functionality
- **indicatif**: Progress bar display

### Shell Integration System

Automatic directory switching on worktree change requires special implementation due to Unix process restrictions:

1. Binary writes path to file specified by `GW_SWITCH_FILE` env var
2. Shell function (`shell/gw.sh`) reads the file and executes `cd`
3. Legacy fallback: `SWITCH_TO:/path` marker on stdout

### Hook System Design

Define lifecycle hooks in `.git-workers.toml`:

```toml
[hooks]
post-create = ["npm install", "cp .env.example .env"]
pre-remove = ["rm -rf node_modules"]
post-switch = ["echo 'Switched to {{worktree_name}}'"]
```

Template variables:

- `{{worktree_name}}`: The worktree name
- `{{worktree_path}}`: Absolute path to worktree

### Worktree Patterns

First worktree creation offers two options:

1. Same level as repository: `../worktree-name`
2. In subdirectory (recommended): `../repo/worktrees/worktree-name`

Subsequent worktrees follow the established pattern automatically.

### ESC Key Handling

All interactive prompts support ESC cancellation through custom `input_esc_raw` module:

- `input_esc_raw()` returns `Option<String>` (None on ESC)
- `Select::interact_opt()` for menu selections
- `Confirm::interact_opt()` for confirmations

### Worktree Rename Implementation

Since Git lacks native rename functionality:

1. Move directory with `fs::rename`
2. Update `.git/worktrees/<name>` metadata directory
3. Update gitdir files in both directions
4. Optionally rename associated branch if it matches worktree name

### CI/CD Configuration

- **GitHub Actions**: `.github/workflows/ci.yml` (test, lint, build)
- **Release workflow**: `.github/workflows/release.yml` (automated releases)
- **Homebrew tap**: Updates `wasabeef/homebrew-gw-tap` on release
- **Pre-commit hooks**: `lefthook.yml` (format, clippy)

### Testing Considerations

- Integration tests in `tests/` directory (30 test files)
- Some tests are flaky in parallel execution (marked with `#[ignore]`)
- CI sets `CI=true` environment variable to skip flaky tests
- Run with `--test-threads=1` for reliable results
- Use `--nocapture` to see test output for debugging
- New test files added:
  - `worktree_path_test.rs`: Tests for path resolution and edge cases
  - `create_worktree_integration_test.rs`: Integration tests for worktree creation
  - `create_worktree_from_tag_test.rs`: Tests for tag listing and worktree creation from tags

### String Formatting

- **ALWAYS use inline variable syntax in format! macros**: `format!("{variable}")` instead of `format!("{}", variable)`
- This applies to ALL format-like macros: `format!`, `println!`, `eprintln!`, `log::info!`, `log::warn!`, `log::error!`, etc.
- Examples:

  ```rust
  // ✅ Correct
  format!("Device {name} created successfully")
  println!("Found {count} devices")
  log::info!("Starting device {identifier}")

  // ❌ Incorrect
  format!("Device {} created successfully", name)
  println!("Found {} devices", count)
  log::info!("Starting device {}", identifier)
  ```

- This rule is enforced by `clippy::uninlined_format_args` which treats violations as errors in CI
- Apply this consistently across ALL files including main source, tests, examples, and binary targets

### Important Constraints

- Only works within Git repositories
- Requires initial commit (bare repositories supported)
- Cannot rename current worktree
- Cannot rename worktrees with detached HEAD
- Shell integration supports Bash/Zsh only
- No Windows support (macOS and Linux only)
- Recent breaking change: CLI arguments removed in favor of menu-only interface

### Configuration Loading Priority

**Bare repositories:**

- Check main/master worktree directories only

**Non-bare repositories:**

1. Current directory (current worktree)
2. Main/master worktree directories (fallback)

## v0.3.0 File Copy Feature (Implemented)

### Overview

Automatically copy ignored files (like `.env`) from main worktree to new worktrees during creation.

### Configuration

```toml
[files]
# Files to copy when creating new worktrees
copy = [".env", ".env.local", "config/local.json"]

# Optional: source directory (defaults to main worktree)
# source = "path/to/source"
```

### Implementation Details

1. **Config Structure**: `FilesConfig` struct with `copy` and `source` fields (destination is always worktree root)
2. **File Detection**: Uses same priority as config file discovery for finding source files
3. **Copy Logic**: Executes after worktree creation but before post-create hooks
4. **Error Handling**: Warns on missing files but continues with worktree creation
5. **Security**: Validates paths to prevent directory traversal attacks
6. **Features**:
   - Supports both files and directories
   - Recursive directory copying
   - Symlink detection with warnings
   - Maximum directory depth limit (50 levels)
   - Preserves file permissions

## Bug Fixes

### v0.3.0 Worktree Creation Path Resolution

Fixed an issue where creating worktrees from HEAD in non-bare repositories could fail when using relative paths like `../worktree-name`. The fix ensures that relative paths are resolved from the current working directory rather than from the git directory.

**Root Cause**: The `git worktree add` command was being executed with `current_dir` set to the git directory, causing relative paths to be interpreted incorrectly.

### v0.3.0 Security and Robustness Improvements

#### Worktree Name Validation

Added comprehensive validation for worktree names to prevent issues:

- **Invalid Characters**: Rejects filesystem-incompatible characters (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, `\0`)
- **Reserved Names**: Prevents conflicts with Git internals (`.git`, `HEAD`, `refs`, etc.)
- **Non-ASCII Warning**: Warns users about potential compatibility issues with non-ASCII characters
- **Length Limits**: Enforces 255-character maximum for filesystem compatibility
- **Hidden Files**: Prevents names starting with `.` to avoid hidden file conflicts

#### File Copy Size Limits

Enhanced file copy functionality with safety checks:

- **Large File Skipping**: Automatically skips files larger than 100MB with warnings
- **Performance Protection**: Prevents accidental copying of build artifacts or large binaries
- **User Feedback**: Clear warnings when files are skipped due to size

#### Concurrent Access Control

Implemented file-based locking to prevent race conditions:

- **Process Locking**: Uses `.git/git-workers-worktree.lock` to prevent concurrent worktree creation
- **Stale Lock Cleanup**: Automatically removes locks older than 5 minutes
- **Error Messages**: Clear feedback when another process is creating worktrees
- **Automatic Cleanup**: Lock files are automatically removed when operations complete

#### Custom Path Validation

Added comprehensive validation for user-specified worktree paths:

- **Path Security**: Validates against path traversal attacks and excessive directory navigation
- **Cross-Platform Compatibility**: Checks for Windows reserved characters even on non-Windows systems
- **Git Reserved Names**: Prevents conflicts with git internal directories in path components
- **Path Format Validation**: Ensures proper relative path format (no absolute paths, no trailing slashes)

**Solution**: Convert relative paths to absolute paths before passing them to the git command, ensuring consistent behavior regardless of the working directory.

## Test Coverage and CI Integration

### Test File Consolidation (v0.5.1+)

Major test restructuring completed to improve maintainability and reduce duplication:

- **File Reduction**: Consolidated from 64 to 40 test files
- **Unified Structure**: Created `unified_*_comprehensive_test.rs` files grouping related functionality
- **Duplication Removal**: Eliminated 15+ duplicate test cases
- **Comment Translation**: Converted all Japanese comments to English for consistency

### CI/CD Configuration

**GitHub Actions Workflows:**

- `.github/workflows/ci.yml`: Comprehensive test, lint, build, and coverage analysis
- `.github/workflows/release.yml`: Automated releases with Homebrew tap updates

**Pre-commit Hooks (lefthook.yml):**

```yaml
pre-commit:
  parallel: false
  commands:
    fmt:
      glob: '*.rs'
      run: cargo fmt --all
      stage_fixed: true
    clippy:
      glob: '*.rs'
      run: cargo clippy --all-targets --all-features -- -D warnings
```

**Test Configuration:**

- Single-threaded execution (`--test-threads=1`) to prevent race conditions
- CI environment variable automatically set for non-interactive test execution
- Coverage analysis with `cargo-tarpaulin` including proper concurrency control

### Package Management Integration

**Bun Integration (package.json):**

```json
{
  "scripts": {
    "test": "bun ./scripts/run-tests.js",
    "format": "cargo fmt --all && prettier --write .",
    "lint": "cargo clippy --all-targets --all-features -- -D warnings",
    "check": "bun run format && bun run lint && bun run test"
  }
}
```

**Test Runner Scripts:**

- `scripts/run-tests.js`: Bun-compatible test wrapper with proper exit handling
- `scripts/test.sh`: Bash fallback for direct cargo test execution

### Test Structure

**Unified Test Files (40 total):**

- `unified_*_comprehensive_test.rs`: Consolidated functionality tests
- `api_contract_basic_test.rs`: Contract-based testing
- Security, edge cases, and integration tests with proper error handling

**Coverage Analysis:**

- Single-threaded execution prevents worktree lock conflicts
- Directory restoration with fallback handling for CI environments
- Error handling for temporary directory cleanup

### Test Execution Best Practices

- Use `CI=true` environment variable for non-interactive execution
- Single-threaded execution prevents resource conflicts
- Comprehensive error handling for CI environment limitations
- Automated cleanup of temporary files and directories

### Legacy Test Files (Pre-consolidation)

The following test files were consolidated into unified versions:

- Individual component tests → `unified_*_comprehensive_test.rs`
- Duplicate functionality tests → Removed
- Japanese comments → Translated to English
