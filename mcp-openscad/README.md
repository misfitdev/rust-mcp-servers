# OpenSCAD MCP Server

Model Context Protocol server for OpenSCAD 3D modeling language. Provides tools for rendering, analyzing, and manipulating OpenSCAD designs through Claude and other LLMs.

## Features

- **17 MCP Tools** - Complete OpenSCAD workflow in Claude and LLMs
- **Real Tool Execution** - All tools call OpenSCAD binary or perform actual filesystem operations
- **Rendering** - Convert SCAD to PNG with camera control and perspective views
- **Export** - Convert to 3D formats: STL, 3MF, AMF, OFF, DXF, SVG
- **File Management** - Create, read, update, list, delete SCAD files
- **Analysis** - Parse dependencies, detect circular references, validate syntax, analyze models
- **System Integration** - Check OpenSCAD installation, discover libraries, manage project files
- **Caching** - File-based cache with LRU eviction and SHA-256 keying
- **Quality Control** - Draft, normal, and high-quality render presets
- **Observability** - Structured logging and performance metrics
- **Graceful Degradation** - Tests skip OpenSCAD-dependent features if binary not found

## Prerequisites

- **OpenSCAD** - Must be installed and in PATH
  ```bash
  # macOS
  brew install openscad

  # Linux
  sudo apt-get install openscad

  # Windows
  choco install openscad
  ```

- **Rust 1.70+** - For building from source

## Build

```bash
just build-openscad         # Debug build
just build-openscad-release # Release build
```

Binary: `target/release/openscad-mcp`

## Install

### From Source

```bash
just build-openscad-release
cp target/release/openscad-mcp /usr/local/bin/
```

### Verify Installation

```bash
openscad-mcp --help
just run-openscad
```

## Configuration

Environment variables:

```bash
OPENSCAD_PATH=/path/to/openscad          # Custom OpenSCAD binary
RUST_LOG=openscad_mcp=debug              # Logging level
SCAD_CACHE_DIR=/tmp/scad-cache           # Cache directory
SCAD_CACHE_SIZE=1073741824               # Cache size (1GB default)
SCAD_RENDER_TIMEOUT=300                  # Render timeout (seconds)
```

Or create `.env` file in working directory.

## Usage

### As MCP Server

Configure in your LLM client (Claude, etc.):

```json
{
  "mcpServers": {
    "openscad": {
      "command": "openscad-mcp"
    }
  }
}
```

### Tools Available (17 total)

**Rendering & Comparison:**
- **render_scad** - Render SCAD file to PNG with camera control
- **render_perspectives** - Render 6 perspectives (front, back, left, right, top, bottom)
- **compare_renders** - Render two designs side-by-side for comparison
- **export_scad** - Export to 3D formats: STL, 3MF, AMF, OFF, DXF, SVG

**Analysis & Validation:**
- **validate_scad** - Syntax-check SCAD code without rendering
- **analyze_model** - Analyze model geometry and metrics
- **parse_dependencies** - Extract include/use statements from files
- **detect_circular** - Find circular dependencies in file includes

**File Management:**
- **create_model** - Create new SCAD files
- **get_model** - Read SCAD file content and metadata
- **update_model** - Modify existing SCAD files
- **list_models** - List all SCAD files in directory
- **delete_model** - Delete SCAD files

**System & Configuration:**
- **check_openscad** - Verify OpenSCAD installation and version
- **get_libraries** - Discover installed OpenSCAD libraries
- **get_project_files** - List project files and build dependency graph
- **clear_cache** - Clear the render cache

### Example (via Claude)

```
User: "Render this OpenSCAD design and tell me if there are any circular dependencies"

Claude uses render_scad and detect_circular tools to:
1. Render the design to PNG
2. Parse dependencies and check for cycles
3. Report findings
```

## Development

### Build & Test

```bash
just build-openscad        # Debug
just test-openscad         # Run tests
just check-openscad        # Format, lint, test
just fmt-openscad          # Format code only
```

### Run Tests

```bash
just test-openscad         # Run all tests (237 tests)
just test-openscad-verbose # Show test output
```

**Test coverage**: 237 unit tests covering:
- All 17 MCP tools with real execution
- Rendering pipeline (PNG generation)
- Export formats (STL, 3MF, AMF, OFF, DXF, SVG)
- File operations (create, read, update, list, delete)
- Dependency analysis and circular detection
- Cache behavior and metrics
- Error handling and validation
- OpenSCAD availability detection
- JSON-RPC protocol compliance

Tests automatically skip OpenSCAD-dependent features if binary not found.

### Architecture

```
src/
├── main.rs              # Entry point, stdio loop
├── mcp.rs               # MCP protocol implementation
│   ├── ToolRegistry    # 17 MCP tools registration
│   ├── execute_tool    # Real tool execution
│   └── JSON-RPC 2.0    # Protocol handlers
├── render/              # Rendering engine
│   ├── engine.rs       # OpenSCAD subprocess & binary detection
│   ├── params.rs       # Render parameters
│   └── quality.rs      # Quality presets (draft/normal/high)
├── models/              # Data models
│   ├── store.rs        # SCAD file management
│   └── dependency.rs   # Dependency graph analysis
├── analysis/            # SCAD analysis
│   ├── validator.rs    # OpenSCAD syntax validation
│   └── mesh.rs         # STL mesh parsing
├── cache/               # Caching layer
│   ├── file_cache.rs   # File-based LRU cache
│   └── metrics.rs      # Cache hit/miss tracking
├── config/              # Configuration & environment
└── error.rs             # Error types and handling
```

### Key Dependencies

- **tokio** 1.50+ - Async runtime
- **serde** - Serialization
- **tempfile** - Temporary files
- **sha2** - Content hashing
- **regex** - Pattern matching

See `Cargo.toml` for full dependency list.

## Performance

- **Real Execution** - All tools call OpenSCAD binary with actual subprocess management
- **Caching** - SHA-256 content-addressed cache with persistent LRU index (O(1) eviction)
- **Concurrency** - Parallel rendering via Tokio async runtime
- **Efficiency** - Zero-copy rendering parameters, direct file operations
- **Cleanup** - Automatic temp file cleanup via tempfile crate
- **Timeouts** - Configurable render timeouts (default 60s, export 120s)

Typical performance:
- Render time: 100-500ms (depends on quality and model complexity)
- File operations: < 10ms
- Cache lookups: < 1ms

## Troubleshooting

### "OpenSCAD not found"

```bash
# Verify OpenSCAD is installed
which openscad

# Set custom path
export OPENSCAD_PATH=/path/to/openscad
```

### Render timeout

Increase timeout:
```bash
export SCAD_RENDER_TIMEOUT=600
```

### Cache issues

Clear cache:
```bash
rm -rf ~/.cache/openscad-mcp
```

### Tests failing

Tests automatically skip OpenSCAD-dependent tools if binary not found. To use rendering/export tools:

```bash
# Verify OpenSCAD is installed
openscad --version

# Or set custom path
export OPENSCAD_PATH=/path/to/openscad
```

File management tools (create/read/update/list/delete) work without OpenSCAD.

## Contributing

See parent directory [README](../README.md) for contribution guidelines.

Contributions via **issue + LLM prompt only** (no direct code PRs except core team).

## License

MIT

## References

- [OpenSCAD Docs](https://openscad.org/documentation.html)
- [MCP Spec](https://modelcontextprotocol.io/)
- [Tokio Runtime](https://tokio.rs/)
