//! Integration tests for Mole-RS CLI

use assert_cmd::Command;
use predicates::prelude::*;

/// Test help command
#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Deep clean and optimize"));
}

/// Test version command
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("mo"));
}

/// Test clean command with dry-run
#[test]
fn test_clean_dry_run() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.args(["clean", "--dry-run"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));
}

/// Test clean command with debug flag
#[test]
fn test_clean_debug() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.args(["clean", "--dry-run", "--debug"]);
    cmd.assert()
        .success();
}

/// Test analyze command on temp directory
#[test]
fn test_analyze_temp_dir() {
    let temp = tempfile::TempDir::new().unwrap();
    
    // Create some test files
    std::fs::write(temp.path().join("file1.txt"), "hello").unwrap();
    std::fs::write(temp.path().join("file2.txt"), "world").unwrap();
    
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.arg("analyze").arg(temp.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Disk Analyzer"))
        .stdout(predicate::str::contains("file1.txt").or(predicate::str::contains("file2.txt")));
}

/// Test analyze command on empty directory
#[test]
fn test_analyze_empty_dir() {
    let temp = tempfile::TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.arg("analyze").arg(temp.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No files found"));
}

/// Test purge command with dry-run
#[test]
fn test_purge_dry_run() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.args(["purge", "--dry-run"]);
    cmd.assert()
        .success();
}

/// Test purge command with custom paths
#[test]
fn test_purge_with_paths() {
    let temp = tempfile::TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.args(["purge", "--dry-run", "--paths", temp.path().to_str().unwrap()]);
    cmd.assert()
        .success();
}

/// Test optimize command with dry-run
#[test]
fn test_optimize_dry_run() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.args(["optimize", "--dry-run"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));
}

/// Test unknown command returns error
#[test]
fn test_unknown_command() {
    let mut cmd = Command::cargo_bin("mo").unwrap();
    cmd.arg("unknown_command");
    cmd.assert()
        .failure();
}
