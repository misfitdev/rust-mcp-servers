# Product Guidelines

## Code Style & Conventions

### Rust Best Practices
- Follow Rust API guidelines (https://rust-lang.github.io/api-guidelines/)
- Use clippy strict lints: `RUSTFLAGS="-W clippy::all" cargo check`
- Aim for zero unsafe code where possible; document all unsafe blocks with SAFETY comments
- Use idiomatic Rust: prefer pattern matching over if/else, iterators over loops, Result/Option over null checks

### Error Handling
- Use typed errors with `thiserror` or `anyhow`
- Provide context via error chains
- Return `Result<T, E>` for fallible operations; use `?` operator
- Log errors at appropriate levels (error, warn, debug)

### Testing
- Minimum 80% code coverage (as per workflow.md)
- Test-driven development: write tests before implementation
- Unit tests in modules; integration tests in `/tests`
- Mock external dependencies (OpenSCAD subprocess, filesystem)

### Documentation
- Public API must have doc comments (`///`)
- Include examples in doc comments
- Maintain README with architecture overview
- Document async/await behavior and concurrency

### Performance
- Profile before optimizing
- Avoid allocations in hot paths
- Use references, not clones
- Leverage tokio for concurrent operations

## Naming Conventions

### Modules & Types
- Modules: `snake_case` (e.g., `render_engine`)
- Types: `PascalCase` (e.g., `RenderRequest`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_CONCURRENT_RENDERS`)
- Functions: `snake_case` (e.g., `render_single`)

### MCP Tools
- Tool names: `snake_case` (e.g., `render_single`, `get_cache_stats`)
- Tool descriptions: Clear, concise, action-oriented

## Architecture Principles

### Modularity
- Separate concerns: rendering, caching, config, validation, resource management
- Use traits for abstraction (e.g., `Cache`, `Renderer`)
- Minimize coupling between modules

### Async/Concurrency
- Use `tokio` for async runtime
- Use `tokio::spawn` or `join_all` for parallelism
- Protect shared state with `Arc<RwLock<T>>` or `Arc<Mutex<T>>`
- Document which functions are async and why

### Resource Safety
- Use RAII patterns (Drop trait)
- Ensure temp files are cleaned up (via `tempfile` crate with `drop` on scope exit)
- Explicitly kill child processes on abort
- No manual memory management

### Configuration
- Centralized config module with defaults
- Support env vars, YAML, and .env files
- Provide `config::reload()` for hot-reloads
- Validate config on startup

## Observability

### Logging
- Use `tracing` or `log` crate
- Log at appropriate levels: error, warn, info (default), debug (verbose), trace (very verbose)
- Include structured fields (e.g., render_id, duration, cache_hit)
- No logging of secrets

### Metrics
- Track: render count, duration, success/failure rate
- Track: cache hit/miss ratio, size, evictions
- Track: memory usage, active processes, queue depth
- Expose via introspection tools (get_server_metrics, get_cache_stats)

### Error Reporting
- Capture and return OpenSCAD stderr in responses
- Distinguish between user errors and system errors
- Provide actionable error messages and suggestions

## Quality Assurance

### Code Review
- Focus on correctness, safety, and maintainability
- Check for resource leaks, unsafe blocks, unhandled errors
- Verify tests cover both happy path and error cases

### CI/CD Integration
- Run on every commit: `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- Maintain 80%+ coverage
- No warnings allowed in CI

### Security
- Validate all inputs (file paths, variable names, parameter ranges)
- Use path canonicalization to prevent directory traversal
- Subprocess timeouts to prevent hung processes
- No shell injection in subprocess calls (use `Command::arg()`, not shell strings)

