//! Tests for the purge command (dev artifact detection)

use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Create a mock Node.js project
fn create_node_project(base: &std::path::Path) -> std::path::PathBuf {
    let project = base.join("my-node-app");
    fs::create_dir_all(&project).unwrap();
    
    // Create package.json
    let mut pkg = File::create(project.join("package.json")).unwrap();
    pkg.write_all(br#"{"name": "test", "version": "1.0.0"}"#).unwrap();
    
    // Create node_modules
    let node_modules = project.join("node_modules");
    fs::create_dir_all(&node_modules).unwrap();
    
    // Create some fake modules
    let lodash = node_modules.join("lodash");
    fs::create_dir_all(&lodash).unwrap();
    let mut file = File::create(lodash.join("index.js")).unwrap();
    file.write_all(b"module.exports = {};").unwrap();
    
    project
}

/// Create a mock Rust project
fn create_rust_project(base: &std::path::Path) -> std::path::PathBuf {
    let project = base.join("my-rust-app");
    fs::create_dir_all(&project).unwrap();
    
    // Create Cargo.toml
    let mut cargo = File::create(project.join("Cargo.toml")).unwrap();
    cargo.write_all(br#"[package]
name = "test"
version = "0.1.0"
"#).unwrap();
    
    // Create target directory
    let target = project.join("target");
    fs::create_dir_all(target.join("debug")).unwrap();
    
    // Create some build artifacts
    let mut file = File::create(target.join("debug/test")).unwrap();
    file.write_all(&[0u8; 5000]).unwrap();
    
    project
}

/// Create a mock Python project
fn create_python_project(base: &std::path::Path) -> std::path::PathBuf {
    let project = base.join("my-python-app");
    fs::create_dir_all(&project).unwrap();
    
    // Create requirements.txt
    let mut req = File::create(project.join("requirements.txt")).unwrap();
    req.write_all(b"flask==2.0\nrequests==2.28\n").unwrap();
    
    // Create venv directory
    let venv = project.join("venv");
    fs::create_dir_all(venv.join("lib/python3.10/site-packages")).unwrap();
    
    // Create __pycache__
    let pycache = project.join("__pycache__");
    fs::create_dir_all(&pycache).unwrap();
    let mut file = File::create(pycache.join("app.cpython-310.pyc")).unwrap();
    file.write_all(&[0u8; 500]).unwrap();
    
    project
}

#[test]
fn test_node_project_structure() {
    let temp = TempDir::new().unwrap();
    let project = create_node_project(temp.path());
    
    assert!(project.join("package.json").exists());
    assert!(project.join("node_modules").exists());
    assert!(project.join("node_modules/lodash/index.js").exists());
}

#[test]
fn test_rust_project_structure() {
    let temp = TempDir::new().unwrap();
    let project = create_rust_project(temp.path());
    
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("target").exists());
    assert!(project.join("target/debug").exists());
}

#[test]
fn test_python_project_structure() {
    let temp = TempDir::new().unwrap();
    let project = create_python_project(temp.path());
    
    assert!(project.join("requirements.txt").exists());
    assert!(project.join("venv").exists());
    assert!(project.join("__pycache__").exists());
}

#[test]
fn test_mixed_projects() {
    let temp = TempDir::new().unwrap();
    
    create_node_project(temp.path());
    create_rust_project(temp.path());
    create_python_project(temp.path());
    
    // Count directories
    let dirs: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    
    assert_eq!(dirs.len(), 3);
}
