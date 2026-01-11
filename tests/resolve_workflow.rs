//! Integration tests for `jin resolve` command workflow
//!
//! These tests verify the complete resolve command workflow from conflict
//! detection through resolution, including:
//! - Real conflict creation using Jin's layer system (not mocked paused state)
//! - `jin apply` creates `.jinmerge` files with proper conflict markers
//! - Manual conflict resolution by editing `.jinmerge` files
//! - `jin resolve <file>` validates resolution and applies content
//! - `.jinmerge` files are removed after successful resolve
//! - Paused state is updated or deleted appropriately
//! - Apply operation completes automatically when all conflicts resolved
//! - Workflow handles multiple conflicts (partial and full resolution)

mod common;
use common::fixtures::{setup_test_repo, unique_test_id};

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_resolve_command_workflows() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== Setup Phase: Create mode and real conflict ==========

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add file to global layer with content A
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify file to content B and add to mode layer (creates conflict)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove file from workspace (triggers conflict on apply)
    fs::remove_file(&config_path).unwrap();

    // ========== Conflict Detection Phase: Run jin apply ==========

    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");

    // Run apply - should create .jinmerge file and pause
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // Verify .jinmerge file was created
    assert!(
        jinmerge_path.exists(),
        ".jinmerge file should exist after apply conflict"
    );

    // Verify paused state was created
    assert!(
        paused_state_path.exists(),
        "Paused state should exist after apply conflict"
    );

    // ========== Verify .jinmerge file format ==========

    let jinmerge_content = fs::read_to_string(&jinmerge_path).unwrap();

    // Assert contains required header
    assert!(
        jinmerge_content.contains("# Jin merge conflict"),
        ".jinmerge should contain Jin conflict header"
    );

    // Assert contains Git-style conflict markers
    assert!(
        jinmerge_content.contains("<<<<<<<"),
        ".jinmerge should contain opening conflict marker"
    );
    assert!(
        jinmerge_content.contains("======="),
        ".jinmerge should contain separator conflict marker"
    );
    assert!(
        jinmerge_content.contains(">>>>>>>"),
        ".jinmerge should contain closing conflict marker"
    );

    // Assert contains layer information
    assert!(
        jinmerge_content.contains("global") || jinmerge_content.contains("mode"),
        ".jinmerge should contain layer information"
    );

    // ========== Manual Resolution Phase ==========

    // User manually resolves conflict by editing .jinmerge file
    // Remove conflict markers and choose a merged resolution
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9999}",
    )
    .unwrap();

    // ========== Resolve Validation Phase ==========

    // Run resolve command
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // Verify resolved file was written to workspace
    assert!(
        config_path.exists(),
        "Resolved file should exist in workspace"
    );
    let resolved_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        resolved_content.contains("9999"),
        "Resolved file should contain merged value"
    );

    // Verify .jinmerge file was deleted
    assert!(
        !jinmerge_path.exists(),
        ".jinmerge file should be deleted after successful resolve"
    );

    // Verify paused state was deleted (all conflicts resolved)
    assert!(
        !paused_state_path.exists(),
        "Paused state should be deleted when all conflicts resolved"
    );
}

