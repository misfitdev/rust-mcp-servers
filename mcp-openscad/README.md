# OpenSCAD MCP Server

Model Context Protocol server for OpenSCAD 3D modeling language. Provides tools for rendering, analyzing, and manipulating OpenSCAD designs through Claude and other LLMs.

## Features

- **Rendering** - Convert SCAD files to images (PNG) and models (STL, 3MF, etc.)
- **Comparison** - Render and compare two designs side-by-side
- **Analysis** - Parse dependencies, detect circular references, validate syntax
- **Caching** - File-based cache with LRU eviction and SHA-256 keying
- **Quality Control** - Draft, normal, and high-quality render presets
- **Observability** - Structured logging and performance metrics

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

### Tools Available

- **render_scad** - Render SCAD to PNG
- **export_scad** - Export to STL, 3MF, AMF, OFF, DXF, SVG
- **compare_renders** - Compare two designs side-by-side
- **parse_dependencies** - Extract file dependencies
- **analyze_model** - Validate and analyze SCAD file
- **detect_circular** - Find circular dependencies

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
just test-openscad-verbose  # Show test output
```

**Test coverage**: 207+ unit tests covering:
- Rendering pipeline
- Export formats
- Dependency analysis
- Circular reference detection
- Caching behavior
- Error handling

### Architecture

```
src/
├── main.rs              # Entry point
├── server.rs            # MCP server implementation
├── tools/               # Tool definitions
├── render/              # Rendering engine
│   ├── engine.rs       # OpenSCAD subprocess
│   ├── params.rs       # Render parameters
│   └── quality.rs      # Quality presets
├── models/              # Data models
│   ├── store.rs        # File management
│   └── dependency.rs   # Dependency graph
├── analysis/            # SCAD analysis
│   ├── validator.rs    # Syntax validation
│   └── mesh.rs         # Mesh parsing
├── cache/               # Caching layer
│   ├── file_cache.rs   # File-based cache
│   └── metrics.rs      # Cache metrics
├── config/              # Configuration
└── error.rs             # Error types
```

### Key Dependencies

- **tokio** 1.50+ - Async runtime
- **serde** - Serialization
- **tempfile** - Temporary files
- **sha2** - Content hashing
- **regex** - Pattern matching

See `Cargo.toml` for full dependency list.

## Performance

- **Caching** - SHA-256 content-addressed, LRU eviction
- **Concurrency** - Parallel rendering via Tokio
- **Efficiency** - Zero-copy rendering parameters
- **Cleanup** - Automatic temp file cleanup

Typical render time: 100-500ms depending on quality and complexity.

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

Ensure OpenSCAD binary is in PATH:
```bash
openscad --version
```

## Contributing

See parent directory [README](../README.md) for contribution guidelines.

Contributions via **issue + LLM prompt only** (no direct code PRs except core team).

## License

MIT

## References

- [OpenSCAD Docs](https://openscad.org/documentation.html)
- [MCP Spec](https://modelcontextprotocol.io/)
- [Tokio Runtime](https://tokio.rs/)
