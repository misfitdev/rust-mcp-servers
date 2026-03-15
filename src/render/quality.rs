//! Quality presets and OpenSCAD parameter mapping
//!
//! Maps quality presets (draft/normal/high) to OpenSCAD $fn, $fa, $fs parameters.

use serde::{Deserialize, Serialize};

/// OpenSCAD quality parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    /// $fn: fragments (0 = adaptive)
    pub fn_param: u32,

    /// $fa: fragment angle (degrees)
    pub fa_param: f64,

    /// $fs: fragment size (mm)
    pub fs_param: f64,
}

impl QualitySettings {
    /// Create quality settings for draft mode
    pub fn draft() -> Self {
        Self {
            fn_param: 0,
            fa_param: 12.0,
            fs_param: 2.0,
        }
    }

    /// Create quality settings for normal mode
    pub fn normal() -> Self {
        Self {
            fn_param: 0,
            fa_param: 6.0,
            fs_param: 1.0,
        }
    }

    /// Create quality settings for high quality mode
    pub fn high() -> Self {
        Self {
            fn_param: 0,
            fa_param: 2.0,
            fs_param: 0.5,
        }
    }

    /// Get OpenSCAD command-line parameters
    pub fn to_openscad_args(&self) -> Vec<String> {
        vec![
            format!("$fn={}", self.fn_param),
            format!("$fa={}", self.fa_param),
            format!("$fs={}", self.fs_param),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_draft() {
        let q = QualitySettings::draft();
        assert_eq!(q.fn_param, 0);
        assert_eq!(q.fa_param, 12.0);
        assert_eq!(q.fs_param, 2.0);
    }

    #[test]
    fn test_quality_normal() {
        let q = QualitySettings::normal();
        assert_eq!(q.fn_param, 0);
        assert_eq!(q.fa_param, 6.0);
        assert_eq!(q.fs_param, 1.0);
    }

    #[test]
    fn test_quality_high() {
        let q = QualitySettings::high();
        assert_eq!(q.fn_param, 0);
        assert_eq!(q.fa_param, 2.0);
        assert_eq!(q.fs_param, 0.5);
    }

    #[test]
    fn test_quality_draft_args() {
        let q = QualitySettings::draft();
        let args = q.to_openscad_args();
        assert_eq!(args.len(), 3);
        assert!(args[0].contains("$fn=0"));
        assert!(args[1].contains("$fa=12"));
        assert!(args[2].contains("$fs=2"));
    }

    #[test]
    fn test_quality_normal_args() {
        let q = QualitySettings::normal();
        let args = q.to_openscad_args();
        assert_eq!(args.len(), 3);
        assert!(args[0].contains("$fn=0"));
        assert!(args[1].contains("$fa=6"));
        assert!(args[2].contains("$fs=1"));
    }

    #[test]
    fn test_quality_high_args() {
        let q = QualitySettings::high();
        let args = q.to_openscad_args();
        assert_eq!(args.len(), 3);
        assert!(args[0].contains("$fn=0"));
        assert!(args[1].contains("$fa=2"));
        assert!(args[2].contains("$fs=0.5"));
    }

    #[test]
    fn test_quality_serialization() {
        let q = QualitySettings::normal();
        let json = serde_json::to_string(&q);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("fn_param"));
    }

    #[test]
    fn test_quality_deserialization() {
        let json = r#"{"fn_param": 24, "fa_param": 5.0, "fs_param": 0.8}"#;
        let result: std::result::Result<QualitySettings, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_progressive_refinement() {
        // Quality should improve (finer resolution) as we go up
        let draft = QualitySettings::draft();
        let normal = QualitySettings::normal();
        let high = QualitySettings::high();

        assert!(draft.fa_param > normal.fa_param);
        assert!(normal.fa_param > high.fa_param);
        assert!(draft.fs_param > normal.fs_param);
        assert!(normal.fs_param > high.fs_param);
    }

    #[test]
    fn test_quality_clone() {
        let q1 = QualitySettings::high();
        let q2 = q1.clone();
        assert_eq!(q1.fn_param, q2.fn_param);
    }

    #[test]
    fn test_quality_debug() {
        let q = QualitySettings::normal();
        let debug_str = format!("{:?}", q);
        assert!(debug_str.contains("QualitySettings"));
    }
}
