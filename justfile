#!/usr/bin/env just --justfile

# Default target: show help
default:
    @just --list

# Build all crates in the workspace
build:
    cargo build --workspace

# Build a specific crate
build-crate crate="openscad-mcp":
    cargo build -p {{ crate }}

# Build in release mode
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace --lib

# Run tests with output
test-verbose:
    cargo test --workspace --lib -- --nocapture

# Run tests for a specific crate
test-crate crate="openscad-mcp":
    cargo test -p {{ crate }} --lib

# Run clippy lints
lint:
    cargo clippy --workspace --lib -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without making changes
fmt-check:
    cargo fmt --all -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test
    @echo "✓ All checks passed"

# Clean build artifacts
clean:
    cargo clean

# Check for security vulnerabilities
audit:
    cargo audit

# Full development workflow
dev: fmt lint test
    @echo "✓ Ready to commit"

# Build and test release
release: build-release test
    @echo "✓ Release build successful"

# Show workspace structure
info:
    @echo "Workspace members:"
    @cargo metadata --format-version 1 | jq -r '.workspace_members[] | split(" ") | .[0]'

# Run OpenSCAD MCP binary (if applicable)
run-openscad:
    cargo run -p openscad-mcp --bin openscad-mcp

# Development watch mode (requires watchexec)
watch task="test":
    watchexec -e rs just {{ task }}
