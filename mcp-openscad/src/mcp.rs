//! MCP (Model Context Protocol) server implementation
//! Implements JSON-RPC 2.0 over stdin/stdout for Model Context Protocol

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

/// Tool definition for MCP
#[derive(Debug, Clone)]
struct ToolDef {
    name: String,
    description: String,
    input_schema: Value,
}

impl ToolDef {
    fn new(name: &str, description: &str, input_schema: Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema
        })
    }
}

/// Registry of available tools
struct ToolRegistry {
    tools: Vec<ToolDef>,
}

impl ToolRegistry {
    fn new() -> Self {
        let mut tools = Vec::new();

        // Rendering tools
        tools.push(ToolDef::new(
            "render_scad",
            "Render an OpenSCAD file to PNG with camera parameters",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"},
                    "camera_pos": {"type": "string", "description": "Camera position (x,y,z)"},
                    "camera_target": {"type": "string", "description": "Camera target (x,y,z)"},
                    "quality": {"type": "string", "enum": ["draft", "normal", "high"]}
                },
                "required": ["file"]
            }),
        ));

        tools.push(ToolDef::new(
            "render_perspectives",
            "Render 8 perspectives of an OpenSCAD model",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"},
                    "quality": {"type": "string", "enum": ["draft", "normal", "high"]}
                },
                "required": ["file"]
            }),
        ));

        tools.push(ToolDef::new(
            "compare_renders",
            "Compare two OpenSCAD designs side-by-side",
            json!({
                "type": "object",
                "properties": {
                    "left_file": {"type": "string", "description": "Left model path"},
                    "right_file": {"type": "string", "description": "Right model path"},
                    "left_name": {"type": "string", "description": "Label for left model"},
                    "right_name": {"type": "string", "description": "Label for right model"}
                },
                "required": ["left_file", "right_file"]
            }),
        ));

        // Export tools
        tools.push(ToolDef::new(
            "export_scad",
            "Export OpenSCAD file to 3D format (STL, 3MF, AMF, OFF, DXF, SVG)",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"},
                    "format": {"type": "string", "enum": ["stl", "3mf", "amf", "off", "dxf", "svg"]}
                },
                "required": ["file", "format"]
            }),
        ));

        // Analysis tools
        tools.push(ToolDef::new(
            "analyze_model",
            "Validate and analyze an OpenSCAD file",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"}
                },
                "required": ["file"]
            }),
        ));

        tools.push(ToolDef::new(
            "parse_dependencies",
            "Extract file dependencies from an OpenSCAD file",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"}
                },
                "required": ["file"]
            }),
        ));

        tools.push(ToolDef::new(
            "detect_circular",
            "Detect circular dependencies in OpenSCAD files",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"}
                },
                "required": ["file"]
            }),
        ));

        Self { tools }
    }

    fn list(&self) -> Vec<Value> {
        self.tools.iter().map(|t| t.to_json()).collect()
    }

    fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.iter().find(|t| t.name == name)
    }
}

/// MCP Server for OpenSCAD tools
pub struct OpenSCADMCPServer {
    registry: ToolRegistry,
}

impl OpenSCADMCPServer {
    fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Create and run the MCP server on stdin/stdout
    pub async fn run() -> Result<()> {
        tracing::info!("Starting OpenSCAD MCP server");
        let server = Self::new();
        run_stdio_server(server).await
    }
}

/// Run MCP server over stdin/stdout
async fn run_stdio_server(mut server: OpenSCADMCPServer) -> Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut stdout = io::stdout();
    let mut lines = reader.lines();

    tracing::debug!("MCP server listening on stdin/stdout");

    while let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match process_message(trimmed, &mut server) {
            Ok(Some(response)) => {
                writeln!(stdout, "{}", response)?;
                stdout.flush()?;
            }
            Ok(None) => {
                // No response needed
            }
            Err(e) => {
                tracing::error!("Error processing message: {}", e);
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": format!("Internal error: {}", e)
                    }
                });
                writeln!(stdout, "{}", error_response)?;
                stdout.flush()?;
            }
        }
    }

    tracing::info!("MCP server shutdown complete");
    Ok(())
}

/// Process incoming MCP message (JSON-RPC 2.0)
fn process_message(message: &str, server: &mut OpenSCADMCPServer) -> Result<Option<String>> {
    // Parse JSON-RPC message
    let json: Value = serde_json::from_str(message)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

    // Get method field
    let method = json
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid method field"))?;

    // Route to appropriate handler
    match method {
        "initialize" => handle_initialize(&json),
        "tools/list" => handle_tools_list(&json, server),
        "tools/call" => handle_tools_call(&json, server),
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}

/// Handle MCP initialize request
fn handle_initialize(message: &Value) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "openscad-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    Ok(Some(response.to_string()))
}

/// Handle tools/list request
fn handle_tools_list(message: &Value, server: &OpenSCADMCPServer) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": server.registry.list()
        }
    });

    Ok(Some(response.to_string()))
}

/// Handle tools/call request
fn handle_tools_call(message: &Value, server: &OpenSCADMCPServer) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let tool_name = message
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str());

    if let Some(name) = tool_name {
        if server.registry.get(name).is_some() {
            // Tool exists - execution would happen here
            let response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "output": "Tool execution not yet implemented"
                }
            });
            Ok(Some(response.to_string()))
        } else {
            let response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32001,
                    "message": format!("Unknown tool: {}", name)
                }
            });
            Ok(Some(response.to_string()))
        }
    } else {
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": "Missing tool name in params"
            }
        });
        Ok(Some(response.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tools.len(), 7);
    }

    #[test]
    fn test_tool_exists() {
        let registry = ToolRegistry::new();
        assert!(registry.get("render_scad").is_some());
        assert!(registry.get("export_scad").is_some());
        assert!(registry.get("analyze_model").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_tool_to_json() {
        let tool = ToolDef::new("test_tool", "Test description", json!({"type": "object"}));
        let json = tool.to_json();
        assert_eq!(json["name"], "test_tool");
        assert_eq!(json["description"], "Test description");
    }

    #[test]
    fn test_initialize_response() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });

        let result = handle_initialize(&msg);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn test_tools_list_response() {
        let server = OpenSCADMCPServer::new();
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        let result = handle_tools_list(&msg, &server);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 2);
        assert_eq!(parsed["result"]["tools"].as_array().unwrap().len(), 7);
    }

    #[test]
    fn test_tools_call_with_valid_tool() {
        let server = OpenSCADMCPServer::new();
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "render_scad",
                "arguments": {
                    "file": "test.scad"
                }
            }
        });

        let result = handle_tools_call(&msg, &server);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 3);
        assert!(parsed["result"].is_object());
    }

    #[test]
    fn test_tools_call_with_invalid_tool() {
        let server = OpenSCADMCPServer::new();
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool"
            }
        });

        let result = handle_tools_call(&msg, &server);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed["error"].is_object());
    }

    #[test]
    fn test_process_initialize() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let mut server = OpenSCADMCPServer::new();
        let result = process_message(json, &mut server);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_tools_list() {
        let json = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let mut server = OpenSCADMCPServer::new();
        let result = process_message(json, &mut server);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["tools"].as_array().unwrap().len(), 7);
    }
}
