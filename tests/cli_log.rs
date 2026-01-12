//! Integration tests for `jin log` command
//!
//! Tests the log command's ability to display commit history from all layers,
//! including ModeScope, ModeProject, and ModeScopeProject layers.

use predicates::prelude::*;

mod common;
use common::fixtures::*;

/// Test that jin log shows commits from all layers
///
/// This test reproduces the bug scenario where commits to ModeScope layer
/// were not displayed by jin log. After the fix (S1-S2), the log command
/// uses dynamic ref discovery to find all layer commits.
#[test]
fn test_log_shows_all_layer_commits() {
    // ===== Setup: Isolated test environment =====
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize Jin repository
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // ===== Step 1: Create and activate mode =====
    let mode_name = format!("testmode_{}", unique_test_id());

    jin()
        .args(["mode", "create", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ===== Step 2: Create and activate scope =====
    let scope_name = format!("testscope_{}", unique_test_id());

    jin()
        .args(["scope", "create", &scope_name, "--mode", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", &scope_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ===== Step 3: Commit to ModeBase layer =====
    // File committed without --scope flag goes to ModeBase
    let mode_file = fixture.path().join("mode.json");
    std::fs::write(&mode_file, "{\"mode\": \"base\"}").unwrap();

    jin()
        .args(["add", "mode.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode base commit"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ===== Step 4: Commit to ModeScope layer =====
    // File committed with --scope flag goes to ModeScope
    let scope_file = fixture.path().join("scope.json");
    std::fs::write(&scope_file, "{\"scope\": \"test\"}").unwrap();

    jin()
        .args(["add", "scope.json", "--mode", "--scope", &scope_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode scope commit"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ===== Step 5: Verify jin log shows both commits =====
    jin()
        .arg("log")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Mode base commit"))
        .stdout(predicate::str::contains("Mode scope commit"));
}
