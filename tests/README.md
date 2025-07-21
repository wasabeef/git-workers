# Test Structure

This directory contains all tests for the Git Workers project, organized by test type and module.

## Directory Structure

```
tests/
├── unit/               # Unit tests for individual modules
│   ├── commands/       # Tests for command implementations
│   ├── core/          # Tests for core business logic
│   ├── infrastructure/ # Tests for external integrations (git, fs, etc.)
│   └── ui/            # Tests for UI components and interactions
├── integration/        # Integration tests for multi-component scenarios
├── e2e/               # End-to-end tests for complete workflows
└── performance/       # Performance benchmarks and tests
```

## Test Categories

### Unit Tests (`unit/`)

Unit tests focus on testing individual modules in isolation with mocked dependencies.

- **commands/**: Tests for each command (create, delete, rename, switch, etc.)
- **core/**: Tests for business logic (validation, worktree patterns, etc.)
- **infrastructure/**: Tests for external integrations (git operations, file system, hooks)
- **ui/**: Tests for UI components (menu, input handling, display)

### Integration Tests (`integration/`)

Integration tests verify that multiple components work correctly together.

- `git_flow.rs`: Tests for git-flow style workflows
- `multi_repo.rs`: Tests for handling multiple repositories
- `worktree_lifecycle.rs`: Tests for complete worktree lifecycle

### End-to-End Tests (`e2e/`)

E2E tests simulate real user workflows from start to finish.

- `workflow.rs`: Complete user workflow tests

### Performance Tests (`performance/`)

Performance tests measure and benchmark critical operations.

- `benchmark.rs`: Performance benchmarks for key operations

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test --test unit
cargo test --test integration
cargo test --test e2e

# Run tests for specific module
cargo test --test unit::commands

# Run with single thread (for flaky tests)
cargo test -- --test-threads=1

# Run with output
cargo test -- --nocapture
```

## Test Guidelines

1. **Unit tests** should be fast and isolated, using mocks for external dependencies
2. **Integration tests** can use real git repositories but should clean up after themselves
3. **E2E tests** should simulate realistic user scenarios
4. **Performance tests** should establish baseline metrics and detect regressions

## Adding New Tests

When adding new tests:

1. Place them in the appropriate directory based on test type
2. Follow the existing naming conventions
3. Use descriptive test names that explain what is being tested
4. Clean up any temporary resources (files, directories, git repos) after tests
5. Mark flaky tests with `#[ignore]` and document why they're flaky
