# Contributing to API Switch

Thanks for your interest in contributing! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- SQLite 3
- Docker (optional, for containerized deployment)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/simer11-jing/api-switch.git
cd api-switch

# Build the project
cargo build

# Run tests
cargo test

# Run the server
cargo run
```

### Project Structure

```
api-switch/
├── src/
│   ├── main.rs         # Application entry point
│   ├── handlers.rs     # HTTP route handlers
│   ├── models.rs       # Data models
│   ├── db.rs           # Database operations
│   ├── circuit_breaker.rs  # Circuit breaker implementation
│   └── auth.rs         # Authentication
├── static/             # Frontend static files
├── tests/              # Integration tests
└── docs/               # Documentation
```

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/simer11-jing/api-switch/issues)
2. If not, create a new issue using the bug report template
3. Include:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment details (OS, Rust version, etc.)

### Suggesting Features

1. Open an issue with the feature request template
2. Describe the feature and why it would be useful
3. If possible, outline how it could be implemented

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Ensure tests pass (`cargo test`)
6. Run clippy (`cargo clippy -- -D warnings`)
7. Format code (`cargo fmt`)
8. Commit with a clear message
9. Push to your fork
10. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings
- Write meaningful commit messages
- Add documentation for public APIs

## Development Workflow

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

## Questions?

Feel free to open an issue for any questions or discussions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
