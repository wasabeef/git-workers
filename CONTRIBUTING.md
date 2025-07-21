# Contributing to Git Workers

First off, thank you for considering contributing to Git Workers! It's people like you that make Git Workers such a great tool.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or bug fix
4. Make your changes
5. Push to your fork and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70+
- Git 2.20+
- macOS or Linux (Windows not supported)

### Setup

```bash
# Clone your fork
git clone https://github.com/yourusername/git-workers.git
cd git-workers

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

### Building from Source

```bash
# Development build
cargo build

# Release build
cargo build --release

# Install locally
cargo install --path .

# Run with logging
RUST_LOG=debug cargo run
```

### Recommended Tools

- **rustfmt**: For code formatting

  ```bash
  rustup component add rustfmt
  ```

- **clippy**: For linting

  ```bash
  rustup component add clippy
  ```

## Project Structure

```
git-workers/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands.rs          # Command implementations
│   ├── git.rs              # Git worktree operations
│   ├── menu.rs             # Menu definitions
│   ├── config.rs           # Configuration management
│   ├── hooks.rs            # Hook system
│   ├── repository_info.rs  # Repository information display
│   ├── input_esc_raw.rs    # Custom input handling with ESC support
│   └── utils.rs            # Utility functions
├── shell/
│   └── gw.sh               # Shell integration script
├── tests/                  # Integration tests
├── .github/
│   ├── workflows/          # CI/CD workflows
│   └── ISSUE_TEMPLATE/     # Issue templates
├── Cargo.toml              # Project dependencies
├── lefthook.yml            # Git hooks configuration
└── README.md               # Project documentation
```

## Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md)
- Use `cargo fmt` before committing
- Run `cargo clippy` and address any warnings
- Write idiomatic Rust code

### Code Organization

- Keep functions small and focused
- Use descriptive variable and function names
- Add comments for complex logic
- Group related functionality in modules

### Error Handling

- Use `Result<T, E>` for operations that can fail
- Provide meaningful error messages
- Use `anyhow` for error handling in application code
- Use specific error types in library code

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
   - Located in `src/*.rs` files within `#[cfg(test)]` modules
2. **Integration Tests**: Test the entire application flow
   - Located in `tests/` directory
3. **Manual Tests**: For interactive features
   - Documented in `tests/manual_test_guide.md`

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Writing Tests

- Write tests for new features
- Update tests when modifying existing functionality
- Aim for high test coverage
- Test edge cases and error conditions

## Pull Request Process

1. **Before submitting**:
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`
   - Update documentation if needed
   - Add tests for new functionality
2. **PR Guidelines**:
   - Use a clear and descriptive title
   - Reference any related issues
   - Provide a detailed description of changes
   - Include screenshots for UI changes
3. **PR Template**:

   ```markdown
   ## Description

   Brief description of changes

   ## Type of Change

   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing

   - [ ] Tests pass locally
   - [ ] Added new tests
   - [ ] Manual testing completed

   ## Checklist

   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] No new warnings
   ```

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- Git Workers version (`gw --version`)
- Operating system and version
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages (if any)

### Feature Requests

For feature requests, please include:

- Use case description
- Expected behavior
- Why this feature would be useful
- Any implementation suggestions

## Development Workflow

1. **Pick an issue**: Look for issues labeled `good first issue` or `help wanted`
2. **Discuss**: Comment on the issue before starting work
3. **Branch**: Create a feature branch from `main`
4. **Develop**: Make your changes following the coding standards
5. **Test**: Ensure all tests pass
6. **Commit**: Use clear, conventional commit messages
7. **Push**: Push to your fork
8. **PR**: Open a pull request

## Commit Message Convention

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks

Example:

```
feat(worktree): add fuzzy search functionality

Implement fuzzy search for worktree names and branches using
skim algorithm. This allows users to quickly find worktrees
by typing partial names.

Closes #123
```

## Questions?

Feel free to open an issue for any questions about contributing!
