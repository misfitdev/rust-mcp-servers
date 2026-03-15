//! Rendering parameter parsing and validation
//!
//! Handles camera position, target, image size, variables, and quality settings.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rendering parameters for OpenSCAD output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderParams {
    /// Camera position [x, y, z]
    pub camera_pos: [f64; 3],

    /// Camera target/look-at [x, y, z]
    pub camera_target: [f64; 3],

    /// Image dimensions [width, height]
    pub image_size: [u32; 2],

    /// Quality preset: draft, normal, high
    pub quality: QualityPreset,

    /// OpenSCAD variables
    pub variables: HashMap<String, String>,

    /// Color scheme name
    pub color_scheme: Option<String>,
}

/// Quality preset for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    Draft,
    Normal,
    High,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            camera_pos: [0.0, 0.0, 100.0],
            camera_target: [0.0, 0.0, 0.0],
            image_size: [800, 600],
            quality: QualityPreset::Normal,
            variables: HashMap::new(),
            color_scheme: None,
        }
    }
}

impl RenderParams {
    /// Parse camera position from string
    /// Supports formats: "x,y,z" or "[x, y, z]" or "{x: x, y: y, z: z}"
    pub fn parse_camera_pos(input: &str) -> Result<[f64; 3]> {
        let trimmed = input.trim().trim_start_matches('[').trim_end_matches(']');

        if trimmed.contains(',') {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() != 3 {
                return Err(Error::Validation(
                    "Camera position must have 3 values".to_string(),
                ));
            }

            let x = parts[0]
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Validation("Invalid camera x coordinate".to_string()))?;
            let y = parts[1]
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Validation("Invalid camera y coordinate".to_string()))?;
            let z = parts[2]
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Validation("Invalid camera z coordinate".to_string()))?;

            Ok([x, y, z])
        } else {
            Err(Error::Validation(
                "Camera position format: x,y,z".to_string(),
            ))
        }
    }

    /// Parse camera target
    pub fn parse_camera_target(input: &str) -> Result<[f64; 3]> {
        Self::parse_camera_pos(input)
    }

    /// Parse image size
    pub fn parse_image_size(input: &str) -> Result<[u32; 2]> {
        let trimmed = input
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim_start_matches('{')
            .trim_end_matches('}');

        if trimmed.contains(',') {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() != 2 {
                return Err(Error::Validation(
                    "Image size must be [width, height]".to_string(),
                ));
            }

            let width = parts[0]
                .trim()
                .parse::<u32>()
                .map_err(|_| Error::Validation("Invalid image width".to_string()))?;
            let height = parts[1]
                .trim()
                .parse::<u32>()
                .map_err(|_| Error::Validation("Invalid image height".to_string()))?;

            if width == 0 || height == 0 {
                return Err(Error::Validation(
                    "Image dimensions must be positive".to_string(),
                ));
            }

            Ok([width, height])
        } else {
            Err(Error::Validation(
                "Image size format: width,height".to_string(),
            ))
        }
    }

    /// Parse quality preset
    pub fn parse_quality(input: &str) -> Result<QualityPreset> {
        match input.to_lowercase().as_str() {
            "draft" => Ok(QualityPreset::Draft),
            "normal" => Ok(QualityPreset::Normal),
            "high" => Ok(QualityPreset::High),
            _ => Err(Error::Validation(
                "Quality must be: draft, normal, or high".to_string(),
            )),
        }
    }

    /// Parse variables from JSON string
    pub fn parse_variables(input: &str) -> Result<HashMap<String, String>> {
        serde_json::from_str(input)
            .map_err(|e| Error::Serialization(format!("Failed to parse variables: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_camera_pos_csv() {
        let result = RenderParams::parse_camera_pos("10.0,20.0,30.0");
        assert!(result.is_ok());
        let pos = result.unwrap();
        assert_eq!(pos, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_parse_camera_pos_with_brackets() {
        let result = RenderParams::parse_camera_pos("[10.0, 20.0, 30.0]");
        assert!(result.is_ok());
        let pos = result.unwrap();
        assert_eq!(pos, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_parse_camera_pos_invalid_count() {
        let result = RenderParams::parse_camera_pos("10.0,20.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_camera_pos_invalid_number() {
        let result = RenderParams::parse_camera_pos("x,y,z");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_camera_pos_zero_values() {
        let result = RenderParams::parse_camera_pos("0.0,0.0,0.0");
        assert!(result.is_ok());
        let pos = result.unwrap();
        assert_eq!(pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_camera_target() {
        let result = RenderParams::parse_camera_target("0.0,0.0,0.0");
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_image_size_csv() {
        let result = RenderParams::parse_image_size("800,600");
        assert!(result.is_ok());
        let size = result.unwrap();
        assert_eq!(size, [800, 600]);
    }

    #[test]
    fn test_parse_image_size_with_brackets() {
        let result = RenderParams::parse_image_size("[1024, 768]");
        assert!(result.is_ok());
        let size = result.unwrap();
        assert_eq!(size, [1024, 768]);
    }

    #[test]
    fn test_parse_image_size_zero_width() {
        let result = RenderParams::parse_image_size("0,600");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_image_size_zero_height() {
        let result = RenderParams::parse_image_size("800,0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_image_size_invalid_format() {
        let result = RenderParams::parse_image_size("800x600");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_quality_draft() {
        let result = RenderParams::parse_quality("draft");
        assert_eq!(result.unwrap(), QualityPreset::Draft);
    }

    #[test]
    fn test_parse_quality_normal() {
        let result = RenderParams::parse_quality("normal");
        assert_eq!(result.unwrap(), QualityPreset::Normal);
    }

    #[test]
    fn test_parse_quality_high() {
        let result = RenderParams::parse_quality("high");
        assert_eq!(result.unwrap(), QualityPreset::High);
    }

    #[test]
    fn test_parse_quality_case_insensitive() {
        assert_eq!(
            RenderParams::parse_quality("DRAFT").unwrap(),
            QualityPreset::Draft
        );
        assert_eq!(
            RenderParams::parse_quality("Normal").unwrap(),
            QualityPreset::Normal
        );
    }

    #[test]
    fn test_parse_quality_invalid() {
        let result = RenderParams::parse_quality("ultra");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_variables_json() {
        let json = r#"{"thickness": "2.5", "width": "10"}"#;
        let result = RenderParams::parse_variables(json);
        assert!(result.is_ok());
        let vars = result.unwrap();
        assert_eq!(vars.get("thickness"), Some(&"2.5".to_string()));
    }

    #[test]
    fn test_parse_variables_empty() {
        let result = RenderParams::parse_variables("{}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_variables_invalid_json() {
        let result = RenderParams::parse_variables("{invalid}");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_params() {
        let params = RenderParams::default();
        assert_eq!(params.camera_pos, [0.0, 0.0, 100.0]);
        assert_eq!(params.image_size, [800, 600]);
        assert_eq!(params.quality, QualityPreset::Normal);
    }

    #[test]
    fn test_quality_preset_copy() {
        let q1 = QualityPreset::High;
        let q2 = q1;
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_render_params_serialization() {
        let params = RenderParams::default();
        let json = serde_json::to_string(&params);
        assert!(json.is_ok());
    }
}
