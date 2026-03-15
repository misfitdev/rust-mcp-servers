# Technology Stack

## Language & Runtime

**Primary Language:** Rust (Edition 2021, MSRV 1.70+)

**Runtime:** Tokio async runtime (latest stable)

## Core Dependencies

### MCP & Protocol
- **rmcp** (v0.16+) — Official Rust MCP SDK from Anthropic
  - Provides: stdio/HTTP/SSE transports, tool macros, resource handling, schemas
  - Features: `server`, `macros`, `schemars`, `auth`

### Async & Concurrency
- **tokio** (v1.35+) — Async runtime and utilities
  - Features: `full` (macros, time, sync, process, fs, io-util)
  - Used for: concurrent renders, subprocess management, task spawning

### CLI & Subprocess
- **tokio::process::Command** — Subprocess execution (built-in to tokio)
- **which** (v5.0+) — Find OpenSCAD binary in PATH

### Configuration
- **serde** + **serde_yaml** (v0.9+) — YAML parsing
- **dotenvy** (v0.15+) — .env file support
- **anyhow** (v1.0+) — Error handling with context

### File & Path Handling
- **tempfile** (v3.8+) — Safe temp file creation and cleanup
- **camino** (v1.1+) — UTF-8 paths (optional, for better ergonomics)

### Image Processing
- **image** (v0.24+) — PNG encoding/decoding and compression
  - Replaces PIL/Pillow from Python version
  - Features: `png`, `image` for format support

### Caching & Data Structures
- **lru** (v0.12+) — LRU cache for render results
- **sha2** (v0.10+) — SHA-256 hashing for cache keys

### Logging & Observability
- **tracing** (v0.1+) — Structured logging
- **tracing-subscriber** (v0.3+) — Log formatting and output
  - Features: `env-filter`, `fmt` for console output

### Serialization
- **serde_json** (v1.0+) — JSON for responses and config

### Testing
- **tokio::test** — Async test framework (built-in to tokio)
- **mockall** (v0.12+) — Mock generation for subprocess, filesystem
- **tempfile** — Temp files in tests

## Development Dependencies

- **cargo-tarpaulin** (v0.20+) — Code coverage measurement
- **cargo-watch** (v8.4+) — Watch and re-run tests/builds
- **clippy** (latest) — Linting (built-in to rustup)
- **rustfmt** (latest) — Code formatting (built-in to rustup)

## Code Quality Tools

- **cargo clippy** — Static analysis (zero warnings required)
- **cargo fmt** — Code formatting (enforced on CI)
- **cargo test** — Unit and integration tests
- **tarpaulin** — Coverage reporting (80% minimum)

## Build Configuration

**Cargo.toml targets:**
- `bin/openscad-mcp` — Main server binary
- `lib` — Public library for embedding or extending
- `tests/` — Integration tests

**Feature flags:**
- `default` = `["server"]`
- `server` — Compile MCP server (enabled by default)
- `client` — Compile MCP client library (optional)

## Deployment

**Binary Output:** Single statically-linked Rust binary (Linux, macOS, Windows x86_64/ARM64)

**External Runtime:** OpenSCAD (must be installed separately; auto-detected via PATH)

**Container:** distroless/rust or scratch base (future enhancement)

## Performance Targets

- Server startup: <100ms
- Single render: depends on model complexity (baseline: 0.5-2s for simple models)
- Parallel 8-view renders: ~1x single render time (true parallelism via tokio)
- Memory footprint: <20MB base + per-concurrent-render overhead

## Compatibility

**Rust MSRV:** 1.70+ (for Tokio 1.35+, Edition 2021)

**OpenSCAD:** 2019.05+ (protocol compatibility; tested against 2021.01, 2024.09)

**MCP Protocol:** 1.0 (compatible with Claude 3.5+)

**Operating Systems:**
- Linux (glibc 2.28+, musl 1.1.24+)
- macOS 10.13+
- Windows 10+

