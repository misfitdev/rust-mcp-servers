# Implementation Plan: Core OpenSCAD MCP Server (MVP)

## Phase 1: Project Scaffolding & Core Infrastructure

### 1.1 Cargo Project Setup
- [ ] Create Cargo.toml with dependencies (tokio, rmcp, tempfile, serde, etc.)
- [ ] Configure workspace (if needed) for openscad-mcp binary + library
- [ ] Set up Cargo.lock for deterministic builds
- [ ] Configure build targets: bin/openscad-mcp, lib
- [ ] Add dev dependencies: tokio-test, mockall, tarpaulin

### 1.2 Foundational Modules
- [ ] Write tests: error.rs (custom error types for all failure modes)
  - Test: Render errors, cache errors, filesystem errors, validation errors
- [ ] Implement: error.rs (thiserror-based Error enum)
- [ ] Write tests: logging.rs (initialization, filtering)
  - Test: RUST_LOG env var parsing, span creation
- [ ] Implement: logging.rs (tracing + tracing-subscriber)
- [ ] Write tests: config/loader.rs (env vars, YAML, .env support)
  - Test: Load from each source, priority, defaults, validation
- [ ] Implement: config/loader.rs and config/mod.rs (ConfigBuilder pattern)

### 1.3 MCP Server Bootstrap
- [ ] Write tests: server.rs (tool registration, basic request handling)
  - Test: Tools list, tool invocation, error responses
- [ ] Implement: server.rs (FastMCP server, tool registration, main entry point)
- [ ] Write tests: main.rs (CLI arg parsing, server startup)
  - Test: Help text, config file path, version flag
- [ ] Implement: main.rs (simple CLI with arg parsing)
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Project Scaffolding & Core Infrastructure' (Protocol in workflow.md)

---

## Phase 2: Rendering Engine

### 2.1 OpenSCAD Integration
- [ ] Write tests: render/engine.rs (find_openscad, version detection)
  - Test: OpenSCAD in PATH, OPENSCAD_PATH env var, common install locations, not found
- [ ] Implement: render/engine.rs (find_openscad, detect_version)
- [ ] Write tests: render/engine.rs (subprocess execution, output capture)
  - Test: stdout/stderr capture, returncode handling, timeout enforcement
- [ ] Implement: render/engine.rs (Command builder, output capture, timeout)

### 2.2 Parameter Parsing
- [ ] Write tests: render/params.rs (camera_position, camera_target, image_size)
  - Test: List format, dict format, JSON string, CSV, invalid inputs
- [ ] Implement: render/params.rs (unified parameter parser with serde)
- [ ] Write tests: render/params.rs (variables, color_scheme, quality presets)
  - Test: Type coercion, validation, defaults
- [ ] Implement: render/quality.rs (quality preset mapping: draft/normal/high → $fn/$fa/$fs)

### 2.3 Single-View Rendering
- [ ] Write tests: render/mod.rs (render_scad_to_png with mocked subprocess)
  - Test: Camera positioning, quality selection, output validation
- [ ] Implement: render/mod.rs (render_scad_to_png orchestrator)
- [ ] Write tests: render_single tool (integration with parameter parsing)
  - Test: Content vs file inputs, error handling, response format
- [ ] Implement: render_single tool (register with MCP server)

### 2.4 Multi-View Parallel Rendering
- [ ] Write tests: render/mod.rs (render_perspectives with tokio::join! or join_all)
  - Test: All 8 views render in parallel, ~1x single render time
  - Test: Error handling (one view fails, others continue or all abort?)
- [ ] Implement: render/mod.rs (render_perspectives using tokio::join_all)
- [ ] Implement: render_perspectives tool (register with MCP server)
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Rendering Engine' (Protocol in workflow.md)

---

## Phase 3: Caching System

### 3.1 Cache Key & Storage
- [ ] Write tests: cache/file_cache.rs (compute cache key: SHA-256 of canonical path + content + params)
  - Test: Different content = different keys, same content = same key (no path collision)
  - Test: Parameter order doesn't matter (deterministic JSON)
- [ ] Implement: cache/file_cache.rs (cache key computation)
- [ ] Write tests: cache/file_cache.rs (save to cache with TTL metadata)
  - Test: Create cache directory, write PNG + metadata file, handle errors
- [ ] Implement: cache/file_cache.rs (save_to_cache)

### 3.2 Cache Lookup & Eviction
- [ ] Write tests: cache/file_cache.rs (check cache for hits, TTL expiration)
  - Test: Hit, miss, expired entry, missing file
