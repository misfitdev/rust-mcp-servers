#!/usr/bin/env just --justfile

import 'mcp-openscad/justfile'
import 'shared/justfile'

# Default target: show help
default:
    @just --list

# Workspace orchestration

# Build all crates
build:
    cargo build --workspace

# Build all in release mode
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace --lib

# Run tests with output
test-verbose:
    cargo test --workspace --lib -- --nocapture

# Run all lints
lint:
    cargo clippy --workspace --lib -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run all checks
check: fmt-check lint test
    @echo "✓ All checks passed"

# Clean build artifacts
clean:
    cargo clean

# Check security vulnerabilities
audit:
    cargo audit

# Full dev workflow
dev: fmt lint test
    @echo "✓ Ready to commit"

# Build and test release
release: build-release test
    @echo "✓ Release build successful"

# Show workspace info
info:
    @echo "Workspace members:"
    @cargo metadata --format-version 1 | jq -r '.workspace_members[] | split(" ") | .[0]'

# Watch and rerun task
watch task="test":
    watchexec -e rs just {{ task }}
