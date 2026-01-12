//! Mode and scope workflow integration tests for Jin
//!
//! Tests layer routing and precedence in the 9-layer hierarchy:
//! 1. GlobalBase
//! 2. ModeBase
//! 3. ScopeBase
//! 4. GlobalProject
//! 5. ModeProject
//! 6. ScopeProject
//! 7. ModeScope
//! 8. ModeScopeProject
//! 9. ProjectBase

use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

// Use serial_test to ensure tests that use JIN_DIR environment variable run sequentially
use serial_test::serial;

/// Test layer routing: mode base (Layer 2)
#[test]
#[serial]
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize project
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create and add file with --mode flag (mode base layer)
    fs::write(
        project_path.join("config.json"),
        r#"{"layer": "mode-base"}"#,
    )?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode base"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for mode base layer
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test layer routing: mode + project (Layer 5)
#[test]
#[serial]
fn test_layer_routing_mode_project() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add file with --mode --project flags (mode project layer)
    fs::write(
        project_path.join("project.json"),
        r#"{"layer": "mode-project"}"#,
    )?;

    jin()
        .args(["add", "project.json", "--mode", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for mode project layer
    // Note: project name is derived from directory
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("Failed to get project name");
    let ref_path = format!(
        "refs/jin/layers/mode/{}/project/{}",
        mode_name, project_name
    );
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test layer routing: mode + scope (Layer 7)
#[test]
#[serial]
fn test_layer_routing_mode_scope() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    let scope_name = format!("env:test_{}", unique_test_id());

    create_mode(&mode_name, Some(jin_dir))?;
    create_scope(&scope_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", &scope_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add file with --mode --scope flags
    fs::write(
        project_path.join("scope.json"),
        r#"{"layer": "mode-scope"}"#,
    )?;

    jin()
        .args([
            "add",
            "scope.json",
            "--mode",
            &format!("--scope={}", scope_name),
        ])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode scope"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for mode scope layer
    let ref_path = format!("refs/jin/layers/mode/{}/scope/{}", mode_name, scope_name);
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test layer routing: mode + scope + project (Layer 8)
#[test]
#[serial]
fn test_layer_routing_mode_scope_project() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    let scope_name = format!("env:test_{}", unique_test_id());

    create_mode(&mode_name, Some(jin_dir))?;
    create_scope(&scope_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", &scope_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add file with --mode --scope --project flags (highest precedence user layer)
    fs::write(
        project_path.join("full.json"),
        r#"{"layer": "mode-scope-project"}"#,
    )?;

    jin()
        .args([
            "add",
            "full.json",
            "--mode",
            &format!("--scope={}", scope_name),
            "--project",
        ])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode scope project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for mode scope project layer
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("Failed to get project name");
    let ref_path = format!(
        "refs/jin/layers/mode/{}/scope/{}/project/{}",
        mode_name, scope_name, project_name
    );
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test layer precedence: higher layer wins
#[test]
#[serial]
fn test_layer_precedence_higher_wins() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add file to mode base (Layer 2)
    fs::write(
        project_path.join("config.json"),
        r#"{"layer": "mode-base", "value": 2}"#,
    )?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode base"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add same file to mode-project (Layer 5 - higher precedence)
    fs::write(
        project_path.join("config.json"),
        r#"{"layer": "mode-project", "value": 5}"#,
    )?;

    jin()
        .args(["add", "config.json", "--mode", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply and verify higher layer wins
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("config.json"))?;
    assert!(
        content.contains(r#""layer": "mode-project""#),
        "Mode-project (Layer 5) should override mode-base (Layer 2). Content: {}",
        content
    );
    assert!(
        content.contains(r#""value": 5"#),
        "Higher layer value should win. Content: {}",
        content
    );

    Ok(())
}

/// Test deep merge of JSON files across layers
#[test]
#[serial]
fn test_mode_scope_deep_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add base configuration in mode layer
    fs::write(
        project_path.join("settings.json"),
        r#"{"debug": false, "timeout": 30, "features": {"auth": true}}"#,
    )?;

    jin()
        .args(["add", "settings.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base settings"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Override some values in higher layer
    fs::write(
        project_path.join("settings.json"),
        r#"{"debug": true, "features": {"logging": true}}"#,
    )?;

    jin()
        .args(["add", "settings.json", "--mode", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Project settings"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply and verify deep merge
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("settings.json"))?;

    // Verify merged values:
    // - debug: true (overridden)
    // - timeout: 30 (from base)
    // - features.auth: true (from base)
    // - features.logging: true (from override)
    assert!(
        content.contains(r#""debug": true"#),
        "debug should be overridden to true. Content: {}",
        content
    );
    assert!(
        content.contains(r#""timeout": 30"#),
        "timeout should be preserved from base. Content: {}",
        content
    );
    assert!(
        content.contains(r#""auth": true"#),
        "auth feature should be preserved. Content: {}",
        content
    );
    assert!(
        content.contains(r#""logging": true"#),
        "logging feature should be added. Content: {}",
        content
    );

    Ok(())
}

/// Test global layer (Layer 1)
#[test]
#[serial]
fn test_layer_routing_global_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    // Add file to global layer
    fs::write(project_path.join("global.json"), r#"{"layer": "global"}"#)?;

    jin()
        .args(["add", "global.json", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Global config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for global layer
    assert_layer_ref_exists("refs/jin/layers/global", Some(jin_dir));

    Ok(())
}

/// Test project base layer (Layer 9 - lowest precedence user layer)
#[test]
#[serial]
fn test_layer_routing_project_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    // Add file to project layer (no flags)
    fs::write(project_path.join("project.json"), r#"{"layer": "project"}"#)?;

    jin()
        .args(["add", "project.json"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Project config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify ref created for project layer
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("Failed to get project name");
    let ref_path = format!("refs/jin/layers/project/{}", project_name);
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test error: use scope without mode when scope is mode-scoped
#[test]
#[serial]
fn test_scope_requires_mode_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    let scope_name = format!("env:test_{}", unique_test_id());

    // Create mode-scoped scope
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args([
            "scope",
            "create",
            &scope_name,
            &format!("--mode={}", mode_name),
        ])
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Try to use scope without activating mode
    let result = jin()
        .args(["scope", "use", &scope_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    // Should fail or warn about mode requirement
    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success() || stderr_str.contains("mode") || stderr_str.contains("requires"),
        "Using mode-scoped scope without mode should fail or warn"
    );

    Ok(())
}

/// Test multiple modes don't interfere
#[test]
#[serial]
fn test_multiple_modes_isolated() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_a = format!("mode_a_{}", unique_test_id());
    let mode_b = format!("mode_b_{}", unique_test_id());

    create_mode(&mode_a, Some(jin_dir))?;
    create_mode(&mode_b, Some(jin_dir))?;

    // Add file to mode A
    jin()
        .args(["mode", "use", &mode_a])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(project_path.join("a.txt"), "mode A content")?;

    jin()
        .args(["add", "a.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode A"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Switch to mode B and add different file
    jin()
        .args(["mode", "use", &mode_b])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(project_path.join("b.txt"), "mode B content")?;

    jin()
        .args(["add", "b.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode B"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify both mode refs exist independently
    assert_layer_ref_exists(&format!("refs/jin/layers/mode/{}", mode_a), Some(jin_dir));
    assert_layer_ref_exists(&format!("refs/jin/layers/mode/{}", mode_b), Some(jin_dir));

    Ok(())
}

/// Test that switching modes automatically clears workspace metadata
///
/// Workflow:
/// 1. Create two modes (mode_a, mode_b)
/// 2. Activate mode_a, add file, commit, apply
/// 3. Switch to mode_b (should clear metadata)
/// 4. Add mode_b file, commit, apply (should work without --force)
/// 5. Verify mode_b content is in workspace
#[test]
#[serial]
fn test_mode_switch_clears_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test fixture
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize Jin repository
    jin_init(project_path, None)?;

    // Create two unique mode names
    let mode_a = format!("mode_a_{}", unique_test_id());
    let mode_b = format!("mode_b_{}", unique_test_id());

    // Create both modes
    create_mode(&mode_a, Some(jin_dir))?;
    create_mode(&mode_b, Some(jin_dir))?;

    // === STEP 1: Activate mode_a and apply configuration ===
    jin()
        .args(["mode", "use", &mode_a])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add a file for mode_a
    fs::write(project_path.join("config.json"), r#"{"mode": "mode_a"}"#)?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode A config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply mode_a configuration
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify mode_a content is in workspace (JSON is pretty-printed)
    assert_workspace_file(
        project_path,
        "config.json",
        r#"{
  "mode": "mode_a"
}"#,
    );

    // === STEP 2: Switch to mode_b ===
    let result = jin()
        .args(["mode", "use", &mode_b])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    // Verify metadata clear message appears
    // Note: The actual message format may be "activating mode" or "mode changed from X to Y"
    // depending on whether metadata has a mode layer. Both indicate metadata was cleared.
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains(&format!("Cleared workspace metadata ("))
            && (stdout_str.contains(&format!("mode changed from '{}' to '{}'", mode_a, mode_b))
                || stdout_str.contains(&format!("activating mode '{}'", mode_b))),
        "Expected metadata clear message. Got: {}",
        stdout_str
    );

    // === STEP 3: Add mode_b configuration and apply ===
    // Note: File content changed to mode_b
    fs::write(project_path.join("config.json"), r#"{"mode": "mode_b"}"#)?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode B config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // CRITICAL: This apply must succeed without --force
    // If metadata wasn't cleared, this would fail with "detached workspace" error
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify mode_b content is in workspace (JSON is pretty-printed)
    assert_workspace_file(
        project_path,
        "config.json",
        r#"{
  "mode": "mode_b"
}"#,
    );

    Ok(())
}

/// Test that switching scopes automatically clears workspace metadata
///
/// Workflow:
/// 1. Create two scopes (scope_x, scope_y)
/// 2. Activate scope_x, add file, commit, apply
/// 3. Switch to scope_y (should clear metadata)
/// 4. Add scope_y file, commit, apply (should work without --force)
/// 5. Verify scope_y content is in workspace
#[test]
#[serial]
fn test_scope_switch_clears_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test fixture
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize Jin repository
    jin_init(project_path, None)?;

    // Create two unique scope names
    let scope_x = format!("scope_x_{}", unique_test_id());
    let scope_y = format!("scope_y_{}", unique_test_id());

    // Create both scopes
    create_scope(&scope_x, Some(jin_dir))?;
    create_scope(&scope_y, Some(jin_dir))?;

    // === STEP 1: Activate scope_x and apply configuration ===
    jin()
        .args(["scope", "use", &scope_x])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add a file for scope_x (note: --scope= flag format)
    fs::write(project_path.join("config.json"), r#"{"scope": "scope_x"}"#)?;

    jin()
        .args(["add", "config.json", &format!("--scope={}", scope_x)])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Scope X config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply scope_x configuration
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify scope_x content is in workspace (JSON is pretty-printed)
    assert_workspace_file(
        project_path,
        "config.json",
        r#"{
  "scope": "scope_x"
}"#,
    );

    // === STEP 2: Switch to scope_y ===
    let result = jin()
        .args(["scope", "use", &scope_y])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    // Verify metadata clear message appears
    // Note: The actual message format may be "activating scope" or "scope changed from X to Y"
    // depending on whether metadata has a scope layer. Both indicate metadata was cleared.
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains(&format!("Cleared workspace metadata ("))
            && (stdout_str.contains(&format!(
                "scope changed from '{}' to '{}'",
                scope_x, scope_y
            )) || stdout_str.contains(&format!("activating scope '{}'", scope_y))),
        "Expected metadata clear message. Got: {}",
        stdout_str
    );

    // === STEP 3: Add scope_y configuration and apply ===
    // Note: File content changed to scope_y
    fs::write(project_path.join("config.json"), r#"{"scope": "scope_y"}"#)?;

    jin()
        .args(["add", "config.json", &format!("--scope={}", scope_y)])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Scope Y config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // CRITICAL: This apply must succeed without --force
    // If metadata wasn't cleared, this would fail with "detached workspace" error
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify scope_y content is in workspace (JSON is pretty-printed)
    assert_workspace_file(
        project_path,
        "config.json",
        r#"{
  "scope": "scope_y"
}"#,
    );

    Ok(())
}
