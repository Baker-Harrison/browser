# Browser

A modern, cross-platform web browser built from scratch in Rust.

## Overview

This project aims to build a complete web browser from the ground up, focusing on performance, security, and modern web standards. The browser is built entirely in Rust, leveraging the language's memory safety guarantees and performance characteristics.

## Features

- **Cross-platform**: Support for multiple operating systems (Windows, macOS, Linux)
- **Modern Rendering Engine**: Custom-built HTML/CSS/JavaScript rendering engine
- **Security First**: Built-in security features including sandboxing and same-origin policy
- **Extensible**: Plugin architecture for browser extensions
- **Performance**: Optimized for speed and memory efficiency

## Architecture

The browser is organized into distinct components:

- **Rendering Engine**: HTML parsing, CSS styling, layout, and painting
- **JavaScript Engine**: JS execution and runtime
- **Networking Stack**: HTTP/HTTPS, WebSocket, and protocol handling
- **UI Framework**: Tab management, bookmarks, settings, and browser chrome
- **Security Model**: Same-origin policy, sandboxing, and security boundaries
- **Storage Engine**: Cookies, cache, history, and local storage
- **Extension System**: Plugin architecture for browser extensions

See [AGENTS.md](AGENTS.md) for detailed architecture documentation.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- Make (optional, but recommended)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Baker-Harrison/browser.git
   cd browser
   ```

2. Install Rust toolchain:
   ```bash
   rustup install stable
   rustup default stable
   ```

3. Build the project:
   ```bash
   make build
   ```

4. Run the browser:
   ```bash
   make run
   ```

## Development

### Setting Up Development Environment

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development setup instructions.

### Available Commands

```bash
make build          # Build the project
make test           # Run all tests
make fmt            # Format code
make clippy         # Run linter
make check          # Run cargo check
make benchmark      # Run benchmarks
make install-hooks  # Install pre-commit hooks
make audit          # Security audit dependencies
make doc            # Generate and open documentation
```

### Code Quality

The project uses several tools to maintain code quality:

- **rustfmt**: Automatic code formatting
- **clippy**: Rust linter for catching common mistakes
- **pre-commit hooks**: Automated checks before commits
- **cargo-tarpaulin**: Code coverage
- **cargo-audit**: Security vulnerability scanning

## Testing

Run the test suite:

```bash
make test
```

Run tests with coverage:

```bash
make test-coverage
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and quality checks
5. Submit a pull request

## Security

If you discover a security vulnerability, please see [SECURITY.md](SECURITY.md) for the responsible disclosure process.

## License

This project is licensed under the LICENSE file in the repository.

## Roadmap

- [ ] Basic HTML rendering
- [ ] CSS styling support
- [ ] JavaScript execution
- [ ] Network stack implementation
- [ ] UI framework
- [ ] Extension system
- [ ] Performance optimization
- [ ] Security hardening

## Acknowledgments

- The Rust community for excellent tools and libraries
- Contributors who help improve this project

## Contact

For questions, suggestions, or contributions, please open an issue on GitHub.

---

**Note**: This project is in early development. Many features are not yet implemented.
