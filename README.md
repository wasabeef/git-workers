# Git Workers

An interactive CLI tool for managing Git worktrees with ease.

[![CI](https://github.com/wasabeef/git-workers/actions/workflows/ci.yml/badge.svg)](https://github.com/wasabeef/git-workers/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Features

- 📋 List worktrees with status information
- 🔍 Fuzzy search through worktrees
- ➕ Create new worktrees from branches or HEAD
- ➖ Delete single or multiple worktrees
- 🔄 Switch worktrees with automatic directory change
- ✏️ Rename worktrees and optionally their branches
- 🧹 Cleanup old worktrees by age
- 🪝 Execute hooks on worktree lifecycle events

## Installation

### Homebrew

```bash
brew install wasabeef/gw-tap/gw
```

### Shell Integration

To enable automatic directory switching when switching worktrees, add this to your shell config:

```bash
# For bash (~/.bashrc) or zsh (~/.zshrc)
source $(brew --prefix)/share/gw/shell/gw.sh
```

## Usage

Run `gw` in any Git repository:

```bash
gw
```

### Menu Options

- **List worktrees** (`•`): Display all worktrees with status information
- **Search worktrees** (`?`): Fuzzy search through worktrees
- **Create worktree** (`+`): Create a new worktree
- **Delete worktree** (`-`): Delete a single worktree
- **Batch delete** (`=`): Delete multiple worktrees at once
- **Cleanup old worktrees** (`~`): Remove worktrees older than specified days
- **Switch worktree** (`→`): Switch to another worktree (changes directory)
- **Rename worktree** (`*`): Rename an existing worktree
- **Exit** (`x`): Exit the application

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
