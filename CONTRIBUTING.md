# Contributing to dioxus-mosaic

Thank you for your interest in contributing to dioxus-mosaic! This document provides guidelines for contributing.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/dioxus-mosaic.git
   cd dioxus-mosaic
   ```
3. **Install Dioxus CLI**:
   ```bash
   cargo install dioxus-cli
   ```

## Development Workflow

### Running Examples

```bash
# Basic example
dx serve --example basic

# Advanced example
dx serve --example advanced
```

### Running Tests

```bash
cargo test
```

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Run Clippy and fix warnings (`cargo clippy`)
- Add documentation for public APIs
- Write tests for new features

### Before Submitting

1. **Format your code**: `cargo fmt`
2. **Check for warnings**: `cargo clippy`
3. **Run tests**: `cargo test`
4. **Update documentation** if needed
5. **Add examples** for new features

## Pull Request Process

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with clear, descriptive commits:
   ```bash
   git commit -m "Add feature: description"
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Open a Pull Request** on GitHub with:
   - Clear description of changes
   - Reference to any related issues
   - Screenshots/videos if UI changes

5. **Respond to feedback** - maintainers may request changes

## Reporting Issues

When reporting bugs, please include:
- **Rust version**: `rustc --version`
- **Dioxus version**: Check `Cargo.toml`
- **OS**: macOS, Linux, Windows, etc.
- **Minimal reproduction**: Code that demonstrates the issue
- **Expected vs actual behavior**

## Feature Requests

For feature requests, please:
- Check if it already exists in issues
- Describe the use case and benefits
- Provide examples of how you'd use it
- Consider if it fits the library's scope

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help create a welcoming environment

## Questions?

- Open an issue for discussion
- Check existing issues and documentation first

Thank you for contributing! 🎉
