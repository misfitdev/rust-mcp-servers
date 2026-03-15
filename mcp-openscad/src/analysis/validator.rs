//! OpenSCAD syntax validation
//!
//! Validates SCAD code by running through OpenSCAD and parsing error output.

use crate::error::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Validation result for a SCAD file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the code is valid
    pub valid: bool,

    /// List of errors found
    pub errors: Vec<ValidationError>,

    /// List of warnings found
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Line number (if available)
    pub line: Option<u32>,

    /// Column number (if available)
    pub column: Option<u32>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Line number (if available)
    pub line: Option<u32>,
}

impl ValidationResult {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Validate SCAD code by running through OpenSCAD
pub fn validate_scad(content: &str, openscad_path: &str) -> Result<ValidationResult> {
    // Write content to temp file
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| Error::Cache(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_file.path();

    std::fs::write(temp_path, content).map_err(|e| Error::Filesystem(e))?;

    // Run OpenSCAD in validation mode
    let output = Command::new(openscad_path)
        .arg("-o")
        .arg("/dev/null")
        .arg(temp_path)
        .output()
        .map_err(|e| Error::Render(format!("Failed to run openscad: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut result = ValidationResult::default();

    // Parse errors and warnings from stderr
    for line in stderr.lines() {
        if line.contains("error") || line.contains("ERROR") {
            if let Some(error) = parse_error_line(line) {
                result.errors.push(error);
            }
        } else if line.contains("warning") || line.contains("Warning") {
            if let Some(warning) = parse_warning_line(line) {
                result.warnings.push(warning);
            }
        }
    }

    result.valid = result.errors.is_empty();
    Ok(result)
}

/// Parse error line from OpenSCAD output
fn parse_error_line(line: &str) -> Option<ValidationError> {
    // Try to extract line and column numbers from error message
    let re = Regex::new(r#"(?:line|Line)\s*(\d+)(?:[:,\s]+(?:column|Column)\s*(\d+))?"#).ok()?;

    let message = line.to_string();
    let mut line_num = None;
    let mut col_num = None;

    if let Some(caps) = re.captures(line) {
        if let Some(ln) = caps.get(1) {
            line_num = ln.as_str().parse().ok();
        }
        if let Some(cn) = caps.get(2) {
            col_num = cn.as_str().parse().ok();
        }
    }

    Some(ValidationError {
        message,
        line: line_num,
        column: col_num,
    })
}

/// Parse warning line from OpenSCAD output
fn parse_warning_line(line: &str) -> Option<ValidationWarning> {
    let re = Regex::new(r#"(?:line|Line)\s*(\d+)"#).ok()?;

    let message = line.to_string();
    let mut line_num = None;

    if let Some(caps) = re.captures(line) {
        if let Some(ln) = caps.get(1) {
            line_num = ln.as_str().parse().ok();
        }
    }

    Some(ValidationWarning {
        message,
        line: line_num,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.valid);
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_is_valid() {
        let result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
        };
        assert!(result.is_valid());
    }

    #[test]
    fn test_validation_result_invalid_with_errors() {
        let result = ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                message: "Syntax error".to_string(),
                line: Some(5),
                column: None,
            }],
            warnings: vec![],
        };
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError {
            message: "test error".to_string(),
            line: Some(10),
            column: Some(5),
        };
        assert_eq!(error.message, "test error");
        assert_eq!(error.line, Some(10));
    }

    #[test]
    fn test_validation_warning_creation() {
        let warning = ValidationWarning {
            message: "test warning".to_string(),
            line: Some(15),
        };
        assert_eq!(warning.message, "test warning");
    }

    #[test]
    fn test_parse_error_line_with_line_number() {
        let line = "Error at line 5: undefined variable";
        let error = parse_error_line(line);
        assert!(error.is_some());
        let error = error.unwrap();
        assert_eq!(error.line, Some(5));
    }

    #[test]
    fn test_parse_error_line_with_column() {
        let line = "Error at line 10, column 25: syntax error";
        let error = parse_error_line(line);
        assert!(error.is_some());
        let error = error.unwrap();
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(25));
    }

    #[test]
    fn test_parse_error_line_no_numbers() {
        let line = "Generic error message";
        let error = parse_error_line(line);
        assert!(error.is_some());
        let error = error.unwrap();
        assert_eq!(error.line, None);
    }

    #[test]
    fn test_parse_warning_line() {
        let line = "Warning at line 20: unused variable";
        let warning = parse_warning_line(line);
        assert!(warning.is_some());
        let warning = warning.unwrap();
        assert_eq!(warning.line, Some(20));
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![ValidationWarning {
                message: "test warning".to_string(),
                line: Some(5),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("warning"));
    }

    #[test]
    fn test_validation_result_with_multiple_errors() {
        let result = ValidationResult {
            valid: false,
            errors: vec![
                ValidationError {
                    message: "Error 1".to_string(),
                    line: Some(1),
                    column: None,
                },
                ValidationError {
                    message: "Error 2".to_string(),
                    line: Some(5),
                    column: Some(10),
                },
            ],
            warnings: vec![],
        };
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_parse_error_uppercase_line() {
        let line = "Error at Line 42: test error";
        let error = parse_error_line(line);
        assert!(error.is_some());
        assert_eq!(error.unwrap().line, Some(42));
    }
}
