//! Model analysis module
//!
//! Provides validation, mesh analysis, and model inspection capabilities.

pub mod mesh;
pub mod validator;

use crate::error::Result;
use crate::render::params::RenderParams;
use serde::{Deserialize, Serialize};

pub use mesh::{parse_stl, MeshMetrics};
pub use validator::{validate_scad, ValidationResult};

/// Complete analysis result for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAnalysis {
    /// Validation results
    pub validation: ValidationResult,

    /// Mesh metrics (if rendering to STL succeeded)
    pub mesh: Option<MeshMetrics>,
}

/// Analyze a model: validate syntax, export to STL, compute metrics
pub async fn analyze_model(
    _content: &str,
    _params: &RenderParams,
    _openscad_path: &str,
) -> Result<ModelAnalysis> {
    // Placeholder for orchestration logic
    // Would: validate → render → export to STL → parse mesh → return metrics
    Ok(ModelAnalysis {
        validation: ValidationResult::default(),
        mesh: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_analysis_default_validation() {
        let analysis = ModelAnalysis {
            validation: ValidationResult::default(),
            mesh: None,
        };
        assert!(analysis.validation.valid);
        assert!(analysis.mesh.is_none());
    }

    #[test]
    fn test_model_analysis_with_mesh() {
        let analysis = ModelAnalysis {
            validation: ValidationResult::default(),
            mesh: Some(MeshMetrics {
                bbox_min: [0.0, 0.0, 0.0],
                bbox_max: [10.0, 10.0, 10.0],
                triangle_count: 12,
                vertex_count: 8,
            }),
        };
        assert!(analysis.mesh.is_some());
        let mesh = analysis.mesh.unwrap();
        assert_eq!(mesh.triangle_count, 12);
    }

    #[test]
    fn test_model_analysis_serialization() {
        let analysis = ModelAnalysis {
            validation: ValidationResult::default(),
            mesh: Some(MeshMetrics {
                bbox_min: [1.0, 2.0, 3.0],
                bbox_max: [4.0, 5.0, 6.0],
                triangle_count: 100,
                vertex_count: 300,
            }),
        };
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("validation"));
        assert!(json.contains("mesh"));
    }

    #[tokio::test]
    async fn test_analyze_model_placeholder() {
        let content = "cube(10);";
        let params = RenderParams::default();
        let result = analyze_model(content, &params, "/usr/bin/openscad").await;
        assert!(result.is_ok());
    }
}
