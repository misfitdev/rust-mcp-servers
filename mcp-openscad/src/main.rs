use std::env;

mod cli;

use cli::parse_args;
use openscad_mcp::logging;
use openscad_mcp::mcp::OpenSCADMCPServer;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();
    match parse_args(&args) {
        Ok(args) => {
            if args.version {
                println!("openscad-mcp {}", VERSION);
            } else if args.help {
                print_help();
            } else {
                // Initialize logging
                logging::init();

                // Start MCP server
                match OpenSCADMCPServer::run().await {
                    Ok(_) => {
                        tracing::info!("MCP server exited successfully");
                    }
                    Err(e) => {
                        tracing::error!("MCP server error: {}", e);
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!(
        r#"openscad-mcp {}

Usage: openscad-mcp [OPTIONS]

Options:
  -h, --help           Print help information
  -v, --version        Print version information
  -c, --config FILE    Path to configuration file
"#,
        VERSION
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_const() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_help_output() {
        // Just ensure it compiles and doesn't panic
        print_help();
    }
}
