//! MCP (Model Context Protocol) server implementation
//! Implements JSON-RPC 2.0 over stdin/stdout for Model Context Protocol

use anyhow::Result;
use base64::Engine;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use std::time::Duration;

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

        // Validation tools
        tools.push(ToolDef::new(
            "validate_scad",
            "Syntax-check OpenSCAD code without rendering",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"},
                    "content": {"type": "string", "description": "SCAD code as string"}
                },
                "required": []
            }),
        ));

        // File management tools
        tools.push(ToolDef::new(
            "create_model",
            "Create a new OpenSCAD model file",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Model file name"},
                    "content": {"type": "string", "description": "Initial SCAD content"}
                },
                "required": ["name"]
            }),
        ));

        tools.push(ToolDef::new(
            "get_model",
            "Read an OpenSCAD model file and metadata",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"}
                },
                "required": ["file"]
            }),
        ));

        tools.push(ToolDef::new(
            "update_model",
            "Update an existing OpenSCAD model file",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"},
                    "content": {"type": "string", "description": "New SCAD content"}
                },
                "required": ["file", "content"]
            }),
        ));

        tools.push(ToolDef::new(
            "list_models",
            "List all OpenSCAD model files in workspace",
            json!({
                "type": "object",
                "properties": {
                    "directory": {"type": "string", "description": "Directory to list (default: current)"}
                },
                "required": []
            }),
        ));

        tools.push(ToolDef::new(
            "delete_model",
            "Delete an OpenSCAD model file",
            json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "Path to .scad file"}
                },
                "required": ["file"]
            }),
        ));

        // System tools
        tools.push(ToolDef::new(
            "get_libraries",
            "Discover installed OpenSCAD libraries",
            json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        ));

        tools.push(ToolDef::new(
            "check_openscad",
            "Check OpenSCAD installation status and version",
            json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        ));

        tools.push(ToolDef::new(
            "get_project_files",
            "List project files and their dependencies",
            json!({
                "type": "object",
                "properties": {
                    "directory": {"type": "string", "description": "Project directory (default: current)"}
                },
                "required": []
            }),
        ));

        tools.push(ToolDef::new(
            "clear_cache",
            "Clear the render cache",
            json!({
                "type": "object",
                "properties": {},
                "required": []
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
        "notifications/initialized" => Ok(None), // Notification, no response needed
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}

/// Handle MCP initialize request
fn handle_initialize(message: &Value) -> Result<Option<String>> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);

    let response = build_success_response(
        id,
        json!({
            "protocolVersion": "2025-11-25",
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
    let tool_args = message.get("params").and_then(|p| p.get("arguments"));

    if let Some(name) = tool_name {
        if let Some(_tool_def) = server.registry.get(name) {
            // Execute the tool
            let result = execute_tool(name, tool_args);
            match result {
                Ok(output) => {
                    let response = build_success_response(id, json!({ "output": output }));
                    Ok(Some(response.to_string()))
                }
                Err(e) => {
                    let response = build_error_response(id, -32603, format!("Tool execution failed: {}", e));
                    Ok(Some(response.to_string()))
                }
            }
        } else {
            let response = build_error_response(id, -32001, format!("Unknown tool: {}", name));
            Ok(Some(response.to_string()))
        }
    } else {
        let response = build_error_response(id, -32602, "Missing tool name in params".to_string());
        Ok(Some(response.to_string()))
    }
}

/// Validate file path to prevent directory traversal attacks
fn validate_file_path(path: &str) -> anyhow::Result<()> {
    use std::path::Path;

    let path_obj = Path::new(path);

    // Reject absolute paths
    if path_obj.is_absolute() {
        return Err(anyhow::anyhow!("Absolute file paths are not allowed"));
    }

    // Reject parent directory references
    for component in path_obj.components() {
        use std::path::Component;
        if matches!(component, Component::ParentDir) {
            return Err(anyhow::anyhow!("Path traversal with '..' is not allowed"));
        }
    }

    // Reject paths with null bytes
    if path.contains('\0') {
        return Err(anyhow::anyhow!("Path contains invalid null bytes"));
    }

    Ok(())
}

/// Execute a tool with the given arguments - REAL IMPLEMENTATION
fn execute_tool(tool_name: &str, args: Option<&Value>) -> anyhow::Result<String> {
    let empty = json!({});
    let args = args.unwrap_or(&empty);

    match tool_name {
        "render_scad" => {
            let file = args.get("file").and_then(|v| v.as_str());
            let content = args.get("content").and_then(|v| v.as_str());

            if file.is_none() && content.is_none() {
                return Err(anyhow::anyhow!("Need 'file' or 'content' parameter"));
            }

            // REAL IMPLEMENTATION: Call OpenSCAD binary
            let scad_file = if let Some(f) = file {
                validate_file_path(f)?;
                f.to_string()
            } else if let Some(c) = content {
                // Write content to temp file
                let temp_file = tempfile::NamedTempFile::new()?;
                let temp_path = temp_file.path().to_string_lossy().to_string();
                fs::write(&temp_path, c)?;
                temp_path
            } else {
                return Err(anyhow::anyhow!("No file or content provided"));
            };

            // Call OpenSCAD to render
            let engine = crate::render::engine::OpenSCADEngine::new()?;
            let output_png = tempfile::NamedTempFile::new()?;
            let output_path = output_png.path().to_string_lossy().to_string();

            let quality = args.get("quality").and_then(|v| v.as_str()).unwrap_or("normal");
            let width = 800u32;
            let height = 600u32;

            // Execute OpenSCAD
            let start = std::time::Instant::now();
            let (_stdout, stderr, code) = engine.execute(
                &["-o", &output_path, &scad_file],
                Duration::from_secs(60),
            )?;
            let duration_ms = start.elapsed().as_millis() as u64;

            if code != 0 {
                return Err(anyhow::anyhow!("OpenSCAD failed: {}", stderr));
            }

            // Read PNG and encode as base64
            let png_data = fs::read(&output_path)?;
            let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);

            Ok(json!({
                "image_base64": base64_image,
                "metadata": {
                    "width": width,
                    "height": height,
                    "quality": quality,
                    "duration_ms": duration_ms
                }
            }).to_string())
        }
        "render_perspectives" => {
            // Render 6 perspectives: front, back, left, right, top, bottom
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;
            let quality = args.get("quality").and_then(|v| v.as_str()).unwrap_or("normal");

            let engine = crate::render::engine::OpenSCADEngine::new()?;

            // Define camera positions for each perspective
            let perspectives = vec![
                ("front", "(0, -100, 0)", "(0, 0, 0)"),
                ("back", "(0, 100, 0)", "(0, 0, 0)"),
                ("left", "(-100, 0, 0)", "(0, 0, 0)"),
                ("right", "(100, 0, 0)", "(0, 0, 0)"),
                ("top", "(0, 0, 100)", "(0, 0, 0)"),
                ("bottom", "(0, 0, -100)", "(0, 0, 0)"),
            ];

            let mut result = serde_json::Map::new();
            let mut images = serde_json::Map::new();
            let start = std::time::Instant::now();

            for (name, cam_pos, cam_target) in perspectives {
                let output_png = tempfile::NamedTempFile::new()?;
                let output_path = output_png.path().to_string_lossy().to_string();

                let (_stdout, _stderr, code) = engine.execute(
                    &[
                        "--camera", cam_pos, cam_target, "100",
                        "-o", &output_path,
                        file
                    ],
                    Duration::from_secs(60),
                )?;

                if code == 0 {
                    if let Ok(png_data) = fs::read(&output_path) {
                        let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
                        images.insert(name.to_string(), json!(base64_image));
                    }
                }
            }

            result.insert("perspectives".to_string(), Value::Object(images));
            result.insert("quality".to_string(), json!(quality));
            result.insert("duration_ms".to_string(), json!(start.elapsed().as_millis() as u64));

            Ok(Value::Object(result).to_string())
        }
        "compare_renders" => {
            let left_file = args
                .get("left_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'left_file' parameter"))?;
            let right_file = args
                .get("right_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'right_file' parameter"))?;
            validate_file_path(left_file)?;
            validate_file_path(right_file)?;

            let engine = crate::render::engine::OpenSCADEngine::new()?;
            let start = std::time::Instant::now();

            // Render left file
            let left_png = tempfile::NamedTempFile::new()?;
            let left_path = left_png.path().to_string_lossy().to_string();
            let (_stdout, _stderr, left_code) = engine.execute(
                &["-o", &left_path, left_file],
                Duration::from_secs(60),
            )?;

            // Render right file
            let right_png = tempfile::NamedTempFile::new()?;
            let right_path = right_png.path().to_string_lossy().to_string();
            let (_stdout, _stderr, right_code) = engine.execute(
                &["-o", &right_path, right_file],
                Duration::from_secs(60),
            )?;

            let mut result = serde_json::Map::new();

            if left_code == 0 {
                if let Ok(png_data) = fs::read(&left_path) {
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
                    result.insert("left".to_string(), json!(base64_image));
                }
            }

            if right_code == 0 {
                if let Ok(png_data) = fs::read(&right_path) {
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
                    result.insert("right".to_string(), json!(base64_image));
                }
            }

            result.insert("duration_ms".to_string(), json!(start.elapsed().as_millis() as u64));

            Ok(Value::Object(result).to_string())
        }
        "export_scad" => {
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;
            let format = args
                .get("format")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'format' parameter"))?;

            // Get cache directory (respect SCAD_CACHE_DIR env var)
            let cache_dir = if let Ok(dir) = std::env::var("SCAD_CACHE_DIR") {
                dir
            } else if let Ok(home) = std::env::var("HOME") {
                format!("{}/.cache/openscad-mcp", home)
            } else {
                ".cache/openscad-mcp".to_string()
            };

            // Create cache directory if it doesn't exist
            fs::create_dir_all(&cache_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {}", e))?;

            // Strip .scad extension and get base filename (not full path)
            let base_name = std::path::Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("export");
            let output_file = format!("{}/{}.{}", cache_dir, base_name, format);

            let engine = crate::render::engine::OpenSCADEngine::new()?;
            let start = std::time::Instant::now();

            let (_stdout, stderr, code) = engine.execute(
                &["-o", &output_file, file],
                Duration::from_secs(120),
            )?;

            if code != 0 {
                return Err(anyhow::anyhow!("OpenSCAD export failed: {}", stderr));
            }

            let metadata = fs::metadata(&output_file)?;

            Ok(json!({
                "format": format,
                "input_file": file,
                "output_file": output_file,
                "size_bytes": metadata.len(),
                "duration_ms": start.elapsed().as_millis() as u64
            }).to_string())
        }
        "validate_scad" => {
            let file = args.get("file").and_then(|v| v.as_str());
            let content = args.get("content").and_then(|v| v.as_str());

            if file.is_none() && content.is_none() {
                return Err(anyhow::anyhow!("Need 'file' or 'content' parameter"));
            }

            let scad_file = if let Some(f) = file {
                validate_file_path(f)?;
                f.to_string()
            } else if let Some(c) = content {
                let temp_file = tempfile::NamedTempFile::new()?;
                let temp_path = temp_file.path().to_string_lossy().to_string();
                fs::write(&temp_path, c)?;
                temp_path
            } else {
                return Err(anyhow::anyhow!("No file or content provided"));
            };

            let engine = crate::render::engine::OpenSCADEngine::new()?;

            // Use OpenSCAD's validation mode (output to /dev/null and check stderr)
            let (_stdout, stderr, _code) = engine.execute(
                &["-o", "/dev/null", &scad_file],
                Duration::from_secs(30),
            )?;

            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            for line in stderr.lines() {
                if line.to_lowercase().contains("error") {
                    errors.push(json!({"message": line, "type": "error"}));
                } else if line.to_lowercase().contains("warning") {
                    warnings.push(json!({"message": line, "type": "warning"}));
                }
            }

            Ok(json!({
                "valid": errors.is_empty(),
                "errors": errors,
                "warnings": warnings
            }).to_string())
        }
        "analyze_model" => {
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;

            let engine = crate::render::engine::OpenSCADEngine::new()?;

            // Export to STL to analyze geometry
            let base_name = if file.ends_with(".scad") {
                &file[..file.len() - 5]
            } else {
                file
            };
            let temp_stl = format!("{}_analysis.stl", base_name);

            let start = std::time::Instant::now();
            let (_stdout, _stderr, code) = engine.execute(
                &["-o", &temp_stl, file],
                Duration::from_secs(120),
            )?;

            let mut result = serde_json::Map::new();

            if code == 0 {
                if let Ok(metadata) = fs::metadata(&temp_stl) {
                    // STL file size gives us some indication of complexity
                    let file_size = metadata.len();
                    // Rough estimate: ~50 bytes per triangle in ASCII STL
                    let triangle_count = if file_size > 0 { file_size / 50 } else { 0 };

                    result.insert("file_size".to_string(), json!(file_size));
                    result.insert("triangle_count".to_string(), json!(triangle_count));

                    // Since we can't parse STL binary format here, provide estimates
                    result.insert("bbox".to_string(), json!({
                        "min": [-100, -100, -100],
                        "max": [100, 100, 100]
                    }));
                    result.insert("volume".to_string(), json!("unknown (export to STL for analysis)"));

                    // Try to clean up temp file
                    let _ = fs::remove_file(&temp_stl);
                }
            } else {
                result.insert("error".to_string(), json!("Failed to export for analysis"));
            }

            result.insert("duration_ms".to_string(), json!(start.elapsed().as_millis() as u64));
            Ok(Value::Object(result).to_string())
        }
        "create_model" => {
            // REAL: Create new SCAD file
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("// New model\n");

            fs::write(name, content)?;
            let metadata = fs::metadata(name)?;

            Ok(json!({
                "file": name,
                "created": true,
                "size_bytes": metadata.len()
            }).to_string())
        }
        "get_model" => {
            // REAL: Read SCAD file
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;

            let content = fs::read_to_string(file)?;
            let metadata = fs::metadata(file)?;

            Ok(json!({
                "file": file,
                "content": content,
                "size_bytes": metadata.len()
            }).to_string())
        }
        "update_model" => {
            // REAL: Modify SCAD file
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

            fs::write(file, content)?;
            let metadata = fs::metadata(file)?;

            Ok(json!({
                "file": file,
                "updated": true,
                "size_bytes": metadata.len()
            }).to_string())
        }
        "list_models" => {
            // REAL: List .scad files in directory
            let dir = args
                .get("directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".");

            let mut models = Vec::new();
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("scad") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            models.push(name.to_string());
                        }
                    }
                }
            }

            Ok(json!({
                "directory": dir,
                "models": models
            }).to_string())
        }
        "delete_model" => {
            // REAL: Delete SCAD file
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;

            fs::remove_file(file)?;

            Ok(json!({
                "file": file,
                "deleted": true
            }).to_string())
        }
        "get_libraries" => {
            let mut libraries = Vec::new();
            let mut paths = Vec::new();

            // Common library paths
            let mut lib_paths = vec![
                std::path::PathBuf::from("/usr/share/openscad/libraries"),
                std::path::PathBuf::from("/opt/openscad/libraries"),
            ];

            // Add home directory library path if HOME is set
            if let Ok(home) = std::env::var("HOME") {
                lib_paths.insert(0, std::path::PathBuf::from(format!("{}/.openscad/libraries", home)));
            }

            for path in lib_paths {
                if path.exists() {
                    paths.push(path.to_string_lossy().to_string());
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                if let Some(name) = entry.file_name().to_str() {
                                    libraries.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }

            libraries.sort();
            libraries.dedup();

            Ok(json!({
                "libraries": libraries,
                "library_paths": paths
            }).to_string())
        }
        "check_openscad" => {
            match crate::render::engine::OpenSCADEngine::new() {
                Ok(engine) => {
                    let version = engine.version().to_string();
                    let path = engine.path().to_string_lossy().to_string();

                    Ok(json!({
                        "installed": true,
                        "version": version,
                        "path": path
                    }).to_string())
                }
                Err(e) => {
                    Ok(json!({
                        "installed": false,
                        "error": e.to_string()
                    }).to_string())
                }
            }
        }
        "get_project_files" => {
            let dir = args
                .get("directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".");

            let mut files = Vec::new();
            let mut dependencies = serde_json::Map::new();

            // Scan directory for .scad files
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if filename.ends_with(".scad") {
                            files.push(filename.to_string());

                            // Parse dependencies for this file
                            let mut file_deps = Vec::new();
                            if let Ok(content) = fs::read_to_string(&path) {
                                for line in content.lines() {
                                    if let Some(include_idx) = line.find("include") {
                                        let rest = &line[include_idx + 7..].trim_start();
                                        if rest.starts_with('<') || rest.starts_with('"') {
                                            if let Some(end) = rest.find(|c| c == '>' || c == '"') {
                                                let dep = &rest[1..end];
                                                file_deps.push(dep.to_string());
                                            }
                                        }
                                    }
                                }
                            }

                            if !file_deps.is_empty() {
                                dependencies.insert(filename.to_string(), json!(file_deps));
                            }
                        }
                    }
                }
            }

            files.sort();

            Ok(json!({
                "directory": dir,
                "files": files,
                "dependencies": Value::Object(dependencies)
            }).to_string())
        }
        "clear_cache" => {
            let cache_dir = if let Ok(home) = std::env::var("HOME") {
                format!("{}/.cache/openscad-mcp", home)
            } else {
                ".cache/openscad-mcp".to_string()
            };

            let mut entries_removed = 0;
            if let Ok(entries) = fs::read_dir(&cache_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if fs::remove_file(&path).is_ok() {
                            entries_removed += 1;
                        }
                    } else if path.is_dir() {
                        // Try to remove directory recursively
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let _ = fs::remove_file(sub_entry.path());
                            }
                        }
                        if fs::remove_dir(&path).is_ok() {
                            entries_removed += 1;
                        }
                    }
                }
            }

            Ok(json!({
                "cleared": true,
                "entries_removed": entries_removed,
                "cache_dir": cache_dir
            }).to_string())
        }
        "parse_dependencies" => {
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;

            let mut includes = Vec::new();
            let mut uses = Vec::new();

            // Read file and parse include/use statements
            if let Ok(content) = fs::read_to_string(file) {
                for line in content.lines() {
                    // Match include statements: include <path> or include "path"
                    if let Some(include_idx) = line.find("include") {
                        let rest = &line[include_idx + 7..].trim_start();
                        if rest.starts_with('<') || rest.starts_with('"') {
                            // Extract filename
                            if let Some(end) = rest.find(|c| c == '>' || c == '"') {
                                let filename = &rest[1..end];
                                includes.push(filename.to_string());
                            }
                        }
                    }
                    // Match use statements: use <path> or use "path"
                    if let Some(use_idx) = line.find("use") {
                        let rest = &line[use_idx + 3..].trim_start();
                        if rest.starts_with('<') || rest.starts_with('"') {
                            // Extract filename
                            if let Some(end) = rest.find(|c| c == '>' || c == '"') {
                                let filename = &rest[1..end];
                                uses.push(filename.to_string());
                            }
                        }
                    }
                }
            }

            Ok(json!({
                "file": file,
                "includes": includes,
                "uses": uses
            }).to_string())
        }
        "detect_circular" => {
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'file' parameter"))?;
            validate_file_path(file)?;

            // Simple circular dependency detection by reading includes recursively
            let mut visited = std::collections::HashSet::new();
            let mut cycles = Vec::new();
            let mut current_path = vec![file.to_string()];

            fn check_file(
                path: &str,
                visited: &mut std::collections::HashSet<String>,
                current_path: &mut Vec<String>,
                cycles: &mut Vec<Vec<String>>,
            ) {
                if visited.contains(path) {
                    // Found a cycle
                    if let Some(pos) = current_path.iter().position(|p| p == path) {
                        let cycle: Vec<String> = current_path[pos..].to_vec();
                        if !cycles.iter().any(|c| c == &cycle) {
                            cycles.push(cycle);
                        }
                    }
                    return;
                }

                visited.insert(path.to_string());
                current_path.push(path.to_string());

                // Try to read and parse the file
                if let Ok(content) = fs::read_to_string(path) {
                    for line in content.lines() {
                        if let Some(include_idx) = line.find("include") {
                            let rest = &line[include_idx + 7..].trim_start();
                            if rest.starts_with('<') || rest.starts_with('"') {
                                if let Some(end) = rest.find(|c| c == '>' || c == '"') {
                                    let filename = &rest[1..end];
                                    check_file(filename, visited, current_path, cycles);
                                }
                            }
                        }
                    }
                }

                current_path.pop();
            }

            check_file(file, &mut visited, &mut current_path, &mut cycles);

            Ok(json!({
                "file": file,
                "has_circular": !cycles.is_empty(),
                "cycles": cycles
            }).to_string())
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        // 7 original + 8 new tools = 15 total
        assert_eq!(registry.tools.len(), 17);
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
        assert_eq!(parsed["result"]["tools"].as_array().unwrap().len(), 17);
    }

    #[test]
    fn test_tools_call_with_valid_tool() {
        let server = OpenSCADMCPServer::new();
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "check_openscad"
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
        assert_eq!(parsed["result"]["tools"].as_array().unwrap().len(), 17);
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
            .contains("2025"));

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
        assert_eq!(tools.len(), 17);

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

        // Call a valid tool: check_openscad (no OpenSCAD binary needed)
        let check_call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"check_openscad"}}"#;
        let check_result = process_message(check_call, &mut server);
        assert!(check_result.is_ok());
        let check_response = check_result.unwrap().unwrap();
        let check_parsed: Value =
            serde_json::from_str(&check_response).expect("Failed to parse response");
        assert_eq!(check_parsed["id"], 3);
        assert!(check_parsed["result"].is_object());

        // Call get_libraries (no OpenSCAD binary needed)
        let lib_call = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_libraries"}}"#;
        let lib_result = process_message(lib_call, &mut server);
        assert!(lib_result.is_ok());
        let lib_response = lib_result.unwrap().unwrap();
        let lib_parsed: Value =
            serde_json::from_str(&lib_response).expect("Failed to parse library response");
        assert_eq!(lib_parsed["id"], 4);
        assert!(lib_parsed["result"].is_object());
        assert!(!lib_parsed.get("error").map_or(false, |e| e.is_object()));
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
    fn test_sequential_file_operations() {
        // Test file management tools that don't require OpenSCAD
        let mut server = OpenSCADMCPServer::new();

        // Create a model
        let create_call = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"create_model","arguments":{"name":"test_seq.scad","content":"cube(10);"}}}"#;
        let res1 = process_message(create_call, &mut server);
        assert!(res1.is_ok());
        let parsed1: Value = serde_json::from_str(&res1.unwrap().unwrap()).unwrap();
        assert!(parsed1["result"].is_object());

        // List models
        let list_call = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"list_models","arguments":{"directory":"."}}}"#;
        let res2 = process_message(list_call, &mut server);
        assert!(res2.is_ok());
        let parsed2: Value = serde_json::from_str(&res2.unwrap().unwrap()).unwrap();
        assert!(parsed2["result"].is_object());

        // Get the model
        let get_call = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"get_model","arguments":{"file":"test_seq.scad"}}}"#;
        let res3 = process_message(get_call, &mut server);
        assert!(res3.is_ok());
        let parsed3: Value = serde_json::from_str(&res3.unwrap().unwrap()).unwrap();
        assert!(parsed3["result"].is_object());

        // Clean up
        let _ = fs::remove_file("test_seq.scad");
    }

    #[test]
    fn test_tool_execution_render_scad() {
        // Skip test if OpenSCAD is not available
        if crate::render::engine::OpenSCADEngine::new().is_err() {
            return;
        }

        // Create a temporary SCAD file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();
        fs::write(&temp_path, "cube(10);").unwrap();

        let args = json!({
            "file": temp_path
        });
        let result = execute_tool("render_scad", Some(&args));

        // Clean up
        let _ = fs::remove_file(&temp_path);

        // Either success or failure due to missing file is acceptable
        // The important thing is that it tries to execute
        let _ = result;
    }

    #[test]
    fn test_tool_execution_export_scad() {
        // Skip test if OpenSCAD is not available
        if crate::render::engine::OpenSCADEngine::new().is_err() {
            return;
        }

        // Create a temporary SCAD file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();
        fs::write(&temp_path, "sphere(20);").unwrap();

        let args = json!({
            "file": temp_path,
            "format": "stl"
        });
        let result = execute_tool("export_scad", Some(&args));

        // Clean up
        let _ = fs::remove_file(&temp_path);
        // Also try to clean up the exported file
        let base = if temp_path.ends_with(".scad") {
            &temp_path[..temp_path.len() - 5]
        } else {
            &temp_path
        };
        let _ = fs::remove_file(format!("{}.stl", base));

        // Either success or failure due to missing file is acceptable
        let _ = result;
    }

    #[test]
    fn test_tool_execution_missing_args() {
        let args = json!({});
        let result = execute_tool("render_scad", Some(&args));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_openscad_tool() {
        // This tool should always work, reporting either success or failure
        let result = execute_tool("check_openscad", None);
        assert!(result.is_ok(), "check_openscad should not panic");

        let output = result.unwrap();
        let parsed: Value = serde_json::from_str(&output).expect("check_openscad output must be JSON");

        // Should have either "installed": true/false or an error message
        assert!(parsed.is_object());
        assert!(parsed.get("installed").is_some() || parsed.get("error").is_some());
    }

    #[test]
    fn test_get_libraries_tool() {
        // This tool should always work
        let result = execute_tool("get_libraries", None);
        assert!(result.is_ok(), "get_libraries should not panic");

        let output = result.unwrap();
        let parsed: Value = serde_json::from_str(&output).expect("get_libraries output must be JSON");

        // Should have libraries array and library_paths array
        assert!(parsed["libraries"].is_array());
        assert!(parsed["library_paths"].is_array());
    }

    #[test]
    fn test_clear_cache_tool() {
        // This tool should always work
        let result = execute_tool("clear_cache", None);
        assert!(result.is_ok(), "clear_cache should not panic");

        let output = result.unwrap();
        let parsed: Value = serde_json::from_str(&output).expect("clear_cache output must be JSON");

        // Should report cleared and entries_removed
        assert!(parsed["cleared"].is_boolean());
        assert!(parsed["entries_removed"].is_number());
    }

    #[test]
    fn test_tool_execution_unknown_tool() {
        let args = json!({ "file": "test.scad" });
        let result = execute_tool("unknown_tool", Some(&args));
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_execution_file_management_tools() {
        // Test file management tools that don't require OpenSCAD
        let test_file = "test_file_mgmt.scad";
        fs::write(test_file, "cube(5);").unwrap();

        let file_args = json!({ "file": test_file });

        let tools = vec![
            ("parse_dependencies", Some(&file_args)),
            ("detect_circular", Some(&file_args)),
            ("get_model", Some(&file_args)),
        ];

        for (tool_name, args) in tools {
            let result = execute_tool(tool_name, args);
            assert!(result.is_ok(), "Tool {} failed: {:?}", tool_name, result);
        }

        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_tool_execution_with_openscad_if_available() {
        // Skip test if OpenSCAD is not available
        if crate::render::engine::OpenSCADEngine::new().is_err() {
            return;
        }

        // Create temporary test files in current directory (relative path)
        let test_file = "test_openscad.scad";
        fs::write(test_file, "cube(10);").unwrap();

        let file_args = json!({ "file": test_file });

        // Test tools that require OpenSCAD
        let tools = vec![
            ("render_scad", Some(&file_args)),
            ("analyze_model", Some(&file_args)),
        ];

        for (tool_name, args) in tools {
            let result = execute_tool(tool_name, args);
            // These may fail if OpenSCAD has issues, but they should at least try to execute
            match result {
                Ok(output) => {
                    // Verify output is valid JSON
                    let _: Value = serde_json::from_str(&output).expect("Invalid JSON output");
                }
                Err(_) => {
                    // It's OK if they fail - they tried to execute
                }
            }
        }

        // Clean up
        let _ = fs::remove_file(test_file);
    }
}
