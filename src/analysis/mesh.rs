//! Mesh analysis for STL files
//!
//! Parses STL format and computes mesh metrics (bounding box, triangle count).

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Mesh metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMetrics {
    /// Bounding box minimum coordinates [x, y, z]
    pub bbox_min: [f64; 3],

    /// Bounding box maximum coordinates [x, y, z]
    pub bbox_max: [f64; 3],

    /// Number of triangles/facets
    pub triangle_count: u32,

    /// Total number of vertices
    pub vertex_count: u32,
}

impl MeshMetrics {
    /// Get bounding box dimensions
    pub fn dimensions(&self) -> [f64; 3] {
        [
            self.bbox_max[0] - self.bbox_min[0],
            self.bbox_max[1] - self.bbox_min[1],
            self.bbox_max[2] - self.bbox_min[2],
        ]
    }

    /// Get bounding box volume
    pub fn volume(&self) -> f64 {
        let dims = self.dimensions();
        dims[0] * dims[1] * dims[2]
    }

    /// Get bounding box center
    pub fn center(&self) -> [f64; 3] {
        [
            (self.bbox_min[0] + self.bbox_max[0]) / 2.0,
            (self.bbox_min[1] + self.bbox_max[1]) / 2.0,
            (self.bbox_min[2] + self.bbox_max[2]) / 2.0,
        ]
    }
}

/// Parse ASCII STL file
pub fn parse_stl(path: impl AsRef<Path>) -> Result<MeshMetrics> {
    let file = File::open(path.as_ref()).map_err(|e| Error::Filesystem(e))?;
    let reader = BufReader::new(file);

    let mut metrics = MeshMetrics {
        bbox_min: [f64::INFINITY; 3],
        bbox_max: [f64::NEG_INFINITY; 3],
        triangle_count: 0,
        vertex_count: 0,
    };

    let mut in_facet = false;
    let mut in_loop = false;

    for line in reader.lines() {
        let line = line.map_err(|e| Error::Cache(format!("Error reading STL file: {}", e)))?;
        let trimmed = line.trim();

        if trimmed.starts_with("facet") {
            in_facet = true;
            metrics.triangle_count += 1;
        } else if trimmed.starts_with("endfacet") {
            in_facet = false;
        } else if trimmed.starts_with("outer loop") {
            in_loop = true;
        } else if trimmed.starts_with("endloop") {
            in_loop = false;
        } else if trimmed.starts_with("vertex") && in_loop {
            // Parse vertex coordinates
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() == 4 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[1].parse::<f64>(),
                    parts[2].parse::<f64>(),
                    parts[3].parse::<f64>(),
                ) {
                    metrics.vertex_count += 1;

                    // Update bounding box
                    metrics.bbox_min[0] = metrics.bbox_min[0].min(x);
                    metrics.bbox_min[1] = metrics.bbox_min[1].min(y);
                    metrics.bbox_min[2] = metrics.bbox_min[2].min(z);

                    metrics.bbox_max[0] = metrics.bbox_max[0].max(x);
                    metrics.bbox_max[1] = metrics.bbox_max[1].max(y);
                    metrics.bbox_max[2] = metrics.bbox_max[2].max(z);
                }
            }
        }
    }

    // Validate we found data
    if metrics.triangle_count == 0 {
        return Err(Error::Validation(
            "No triangles found in STL file".to_string(),
        ));
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_metrics_dimensions() {
        let metrics = MeshMetrics {
            bbox_min: [0.0, 0.0, 0.0],
            bbox_max: [10.0, 20.0, 30.0],
            triangle_count: 100,
            vertex_count: 300,
        };
        let dims = metrics.dimensions();
        assert_eq!(dims, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_mesh_metrics_volume() {
        let metrics = MeshMetrics {
            bbox_min: [0.0, 0.0, 0.0],
            bbox_max: [10.0, 10.0, 10.0],
            triangle_count: 12,
            vertex_count: 8,
        };
        assert_eq!(metrics.volume(), 1000.0);
    }

    #[test]
    fn test_mesh_metrics_center() {
        let metrics = MeshMetrics {
            bbox_min: [0.0, 0.0, 0.0],
            bbox_max: [10.0, 20.0, 30.0],
            triangle_count: 100,
            vertex_count: 300,
        };
        let center = metrics.center();
        assert_eq!(center, [5.0, 10.0, 15.0]);
    }

    #[test]
    fn test_mesh_metrics_negative_coordinates() {
        let metrics = MeshMetrics {
            bbox_min: [-10.0, -10.0, -10.0],
            bbox_max: [10.0, 10.0, 10.0],
            triangle_count: 12,
            vertex_count: 8,
        };
        let center = metrics.center();
        assert_eq!(center, [0.0, 0.0, 0.0]);
        assert_eq!(metrics.dimensions(), [20.0, 20.0, 20.0]);
    }

    #[test]
    fn test_mesh_metrics_single_point() {
        let metrics = MeshMetrics {
            bbox_min: [5.0, 5.0, 5.0],
            bbox_max: [5.0, 5.0, 5.0],
            triangle_count: 0,
            vertex_count: 1,
        };
        let dims = metrics.dimensions();
        assert_eq!(dims, [0.0, 0.0, 0.0]);
        assert_eq!(metrics.volume(), 0.0);
    }

    #[test]
    fn test_parse_stl_nonexistent() {
        let result = parse_stl("nonexistent.stl");
        assert!(result.is_err());
    }

    #[test]
    fn test_mesh_metrics_serialization() {
        let metrics = MeshMetrics {
            bbox_min: [1.0, 2.0, 3.0],
            bbox_max: [4.0, 5.0, 6.0],
            triangle_count: 50,
            vertex_count: 150,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("bbox_min"));
        assert!(json.contains("triangle_count"));
    }

    #[test]
    fn test_mesh_metrics_large_counts() {
        let metrics = MeshMetrics {
            bbox_min: [0.0, 0.0, 0.0],
            bbox_max: [100.0, 100.0, 100.0],
            triangle_count: 1_000_000,
            vertex_count: 3_000_000,
        };
        assert_eq!(metrics.triangle_count, 1_000_000);
        assert_eq!(metrics.vertex_count, 3_000_000);
    }

    #[test]
    fn test_mesh_metrics_asymmetric_bbox() {
        let metrics = MeshMetrics {
            bbox_min: [-5.0, 0.0, 2.0],
            bbox_max: [15.0, 100.0, 8.0],
            triangle_count: 42,
            vertex_count: 126,
        };
        let dims = metrics.dimensions();
        assert_eq!(dims[0], 20.0);
        assert_eq!(dims[1], 100.0);
        assert_eq!(dims[2], 6.0);
    }
}
