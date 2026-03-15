# Specification: Core OpenSCAD MCP Server (MVP)

## Overview

This specification defines the MVP for the Rust-based OpenSCAD MCP server. The MVP focuses on delivering production-grade rendering, caching, observability, and resource safety—addressing all P1 issues from the Grimes review of the Python implementation.

## Requirements

### Functional Requirements

#### FR-1: Core Rendering
- Single-view rendering with full camera control (eye, center, up vectors)
- Multi-view parallel rendering (8 standard views: front, back, left, right, top, bottom, isometric, dimetric)
- Rendering completes in expected time: single view ~1-2s, 8 parallel views ~1-2s (not 8x)
- Support for quality presets: draft ($fn=12), normal ($fn=32), high ($fn=100)
- Color scheme selection

#### FR-2: OpenSCAD Integration
- Auto-detect OpenSCAD binary from PATH or OPENSCAD_PATH env var
- Full subprocess stdout + stderr capture (not discarded)
- Subprocess timeout enforcement (default 300s, configurable)
- Explicit process termination on timeout (SIGKILL if needed)
- Support for include paths (-I flags)

#### FR-3: Caching
- SHA-256 keyed cache by render parameters
- Content-addressed: cache key includes canonical file path + content hash
- TTL-based expiration (configurable, default 24h)
- LRU eviction when cache size exceeds limit (default 500MB)
- No cache collisions on identical content from different paths

#### FR-4: Model Management
- Create/read/update/delete .scad files (atomic writes via tempfile crate)
- List models in workspace
- Support for include/use dependencies
- Full filesystem error handling

#### FR-5: Analysis & Validation
- Syntax validation (returns stderr output, not swallowed)
- Bounding box calculation via STL export
- Triangle count from STL geometry
- Library discovery (list installed OpenSCAD libraries)
- OpenSCAD version detection

#### FR-6: Export
- Multi-format export: STL, 3MF, AMF, OFF, DXF, SVG
- Configurable output format

#### FR-7: Observability Tools (NEW)
- `get_cache_stats()` — Returns: hit rate, miss rate, current size, entry count, TTL remaining
- `get_server_metrics()` — Returns: active renders, queue depth, avg render time, memory usage, uptime
- `list_renders()` — Returns: active render tasks with status, duration, model name
- `kill_render(task_id)` — Terminates a running render

#### FR-8: Dependency Analysis (NEW)
- `get_project_files()` — Returns: file list + include graph (from Python version)
- `get_dependency_order()` — Topological sort of dependencies
- `detect_circular_includes()` — Circular dependency detection

### Non-Functional Requirements

#### NFR-1: Performance
- Server startup: <100ms
- Cache hit: <10ms (memory lookup)
- Single render: 0.5-2s (depends on model complexity)
- Parallel 8-view renders: ~1-2s (true parallelism, not 8x sequential)

#### NFR-2: Resource Safety
- No memory leaks
- No zombie OpenSCAD processes (explicit Child::kill() on abort)
- Temp files cleaned up on scope exit (via tempfile crate Drop)
- Atomic file writes (tempfile + rename pattern)
- All unsafe code justified with SAFETY comments

#### NFR-3: Reliability
- Subprocess timeouts enforced (no hung renders)
- All errors captured and returned to client
- OpenSCAD stderr included in error responses (not swallowed)
- Graceful degradation if OpenSCAD not found

#### NFR-4: Observability
- Structured logging (tracing crate) with render_id, duration, cache_hit
- Metrics exported via introspection tools
- No secrets in logs
- Error messages actionable and contextual

#### NFR-5: Security
- Path validation (canonical paths, no directory traversal)
- File size limits for SCAD content
- Variable name validation (alphanumeric + underscore)
- No shell injection (use Command::arg(), not shell strings)
- Subprocess execution isolated from untrusted input

#### NFR-6: Code Quality
- 80%+ code coverage (tarpaulin measured)
- Zero clippy warnings (RUSTFLAGS="-W clippy::all")
- Idiomatic Rust (pattern matching, iterators, Result/Option)
- Doc comments on all public items
- TDD: tests before implementation

## Architecture

### Modules

```
src/
├── lib.rs                 # Public library interface
├── main.rs               # Binary entry point
├── server.rs             # MCP server + tool registration
├── render/
│   ├── mod.rs            # Rendering orchestration
│   ├── engine.rs         # OpenSCAD subprocess + command building
│   ├── params.rs         # Parameter parsing (camera, size, variables)
│   └── quality.rs        # Quality presets ($fn, $fa, $fs)
├── cache/
│   ├── mod.rs            # Cache manager interface
│   ├── file_cache.rs     # File-backed cache (TTL + LRU)
│   └── metrics.rs        # Cache statistics
├── config/
│   ├── mod.rs            # Configuration struct
│   ├── loader.rs         # Load from env/YAML/.env
│   └── validator.rs      # Validation logic
├── models/
│   ├── mod.rs            # Model CRUD
│   ├── store.rs          # Filesystem-based model store
│   └── dependency.rs     # Dependency graph + analysis
├── analysis/
│   ├── mod.rs            # Analysis tools
│   ├── validator.rs      # SCAD syntax validation
│   ├── mesh.rs           # STL parsing for bounding box / tri count
│   └── libraries.rs      # Library discovery
├── error.rs              # Typed error enum (thiserror)
├── logging.rs            # Logging initialization
└── metrics.rs            # Metrics collection + introspection

tests/
├── integration/
│   ├── rendering.rs      # End-to-end render tests
│   ├── caching.rs        # Cache behavior tests
│   └── models.rs         # CRUD operation tests
└── unit/                 # Unit test modules (colocated with src/)
```

### Key Design Decisions

1. **True Parallelism**: Use `tokio::join!` and `tokio::task::join_all()` for rendering multiple views in parallel, not sequential awaits.

2. **Content-Addressed Cache**: Cache key = SHA-256(canonical_path + file_content + all_render_params). Prevents collisions on identical content from different paths.

3. **Atomic File I/O**: Use `tempfile::NamedTempFile` with drop-on-scope to ensure writes complete or roll back atomically.

4. **Full Subprocess Output**: Capture both stdout and stderr. Include stderr in response (not swallowed).

5. **Explicit Process Cleanup**: On timeout, use `Child::kill()` (SIGTERM) and if needed, `Child::wait_with_output()` to ensure process terminates.

6. **Structured Logging**: Use `tracing` with span macros for render_id, duration, and cache hits. Enable filtering by RUST_LOG env var.

7. **Configuration Flexibility**: Support env vars, YAML config file, and .env files. Validate on startup.

## Success Criteria

- [ ] All 15 tools from Python version implemented and tested
- [ ] All 4 NEW tools (observability + dependency) implemented and tested
- [ ] 80%+ code coverage (tarpaulin)
- [ ] Zero clippy warnings
- [ ] Parallel renders (8 views) complete in ~1x single render time
- [ ] Cache hits occur for identical renders <10ms
- [ ] No zombie processes (explicit cleanup tests)
- [ ] All stderr output captured and returned (not swallowed)
- [ ] Atomic file I/O verified (tests for crash scenarios)
- [ ] Integration tests pass with real OpenSCAD (if available)

## Out of Scope (Post-MVP)

- Animation/GIF generation
- GPU acceleration
- Real-time preview
- Multi-user session management
- Web UI
- S3/remote cache backend (future enhancement)

