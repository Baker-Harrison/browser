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

### GitHub CLI Installation
For Windows systems without admin access, use the portable version:
 
```bash
# Download the latest portable version
curl -L https://github.com/cli/cli/releases/latest/download/gh_*_windows_amd64.zip -o gh.zip
unzip gh.zip -d gh
# Add to PATH or use directly: ./gh/bin/gh.exe
```

For systems with admin access:
```bash
# Using winget
winget install --id GitHub.cli

# Using chocolatey (requires admin)
choco install gh -y
```

Authentication: Always use `gh auth login` with browser flow for security. NEVER use PAT tokens directly in commands or scripts.

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

## Branch Management Rules (CRITICAL)

- **NEVER work directly on main branch** - Always create a feature branch for any work
- **Use correct branch naming** - Main branch is `main`, NOT `master`
- **Always create feature branches** - For any new feature, bug fix, or significant change
- **Feature branch naming** - Use `feature/description` or `fix/description` format
- **Pull requests required** - All changes must be submitted via PR, never directly to main
- **Branch workflow**:
  1. Create feature branch from main
  2. Make changes and test thoroughly
  3. Submit PR for review
  4. Address feedback
  5. Merge via PR after approval
- **No direct commits** - Never push directly to main branch
- **Branch verification** - Always verify you're on the correct branch before making changes
- **Test before pushing** - Always run full test suite locally before pushing:
  - `cargo test --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo build --release --all-features`
  - Only push after all checks pass

## Performance Rules (CRITICAL)

- **Never request redraw in redraw handler** - Calling request_redraw() inside RedrawRequested event creates infinite loop
- **Only redraw on state changes** - Request redraws only when actual changes occur (resize, input, content updates)
- **Event-driven rendering** - Use specific events to trigger renders, not continuous loops
- **Monitor resource usage** - Be aware of CPU/GPU implications of rendering loops

## Error Handling Rules (CRITICAL)

- **Never use .expect() in production code** - Use proper error handling with Result types
- **Propagate errors gracefully** - Allow errors to bubble up through the call stack
- **Avoid panics on recoverable errors** - Exit gracefully and report errors instead of crashing
- **Use event_loop.exit() for fatal errors** - In event loops, exit gracefully instead of panicking
- **Report errors to user** - Provide meaningful error messages for debugging

## Security Rules (CRITICAL)

- **NEVER use PAT tokens directly** - Never pass Personal Access Tokens as command-line arguments or in scripts
- **Use secure authentication** - Always use interactive authentication flows (like `gh auth login`)
- **No credentials in code** - Never commit API keys, tokens, or passwords to the repository
- **Use environment variables** - Store sensitive credentials in environment variables or secret managers
- **Token rotation** - Regularly rotate access tokens and credentials
- **Audit dependencies** - Run `cargo audit` regularly to check for security vulnerabilities

## Parallel Agent Workflow (HOW TO GO FAST)

Building this browser fast requires running multiple agents in parallel on
independent subsystems. Follow this workflow:

### The Golden Rule
**Agents work against interfaces, not against each other.**
All subsystem boundaries are defined as Rust traits in `INTERFACES.md`.
Read that file before starting any new subsystem. Never change a trait
without updating `INTERFACES.md` first.

### How to assign parallel work
1. Read `INTERFACES.md` to find subsystems marked "NOT YET BUILT"
2. Pick two or more from the same priority tier (P1, P2, etc.)
3. Assign each to a separate agent with this prompt template:

```
You are implementing the [SUBSYSTEM] for a Rust browser built from scratch.
Read INTERFACES.md for the trait you must satisfy.
Read AGENTS.md for project rules.
Create branch feature/[subsystem-name] from main.
Write the implementation + tests. Do NOT modify any other subsystem.
Run cargo test, cargo clippy -- -D warnings, cargo fmt --check before pushing.
Submit a PR when done.
```

### What can be parallelized right now (P1)
- `feature/html-parser` — real HTML5 tokenizer state machine (`src/html/`)
- `feature/css-parser` — CSS tokenizer + rule parser (`src/css/`)
- `feature/network-raw-response` — refactor `HttpClient` to return `RawResponse`
- `feature/font-rendering` — integrate `fontdue` crate for text rendering

### What must be sequential
- Layout engine needs style engine needs HTML + CSS parsers
- Compositor integration needs paint layer needs layout
- JS engine needs a working DOM

### Agent task sizing
- Each agent task should be completable in one session (~200-500 lines of Rust)
- If a task is larger, split it: e.g. "HTML tokenizer" and "HTML tree builder" are two tasks
- Include acceptance criteria (which existing tests must pass, what new tests to write)

### Integration
- Each agent works on its own branch
- Merges happen in priority order (all P1 before any P2)
- Integration tests live in `tests/integration/` and are written when two subsystems are first connected

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