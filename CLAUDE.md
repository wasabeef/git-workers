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

### Branch Option Simplification
- Reduced from 3 options to 2: "Create from current HEAD" and "Select branch (smart mode)"
- Smart mode automatically handles branch conflicts and offers appropriate actions

### Key Methods Added/Modified
- **`get_branch_worktree_map()`**: Maps branch names to worktree names, including main worktree detection
- **`list_all_branches()`**: Returns both local and remote branches (remote without "origin/" prefix)
- **`create_worktree_with_new_branch()`**: Creates worktree with new branch from base branch (supports git-flow style workflows)

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

- Integration tests in `tests/` directory (27 test files)
- Some tests are flaky in parallel execution (marked with `#[ignore]`)
- CI sets `CI=true` environment variable to skip flaky tests
- Run with `--test-threads=1` for reliable results
- Use `--nocapture` to see test output for debugging

### Important Constraints

- Only works within Git repositories
- Requires initial commit (bare repositories supported)
- Cannot rename current worktree
- Cannot rename worktrees with detached HEAD
- Shell integration supports Bash/Zsh only
- No Windows support (macOS and Linux only)
- Recent breaking change: CLI arguments removed in favor of menu-only interface
