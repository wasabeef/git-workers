# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

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

# Run with logging enabled
RUST_LOG=debug cargo run
RUST_LOG=git_workers=trace cargo run
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

# Run all checks (using bun if available)
bun run check

# Coverage report (requires cargo-llvm-cov)
cargo llvm-cov --html --lib --ignore-filename-regex '(tests/|src/main\.rs|src/bin/)' --open
```

### Commit Conventions

- Follow `Conventional Commits` for all commit messages
- Format: `<type>(<scope>)?: <description>`
- Common types in this repository:
  - `feat`: user-facing feature additions
  - `fix`: bug fixes and behavior corrections
  - `refactor`: structural changes without behavior changes
  - `test`: test additions or test-only refactors
  - `docs`: documentation-only changes
  - `chore`: maintenance work with no product behavior impact
  - `ci`: CI or automation workflow changes
  - `build`: build system or dependency management changes
- Keep the subject concise, imperative, and lowercase where natural
- Do not mix structural changes and behavior changes in the same commit
- Examples:
  - `refactor(app): move menu dispatch into app module`
  - `fix(create): preserve selected tag when creating worktree`
  - `test(rename): cover cancel flow in rename prompt`

### Installation

```bash
# Install locally from source
cargo install --path .

# Setup shell integration
./setup.sh

# Or manually add to ~/.bashrc or ~/.zshrc:
source /path/to/git-workers/shell/gw.sh
```

## Current Focus Areas

- Interactive worktree operations are driven from the `app` layer and delegated into `usecases`
- Existing public paths such as `commands`, `infrastructure`, and `repository_info` are kept as compatibility facades
- The project supports shell-assisted directory switching, lifecycle hooks, file copying, tag-based creation, and validated custom paths
- Current refactoring policy prioritizes preserving observable behavior over aggressively removing compatibility layers

## Architecture

### Core Module Structure

```
src/
├── main.rs                 # Thin CLI entry point (`--version` + app startup)
├── lib.rs                  # Public module exports and backward-compatible re-exports
├── app/                    # Menu loop, action dispatch, presenter helpers
├── usecases/               # Main worktree operations (create/delete/list/rename/switch/search)
├── adapters/               # Config, shell, filesystem, Git, UI, and hook adapters
├── domain/                 # Repository context and domain-level helpers
├── commands/               # Backward-compatible facades over usecases
├── config.rs               # Configuration model and access helpers
├── repository_info.rs      # Backward-compatible facade for repo context display
├── infrastructure/         # Backward-compatible exports for older module paths
├── core/                   # Legacy core logic retained during migration
├── ui.rs                   # User interface abstraction used by prompts and tests
├── input_esc_raw.rs        # ESC-aware input helpers
├── constants.rs            # Centralized strings and formatting constants
├── support/                # Terminal and styling support utilities
└── utils.rs                # Shared utilities and compatibility helpers
```

### Dependency Direction

- `main` -> `app`
- `app` -> `usecases`
- `usecases` -> `adapters`, `domain`, `ui`, `config`, `infrastructure`
- `commands` and `repository_info` should stay thin and delegate to the newer modules
- Public compatibility paths are intentionally preserved unless a breaking change is explicitly planned

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

1. **Same level as repository**: `../worktree-name` - Creates worktrees as siblings to the repository
2. **Custom path**: User specifies any relative path (e.g., `main`, `branches/feature`, `worktrees/name`)

For bare repositories with `.bare` pattern, use custom path to create worktrees inside the project directory:
- Custom path: `main` → `my-project/main/`
- Custom path: `feature-1` → `my-project/feature-1/`

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

- The repository currently has 51 test files across `unit`, `integration`, `e2e`, and `performance`
- The safest full verification command is `cargo test --all-features -- --test-threads=1`
- `cargo fmt --check` and `cargo clippy --all-features -- -D warnings` are expected before shipping significant changes
- Some tests remain sensitive to parallel execution because they manipulate Git repositories and process-wide state
- Use `--nocapture` when debugging interactive or repository-context behavior

### Common Error Patterns and Solutions

1. **"Permission denied" when running tests**: Tests create temporary directories; ensure proper permissions
2. **"Repository not found" errors**: Tests require git to be configured (`git config --global user.name/email`)
3. **Flaky test failures**: Use `--test-threads=1` to avoid race conditions in worktree operations
4. **"Lock file exists" errors**: Clean up `.git/git-workers-worktree.lock` if tests are interrupted

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
- The CLI is primarily interactive, with `--version` as the supported non-interactive flag

### Configuration Loading Priority

**Bare repositories:**

- Check main/master worktree directories only

**Non-bare repositories:**

1. Current directory (current worktree)
2. Main/master worktree directories (fallback)

### File Copy Behavior

Ignored files such as `.env` can be copied into new worktrees through `.git-workers.toml`.

```toml
[files]
copy = [".env", ".env.local", "config/local.json"]
# source = "path/to/source"
```

- Copy runs after worktree creation and before post-create hooks
- Missing source files warn but do not abort worktree creation
- Paths are validated to prevent traversal and invalid destinations
- File handling includes symlink checks, depth limits, and permission preservation where applicable

## CI/CD and Tooling

- `.github/workflows/ci.yml` runs the main validation pipeline
- `.github/workflows/release.yml` handles release automation
- `lefthook.yml` runs pre-commit checks such as `fmt` and `clippy`
- `package.json` provides helper scripts:
  - `bun run format`
  - `bun run lint`
  - `bun run test`
  - `bun run check`

## Key Implementation Patterns

### Git Operations

The codebase uses two approaches for Git operations:

1. **git2 library**: For read operations (listing branches, getting commit info)
2. **std::process::Command**: For write operations (worktree add/remove) to ensure compatibility

Example pattern:

```rust
// Read operation using git2
let repo = Repository::open(".")?;
let branches = repo.branches(Some(BranchType::Local))?;

// Write operation using Command
Command::new("git")
    .args(&["worktree", "add", path, branch])
    .output()?;
```

### Error Handling Philosophy

- Use `anyhow::Result` for application-level errors
- Provide context with `.context()` for better error messages
- Show user-friendly messages via `utils::display_error()`
- Never panic in production code; handle all error cases gracefully

### UI Abstraction

The `ui::UserInterface` trait enables testing of interactive features:

- Mock implementations for tests
- Real implementation wraps dialoguer
- All user interactions go through this abstraction
