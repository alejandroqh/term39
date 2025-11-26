# Contributing to term39

Thank you for your interest in contributing to term39! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Create a new branch for your feature or bugfix

## Development Setup

### Prerequisites

- Rust (Edition 2024)
- Cargo

### Building

```bash
# Standard build (terminal backend)
cargo build

# Release build
cargo build --release

# With framebuffer support (Linux only)
cargo build --features framebuffer-backend
```

### Running

```bash
# Default Unicode mode
cargo run

# ASCII compatibility mode
cargo run -- --ascii

# Framebuffer mode (Linux console only, requires root or video group)
sudo ./target/release/term39 --fb-mode=80x25
```

## Code Quality

Before submitting a pull request, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Quick compile check
cargo check
```

## Pull Request Process

1. Ensure your code follows the existing style and conventions
2. Run `cargo fmt` to format your code
3. Run `cargo clippy` and address any warnings
4. Run `cargo test` to ensure all tests pass
5. Update documentation if you're changing functionality
6. Create a pull request with a clear description of your changes

## Code Style Guidelines

- Follow standard Rust conventions
- Use meaningful variable and function names
- Keep functions focused and reasonably sized
- Add comments only where the logic isn't self-evident
- Maintain the MS-DOS aesthetic for UI-related changes

## Architecture Notes

When contributing, please be mindful of:

- **Rendering System**: Uses double-buffered rendering with dirty region tracking
- **Backend Abstraction**: Support both terminal and framebuffer backends
- **Charset Modes**: Maintain compatibility for both Unicode and ASCII modes
- **Cross-platform**: Terminal backend should work on Linux, macOS, and Windows

## Reporting Issues

When reporting bugs, please include:

- Operating system and version
- Terminal emulator (if applicable)
- Steps to reproduce the issue
- Expected vs actual behavior

## Questions?

Feel free to open an issue for questions or discussions about the project.
