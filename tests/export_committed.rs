//! Integration tests for `jin export` of committed files
//!
//! These tests verify that the export command can export files committed to
//! Jin layers without requiring them to be in the staging index, validating
//! JinMap lookups and layer content extraction.
//!
//! Tests cover:
//! - Export of committed files without staging (happy path)
//! - Export rejection for untracked files
//! - Export from different layer types (global, mode, project, scope)
//! - Edge cases (file modified locally, JinMap missing, layer ref missing)

mod common;
use common::fixtures::TestFixture;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_export_committed_file_without_staging() {
    // Test the happy path: export a committed file via JinMap lookup
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    // CRITICAL: Set JIN_DIR before any Jin operations
    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create test file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    // Add file to staging
    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit file (removes from staging, adds to JinMap)
    jin_cmd()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify file is in JinMap
    let jinmap_path = fixture.path().join(".jin").join(".jinmap");
    assert!(jinmap_path.exists(), "JinMap should exist after commit");

    // Export should succeed via JinMap lookup
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    // Verify file is in Git index after export
    let repo = git2::Repository::open(fixture.path()).unwrap();
    let index = repo.index().unwrap();
    assert!(
        index.get_path(Path::new("config.json"), 0).is_some(),
        "File should be in Git index after export"
    );
}

#[test]
fn test_export_rejects_untracked_files() {
    // Test error path: export rejects files not tracked by Jin
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create file (not added or committed to Jin)
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    // Export should fail with "not Jin-tracked" error
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not Jin-tracked"));
}

#[test]
fn test_export_from_mode_layer() {
    // Test export from mode layer
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    jin_cmd()
        .args(["mode", "create", "testmode"])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate mode
    jin_cmd()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and add file to mode layer
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"debug": true}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit to mode layer
    jin_cmd()
        .args(["commit", "-m", "Add debug config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Export should succeed from mode layer
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}

#[test]
fn test_export_from_mode_project_layer() {
    // Test export from mode+project layer (layer 5 in the hierarchy)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and activate mode
    jin_cmd()
        .args(["mode", "create", "testmode"])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and add file to mode+project layer
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"layer": "mode-project"}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--mode", "--project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit to mode+project layer
    jin_cmd()
        .args(["commit", "-m", "Add to mode-project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Export should succeed from mode+project layer
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}

#[test]
fn test_export_file_modified_locally() {
    // Test export when local file differs from committed version
    // Export should succeed using the local version
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create, add, and commit file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
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

    // Modify file locally
    fs::write(&file_path, r#"{"port": 9090}"#).unwrap();

    // Export should succeed (exports local version)
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify exported version is the local (modified) version
    let repo = git2::Repository::open(fixture.path()).unwrap();
    let index = repo.index().unwrap();
    let entry = index.get_path(Path::new("config.json"), 0).unwrap();
    let blob = repo.find_blob(entry.id).unwrap();
    let content = String::from_utf8_lossy(blob.content());
    assert_eq!(content, r#"{"port": 9090}"#);
}

#[test]
fn test_export_with_missing_jinmap() {
    // Test export when JinMap file is missing (first-run pattern)
    // Missing JinMap means no committed files, so export should fail
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create, add, and commit file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
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

    // Delete JinMap file to simulate missing state
    let jinmap_path = fixture.path().join(".jin").join(".jinmap");
    fs::remove_file(&jinmap_path).unwrap();

    // Export should fail (file not in empty JinMap)
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not Jin-tracked"));
}

#[test]
fn test_export_with_missing_layer_ref() {
    // Test export when layer ref is missing (corrupted state)
    // NOTE: This test documents current behavior where export succeeds
    // because the file still exists in the working directory.
    // The validation checks JinMap but if layer tree lookup fails,
    // the export still proceeds via `git add` on the working file.
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create, add, and commit file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
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

    // Manually delete layer ref to simulate corrupted state
    let layers_dir = jin_dir
        .join("layers")
        .join("refs")
        .join("jin")
        .join("layers");
    let global_ref = layers_dir.join("global");
    if global_ref.exists() {
        fs::remove_file(&global_ref).unwrap();
    }

    // Current behavior: Export succeeds because file is in working directory
    // The validation would fail layer tree lookup but `git add` still works
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}

#[test]
fn test_export_still_works_for_staged_files() {
    // Regression test: verify existing staged file export still works
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and add file to staging (don't commit)
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Export should succeed (staging fast path)
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify file is in Git index after export
    let repo = git2::Repository::open(fixture.path()).unwrap();
    let index = repo.index().unwrap();
    assert!(
        index.get_path(Path::new("config.json"), 0).is_some(),
        "File should be in Git index after export"
    );
}

#[test]
fn test_export_multiple_files_from_different_layers() {
    // Test exporting files from multiple layers
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    jin_cmd()
        .args(["mode", "create", "testmode"])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate mode
    jin_cmd()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add file1 to global layer
    let file1_path = fixture.path().join("config.json");
    fs::write(&file1_path, r#"{"port": 8080}"#).unwrap();

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

    // Switch back to no mode, add file2 to mode layer
    jin_cmd()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let file2_path = fixture.path().join("settings.json");
    fs::write(&file2_path, r#"{"debug": true}"#).unwrap();

    jin_cmd()
        .args(["add", "settings.json", "--mode"])
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

    // Export both files - should succeed from both layers
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .arg("export")
        .arg("settings.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify both files are in Git index
    let repo = git2::Repository::open(fixture.path()).unwrap();
    let index = repo.index().unwrap();
    assert!(
        index.get_path(Path::new("config.json"), 0).is_some(),
        "config.json should be in Git index"
    );
    assert!(
        index.get_path(Path::new("settings.json"), 0).is_some(),
        "settings.json should be in Git index"
    );
}

#[test]
fn test_export_uses_jinmap_lookup() {
    // Verify that export actually uses JinMap lookup for committed files
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Initialize Jin
    jin_cmd()
        .args(["init"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create, add, and commit file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
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

    // Verify file is in JinMap
    let jinmap_path = fixture.path().join(".jin").join(".jinmap");
    assert!(jinmap_path.exists(), "JinMap should exist");

    let jinmap_content = fs::read_to_string(&jinmap_path).unwrap();
    assert!(
        jinmap_content.contains("config.json"),
        "JinMap should contain config.json"
    );

    // Export should succeed using JinMap
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify committed files stay in Jin after export
    // (export only removes from staging, not JinMap)
    let jinmap_after = fs::read_to_string(&jinmap_path).unwrap();
    assert!(
        jinmap_after.contains("config.json"),
        "File should still be in JinMap after export"
    );
}
