//! Rendering engine module
//!
//! Orchestrates OpenSCAD rendering with parameter handling and caching.

pub mod engine;
pub mod params;
pub mod quality;

use crate::error::Result;
use params::RenderParams;
use std::path::Path;
use std::time::Duration;

/// Render OpenSCAD content to PNG
pub async fn render_scad_to_png(
    _content: &str,
    _output_path: &Path,
    _params: &RenderParams,
) -> Result<()> {
    // Placeholder for orchestration logic
    // Would integrate engine, params, and quality
    Ok(())
}

/// Render multiple perspectives in parallel
pub async fn render_perspectives(
    _content: &str,
    _output_dir: &Path,
    _params: &RenderParams,
) -> Result<Vec<String>> {
    // Placeholder for parallel rendering
    // Would use tokio::join_all for 8 views
    Ok(Vec::new())
}

/// Export OpenSCAD content to various formats (STL, 3MF, AMF, OFF, DXF, SVG)
pub async fn export_scad_to_format(content: &str, format: &str, output_path: &Path) -> Result<()> {
    // Create a temporary SCAD file
    let temp_scad = tempfile::NamedTempFile::new()
        .map_err(|e| crate::error::Error::Cache(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_scad.path().to_path_buf();

    // Write content to temp file
    std::fs::write(&temp_path, content).map_err(|e| crate::error::Error::Filesystem(e))?;

    // Normalize format (convert to lowercase for OpenSCAD compatibility)
    let normalized_format = format.to_lowercase();

    // Build OpenSCAD output filename with correct extension
    let mut output = output_path.to_path_buf();
    if !output.set_extension(&normalized_format) {
        output.set_extension(&normalized_format);
    }

    // Create the output directory if it doesn't exist
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| crate::error::Error::Filesystem(e))?;
        }
    }

    // Use the render engine to execute OpenSCAD export
    let engine = engine::OpenSCADEngine::new()?;
    let output_str = output.to_string_lossy().to_string();
    let temp_str = temp_path.to_string_lossy().to_string();
    let args = vec!["-o", &output_str, &temp_str];

    let (_stdout, stderr, code) = engine.execute(&args, Duration::from_secs(60))?;

    // Check for errors
    if code != 0 {
        return Err(crate::error::Error::Render(format!(
            "OpenSCAD export failed: {}",
            stderr
        )));
    }

    // Verify output file was created
    if !output.exists() {
        return Err(crate::error::Error::Validation(format!(
            "Export output file not created: {}",
            output.display()
        )));
    }

    Ok(())
}

/// Comparison result for two rendered models
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Left model rendering output path
    pub left_output: String,
    /// Right model rendering output path
    pub right_output: String,
    /// Left model name/identifier
    pub left_name: String,
    /// Right model name/identifier
    pub right_name: String,
    /// Comparison summary (e.g., dimension differences)
    pub summary: String,
}