- [ ] Implement: cache/file_cache.rs (check_cache, expire old entries)
- [ ] Write tests: cache/file_cache.rs (LRU eviction when cache size exceeded)
  - Test: Track file sizes, evict oldest files until under limit
- [ ] Implement: cache/file_cache.rs (evict_cache_if_needed)

### 3.3 Cache Integration
- [ ] Write tests: render/mod.rs (render with cache: hit path, miss path)
  - Test: Cache hit skips subprocess call, cache miss executes render + saves
- [ ] Implement: render/mod.rs (integrate cache check/save into render_scad_to_png)
- [ ] Write tests: cache/metrics.rs (collect cache stats)
  - Test: Track hits, misses, current size, entry count
- [ ] Implement: cache/metrics.rs (CacheMetrics struct + update methods)
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Caching System' (Protocol in workflow.md)

---

## Phase 4: Model Management

### 4.1 Model Store (CRUD)
- [ ] Write tests: models/store.rs (create_model with atomic write)
  - Test: Create file, verify atomic (tempfile + rename), handle errors
- [ ] Implement: models/store.rs (create, with tempfile::NamedTempFile)
- [ ] Write tests: models/store.rs (read, update, delete with error handling)
  - Test: File exists, file missing, permission errors, update atomicity
- [ ] Implement: models/store.rs (read, update, delete)
- [ ] Write tests: models/store.rs (list models in directory)
  - Test: Filter .scad files, handle empty directory, permission errors
- [ ] Implement: models/store.rs (list_models)

### 4.2 Dependency Graph
- [ ] Write tests: models/dependency.rs (parse includes/uses from SCAD)
  - Test: Single include, multiple includes, nested includes, invalid syntax
- [ ] Implement: models/dependency.rs (parse_includes)
- [ ] Write tests: models/dependency.rs (build dependency graph)
  - Test: Direct deps, transitive deps, circular detection
- [ ] Implement: models/dependency.rs (build_graph, detect_cycles)

### 4.3 MCP Tools
- [ ] Implement: create_model tool
- [ ] Implement: get_model tool
- [ ] Implement: update_model tool
- [ ] Implement: delete_model tool
- [ ] Implement: list_models tool
- [ ] Implement: get_project_files tool (list + dependency graph)
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Model Management' (Protocol in workflow.md)

---

## Phase 5: Analysis & Validation

### 5.1 Syntax Validation
- [ ] Write tests: analysis/validator.rs (validate_scad with stderr capture)
  - Test: Valid code, invalid syntax, warnings, undefined variables
- [ ] Implement: analysis/validator.rs (call openscad -o /dev/null, capture stderr)
- [ ] Implement: validate_scad tool
- [ ] Write tests: analysis/validator.rs (parse OpenSCAD output for errors/warnings)
  - Test: Error format parsing, warning detection
- [ ] Implement: analysis/validator.rs (parse_openscad_output)

### 5.2 Mesh Analysis
- [ ] Write tests: analysis/mesh.rs (parse STL, compute bounding box)
  - Test: Valid STL, compute min/max, triangles
- [ ] Implement: analysis/mesh.rs (STLReader for bounding box calculation)
- [ ] Write tests: analysis/mesh.rs (triangle counting)
  - Test: Facet count extraction
- [ ] Implement: analysis/mesh.rs (count_triangles)

### 5.3 Model Analysis Tool
- [ ] Write tests: analysis/mod.rs (analyze_model: export to STL, parse, return metrics)
  - Test: Valid model, complex geometry, export failure
- [ ] Implement: analysis/mod.rs (orchestrate STL export + mesh analysis)
- [ ] Implement: analyze_model tool
- [ ] Implement: get_libraries tool (discover installed OpenSCAD libraries)
- [ ] Implement: check_openscad tool (version detection + health check)
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Analysis & Validation' (Protocol in workflow.md)

---

## Phase 6: Export Tools

### 6.1 Export Implementation
- [ ] Write tests: render/mod.rs (export to STL, 3MF, AMF, OFF, DXF, SVG)
  - Test: Each format, error handling, file output
- [ ] Implement: render/mod.rs (export_scad_to_format orchestrator)
- [ ] Implement: export_model tool (multi-format export)
- [ ] Task: Conductor - User Manual Verification 'Phase 6: Export Tools' (Protocol in workflow.md)

---

## Phase 7: Comparison Rendering

### 7.1 Before/After Comparison
- [ ] Write tests: render/mod.rs (render_perspectives for two models, side-by-side)
  - Test: Both succeed, one fails, compare metadata
