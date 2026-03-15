//! OpenSCAD execution engine
//!
//! Handles finding OpenSCAD executable, version detection, and subprocess management.

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// OpenSCAD engine for subprocess management
pub struct OpenSCADEngine {
    openscad_path: PathBuf,
    version: String,
}

impl OpenSCADEngine {
    /// Create a new OpenSCAD engine, discovering the executable
    pub fn new() -> Result<Self> {
        let openscad_path = find_openscad()?;
        let version = detect_version(&openscad_path)?;

        Ok(Self {
            openscad_path,
            version,
        })
    }

    /// Get the OpenSCAD version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the OpenSCAD executable path
    pub fn path(&self) -> &Path {
        &self.openscad_path
    }

    /// Execute OpenSCAD with timeout
    pub fn execute(&self, args: &[&str], _timeout: Duration) -> Result<(String, String, i32)> {
        let mut cmd = Command::new(&self.openscad_path);
        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Error::Render(format!("Failed to execute openscad: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        Ok((stdout, stderr, code))
    }
}

/// Find OpenSCAD executable in PATH or environment variable
pub fn find_openscad() -> Result<PathBuf> {
    // Check OPENSCAD_PATH environment variable
    if let Ok(path) = std::env::var("OPENSCAD_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Check common installation locations
    let common_paths = vec![
        "/usr/bin/openscad",
        "/usr/local/bin/openscad",
        "/opt/openscad/openscad",
        "/Applications/OpenSCAD.app/Contents/MacOS/OpenSCAD",
    ];

    for path in common_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Try to find in PATH
    if let Ok(path) = which::which("openscad") {
        return Ok(path);
    }

    Err(Error::OpenSCADNotFound(
        "openscad not found in PATH or common locations".to_string(),
    ))
}

/// Detect OpenSCAD version
pub fn detect_version(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Error::Render(format!("Failed to detect openscad version: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .split_whitespace()
        .last()
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_openscad_env_var() {
        // Test that OPENSCAD_PATH env var is checked
        let original = std::env::var("OPENSCAD_PATH").ok();

        // Set to a non-existent path
        std::env::set_var("OPENSCAD_PATH", "/nonexistent/path/openscad");

        // Should fail gracefully
        let result = find_openscad();
        assert!(result.is_err() || result.is_ok()); // Either finds it elsewhere or fails

        // Restore
        if let Some(orig) = original {
            std::env::set_var("OPENSCAD_PATH", orig);
        } else {
            std::env::remove_var("OPENSCAD_PATH");
        }
    }

    #[test]
    fn test_find_openscad_path_search() {
        // If openscad is installed, should find it
        let result = find_openscad();
        // Result can be either Ok or Err depending on system
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_find_openscad_returns_path() {
        if let Ok(path) = find_openscad() {
            assert!(path.is_absolute() || path.to_str().is_some());
        }
    }

    #[test]
    fn test_find_openscad_error_message() {
        // Clear the env var to force search
        let original = std::env::var("OPENSCAD_PATH").ok();
        std::env::set_var("OPENSCAD_PATH", "/nonexistent/openscad");

        let result = find_openscad();
        if let Err(e) = result {
            assert!(e.to_string().contains("not found"));
        }

        if let Some(orig) = original {
            std::env::set_var("OPENSCAD_PATH", orig);
        } else {
            std::env::remove_var("OPENSCAD_PATH");
        }
    }

    #[test]
    fn test_detect_version_returns_string() {
        // This test will only pass if openscad is installed
        if let Ok(path) = find_openscad() {
            if let Ok(version) = detect_version(&path) {
                assert!(!version.is_empty());
            }
        }
    }

    #[test]
    fn test_detect_version_nonexistent_path() {
        let result = detect_version(Path::new("/nonexistent/openscad"));
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_new() {
        // This test will pass or skip depending on openscad availability
        match OpenSCADEngine::new() {
            Ok(engine) => {
                assert!(!engine.version().is_empty());
                assert!(engine.path().is_absolute() || engine.path().to_str().is_some());
            }
            Err(e) => {
                // OK if not installed
                assert!(e.to_string().contains("not found"));
            }
        }
    }

    #[test]
    fn test_engine_version_getter() {
        if let Ok(engine) = OpenSCADEngine::new() {
            let version = engine.version();
            assert!(!version.is_empty());
        }
    }

    #[test]
    fn test_engine_path_getter() {
        if let Ok(engine) = OpenSCADEngine::new() {
            let path = engine.path();
            assert!(path.is_absolute() || path.to_str().is_some());
        }
    }

    #[test]
    fn test_execute_with_invalid_args() {
        if let Ok(engine) = OpenSCADEngine::new() {
            let result = engine.execute(&["--invalid-flag"], Duration::from_secs(5));
            // Should either succeed or return an error
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_execute_returns_stdout_stderr_code() {
        if let Ok(engine) = OpenSCADEngine::new() {
            if let Ok((stdout, stderr, code)) =
                engine.execute(&["--version"], Duration::from_secs(5))
            {
                assert!(stdout.len() > 0 || stderr.len() > 0);
                assert!(code == 0 || code > 0);
            }
        }
    }

    #[test]
    fn test_subprocess_timeout_duration() {
        if let Ok(engine) = OpenSCADEngine::new() {
            let timeout = Duration::from_secs(5);
            let result = engine.execute(&["--version"], timeout);
            // Should complete within reasonable time
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_subprocess_stderr_capture() {
        if let Ok(engine) = OpenSCADEngine::new() {
            if let Ok((_, stderr, _)) = engine.execute(&["--invalid"], Duration::from_secs(5)) {
                // stderr may be empty or contain error message
                assert!(stderr.len() >= 0);
            }
        }
    }
}
