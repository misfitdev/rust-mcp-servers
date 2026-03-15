# Product Definition

## Initial Concept

A Rust-based Model Context Protocol (MCP) server that exposes OpenSCAD 3D rendering and modeling capabilities to AI assistants. This is a greenfield rewrite of the Python openscad-mcp server, built to address performance, resource safety, and observability gaps in the original implementation.

## Vision

Provide AI assistants with a production-grade MCP server for:
- High-performance 3D rendering with true parallel multi-view rendering
- Safe resource management (atomic file I/O, guaranteed cleanup, explicit process termination)
- Complete observability (render metrics, cache statistics, process management)
- Advanced workflows (batch rendering, parameter sweeps, dependency analysis)
- Design validation (printability checks, manifold detection, geometric measurements)

## Target Users

- AI assistants (Claude, etc.) building 3D models
- Users building OpenSCAD projects with batch rendering needs
- Teams needing observability into render performance and resource usage
- Developers building CAD automation pipelines

## Core Features (MVP)

### Rendering (from Python baseline, improved)
- Single view rendering with camera control
- Multi-view parallel rendering (8 views in parallel, not sequential)
- Before/after comparison rendering
- Quality presets (draft/normal/high)
- Color scheme support

### Export & Model Management
- Multi-format export (STL, 3MF, AMF, OFF, DXF, SVG)
- CRUD operations for .scad files
- Project file listing with dependency graph

### Analysis & Validation
- SCAD syntax validation with full stderr capture
- Model analysis (bounding box, dimensions, triangle count)
- Library discovery
- OpenSCAD version detection

### NEW: Observability & Operations
- Cache statistics (hit rate, size, entries)
- Server metrics (active renders, queue depth, memory usage)
- Render process management (list, kill, abort)
- Config hot-reload

### NEW: Advanced Operations
- Batch rendering (multiple models × multiple views in parallel)
- Parameter sweep rendering (vary a variable across a range)
- Multi-format export (export one model to all formats at once)

### NEW: Design Validation
- Printability analysis (overhangs, thin walls, unsupported geometry)
- Manifold error detection (non-watertight geometry)
- Geometric measurements and queries

### NEW: Dependency Analysis
- Dependency order calculation (topological sort)
- Affected model analysis (what needs rerendering if X changes)
- Circular dependency detection

## Non-Goals (for MVP)

- Animation/GIF generation
- Interactive 3D viewer
- Real-time rendering preview
- GPU acceleration (future enhancement)
- Multi-user session management
