# Project Purpose

The purpose of this project is to build a full browser from scratch. It is a cross-platform browser that will be available for multiple operating systems.

# Architecture Decisions

## Core Technology Stack
- **Primary Language**: Rust - chosen for performance, memory safety, and modern tooling
- **Rendering Engine**: Built from scratch - complete control over HTML/CSS/JS rendering
- **Architecture Pattern**: Component-based - clear separation between rendering, networking, UI, and other subsystems

## Component Structure
The browser will be organized into distinct components:
- **Rendering Engine**: HTML parsing, CSS styling, layout, and painting
- **JavaScript Engine**: JS execution and runtime
- **Networking Stack**: HTTP/HTTPS, WebSocket, and protocol handling
- **UI Framework**: Tab management, bookmarks, settings, and browser chrome
- **Security Model**: Same-origin policy, sandboxing, and security boundaries
- **Storage Engine**: Cookies, cache, history, and local storage
- **Extension System**: Plugin architecture for browser extensions

## Development Principles
- Each component should have clear interfaces and minimal dependencies
- Prefer composition over inheritance
- Write tests alongside implementation
- Document architectural decisions in this file
- Follow Rust best practices and idioms

## Development Tooling
The project uses the following development tooling:

### Code Quality
- **rustfmt**: Automatic code formatting (configured in rustfmt.toml)
- **clippy**: Rust linter for catching common mistakes (configured in clippy.toml)
- **pre-commit hooks**: Automated checks before commits (configured in .pre-commit-config.yaml)

### Testing & Quality
- **cargo test**: Unit and integration tests
- **criterion**: Benchmarking framework
- **proptest**: Property-based testing
- **cargo-tarpaulin**: Code coverage (install with: cargo install cargo-tarpaulin)
- **cargo-audit**: Security vulnerability scanning (install with: cargo install cargo-audit)

### Build Automation
- **Makefile**: Common development commands (run `make help` for available commands)
- **GitHub Actions CI**: Automated testing on Ubuntu, MacOS, and Windows

### Available Commands
- `make build` - Build the project
- `make test` - Run all tests
- `make fmt` - Format code
- `make clippy` - Run linter
- `make check` - Run cargo check
- `make benchmark` - Run benchmarks
- `make install-hooks` - Install pre-commit hooks
- `make audit` - Security audit dependencies
- `make doc` - Generate and open documentation


## Project rules

- commit often
- review your code before committing
- write tests for all new code
- document all public APIs
- keep changes small and focused
- follow the existing code style
- use meaningful commit messages
- rebase often to keep history clean
- squash commits before merging
- never ever make code that is not what we want in the future. for example, making an interface in the cli when the browser is supposed to be a full desktop interface

## Open Source Workflow

This project follows a structured open source development workflow:

### Branch Strategy
- `main` - Production-ready code, always stable
- `develop` - Integration branch for features (to be created)
- Feature branches - Created from `develop` for specific features

### Development Process
1. Create feature branches from `develop`
2. Make changes following coding standards
3. Run tests, formatting, and linting before committing
4. Submit pull requests for review
5. Ensure all CI checks pass
6. Address review feedback
7. Squash commits before final merge

### Documentation
- See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines
- See [SECURITY.md](SECURITY.md) for security vulnerability reporting
- Use GitHub issue templates for bug reports and feature requests
- Follow the pull request template when submitting changes

### Quality Gates
- All CI checks must pass (tests, formatting, linting, security audit)
- Code coverage should be maintained or improved
- Documentation must be updated for public API changes
- Security vulnerabilities must be addressed before merging