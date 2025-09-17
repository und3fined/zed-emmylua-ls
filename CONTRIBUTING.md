# Contributing to EmmyLua Analyzer Rust Extension for Zed

Thank you for your interest in contributing to the EmmyLua Analyzer Rust extension for Zed! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Zed Editor](https://zed.dev/)
- Basic knowledge of Lua and language server protocol
- Familiarity with Rust (EmmyLua Analyzer Rust is written in Rust)


### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/und3fined/zed-emmylua-ls.git
   cd zed-emmylua-ls
   ```

2. **Build the extension**
   ```bash
   cargo build
   ```

3. **Install for development**
   - Open Zed
   - Go to Extensions (`Cmd+Shift+P` -> "zed: extensions")
   - Click "Install Dev Extension" and select this directory



## Development Guidelines

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run Clippy for linting (`cargo clippy`)
- Write clear, self-documenting code
- Add comments for complex logic

### Testing

Before submitting changes:

1. **Build and test locally**
   ```bash
   cargo build
   cargo clippy
   cargo fmt --check
   ```

2. **Test the extension**
   - Install the extension in Zed
   - Test with various Lua projects
   - Verify language server features work correctly


### Configuration

The extension supports various configuration options. When adding new settings:
When adding new settings:

1. Update the workspace configuration logic in `src/emmylua.rs`
2. Document new settings in `README.md`
3. Add examples to the documentation
4. Ensure compatibility with `.emmyrc.json` schema from EmmyLua Analyzer Rust

### Custom EmmyLua Analyzer Rust Binary

Users can specify a custom EmmyLua Analyzer Rust binary:

```json
{
  "lsp": {
    "emmylua": {
      "binary": {
        "path": "/path/to/custom/emmylua_ls",
        "arguments": []
      },
      "initialization_options": {
        "completion": {
          "enable": true,
          "callSnippet": false,
          "autoRequire": true
        },
        "diagnostics": {
          "enable": true,
          "globals": ["vim", "love"]
        },
        "runtime": {
          "version": "Lua5.4"
        }
      }
    }
  }
}
```

## Submitting Changes

### Pull Request Process

1. **Fork the repository** and create a feature branch
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the guidelines above

3. **Test thoroughly** with different Lua projects and configurations

4. **Update documentation** if you're adding new features

5. **Submit a pull request** with:
   - Clear description of changes
   - Test cases or examples
   - Screenshots if UI changes are involved

### Commit Messages

Use clear, descriptive commit messages:

- `feat: add support for custom configuration paths`
- `fix: resolve binary download issue on Windows`
- `docs: update README with new configuration options`
- `refactor: improve error handling in language server setup`

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

- Zed version
- Extension version
- Operating system
- Lua version (if relevant)
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Feature Requests

For feature requests:

- Describe the use case
- Explain why it would be valuable
- Provide examples if possible
- Consider implementation complexity

## Architecture

### File Structure

```
src/
├── emmylua.rs          # Main extension implementation
extension.toml          # Extension metadata and configuration
Cargo.toml             # Rust package configuration
README.md              # User documentation
CONTRIBUTING.md        # This file
.emmyrc.json           # EmmyLua configuration (project-specific)
```

### Key Components

- **EmmyLuaExtension**: Main extension struct implementing `zed::Extension`
- **Binary Management**: Automatic download and installation of EmmyLua Analyzer Rust
- **Configuration**: Workspace and language server settings
- **Platform Support**: Cross-platform binary selection and installation

## Resources

### EmmyLua Analyzer Rust

- [EmmyLua Analyzer Rust Repository](https://github.com/EmmyLuaLs/emmylua-analyzer-rust)
- [EmmyLua Documentation](https://emmylua.github.io/)
- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)

### Zed Extension Development

- [Zed Extension API Documentation](https://zed.dev/docs/extensions)
- [Zed Extension Examples](https://github.com/zed-industries/zed/tree/main/extensions)

## Code of Conduct

This project follows a standard code of conduct:

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Maintain a welcoming environment

## Getting Help

- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions and general discussion
- **Documentation**: Check the README and this contributing guide first
- **Schema**: Reference the [EmmyLua schema](https://raw.githubusercontent.com/EmmyLuaLs/emmylua-analyzer-rust/refs/heads/main/crates/emmylua_code_analysis/resources/schema.json) for configuration



## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing to the EmmyLua Analyzer Rust extension for Zed!
