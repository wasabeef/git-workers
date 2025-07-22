# Git Workers

[![CI](https://github.com/wasabeef/git-workers/actions/workflows/ci.yml/badge.svg)](https://github.com/wasabeef/git-workers/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

An interactive CLI tool for managing Git worktrees.

<https://github.com/user-attachments/assets/fb5f0213-4a9f-43e2-9557-416070d7e122>

## Features

- List worktrees with detailed status information (branch, changes, ahead/behind)
- Fuzzy search through worktrees and branches
- Create new worktrees from branches, tags, or HEAD
- Automatically copy gitignored files (.env) to new worktrees
- Delete single or multiple worktrees
- Switch worktrees with automatic directory change
- Rename worktrees and optionally their branches
- Cleanup old worktrees by age
- Execute hooks on worktree lifecycle events
- Edit and manage hooks through the interface

## Installation

### Homebrew

```bash
brew install wasabeef/gw-tap/gw
```

### Build from Source

```bash
# Clone and install
git clone https://github.com/wasabeef/git-workers.git
cd git-workers
cargo install --path .
```

### Shell Integration

Add this to your shell config(.bashrc or .zshrc) to enable automatic directory switching when switching worktrees.

```bash
# For bash (~/.bashrc) or zsh (~/.zshrc)
source $(brew --prefix)/share/gw/shell/gw.sh
```

For manual installation, use the path where git-workers is installed:

```bash
source /path/to/git-workers/shell/gw.sh
```

## Usage

Run `gw` in any Git repository:

```bash
gw
```

### Interactive Menu

Git Workers provides an interactive menu-driven interface. Simply run `gw` and navigate through the options:

- List worktrees (`•`) - Display all worktrees with branch, changes, and sync status
- Search worktrees (`?`) - Fuzzy search through worktree names and branches
- Create worktree (`+`) - Create a new worktree with three options:
  - Create from current HEAD - Creates a new worktree with a new branch from the current HEAD
  - Select branch - Choose from local/remote branches with fuzzy search:
    - Shows local branches (💻) and remote branches (⛅️) with usage status
    - Automatically handles branch conflicts (offers to create new branch if already in use)
    - Fuzzy search enabled when >5 branches
  - Select tag - Choose from tags to create a worktree at a specific version:
    - Shows all tags (🏷️) with messages for annotated tags
    - Creates a new branch from the selected tag
    - Fuzzy search enabled when >5 tags
  - Automatically copies configured files (.env, etc.) to new worktrees
- Delete worktree (`-`) - Delete a single worktree with safety checks
- Batch delete (`=`) - Select and delete multiple worktrees at once (optionally deletes orphaned branches)
- Cleanup old worktrees (`~`) - Remove worktrees older than specified days
- Switch worktree (`→`) - Switch to another worktree (automatically changes directory)
- Rename worktree (`*`) - Rename worktree directory and optionally its branch
- Edit hooks (`λ`) - Configure lifecycle hooks in `.git-workers.toml`
- Exit (`x`) - Exit the application

### Configuration

Git Workers uses `.git-workers.toml` for configuration. The loading strategy differs between bare and non-bare repositories:

#### Bare Repositories

For bare repositories (e.g., `repo.git`), configuration is loaded from:

1. Current directory
2. Default branch directory in current directory (e.g., `./main/.git-workers.toml`)
3. Detected worktree pattern (automatically finds where existing worktrees are organized)
4. Common subdirectories (`branch/`, `worktrees/`)

#### Non-bare Repositories

For regular repositories, configuration is loaded from:

1. Current directory
2. Main repository directory (where `.git` is a directory)
3. Parent directories' `main/` or `master/` subdirectories

```toml
[repository]
# Optional: Specify repository URL to ensure hooks only run in the intended repository
# url = "https://github.com/owner/repo.git"

[repository]
# Repository URL for identification (optional)
# This ensures hooks only run in the intended repository
url = "https://github.com/wasabeef/git-workers.git"

[hooks]
# Run after creating a new worktree
post-create = [
    "echo '🤖 Created worktree: {{worktree_name}}'",
    "echo '🤖 Path: {{worktree_path}}'"
]

# Run before removing a worktree
pre-remove = [
    "echo '🤖 Removing worktree: {{worktree_name}}'"
]

# Run after switching to a worktree
post-switch = [
    "echo '🤖 Switched to: {{worktree_name}}'"
]

[files]
# Optional: Specify a custom source directory
# If not specified, automatically finds the main worktree
# source = "/path/to/custom/source"
# source = "./templates"  # Relative to repository root

# Files to copy when creating new worktrees
# These are typically gitignored files needed for development
copy = [
    ".env",
    ".env.local"
]
```

### Hook Variables

- `{{worktree_name}}`: The name of the worktree
- `{{worktree_path}}`: The absolute path to the worktree

### Worktree Patterns

When creating your first worktree, Git Workers offers two patterns:

1. **Same level as repository**: Creates worktrees as siblings to your main repository

   ```
   parent/
   ├── my-repo/
   ├── feature-1/
   └── feature-2/
   ```

2. **In subdirectory** (recommended): Organizes worktrees in a dedicated directory

   ```
   parent/
   └── my-repo/
       └── worktrees/
           ├── feature-1/
           └── feature-2/
   ```

You can also create worktrees with custom paths:

- `../feature`: Creates at the same level as the repository
- `worktrees/feature`: Creates in a subdirectory
- `branch/feature`: Creates in a custom subdirectory structure

### Keyboard Shortcuts

- **ESC**: Cancel current operation and return to menu
- **Ctrl+C**: Exit the application
- **Ctrl+U**: Clear input line
- **Ctrl+W**: Delete word before cursor
- **Arrow Keys**: Navigate menus
- **Enter**: Confirm selection

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [git2](https://github.com/rust-lang/git2-rs) for Git operations
- Uses [dialoguer](https://github.com/console-rs/dialoguer) for interactive prompts
- Terminal styling with [colored](https://github.com/colored-rs/colored)
