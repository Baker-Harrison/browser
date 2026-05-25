# Contributing to Browser

Thank you for your interest in contributing to the Browser project! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this standard. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- Make (optional, but recommended for common tasks)

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/browser.git
   cd browser
   ```
3. Install Rust toolchain:
   ```bash
   rustup install stable
   rustup default stable
   ```
4. Install development tools:
   ```bash
   cargo install cargo-tarpaulin cargo-audit
   pre-commit install
   ```
5. Build the project:
   ```bash
   make build
   ```

## Development Workflow

### Branch Strategy

- `main` - Production-ready code, always stable
- `develop` - Integration branch for features
- Feature branches - Created from `develop` for specific features

### Creating a Feature Branch

1. Ensure your local `main` is up to date:
   ```bash
   git checkout main
   git pull origin main
   ```
2. Create a new feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Making Changes

1. Make your changes following the [Coding Standards](#coding-standards)
2. Run formatting and linting:
   ```bash
   make fmt
   make clippy
   ```
3. Run tests:
   ```bash
   make test
   ```
4. Run security audit:
   ```bash
   make audit
   ```

### Committing Changes

Follow the commit message guidelines:
- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add rendering engine component

Implement basic HTML parsing and CSS styling engine.
This adds the foundation for the rendering subsystem.

Fixes #123
```

### Syncing with Upstream

Regularly sync your fork with the upstream repository:
```bash
git fetch upstream
git rebase upstream/main
```

## Coding Standards

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Write documentation for all public APIs
- Include examples in documentation

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Prefer composition over inheritance
- Minimize dependencies between components
- Follow the component structure defined in AGENTS.md

### Documentation

- Document all public APIs with `///` doc comments
- Include usage examples in documentation
- Keep AGENTS.md updated with architectural decisions
- Add comments for complex logic

## Testing

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-coverage

# Run specific test
cargo test test_name

# Run tests in release mode
cargo test --release
```

### Writing Tests

- Write unit tests for all new functions
- Write integration tests for component interactions
- Use property-based testing for algorithms (proptest)
- Aim for high code coverage
- Test edge cases and error conditions

### Benchmarking

```bash
# Run benchmarks
make benchmark
```

## Submitting Changes

### Pull Request Process

1. Update your branch with the latest from `main`:
   ```bash
   git fetch origin
   git rebase origin/main
   ```
2. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
3. Create a pull request on GitHub
4. Fill out the PR template
5. Wait for CI checks to pass
6. Address review feedback
7. Maintain your PR (keep it up to date)

### Pull Request Guidelines

- Keep PRs focused and small
- Include tests for new functionality
- Update documentation
- Ensure all CI checks pass
- Respond to review comments promptly
- Squash commits before final merge

### Code Review

- Be respectful and constructive
- Focus on the code, not the person
- Explain the reasoning behind suggestions
- Accept feedback gracefully
- Ask for clarification if needed

## Reporting Issues

### Bug Reports

When reporting a bug, include:
- Clear description of the problem
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment details (OS, Rust version)
- Relevant logs or screenshots

### Feature Requests

When requesting a feature, include:
- Clear description of the feature
- Use case or motivation
- Potential implementation approach
- Any relevant examples or references

### Security Issues

**Do not report security issues publicly.** See [SECURITY.md](SECURITY.md) for the security reporting process.

## Getting Help

- Check existing issues and discussions
- Read the documentation in AGENTS.md
- Ask questions in GitHub Discussions
- Join our community chat (if available)

## Recognition

Contributors will be recognized in the project's CONTRIBUTORS file and release notes.

Thank you for contributing to Browser!
