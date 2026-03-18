#!/bin/bash
# Wrapper script to run openscad-mcp from the package directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/openscad-mcp" "$@"
