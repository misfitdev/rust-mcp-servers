//! MCP tools module
//!
//! Implements all tools exposed to the Model Context Protocol.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Request to render a single view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSingleRequest {
    /// OpenSCAD source code as string
    pub content: Option<String>,

    /// Path to OpenSCAD file
    pub file: Option<String>,

    /// Camera position [x, y, z]
    pub camera_pos: Option<String>,

    /// Camera target [x, y, z]
    pub camera_target: Option<String>,

    /// Image size [width, height]
    pub image_size: Option<String>,

    /// Quality: draft, normal, high
    pub quality: Option<String>,
}

/// Response from render tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    /// Base64-encoded PNG image
    pub image_base64: String,

    /// Metadata about the render
    pub metadata: RenderMetadata,
}

/// Metadata about a render operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetadata {
    /// Rendering duration in milliseconds
    pub duration_ms: u64,

    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Quality setting used
    pub quality: String,
}

/// Validate render request
pub fn validate_render_request(req: &RenderSingleRequest) -> Result<()> {
    // Check that either content or file is provided, not both
    match (&req.content, &req.file) {
        (Some(_), Some(_)) => {
            return Err(crate::error::Error::Validation(
                "Provide either content or file, not both".to_string(),
            ))
        }
        (None, None) => {
            return Err(crate::error::Error::Validation(
                "Provide either content or file".to_string(),
            ))
        }
        _ => {}
    }

    // Validate file path if provided
    if let Some(file_path) = &req.file {
        validate_file_path(file_path)?;
    }

    // Validate content size if provided (max 10MB)
    if let Some(content) = &req.content {
        const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;
        if content.len() > MAX_CONTENT_SIZE {
            return Err(crate::error::Error::Validation(format!(
                "Content exceeds maximum size of {} bytes",
                MAX_CONTENT_SIZE
            )));
        }
    }

    Ok(())
}

/// Validate file path to prevent directory traversal attacks
fn validate_file_path(path: &str) -> Result<()> {
    use std::path::Path;

    let path_obj = Path::new(path);

    // Reject absolute paths
    if path_obj.is_absolute() {
        return Err(crate::error::Error::Validation(
            "Absolute file paths are not allowed".to_string(),
        ));
    }

    // Reject parent directory references
    for component in path_obj.components() {
        use std::path::Component;
        if matches!(component, Component::ParentDir) {
            return Err(crate::error::Error::Validation(
                "Path traversal with '..' is not allowed".to_string(),
            ));
        }
    }

    // Reject paths with null bytes
    if path.contains('\0') {
        return Err(crate::error::Error::Validation(
            "Path contains invalid null bytes".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_request_with_content() {
        let req = RenderSingleRequest {
            content: Some("cube(10);".to_string()),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_with_file() {
        let req = RenderSingleRequest {
            content: None,
            file: Some("models/model.scad".to_string()),
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_rejects_absolute_path() {
        let req = RenderSingleRequest {
            content: None,
            file: Some("/etc/passwd".to_string()),
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_request_rejects_path_traversal() {
        let req = RenderSingleRequest {
            content: None,
            file: Some("../../../etc/passwd".to_string()),
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_request_rejects_large_content() {
        let large_content = "x".repeat(11 * 1024 * 1024); // 11MB
        let req = RenderSingleRequest {
            content: Some(large_content),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_request_accepts_valid_size_content() {
        let content = "x".repeat(5 * 1024 * 1024); // 5MB (under limit)
        let req = RenderSingleRequest {
            content: Some(content),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_with_both_fails() {
        let req = RenderSingleRequest {
            content: Some("cube(10);".to_string()),
            file: Some("model.scad".to_string()),
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_request_with_neither_fails() {
        let req = RenderSingleRequest {
            content: None,
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let result = validate_render_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_request_with_camera_pos() {
        let mut req = RenderSingleRequest {
            content: Some("sphere(5);".to_string()),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        req.camera_pos = Some("10.0,20.0,30.0".to_string());
        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_with_quality() {
        let mut req = RenderSingleRequest {
            content: Some("cube(5);".to_string()),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        req.quality = Some("high".to_string());
        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_serialization() {
        let req = RenderSingleRequest {
            content: Some("cube(10);".to_string()),
            file: None,
            camera_pos: Some("0,0,100".to_string()),
            camera_target: None,
            image_size: Some("800,600".to_string()),
            quality: Some("normal".to_string()),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
    }

    #[test]
    fn test_render_response_serialization() {
        let resp = RenderResponse {
            image_base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            metadata: RenderMetadata {
                duration_ms: 1234,
                width: 800,
                height: 600,
                quality: "normal".to_string(),
            },
        };

        let json = serde_json::to_string(&resp);
        assert!(json.is_ok());
    }

    #[test]
    fn test_render_metadata() {
        let metadata = RenderMetadata {
            duration_ms: 500,
            width: 1024,
            height: 768,
            quality: "high".to_string(),
        };

        assert_eq!(metadata.width, 1024);
        assert_eq!(metadata.height, 768);
    }

    #[test]
    fn test_render_request_default_quality() {
        let req = RenderSingleRequest {
            content: Some("cube(10);".to_string()),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        // Should accept None for quality (use default)
        let result = validate_render_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_request_clone() {
        let req = RenderSingleRequest {
            content: Some("cube(10);".to_string()),
            file: None,
            camera_pos: None,
            camera_target: None,
            image_size: None,
            quality: None,
        };

        let cloned = req.clone();
        assert_eq!(req.content, cloned.content);
    }
}
