# Implementation Plan: Implement MCP Protocol Server

## Phase 1: Core MCP Protocol Server ✓ COMPLETE

Implement the foundation of the MCP server using rust-mcp-stack SDK.

- [x] Task: Write tests for MCP initialization and protocol handling d5523f6
    - [x] Test server initialization with proper capabilities announcement
    - [x] Test JSON-RPC request/response handling
    - [x] Test error responses for invalid requests
    - [x] Test server shutdown gracefully
- [x] Task: Implement MCP protocol server scaffold d5523f6
    - [x] Add rust-mcp-stack SDK to Cargo.toml
    - [x] Implement main server loop (stdin/stdout communication)
    - [x] Implement MCP initialization message handler
    - [x] Add error type for MCP responses
    - [x] Update main.rs to start server instead of printing placeholder
- [x] Task: Test protocol implementation end-to-end d5523f6
    - [x] Manual protocol testing with mock JSON-RPC messages
    - [x] Verify server responds correctly to tool discovery requests
- [x] Task: Conductor - Phase 1 Verification Protocol d5523f6

## Phase 2: Tool Integration & Registration ✓ COMPLETE

Wire all existing tools into the MCP server as callable tools.

- [x] Task: Write tests for tool registration and execution a98c4fa
    - [x] Test tool definitions (name, description, input schema)
    - [x] Test tool discovery via MCP protocol
    - [x] Test tool call execution and result formatting
    - [x] Test error handling for missing parameters
- [x] Task: Implement tool registry and wiring a98c4fa
    - [x] Create tool definitions using JSON-RPC schema (no external SDK needed)
    - [x] Register all rendering tools (render_scad, render_perspectives, compare_renders)
    - [x] Register all export tools (export to all formats: STL, 3MF, AMF, OFF, DXF, SVG)
    - [x] Register all analysis tools (analyze, dependencies, circular detection)
    - [x] Map tool calls to registry lookup in handle_tools_call
- [x] Task: Test tool execution through MCP a98c4fa
    - [x] Test tool registry and tool lookup
    - [x] Test tool discovery via MCP tools/list returns all 7 tools
    - [x] Test tool call with valid tool returns success response
    - [x] Test tool call with invalid tool returns appropriate error
- [x] Task: Conductor - Phase 2 Verification Protocol a98c4fa

## Phase 3: Integration, Testing & Validation

Complete integration testing and validate MCP server works end-to-end.

- [ ] Task: Write integration tests
    - [ ] Full MCP handshake → tool call → result flow
    - [ ] Test with real SCAD files (use test fixtures)
    - [ ] Test error scenarios (OpenSCAD not found, invalid file, timeout)
    - [ ] Test concurrent tool calls
- [ ] Task: Manual validation in Claude Code
    - [ ] Update ~/.claude/settings.json with new binary
    - [ ] Verify `/mcp` shows openscad server
    - [ ] Test calling each tool from Claude Code
    - [ ] Verify results are usable by Claude
- [ ] Task: Coverage and cleanup
    - [ ] Run `cargo tarpaulin` - verify ≥80% coverage
    - [ ] Run `cargo clippy` - fix all warnings
    - [ ] Run `cargo fmt` - format all code
    - [ ] Ensure all tests pass: `cargo test --all`
- [ ] Task: Conductor - Phase 3 Verification Protocol
