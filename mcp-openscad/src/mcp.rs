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

/// Helper to build JSON-RPC 2.0 success responses
fn build_success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

/// Helper to build JSON-RPC 2.0 error responses
fn build_error_response(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
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

    let response = build_success_response(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "openscad-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    );

    Ok(Some(response.to_string()))
}

/// Handle tools/list request
fn handle_tools_list(message: &Value, server: &OpenSCADMCPServer) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = build_success_response(
        id,
        json!({
            "tools": server.registry.list()
        }),
    );

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
            let response = build_success_response(
                id,
                json!({
                    "output": "Tool execution not yet implemented"
                }),
            );
            Ok(Some(response.to_string()))
        } else {
            let response = build_error_response(id, -32001, format!("Unknown tool: {}", name));
            Ok(Some(response.to_string()))
        }
    } else {
        let response = build_error_response(id, -32602, "Missing tool name in params".to_string());
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

    // Integration tests: Full MCP client workflow

    #[test]
    fn test_full_mcp_handshake() {
        let mut server = OpenSCADMCPServer::new();

        // Step 1: Initialize
        let init_json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let init_result = process_message(init_json, &mut server);
        assert!(init_result.is_ok());
        let init_response = init_result.unwrap().unwrap();
        let init_parsed: Value =
            serde_json::from_str(&init_response).expect("Failed to parse init response");
        assert_eq!(init_parsed["id"], 1);
        assert!(init_parsed["result"]["protocolVersion"]
            .as_str()
            .unwrap()
            .contains("2024"));

        // Step 2: List tools
        let list_json = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let list_result = process_message(list_json, &mut server);
        assert!(list_result.is_ok());
        let list_response = list_result.unwrap().unwrap();
        let list_parsed: Value =
            serde_json::from_str(&list_response).expect("Failed to parse tools list response");
        assert_eq!(list_parsed["id"], 2);
        let tools = list_parsed["result"]["tools"]
            .as_array()
            .expect("Tools not an array");
        assert_eq!(tools.len(), 7);

        // Verify each tool has required fields
        for tool in tools {
            assert!(tool["name"].is_string());
            assert!(tool["description"].is_string());
            assert!(tool["inputSchema"].is_object());
        }
    }

    #[test]
    fn test_mcp_tool_call_validation() {
        let mut server = OpenSCADMCPServer::new();

        // Call a valid tool: render_scad
        let render_call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"render_scad","arguments":{"file":"test.scad"}}}"#;
        let render_result = process_message(render_call, &mut server);
        assert!(render_result.is_ok());
        let render_response = render_result.unwrap().unwrap();
        let render_parsed: Value =
            serde_json::from_str(&render_response).expect("Failed to parse render response");
        assert_eq!(render_parsed["id"], 3);
        assert!(render_parsed["result"].is_object());
        assert!(!render_parsed.get("error").map_or(false, |e| e.is_object()));
    }

    #[test]
    fn test_mcp_missing_method() {
        let mut server = OpenSCADMCPServer::new();
        let json = r#"{"jsonrpc":"2.0","id":5}"#;
        let result = process_message(json, &mut server);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_invalid_json() {
        let mut server = OpenSCADMCPServer::new();
        let json = r#"{"jsonrpc":"2.0","id":6"#;
        let result = process_message(json, &mut server);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_schema_validation() {
        let registry = ToolRegistry::new();

        // Verify render_scad schema
        let render_tool = registry.get("render_scad").expect("render_scad not found");
        assert_eq!(render_tool.name, "render_scad");
        assert!(render_tool.description.contains("PNG"));
        let schema = &render_tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["file"].is_object());
        assert!(schema["required"].is_array());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&Value::String("file".to_string())));

        // Verify export_scad schema
        let export_tool = registry.get("export_scad").expect("export_scad not found");
        assert_eq!(export_tool.name, "export_scad");
        let export_schema = &export_tool.input_schema;
        assert_eq!(export_schema["type"], "object");
        assert!(export_schema["properties"]["format"]["enum"]
            .as_array()
            .unwrap()
            .contains(&Value::String("stl".to_string())));
    }

    #[test]
    fn test_all_tools_have_descriptions() {
        let registry = ToolRegistry::new();
        for tool in &registry.tools {
            assert!(
                !tool.description.is_empty(),
                "Tool {} has no description",
                tool.name
            );
            assert!(
                tool.description.len() > 5,
                "Tool {} description too short",
                tool.name
            );
        }
    }

    #[test]
    fn test_mcp_error_response_format() {
        let mut server = OpenSCADMCPServer::new();

        // Call nonexistent tool
        let json =
            r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"fake_tool"}}"#;
        let result = process_message(json, &mut server);
        assert!(result.is_ok());
        let response = result.unwrap().unwrap();
        let parsed: Value =
            serde_json::from_str(&response).expect("Failed to parse error response");

        // Verify error response structure
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 7);
        assert!(parsed["error"].is_object());
        assert!(parsed["error"]["code"].is_number());
        assert!(parsed["error"]["message"].is_string());
        assert!(!parsed.get("result").map_or(false, |r| r.is_object()));
    }

    #[test]
    fn test_sequential_tool_calls() {
        let mut server = OpenSCADMCPServer::new();

        // Call render_scad
        let call1 = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"render_scad","arguments":{"file":"model1.scad"}}}"#;
        let res1 = process_message(call1, &mut server);
        assert!(res1.is_ok());

        // Call analyze_model
        let call2 = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"analyze_model","arguments":{"file":"model2.scad"}}}"#;
        let res2 = process_message(call2, &mut server);
        assert!(res2.is_ok());

        // Call export_scad
        let call3 = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"export_scad","arguments":{"file":"model3.scad","format":"stl"}}}"#;
        let res3 = process_message(call3, &mut server);
        assert!(res3.is_ok());

        // All calls succeed
        let parsed1: Value = serde_json::from_str(&res1.unwrap().unwrap()).unwrap();
        let parsed2: Value = serde_json::from_str(&res2.unwrap().unwrap()).unwrap();
        let parsed3: Value = serde_json::from_str(&res3.unwrap().unwrap()).unwrap();

        assert_eq!(parsed1["id"], 8);
        assert_eq!(parsed2["id"], 9);
        assert_eq!(parsed3["id"], 10);
        assert!(parsed1["result"].is_object());
        assert!(parsed2["result"].is_object());
        assert!(parsed3["result"].is_object());
    }
}
