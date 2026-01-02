//! Integration tests for `jin diff` command
//!
//! Tests all four diff modes:
//! 1. `jin diff --staged` - Show staged changes
//! 2. `jin diff <layer1> <layer2>` - Compare two layers
//! 3. `jin diff <layer>` - Compare workspace vs layer
//! 4. `jin diff` - Compare workspace vs workspace-active

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

/// Test: `jin diff --staged` with no staged changes
#[test]
fn test_diff_staged_empty() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("diff")
        .arg("--staged")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("No staged changes"));
}

/// Test: `jin diff --staged` with staged files
#[test]
fn test_diff_staged_with_files() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");
    let mode_name = format!("test_mode_{}", std::process::id());

    // Initialize and create mode
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a config file
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"key": "value"}"#).unwrap();

    // Stage the file to mode-base
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check staged diff
    jin()
        .arg("diff")
        .arg("--staged")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("config.json"))
        .stdout(contains("mode-base"));
}

/// Test: `jin diff <layer1> <layer2>` - Compare two layers
#[test]
fn test_diff_layers() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");
    let mode_name = format!("test_mode_{}", std::process::id());

    // Initialize and create mode
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add a file to mode-base
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"port": 3000}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Compare with itself (should show no differences)
    jin()
        .args(["diff", "mode-base", "mode-base"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("No differences"));
}

/// Test: `jin diff <layer>` - Workspace vs layer comparison
#[test]
fn test_diff_workspace_vs_layer() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");
    let mode_name = format!("test_mode_{}", std::process::id());

    // Initialize and create mode
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add a file to mode-base
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"key": "original"}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the workspace file
    fs::write(&config_path, r#"{"key": "modified"}"#).unwrap();

    // Compare workspace vs mode-base
    jin()
        .args(["diff", "mode-base"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Comparing workspace vs mode-base"));
}

/// Test: `jin diff` - Default workspace vs workspace-active
#[test]
fn test_diff_default() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");
    let mode_name = format!("test_mode_{}", std::process::id());

    // Initialize and create mode
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add a file to mode-base
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"key": "value"}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Apply to create workspace metadata
    jin()
        .arg("apply")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Now run diff - should show no differences
    jin()
        .arg("diff")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Comparing workspace vs workspace-active"));
}

/// Test: `jin diff invalid-layer` - Error on unknown layer
#[test]
fn test_diff_invalid_layer() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["diff", "invalid-layer"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("Unknown layer"));
}

/// Test: `jin diff` when not initialized
#[test]
fn test_diff_not_initialized() {
    let temp = TempDir::new().unwrap();

    jin()
        .arg("diff")
        .current_dir(temp.path())
        .assert()
        .failure();
}

/// Test: Layer name parsing for all 9 layer types
#[test]
fn test_diff_all_layer_names() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Test each valid layer name - they should fail gracefully (no commits)
    // rather than "Unknown layer"
    let layers = [
        "global-base",
        "mode-base",
        "mode-scope",
        "mode-scope-project",
        "mode-project",
        "scope-base",
        "project-base",
        "user-local",
        "workspace-active",
    ];

    for layer in layers {
        let result = jin()
            .args(["diff", layer, layer])
            .current_dir(temp.path())
            .env("JIN_DIR", &jin_dir)
            .assert();

        // Should either succeed (if layer has commits) or fail with "has no commits"
        // but NOT "Unknown layer"
        let output = result.get_output();
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            // If it failed, it should be due to no commits, not unknown layer
            assert!(
                stderr_str.contains("has no commits") || stderr_str.contains("No differences"),
                "Expected 'has no commits' or 'No differences' for layer {}, got: {}",
                layer,
                stderr_str
            );
        }
    }
}

/// Test: `jin diff --staged` shows modified files
#[test]
fn test_diff_staged_modified() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");
    let mode_name = format!("test_mode_{}", std::process::id());

    // Initialize and create mode
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"key": "value1"}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file after staging
    fs::write(&config_path, r#"{"key": "value2"}"#).unwrap();

    // Check staged diff - should show file as modified
    jin()
        .arg("diff")
        .arg("--staged")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("config.json"))
        .stdout(contains("modified since staging"));
}