#[test]
fn test_resolve_multiple_conflicts_partial() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== Setup: Create multiple real conflicts ==========

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Define test files: (name, global_value, mode_value)
    let test_files = [
        ("config.json", r#"{"port": 8080}"#, r#"{"port": 9090}"#),
        ("settings.json", r#"{"debug": true}"#, r#"{"debug": false}"#),
        ("options.json", r#"{"timeout": 30}"#, r#"{"timeout": 60}"#),
    ];

    // Add all files to global layer and commit together
    for (name, global_val, _) in &test_files {
        let path = fixture.path().join(name);
        fs::write(&path, global_val).unwrap();
        jin_cmd()
            .args(["add", name, "--global"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify all files and add to mode layer, commit together
    for (name, _, mode_val) in &test_files {
        let path = fixture.path().join(name);
        fs::write(&path, mode_val).unwrap();
        jin_cmd()
            .args(["add", name, "--mode"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove all files from workspace
    for (name, _, _) in &test_files {
        fs::remove_file(fixture.path().join(name)).unwrap();
    }

    // Run apply - should create all .jinmerge files
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"));

    // ========== Verify all conflicts created ==========

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");

    // Verify all .jinmerge files were created
    for (name, _, _) in &test_files {
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        assert!(
            jinmerge_path.exists(),
            ".jinmerge file for {} should exist",
            name
        );
    }

    // Verify paused state contains all conflicts
    assert!(paused_state_path.exists(), "Paused state should exist");
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(paused_content.contains("config.json"));
    assert!(paused_content.contains("settings.json"));
    assert!(paused_content.contains("options.json"));
    assert!(paused_content.contains("conflict_count: 3"));

    // ========== Partial Resolution: Resolve only one conflict ==========

    let config_jinmerge = fixture.path().join("config.json.jinmerge");

    // Resolve only config.json
    fs::write(
        &config_jinmerge,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9999}",
    )
    .unwrap();

    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Remaining conflicts: 2"));

    // ========== Verify partial resolution ==========

    // Verify config.json was resolved
    let config_path = fixture.path().join("config.json");
    assert!(config_path.exists(), "Resolved config.json should exist");
    assert!(
        !config_jinmerge.exists(),
        "config.json.jinmerge should be deleted"
    );

    // Verify other .jinmerge files still exist
    let settings_jinmerge = fixture.path().join("settings.json.jinmerge");
    let options_jinmerge = fixture.path().join("options.json.jinmerge");
    assert!(
        settings_jinmerge.exists(),
        "settings.json.jinmerge should still exist"
    );
    assert!(
        options_jinmerge.exists(),
        "options.json.jinmerge should still exist"
    );

    // Verify paused state still exists with remaining conflicts
    assert!(
        paused_state_path.exists(),
        "Paused state should still exist"
    );
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(
        !paused_content.contains("config.json"),
        "config.json should be removed from paused state"
    );
    assert!(
        paused_content.contains("settings.json"),
        "settings.json should still be in paused state"
    );
    assert!(
        paused_content.contains("options.json"),
        "options.json should still be in paused state"
    );
    assert!(
        paused_content.contains("conflict_count: 2"),
        "Paused state should show 2 remaining conflicts"
    );
}

#[test]
fn test_resolve_multiple_conflicts_full() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== Setup: Create multiple real conflicts ==========

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Define test files: (name, global_value, mode_value, resolved_value)
    let test_files = [
        ("a.json", r#"{"v":1}"#, r#"{"v":2}"#, r#"{"v":3}"#),
        ("b.json", r#"{"v":1}"#, r#"{"v":2}"#, r#"{"v":3}"#),
        ("c.json", r#"{"v":1}"#, r#"{"v":2}"#, r#"{"v":3}"#),
    ];

    // Add all files to global layer and commit together
    for (name, global_val, _, _) in &test_files {
        let path = fixture.path().join(name);
        fs::write(&path, global_val).unwrap();
        jin_cmd()
            .args(["add", name, "--global"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify all files and add to mode layer, commit together
    for (name, _, mode_val, _) in &test_files {
        let path = fixture.path().join(name);
        fs::write(&path, mode_val).unwrap();
        jin_cmd()
            .args(["add", name, "--mode"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove all files from workspace
    for (name, _, _, _) in &test_files {
        fs::remove_file(fixture.path().join(name)).unwrap();
    }

    // Run apply - should create all .jinmerge files
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"));

    // ========== Manually resolve all conflicts ==========

    for (name, _, _, resolved_val) in &test_files {
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        fs::write(
            &jinmerge_path,
            format!(
                "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{}",
                resolved_val
            ),
        )
        .unwrap();
    }

    // ========== Full Resolution: Resolve all conflicts with --all ==========

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");

    jin_cmd()
        .args(["resolve", "--all"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 3 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // ========== Verify all conflicts resolved ==========

    // Verify all resolved files exist in workspace
    for (name, _, _, resolved_val) in &test_files {
        let path = fixture.path().join(name);
        assert!(path.exists(), "{} should exist after resolution", name);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("v"), "{} should contain content", name);

        // Verify .jinmerge file was deleted
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        assert!(
            !jinmerge_path.exists(),
            "{} should be deleted after resolution",
            jinmerge_path.display()
        );
    }

    // Verify paused state was deleted
    assert!(
        !paused_state_path.exists(),
        "Paused state should be deleted when all conflicts resolved"
    );

    // Verify status shows clean state
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Clean")
                .or(predicate::str::contains("No staged"))
                .or(predicate::str::contains("Working directory clean")),
        );
}

#[test]
fn test_resolve_validates_conflict_markers_removed() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== Setup: Create a real conflict ==========

    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add to global
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add to mode (different value)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace and apply
    fs::remove_file(&config_path).unwrap();
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ========== Test: Resolve without removing markers fails ==========

    let jinmerge_path = fixture.path().join("config.json.jinmerge");

    // Try to resolve without removing conflict markers - should fail
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Conflict markers still present"));

    // Verify .jinmerge still exists and file not created
    assert!(
        jinmerge_path.exists(),
        ".jinmerge should still exist after failed resolve"
    );
    assert!(
        !config_path.exists(),
        "config.json should not exist after failed resolve"
    );

    // ========== Test: Resolve with markers removed succeeds ==========

    // Now properly resolve by removing conflict markers
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9999}",
    )
    .unwrap();

    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"));

    // Verify resolution succeeded
    assert!(
        config_path.exists(),
        "config.json should exist after successful resolve"
    );
    assert!(
        !jinmerge_path.exists(),
        ".jinmerge should be deleted after successful resolve"
    );
}

#[test]
fn test_resolve_preserves_resolved_content_formatting() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== Setup: Create a real conflict ==========

    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add multi-line content to global
    let config_path = fixture.path().join("config.json");
    let global_content = r#"{
  "port": 8080,
  "debug": true,
  "features": ["a", "b"]
}"#;
    fs::write(&config_path, global_content).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add different multi-line content to mode
    let mode_content = r#"{
  "port": 9090,
  "production": true
}"#;
    fs::write(&config_path, mode_content).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace and apply
    fs::remove_file(&config_path).unwrap();
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ========== Test: Custom multi-line resolution is preserved ==========

    let jinmerge_path = fixture.path().join("config.json.jinmerge");

    // User creates a custom merge with their preferred formatting
    let custom_resolution = r#"{
  "port": 9999,
  "debug": false,
  "production": true,
  "features": ["a", "b", "c"]
}"#;

    fs::write(
        &jinmerge_path,
        format!(
            "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{}",
            custom_resolution
        ),
    )
    .unwrap();

    // Run resolve
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"));

    // Verify custom resolution was preserved exactly
    let resolved_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        resolved_content.contains("9999"),
        "Custom port value should be preserved"
    );
    assert!(
        resolved_content.contains("debug"),
        "Debug field should be preserved"
    );
    assert!(
        resolved_content.contains("production"),
        "Production field should be preserved"
    );
    assert!(
        resolved_content.contains("[\"a\", \"b\", \"c\"]"),
        "Custom features array should be preserved"
    );
}
