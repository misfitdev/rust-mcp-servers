//! MCP Server implementation
//!
//! Handles tool registration, request handling, and protocol communication.

use crate::error::Result;

/// MCP Server for OpenSCAD model operations
pub struct MCPServer {
    name: String,
    version: String,
    tools: Vec<ToolDefinition>,
}

/// Tool definition for MCP server
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            tools: Vec::new(),
        }
    }

    /// Register a tool with the server
    pub fn register_tool(&mut self, tool: ToolDefinition) -> Result<()> {
        if self.tools.iter().any(|t| t.name == tool.name) {
            return Err(crate::error::Error::Config(format!(
                "Tool '{}' is already registered",
                tool.name
            )));
        }
        self.tools.push(tool);
        Ok(())
    }

    /// Get all registered tools
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.iter().collect()
    }

    /// Get server info
    pub fn info(&self) -> (String, String) {
        (self.name.clone(), self.version.clone())
    }

    /// Get a specific tool by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.iter().find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = MCPServer::new("openscad-mcp".to_string(), "0.1.0".to_string());
        let (name, version) = server.info();
        assert_eq!(name, "openscad-mcp");
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn test_empty_tool_list() {
        let server = MCPServer::new("test".to_string(), "1.0".to_string());
        assert_eq!(server.list_tools().len(), 0);
    }

    #[test]
    fn test_register_single_tool() {
        let mut server = MCPServer::new("test".to_string(), "1.0".to_string());
        let tool = ToolDefinition {
            name: "render_model".to_string(),
            description: "Render a model".to_string(),
            input_schema: "{}".to_string(),
        };

        let result = server.register_tool(tool);
        assert!(result.is_ok());
        assert_eq!(server.list_tools().len(), 1);
    }

    #[test]
    fn test_register_multiple_tools() {
        let mut server = MCPServer::new("test".to_string(), "1.0".to_string());

        for i in 0..3 {
            let tool = ToolDefinition {
                name: format!("tool_{}", i),
                description: format!("Tool {}", i),
                input_schema: "{}".to_string(),
            };
            assert!(server.register_tool(tool).is_ok());
        }

        assert_eq!(server.list_tools().len(), 3);
    }

    #[test]
    fn test_register_duplicate_tool() {
        let mut server = MCPServer::new("test".to_string(), "1.0".to_string());
        let tool = ToolDefinition {
            name: "render".to_string(),
            description: "Render".to_string(),
            input_schema: "{}".to_string(),
        };

        assert!(server.register_tool(tool.clone()).is_ok());
        assert!(server.register_tool(tool).is_err());
    }

    #[test]
    fn test_get_tool() {
        let mut server = MCPServer::new("test".to_string(), "1.0".to_string());
        let tool = ToolDefinition {
            name: "render".to_string(),
            description: "Render model".to_string(),
            input_schema: "{}".to_string(),
        };

        server.register_tool(tool).unwrap();
        let found = server.get_tool("render");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "render");
    }

    #[test]
    fn test_get_nonexistent_tool() {
        let server = MCPServer::new("test".to_string(), "1.0".to_string());
        let found = server.get_tool("nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_tool_definition_clone() {
        let tool = ToolDefinition {
            name: "test".to_string(),
            description: "test".to_string(),
            input_schema: "{}".to_string(),
        };
        let cloned = tool.clone();
        assert_eq!(tool.name, cloned.name);
    }

    #[test]
    fn test_tool_definition_debug() {
        let tool = ToolDefinition {
            name: "test".to_string(),
            description: "test".to_string(),
            input_schema: "{}".to_string(),
        };
        let debug_str = format!("{:?}", tool);
        assert!(debug_str.contains("test"));
    }
}
