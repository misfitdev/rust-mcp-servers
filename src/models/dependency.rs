//! Dependency tracking for OpenSCAD models
//!
//! Parses include/use statements and builds dependency graphs.

use crate::error::{Error, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Dependency graph for models
pub struct DependencyGraph {
    /// Map from model path to its dependencies
    graph: HashMap<PathBuf, Vec<PathBuf>>,
}

impl DependencyGraph {
    /// Create new dependency graph
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn add_edge(&mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) {
        let from = from.into();
        let to = to.into();
        self.graph.entry(from).or_insert_with(Vec::new).push(to);
    }

    /// Get direct dependencies
    pub fn get_deps(&self, path: &Path) -> Option<&Vec<PathBuf>> {
        self.graph.get(path)
    }

    /// Check for cycles using DFS
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.graph.keys() {
            if !visited.contains(node) {
                if self.has_cycle_dfs(node, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    fn has_cycle_dfs(
        &self,
        node: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(neighbors) = self.graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse include/use statements from OpenSCAD content
pub fn parse_includes(content: &str) -> Result<Vec<String>> {
    let re = Regex::new(r#"(?:include|use)\s*<([^>]+)>|(?:include|use)\s*"([^"]+)""#)
        .map_err(|e| Error::Validation(format!("Regex error: {}", e)))?;

    let mut includes = Vec::new();

    for cap in re.captures_iter(content) {
        if let Some(path) = cap.get(1).or_else(|| cap.get(2)) {
            includes.push(path.as_str().to_string());
        }
    }

    Ok(includes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_include_angle_brackets() {
        let content = "include <part.scad>";
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "part.scad");
    }

    #[test]
    fn test_parse_single_include_quotes() {
        let content = r#"include "part.scad""#;
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "part.scad");
    }

    #[test]
    fn test_parse_single_use() {
        let content = "use <library.scad>";
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "library.scad");
    }

    #[test]
    fn test_parse_multiple_includes() {
        let content = r#"
            include <part1.scad>
            include <part2.scad>
            use "library.scad"
        "#;
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"part1.scad".to_string()));
        assert!(result.contains(&"part2.scad".to_string()));
        assert!(result.contains(&"library.scad".to_string()));
    }

    #[test]
    fn test_parse_with_path() {
        let content = "include <../libraries/utils.scad>";
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "../libraries/utils.scad");
    }

    #[test]
    fn test_parse_no_includes() {
        let content = "cube(10); sphere(5);";
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let content = "include  <part.scad>  ";
        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "part.scad");
    }

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        let path1 = PathBuf::from("model1.scad");
        let path2 = PathBuf::from("model2.scad");

        graph.add_edge(path1.clone(), path2.clone());
        let deps = graph.get_deps(&path1);
        assert!(deps.is_some());
        assert_eq!(deps.unwrap().len(), 1);
    }

    #[test]
    fn test_dependency_graph_no_cycle_simple() {
        let mut graph = DependencyGraph::new();
        let path1 = PathBuf::from("a.scad");
        let path2 = PathBuf::from("b.scad");

        graph.add_edge(path1, path2);
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_no_cycle_linear() {
        let mut graph = DependencyGraph::new();
        let path1 = PathBuf::from("a.scad");
        let path2 = PathBuf::from("b.scad");
        let path3 = PathBuf::from("c.scad");

        graph.add_edge(path1, path2.clone());
        graph.add_edge(path2, path3);
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_self_cycle() {
        let mut graph = DependencyGraph::new();
        let path = PathBuf::from("a.scad");

        graph.add_edge(path.clone(), path);
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_two_cycle() {
        let mut graph = DependencyGraph::new();
        let path1 = PathBuf::from("a.scad");
        let path2 = PathBuf::from("b.scad");

        graph.add_edge(path1.clone(), path2.clone());
        graph.add_edge(path2, path1);
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_three_cycle() {
        let mut graph = DependencyGraph::new();
        let path1 = PathBuf::from("a.scad");
        let path2 = PathBuf::from("b.scad");
        let path3 = PathBuf::from("c.scad");

        graph.add_edge(path1.clone(), path2.clone());
        graph.add_edge(path2, path3.clone());
        graph.add_edge(path3, path1);
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_dependency_graph_complex_no_cycle() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.scad");
        let b = PathBuf::from("b.scad");
        let c = PathBuf::from("c.scad");
        let d = PathBuf::from("d.scad");

        graph.add_edge(a.clone(), b.clone());
        graph.add_edge(a.clone(), c.clone());
        graph.add_edge(b, d.clone());
        graph.add_edge(c, d);

        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_parse_complex_file() {
        let content = r#"
            // Include some utilities
            include <math_utils.scad>

            // Use standard library
            use <MCAD/fasteners.scad>

            // Local includes
            include "parameters.scad"
            include "parts/base.scad"

            // Main geometry
            cube([10, 10, 10]);
        "#;

        let result = parse_includes(content).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result.contains(&"math_utils.scad".to_string()));
        assert!(result.contains(&"MCAD/fasteners.scad".to_string()));
        assert!(result.contains(&"parameters.scad".to_string()));
        assert!(result.contains(&"parts/base.scad".to_string()));
    }
}
