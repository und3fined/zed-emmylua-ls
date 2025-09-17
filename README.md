# EmmyLua Analyzer Rust for Zed

A Zed extension that provides [EmmyLua Analyzer Rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) support for Lua development.

## Features

- **Code Completion**: Intelligent autocomplete for Lua code
- **Diagnostics**: Real-time error detection and warnings
- **Go to Definition**: Navigate to symbol definitions
- **Hover Information**: View documentation and type information
- **Signature Help**: Function parameter hints
- **Code Actions**: Quick fixes and refactoring suggestions
- **Inlay Hints**: Type and parameter hints

Full features guide [check here](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/main/docs/features/features_EN.md)

## Installation

1. Open Zed
2. Open the command palette (`Cmd+Shift+P` on macOS, `Ctrl+Shift+P` on Linux/Windows)
3. Run `zed: extensions`
4. Search for "EmmyLua for Zed" and install

The extension will automatically download and configure the EmmyLua Analyzer Rust binary.

## Configuration

Maybe conflict with [Lua](https://github.com/zed-extensions/lua) language sever.

You can customize the extension settings in your Zed configuration file (`settings.json`).

```jsonc
{
  // Other settings...
  "languages": {
    "Lua": {
      "language_servers": [
        "!lua-language-server",
        "emmylua",
        "..."
      ],
      // ...other Lua settings
    }
  },
  // Other settings...
}
```

### Custom Binary

You can specify a custom EmmyLua Analyzer Rust binary:

```json
{
  "lsp": {
    "emmylua": {
      "binary": {
        "path": "/path/to/emmylua_ls",
        "arguments": []
      }
    }
  }
}
```

### Configuration Files

The extension looks for EmmyLua configuration files in the following`.emmyrc.json` (EmmyLua Analyzer Rust specific)

Example `.emmyrc.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/EmmyLuaLs/emmylua-analyzer-rust/refs/heads/main/crates/emmylua_code_analysis/resources/schema.json",
  "runtime": {
    "version": "Lua5.4"
  },
  "workspace": {
    "library": [
      "/usr/local/share/lua/5.4"
    ]
  },
  "diagnostics": {
    "globals": [
      "vim",
      "love"
    ]
  }
}
```

## Development

### Building from Source

1. Clone the repository
2. Run `cargo build` to build the extension
3. Install in Zed using the local development path

### Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Troubleshooting

### Permission Issues on Linux

If you encounter a "Permission denied" error:

```bash
# Find the binary location
find ~/.local/share/zed/extensions/emmylua -name "emmylua_ls" -type f

# Make it executable
chmod +x ~/.local/share/zed/extensions/emmylua/emmylua_ls/emmylua_ls
```

### Binary Not Found

If the extension can't find the EmmyLua binary:

1. Check the Zed logs for detailed error messages
2. Ensure your platform/architecture is supported
3. Try removing the extension and reinstalling
4. Check your internet connection for downloads

### Configuration Issues

If the language server doesn't start:

1. Verify your `.emmyrc.json` syntax is valid
2. Check that workspace library paths exist
3. Ensure diagnostic settings are correct
4. Try with minimal configuration first

### Getting Help

- Check [Zed logs](https://zed.dev/docs/configuring-zed#log-file) for detailed error messages
- Create an issue on GitHub with your configuration and error logs
- Include your OS, Zed version, and extension version

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [EmmyLua Analyzer Rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) - The underlying language server
- [Zed](https://zed.dev) - The editor this extension is built for
