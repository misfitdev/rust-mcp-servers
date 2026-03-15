# Specification: Implement MCP Protocol Server

## Overview

Complete the OpenSCAD MCP server by implementing full Model Context Protocol support. Wire up all existing tools (rendering, export, analysis) into a functional MCP server that communicates over stdin/stdout using the rust-mcp-stack SDK. This makes the server discoverable and usable by Claude Code and other MCP clients.

## Functional Requirements

### 1. MCP Protocol Implementation
- Implement JSON-RPC 2.0 over stdin/stdout using rust-mcp-stack SDK
- Handle MCP initialization handshake (server announces capabilities)
- Support tool registration and discovery
- Handle tool call requests and return results
- Support error handling and protocol messages

### 2. Tool Integration
Wire all existing tools into the MCP server:

**Rendering Tools:**
- `render_scad` - Render SCAD file to PNG with camera parameters
- `render_perspectives` - Render 8 perspectives of a model
- `compare_renders` - Side-by-side comparison of two renders

**Export Tools:**
- `export_scad` - Export to STL, 3MF, AMF, OFF, DXF, SVG formats

**Analysis Tools:**
- `analyze_model` - Validate and analyze SCAD file
- `parse_dependencies` - Extract file dependencies
- `detect_circular` - Find circular dependencies

### 3. Server Lifecycle
- Parse CLI args (config file, logging level)
- Load configuration (OpenSCAD path, cache settings, timeouts)
- Initialize logging via tracing
- Start MCP server on stdin/stdout
- Handle graceful shutdown

### 4. Error Handling
- Return MCP-compliant error responses
- Propagate tool errors with context
- Handle subprocess failures (OpenSCAD not found, render timeout)

## Non-Functional Requirements

- **Test Coverage:** 80% minimum (unit + integration tests)
- **Protocol Compliance:** MCP 1.0 compatible with Claude Code
- **Performance:** Server startup <100ms, tool execution matches existing implementations
- **Logging:** Structured logging to stderr via tracing

## Acceptance Criteria

1. ✅ Binary runs as standalone MCP server: `/Users/tucker/.claude/bin/openscad-mcp`
2. ✅ Server listens on stdin and outputs JSON-RPC over stdout
3. ✅ Claude Code `/_mcp` shows "openscad" in available servers
4. ✅ All tools (render, export, analyze) callable from Claude Code
5. ✅ Tool calls execute and return results correctly
6. ✅ Errors handled gracefully with MCP error messages
7. ✅ Test coverage ≥80%
8. ✅ Zero clippy warnings, all tests passing

## Out of Scope

- Resource streaming (future enhancement)
- Batch operations (can be added as new tools later)
- Performance optimization beyond existing implementations
- GUI or interactive features
