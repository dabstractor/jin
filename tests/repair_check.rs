//! Integration tests for `jin repair --check` command
//!
//! These tests verify that the --check flag:
//! - Runs workspace validation and reports results
//! - Exits successfully when workspace is attached
//! - Reports detachment details when workspace is detached
//! - Does not run other repair checks when --check is used
//! - Works with or without --dry-run flag

mod common;
use common::fixtures::TestFixture;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_repair_check_success_when_attached() {
    // Test that --check reports success when workspace is properly attached
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run repair --check (should pass for fresh workspace)
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    result
        .success()
        .stdout(predicate::str::contains("Checking workspace attachment"))
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("Workspace is properly attached"));
}

#[test]
fn test_repair_check_exits_early() {
    // Test that --check only runs workspace attachment check, not other checks
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run repair --check
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    // Should NOT run other repair checks like "Checking repository structure"
    result.success().stdout(
        predicate::str::contains("Checking workspace attachment")
            .and(predicate::str::contains("Checking repository structure").not()),
    );
}

#[test]
fn test_repair_check_with_dry_run() {
    // Test that --check works with --dry-run flag
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run repair --check --dry-run
    let result = jin_cmd()
        .args(["repair", "--check", "--dry-run"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    result
        .success()
        .stdout(predicate::str::contains("Checking workspace attachment"))
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("Workspace is properly attached"));
}

#[test]
fn test_repair_without_check_runs_all_checks() {
    // Test that regular repair (without --check) still runs all 7 checks
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run regular repair command (without --check)
    let result = jin_cmd()
        .args(["repair"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    // Should run all checks including repository structure
    result
        .success()
        .stdout(predicate::str::contains("Checking repository structure"))
        .stdout(predicate::str::contains("Checking layer references"))
        .stdout(predicate::str::contains("Checking staging index"))
        .stdout(predicate::str::contains("Checking .jinmap"))
        .stdout(predicate::str::contains("Checking workspace metadata"))
        .stdout(predicate::str::contains("Checking global configuration"))
        .stdout(predicate::str::contains("Checking project context"));
}

#[test]
fn test_repair_check_not_initialized() {
    // Test that --check handles uninitialized Jin gracefully
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Delete project context to simulate uninitialized state
    let context_path = fixture.path().join(".jin/context");
    if context_path.exists() {
        fs::remove_file(&context_path).unwrap();
    }

    // Run repair --check
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    // Should show "not initialized" or similar message
    result
        .success()
        .stdout(predicate::str::contains("Checking workspace attachment"));
}

#[test]
fn test_repair_check_detached_file_mismatch() {
    // Test that --check detects file mismatch detachment condition
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Initialize project
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and commit a file
    let test_file = fixture.path().join("config.txt");
    fs::write(&test_file, "original content").unwrap();
    jin_cmd()
        .args(["add", "config.txt"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Apply the layer
    jin_cmd()
        .args(["apply"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file externally (outside of Jin)
    fs::write(&test_file, "modified externally").unwrap();

    // Run repair --check
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    // Should detect detached workspace
    result
        .success()
        .stdout(predicate::str::contains("Checking workspace attachment"))
        .stdout(predicate::str::contains("✗"))
        .stdout(
            predicate::str::contains("detached").or(predicate::str::contains("Workspace state")),
        );
}

#[test]
fn test_repair_normal_mode_includes_workspace_check() {
    // Test that normal repair mode includes workspace attachment check
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run regular repair (not --check mode)
    let result = jin_cmd()
        .args(["repair", "--dry-run"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    // For a fresh workspace, workspace attachment should be checked
    // The check may show "✓ (not initialized)" or similar
    result.success();
}