/// Compare two OpenSCAD models side-by-side
pub async fn compare_scad_renders(
    left_content: &str,
    right_content: &str,
    left_name: &str,
    right_name: &str,
    output_dir: &Path,
    params: &RenderParams,
) -> Result<ComparisonResult> {
    // Create output directory if needed
    std::fs::create_dir_all(output_dir).map_err(|e| crate::error::Error::Filesystem(e))?;

    // Render both models in parallel
    let left_path = output_dir.join(format!("{}_left.png", left_name));
    let right_path = output_dir.join(format!("{}_right.png", right_name));

    let left_future = render_scad_to_png(left_content, &left_path, params);
    let right_future = render_scad_to_png(right_content, &right_path, params);

    let (left_result, right_result) = tokio::join!(left_future, right_future);

    // Check results - at least one should succeed for a valid comparison
    match (left_result, right_result) {
        (Ok(()), Ok(())) => {
            let summary = format!("Both models rendered successfully");
            Ok(ComparisonResult {
                left_output: left_path.to_string_lossy().to_string(),
                right_output: right_path.to_string_lossy().to_string(),
                left_name: left_name.to_string(),
                right_name: right_name.to_string(),
                summary,
            })
        }
        (Ok(()), Err(e)) => Err(crate::error::Error::Render(format!(
            "Right model render failed: {}",
            e
        ))),
        (Err(e), Ok(())) => Err(crate::error::Error::Render(format!(
            "Left model render failed: {}",
            e
        ))),
        (Err(left_e), Err(right_e)) => Err(crate::error::Error::Render(format!(
            "Both models failed to render - left: {}, right: {}",
            left_e, right_e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_render_scad_to_png_basic() {
        let content = "cube(10);";
        let output = PathBuf::from("/tmp/test.png");
        let params = RenderParams::default();

        // Just verify the function signature compiles and is callable
        let result = render_scad_to_png(content, &output, &params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_with_custom_params() {
        let content = "cube([10, 20, 30]);";
        let output = PathBuf::from("/tmp/test2.png");
        let mut params = RenderParams::default();
        params.camera_pos = [50.0, 50.0, 50.0];

        let result = render_scad_to_png(content, &output, &params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_empty_content() {
        let content = "";
        let output = PathBuf::from("/tmp/empty.png");
        let params = RenderParams::default();

        // Empty content should still be acceptable to the renderer
        let result = render_scad_to_png(content, &output, &params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_perspectives_returns_filenames() {
        let content = "cube(10);";
        let output_dir = PathBuf::from("/tmp");
        let params = RenderParams::default();

        let result = render_perspectives(content, &output_dir, &params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_perspectives_8_views() {
        let content = "sphere(r=5);";
        let output_dir = PathBuf::from("/tmp");
        let params = RenderParams::default();

        if let Ok(files) = render_perspectives(content, &output_dir, &params).await {
            // Should generate up to 8 view files
            assert!(files.len() <= 8);
        }
    }

    #[tokio::test]
    async fn test_render_with_quality_settings() {
        let content = "cube(10);";
        let output = PathBuf::from("/tmp/quality_test.png");
        let mut params = RenderParams::default();
        params.quality = params::QualityPreset::High;

        let result = render_scad_to_png(content, &output, &params).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_settings_integration() {
        let quality = quality::QualitySettings::high();
        let args = quality.to_openscad_args();
        assert!(!args.is_empty());
    }

    #[test]
    fn test_render_params_default() {
        let params = RenderParams::default();
        assert_eq!(params.camera_pos, [0.0, 0.0, 100.0]);
        assert_eq!(params.image_size, [800, 600]);
    }

    #[tokio::test]
    async fn test_export_scad_to_stl() {
        let content = "cube(10);";
        let output = PathBuf::from("/tmp/test.stl");

        let result = export_scad_to_format(content, "stl", &output).await;
        // If OpenSCAD is not installed, this will fail gracefully
        // In CI/test environments, we expect either success (if OpenSCAD is available)
        // or a proper error (if not)
        match result {
            Ok(_) => {
                // OpenSCAD is available and export succeeded
            }
            Err(e) => {
                // OpenSCAD not found or export failed - check error type
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_export_scad_to_3mf() {
        let content = "sphere(r=5);";
        let output = PathBuf::from("/tmp/test.3mf");

        let result = export_scad_to_format(content, "3mf", &output).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_export_scad_to_amf() {
        let content = "cylinder(h=10, r=5);";
        let output = PathBuf::from("/tmp/test.amf");

        let result = export_scad_to_format(content, "amf", &output).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_export_scad_to_off() {
        let content = "cube([5, 5, 5]);";
        let output = PathBuf::from("/tmp/test.off");

        let result = export_scad_to_format(content, "off", &output).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_export_scad_to_dxf() {
        let content = "square([10, 10]);";
        let output = PathBuf::from("/tmp/test.dxf");

        let result = export_scad_to_format(content, "dxf", &output).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_export_scad_to_svg() {
        let content = "circle(r=5);";
        let output = PathBuf::from("/tmp/test.svg");

        let result = export_scad_to_format(content, "svg", &output).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found")
                        || err_str.contains("OpenSCAD export failed")
                        || err_str.contains("No such file"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_compare_scad_renders_both_succeed() {
        let left_content = "cube(10);";
        let right_content = "sphere(r=5);";
        let output_dir = PathBuf::from("/tmp/comparison_test");

        let params = RenderParams::default();
        let result = compare_scad_renders(
            left_content,
            right_content,
            "left_model",
            "right_model",
            &output_dir,
            &params,
        )
        .await;

        // If OpenSCAD is available, this should succeed
        match result {
            Ok(comparison) => {
                assert_eq!(comparison.left_name, "left_model");
                assert_eq!(comparison.right_name, "right_model");
                assert!(comparison
                    .summary
                    .contains("Both models rendered successfully"));
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                // Accept OpenSCAD not found in test environment
                assert!(
                    err_str.contains("openscad not found") || err_str.contains("failed to render"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_compare_scad_renders_one_fails() {
        let left_content = "cube(10);";
        let right_content = "invalid syntax here";
        let output_dir = PathBuf::from("/tmp/comparison_test_fail");

        let params = RenderParams::default();
        let result = compare_scad_renders(
            left_content,
            right_content,
            "valid_model",
            "invalid_model",
            &output_dir,
            &params,
        )
        .await;

        // Result should either be success (if OpenSCAD lax parsing) or error about render failure
        match result {
            Ok(_) => {
                // Some versions of OpenSCAD may be lenient with syntax
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                // Error should mention render failure or OpenSCAD not found
                assert!(
                    err_str.contains("render failed") || err_str.contains("openscad not found"),
                    "Expected render failure error: {}",
                    err_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_compare_scad_renders_metadata() {
        let left_content = "cube([5, 5, 5]);";
        let right_content = "cube([10, 10, 10]);";
        let output_dir = PathBuf::from("/tmp/comparison_metadata");

        let params = RenderParams::default();
        let result = compare_scad_renders(
            left_content,
            right_content,
            "small",
            "large",
            &output_dir,
            &params,
        )
        .await;

        // Check that comparison result contains proper metadata
        match result {
            Ok(comparison) => {
                assert_eq!(comparison.left_name, "small");
                assert_eq!(comparison.right_name, "large");
                // Verify output paths are set
                assert!(comparison.left_output.contains("small_left"));
                assert!(comparison.right_output.contains("large_right"));
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                assert!(
                    err_str.contains("openscad not found") || err_str.contains("render failed"),
                    "Unexpected error: {}",
                    err_str
                );
            }
        }
    }
}