- [ ] Implement: render/mod.rs (compare_scad_renders)
- [ ] Implement: compare_renders tool
- [ ] Task: Conductor - User Manual Verification 'Phase 7: Comparison Rendering' (Protocol in workflow.md)

---

## Phase 8: Observability Tools (NEW)

### 8.1 Cache Statistics
- [ ] Write tests: cache/metrics.rs (expose hit rate, entry count, size)
  - Test: Calculate percentages, handle empty cache
- [ ] Implement: cache/metrics.rs (get_stats method)
- [ ] Implement: get_cache_stats tool
- [ ] Write tests: metrics.rs (server-wide metrics: active renders, queue depth)
  - Test: Track active tasks, record duration, memory introspection
- [ ] Implement: metrics.rs (ServerMetrics struct)

### 8.2 Process Management
- [ ] Write tests: metrics.rs (list_renders: active tasks with status)
  - Test: Track render ID, model, duration, status
- [ ] Implement: metrics.rs (list_renders, with task handles)
- [ ] Write tests: metrics.rs (kill_render: terminate a task)
  - Test: Terminate running task, handle already-completed, invalid ID
- [ ] Implement: metrics.rs (kill_render with tokio task cancellation)
- [ ] Implement: get_server_metrics, list_renders, kill_render tools
- [ ] Task: Conductor - User Manual Verification 'Phase 8: Observability Tools (NEW)' (Protocol in workflow.md)

---

## Phase 9: Dependency Analysis Tools (NEW)

### 9.1 Dependency Order
- [ ] Write tests: models/dependency.rs (topological sort of include graph)
  - Test: Linear deps, tree structure, circular detection (error case)
- [ ] Implement: models/dependency.rs (topological_sort)
- [ ] Implement: get_dependency_order tool
- [ ] Write tests: models/dependency.rs (affected_models: given changed file, return what needs rebuild)
  - Test: Direct deps, transitive deps, no deps
- [ ] Implement: models/dependency.rs (affected_models)

### 9.2 Circular Dependency Detection
- [ ] Write tests: models/dependency.rs (detect_circular_includes)
  - Test: Circular A→B→A, no cycles, complex cycles
- [ ] Implement: models/dependency.rs (detect_circular_includes)
- [ ] Implement: detect_circular_includes tool (as part of get_project_files or standalone)
- [ ] Task: Conductor - User Manual Verification 'Phase 9: Dependency Analysis Tools (NEW)' (Protocol in workflow.md)

---

## Phase 10: Quality Assurance & Integration

### 10.1 Code Coverage & Linting
- [ ] Run `cargo test --all` — verify all tests pass
- [ ] Run `cargo tarpaulin --fail-under 80` — verify coverage
- [ ] Run `RUSTFLAGS="-W clippy::all" cargo clippy --all-targets` — zero warnings
- [ ] Run `cargo fmt --check` — formatting compliance

### 10.2 Integration Tests (Real OpenSCAD)
- [ ] Write tests: integration/rendering.rs (end-to-end with real OpenSCAD if available)
  - Test: Simple cube render, multi-view, quality presets, actual PNG output
- [ ] Write tests: integration/caching.rs (cache behavior end-to-end)
  - Test: First render vs cache hit, TTL expiration, LRU eviction
- [ ] Write tests: integration/models.rs (CRUD operations with real filesystem)
  - Test: Create, read, update, delete files, dependency parsing

### 10.3 Binary Verification
- [ ] Build release binary: `cargo build --release`
- [ ] Verify binary works: `./target/release/openscad-mcp --version`, `--help`
- [ ] Test stdio transport: connect and invoke a tool
- [ ] Generate documentation: `cargo doc --no-deps --open`

### 10.4 Documentation
- [ ] Write README with architecture overview
- [ ] Document tool parameters in code (doc comments)
- [ ] Create examples/ directory with usage samples
- [ ] Update conductor/product.md if scope changed

### 10.5 Final Checklist
- [ ] All tasks in plan marked `[x]`
- [ ] All tests passing: `cargo test --all`
- [ ] Coverage ≥80%: `cargo tarpaulin --fail-under 80`
- [ ] No clippy warnings: `RUSTFLAGS="-W clippy::all" cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Commit each phase with git notes or conventional commits
- [ ] Push all changes to remote

- [ ] Task: Conductor - User Manual Verification 'Phase 10: Quality Assurance & Integration' (Protocol in workflow.md)

