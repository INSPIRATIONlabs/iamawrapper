//! Integration tests for macOS package creation (T023)
//!
//! Tests basic package creation workflow.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// T023: Integration test for basic package creation
#[test]
fn test_macos_pkg_basic_creation() {
    // Create a test source folder
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();

    // Create a test file
    fs::write(source_dir.join("test.txt"), "Hello, World!").unwrap();

    // Create output directory
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&output_dir).unwrap();
    let output_file = output_dir.join("test.pkg");

    // Run the command
    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
    ]);

    cmd.assert().success();

    // Verify output file exists
    assert!(output_file.exists(), "Package file should be created");

    // Verify it starts with XAR magic
    let data = fs::read(&output_file).unwrap();
    assert_eq!(&data[0..4], b"xar!", "Package should be XAR format");
}

#[test]
fn test_macos_pkg_missing_identifier() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("test.txt"), "content").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--version",
        "1.0.0",
        // Missing --identifier
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("identifier"));
}

#[test]
fn test_macos_pkg_missing_version() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("test.txt"), "content").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        // Missing --version
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

#[test]
fn test_macos_pkg_missing_source_folder() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        "/nonexistent/path",
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
    ]);

    cmd.assert().failure();
}

#[test]
fn test_macos_pkg_empty_source_folder() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    // Empty directory - no files

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
    ]);

    cmd.assert().failure();
}

#[test]
fn test_macos_pkg_with_install_location() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("test.txt"), "content").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
        "--install-location",
        "/Applications",
    ]);

    cmd.assert().success();
    assert!(output_file.exists());
}

#[test]
fn test_macos_pkg_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();

    // Create nested structure
    let app_dir = source_dir.join("MyApp.app/Contents/MacOS");
    fs::create_dir_all(&app_dir).unwrap();
    fs::write(app_dir.join("myapp"), "binary content").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
    ]);

    cmd.assert().success();
    assert!(output_file.exists());
}

// T043: Integration tests for package with scripts
#[test]
fn test_macos_pkg_with_scripts() {
    let temp_dir = TempDir::new().unwrap();

    // Create source folder with content
    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("app.txt"), "application content").unwrap();

    // Create scripts folder with preinstall and postinstall
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();
    fs::write(
        scripts_dir.join("preinstall"),
        "#!/bin/bash\necho 'preinstall running'",
    )
    .unwrap();
    fs::write(
        scripts_dir.join("postinstall"),
        "#!/bin/bash\necho 'postinstall running'",
    )
    .unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
        "--scripts",
        scripts_dir.to_str().unwrap(),
    ]);

    cmd.assert().success();
    assert!(
        output_file.exists(),
        "Package with scripts should be created"
    );

    // Verify package is valid XAR
    let data = fs::read(&output_file).unwrap();
    assert_eq!(&data[0..4], b"xar!", "Package should be XAR format");
}

#[test]
fn test_macos_pkg_with_preinstall_only() {
    let temp_dir = TempDir::new().unwrap();

    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("app.txt"), "content").unwrap();

    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();
    fs::write(scripts_dir.join("preinstall"), "#!/bin/bash\necho 'pre'").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
        "--scripts",
        scripts_dir.to_str().unwrap(),
    ]);

    cmd.assert().success();
    assert!(output_file.exists());
}

#[test]
fn test_macos_pkg_with_postinstall_only() {
    let temp_dir = TempDir::new().unwrap();

    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("app.txt"), "content").unwrap();

    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();
    fs::write(scripts_dir.join("postinstall"), "#!/bin/bash\necho 'post'").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
        "--scripts",
        scripts_dir.to_str().unwrap(),
    ]);

    cmd.assert().success();
    assert!(output_file.exists());
}

#[test]
fn test_macos_pkg_scripts_folder_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let source_dir = temp_dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("app.txt"), "content").unwrap();

    let output_file = temp_dir.path().join("test.pkg");

    let mut cmd = Command::cargo_bin("iamawrapper").unwrap();
    cmd.args([
        "macos",
        "pkg",
        "-c",
        source_dir.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--identifier",
        "com.test.app",
        "--version",
        "1.0.0",
        "--scripts",
        "/nonexistent/scripts",
    ]);

    cmd.assert().failure();
}
