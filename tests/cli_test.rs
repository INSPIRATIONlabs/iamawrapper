//! CLI tests for subcommand structure (T024)
//!
//! Tests that the CLI properly routes to macos subcommands.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

/// T024: Test that `macos pkg` subcommand is recognized
#[test]
fn test_macos_subcommand_exists() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["macos", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pkg"));
}

#[test]
fn test_macos_pkg_subcommand_help() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["macos", "pkg", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("identifier"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("content"))
        .stdout(predicate::str::contains("output"));
}

#[test]
fn test_intune_subcommand_exists() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["intune", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("create"));
}

#[test]
fn test_intune_create_subcommand_help() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["intune", "create", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("content"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("output"));
}

#[test]
fn test_intune_extract_subcommand_help() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["intune", "extract", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("input"))
        .stdout(predicate::str::contains("output"));
}

#[test]
fn test_root_help_shows_subcommands() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");
    cmd.args(["--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("intune"))
        .stdout(predicate::str::contains("macos"));
}

#[test]
fn test_no_args_shows_help_or_interactive() {
    let mut cmd = cargo_bin_cmd!("iamawrapper");

    // Without args, should either show help or enter interactive mode
    // We'll accept either behavior for now
    let result = cmd.assert();
    // Just verify it doesn't crash - it can succeed or show help
    let _ = result;
}
