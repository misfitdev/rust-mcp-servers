# Rust MCP Servers

A workspace of Model Context Protocol (MCP) servers written in Rust.

## Overview

This workspace provides high-quality MCP implementations designed for reliability, performance, and extensibility. Built with async Rust (Tokio), comprehensive testing (237+ unit tests), and security best practices.

## Crates

- **mcp-openscad** - MCP server for OpenSCAD 3D modeling language
- **shared** - Shared utilities and common patterns for MCP servers

## Quick Start

### Prerequisites

- Rust 1.70+ (MSRV)
- `just` task runner
- For mcp-openscad: OpenSCAD binary in PATH

### Build

```bash
just build              # Debug build all crates
just build-release      # Release build
just build-openscad     # Build specific crate
```

### Test

```bash
just test               # Run all tests
just check              # Format, lint, test
just dev                # Pre-commit workflow
```

### Install

```bash
just build-release
# Binaries in target/release/
```

Each crate has its own README with server-specific installation and usage.

## Development

### Workflow

```bash
just fmt        # Format code
just lint       # Check with clippy
just test       # Run all tests
just dev        # Full pre-commit check
just watch      # Watch mode - auto-rerun on changes
```

### Testing

- Test-driven development (TDD) enforced
- All changes must pass `just check` before commit
- Pre-commit hooks via `lefthook` prevent bad commits
- 100% unit test coverage required for new features

### Architecture

- **Async-first**: Tokio runtime for concurrent operations
- **Error handling**: Custom error types with `thiserror`
- **Observability**: Structured logging with `tracing`
- **Security**: No unsafe code except in pinned dependencies
- **Workspace**: Cargo workspace for multiple MCPs

## Contributing

Contributions welcome via **issue + LLM prompt only**.

### Contribution Process

1. **Open an issue** - Describe the problem or feature
2. **Submit an LLM prompt** - Provide a Claude/LLM prompt that solves it
3. **Core team reviews** - We implement based on your prompt
4. **No code PRs** - Direct code submissions not accepted (except core team)

### Why LLM Prompts?

- Enforces consistent architecture and patterns
- Guarantees TDD compliance and test coverage
- Maintains code quality standards
- Reduces review friction through clear intent
- Enables contributions without deep codebase knowledge

### Development Guidelines

- **One phase at a time** - Sequential implementation
- **Tests first** - Write tests before code
- **Async-safe** - No blocking operations
- **No unsafe code** - Unless explicitly documented
- **Latest dependencies** - Track upstream releases closely

## Dependencies

All dependencies pinned to latest releases (no LTS thinking):

```bash
tokio 1.50+         # Async runtime
serde 1.0+          # Serialization
tracing 0.1+        # Observability
thiserror 2.0+      # Error handling
```

See `Cargo.toml` for complete dependency list.

## Troubleshooting

### Build fails

```bash
cargo clean
just build
```

### Tests fail

Ensure OpenSCAD is installed and in PATH:
```bash
which openscad
```

### Pre-commit hook blocks commit

Run `just check` to see issues:
```bash
just dev
```

## Resources

- [Model Context Protocol](https://modelcontextprotocol.io/)
- [OpenSCAD Documentation](https://openscad.org/documentation.html)
- [Rust Book](https://doc.rust-lang.org/book/)

## License

See individual crates for license information.
