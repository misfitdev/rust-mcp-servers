//! Command-line argument parsing

/// Command-line arguments
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub help: bool,
    pub version: bool,
    pub config: Option<String>,
}

/// Parse command-line arguments
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut parsed = Args::default();

    let mut i = 1; // Skip program name
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                parsed.help = true;
            }
            "-v" | "--version" => {
                parsed.version = true;
            }
            "-c" | "--config" => {
                i += 1;
                if i >= args.len() {
                    return Err("--config requires a value".to_string());
                }
                parsed.config = Some(args[i].clone());
            }
            arg => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
        i += 1;
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help_short() {
        let args = vec!["prog".to_string(), "-h".to_string()];
        let result = parse_args(&args).unwrap();
        assert!(result.help);
        assert!(!result.version);
    }

    #[test]
    fn test_parse_help_long() {
        let args = vec!["prog".to_string(), "--help".to_string()];
        let result = parse_args(&args).unwrap();
        assert!(result.help);
    }

    #[test]
    fn test_parse_version_short() {
        let args = vec!["prog".to_string(), "-v".to_string()];
        let result = parse_args(&args).unwrap();
        assert!(result.version);
        assert!(!result.help);
    }

    #[test]
    fn test_parse_version_long() {
        let args = vec!["prog".to_string(), "--version".to_string()];
        let result = parse_args(&args).unwrap();
        assert!(result.version);
    }

    #[test]
    fn test_parse_config_short() {
        let args = vec![
            "prog".to_string(),
            "-c".to_string(),
            "/path/to/config.yaml".to_string(),
        ];
        let result = parse_args(&args).unwrap();
        assert_eq!(result.config, Some("/path/to/config.yaml".to_string()));
    }

    #[test]
    fn test_parse_config_long() {
        let args = vec![
            "prog".to_string(),
            "--config".to_string(),
            "/path/to/config.yaml".to_string(),
        ];
        let result = parse_args(&args).unwrap();
        assert_eq!(result.config, Some("/path/to/config.yaml".to_string()));
    }

    #[test]
    fn test_parse_combined_args() {
        let args = vec![
            "prog".to_string(),
            "-v".to_string(),
            "-c".to_string(),
            "config.yaml".to_string(),
        ];
        let result = parse_args(&args).unwrap();
        assert!(result.version);
        assert_eq!(result.config, Some("config.yaml".to_string()));
    }

    #[test]
    fn test_parse_config_missing_value() {
        let args = vec!["prog".to_string(), "-c".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a value"));
    }

    #[test]
    fn test_parse_unknown_arg() {
        let args = vec!["prog".to_string(), "-x".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown argument"));
    }

    #[test]
    fn test_parse_empty_args() {
        let args = vec!["prog".to_string()];
        let result = parse_args(&args).unwrap();
        assert!(!result.help);
        assert!(!result.version);
        assert!(result.config.is_none());
    }

    #[test]
    fn test_default_args() {
        let args = Args::default();
        assert!(!args.help);
        assert!(!args.version);
        assert!(args.config.is_none());
    }
}
