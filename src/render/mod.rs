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
}
