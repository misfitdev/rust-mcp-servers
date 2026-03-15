//! MCP (Model Context Protocol) server implementation
//! Implements JSON-RPC 2.0 over stdin/stdout for Model Context Protocol

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

/// MCP Server for OpenSCAD tools
pub struct OpenSCADMCPServer;

impl OpenSCADMCPServer {
    /// Create and run the MCP server on stdin/stdout
    pub async fn run() -> Result<()> {
        tracing::info!("Starting OpenSCAD MCP server");
        run_stdio_server().await
    }
}

/// Run MCP server over stdin/stdout
async fn run_stdio_server() -> Result<()> {
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

        match process_message(trimmed) {
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
fn process_message(message: &str) -> Result<Option<String>> {
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
        "tools/list" => handle_tools_list(&json),
        "tools/call" => handle_tools_call(&json),
        _ => Err(anyhow::anyhow!("Unknown method: {}", method))
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
fn handle_tools_list(message: &Value) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": []
        }
    });

    Ok(Some(response.to_string()))
}

/// Handle tools/call request
fn handle_tools_call(message: &Value) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32601,
            "message": "Tool execution not yet implemented"
        }
    });

    Ok(Some(response.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(parsed["result"]["serverInfo"]["name"].as_str().unwrap().contains("openscad"));
    }

    #[test]
    fn test_tools_list_response() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        let result = handle_tools_list(&msg);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 2);
        assert!(parsed["result"]["tools"].is_array());
    }

    #[test]
    fn test_invalid_json() {
        let result = serde_json::from_str::<Value>("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_tools_call_error() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call"
        });

        let result = handle_tools_call(&msg);
        assert!(result.is_ok());

        let response = result.unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed["error"].is_object());
    }

    #[test]
    fn test_process_initialize() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let result = process_message(json);
        assert!(result.is_ok());
    }
}
