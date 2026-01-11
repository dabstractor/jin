//! Integration tests for `jin import` command
//!
//! Tests the complete import workflow:
//! 1. Import Git-tracked files into Jin staging
//! 2. Files removed from Git index (kept in workspace)
//! 3. Files added to .gitignore managed block
//! 4. Atomic rollback on error
//! 5. Batch operations with per-file error handling

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

/// Initialize a Git repository with user config
fn git_init_with_config(path: &tempfile::TempDir) {
    StdCommand::new("git")
        .arg("init")
        .current_dir(path.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path.path())
        .output()
        .unwrap();
}

/// Test: `jin import` with a single Git-tracked file
#[test]
fn test_import_single_file() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit a file to Git
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, r#"{"key": "value"}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import the file
    jin()
        .arg("import")
        .arg("config.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 1 file(s)"));

    // Verify file removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("config.json"));

    // Verify file still exists in workspace
    assert!(config_path.exists());

    // Verify file in Jin staging
    let staging_index_path = jin_dir.join("staging").join("index.json");
    assert!(staging_index_path.exists());
    let staging_content = fs::read_to_string(&staging_index_path).unwrap();
    assert!(staging_content.contains("config.json"));

    // Verify .gitignore updated
    let gitignore_content = fs::read_to_string(temp.path().join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("config.json"));
}

/// Test: `jin import` with multiple files
#[test]
fn test_import_multiple_files() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit multiple files
    fs::write(temp.path().join("config1.json"), r#"{"a": 1}"#).unwrap();
    fs::write(temp.path().join("config2.json"), r#"{"b": 2}"#).unwrap();
    fs::write(temp.path().join("config3.json"), r#"{"c": 3}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import all files
    jin()
        .args(["import", "config1.json", "config2.json", "config3.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 3 file(s)"));

    // Verify all files removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("config1.json"));
    assert!(!git_files.contains("config2.json"));
    assert!(!git_files.contains("config3.json"));

    // Verify all files in Jin staging
    let staging_content = fs::read_to_string(jin_dir.join("staging/index.json")).unwrap();
    assert!(staging_content.contains("config1.json"));
    assert!(staging_content.contains("config2.json"));
    assert!(staging_content.contains("config3.json"));
}

/// Test: `jin import` with directory (auto-expansion)
#[test]
fn test_import_directory() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create directory with files
    let vscode_dir = temp.path().join(".vscode");
    fs::create_dir(&vscode_dir).unwrap();
    fs::write(
        vscode_dir.join("settings.json"),
        r#"{"editor.fontSize": 14}"#,
    )
    .unwrap();
    fs::write(vscode_dir.join("launch.json"), r#"{"version": "0.2.0"}"#).unwrap();

    StdCommand::new("git")
        .args(["add", ".vscode"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import directory
    jin()
        .arg("import")
        .arg(".vscode")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 2 file(s)"));

    // Verify files removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("settings.json"));
    assert!(!git_files.contains("launch.json"));
}

/// Test: `jin import` with non-Git file should error
#[test]
fn test_import_not_tracked() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    StdCommand::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create file but don't add to Git
    fs::write(temp.path().join("config.json"), r#"{"key": "value"}"#).unwrap();

    // Try to import - should fail with helpful message
    jin()
        .arg("import")
        .arg("config.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("not tracked by Git"))
        .stderr(contains("jin add"));
}

/// Test: `jin import` with modified file (without --force)
#[test]
fn test_import_modified_file() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit file
    fs::write(temp.path().join("config.json"), r#"{"key": "value"}"#).unwrap();
    StdCommand::new("git")
        .args(["add", "config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file
    fs::write(temp.path().join("config.json"), r#"{"key": "modified"}"#).unwrap();

    // Try to import without --force - should fail
    jin()
        .arg("import")
        .arg("config.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("uncommitted changes"));
}

/// Test: `jin import --force` with modified file should succeed
#[test]
fn test_import_modified_with_force() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit file
    fs::write(temp.path().join("config.json"), r#"{"key": "value"}"#).unwrap();
    StdCommand::new("git")
        .args(["add", "config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file
    fs::write(temp.path().join("config.json"), r#"{"key": "modified"}"#).unwrap();

    // Import with --force - should succeed
    jin()
        .args(["import", "--force", "config.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 1 file(s)"));
}

/// Test: `jin import` when Jin not initialized
#[test]
fn test_import_not_initialized() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit file
    fs::write(temp.path().join("config.json"), r#"{"key": "value"}"#).unwrap();
    StdCommand::new("git")
        .args(["add", "config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Try to import WITHOUT initializing Jin - should fail
    jin()
        .arg("import")
        .arg("config.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure();
}

/// Test: `jin import` atomic rollback on error
#[test]
fn test_import_rollback() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit file1
    fs::write(temp.path().join("file1.json"), r#"{"a": 1}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "file1.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify file1 so it will fail the modified check
    fs::write(temp.path().join("file1.json"), r#"{"a": 2}"#).unwrap();

    // Try to import a mix: valid file (doesn't exist) and modified file
    // The non-existent file should fail first before any git changes
    jin()
        .args(["import", "nonexistent.json", "file1.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("not found"));

    // Verify file1.json is still in Git index (no rollback needed since
    // nothing was removed - validation failed first)
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(git_files.contains("file1.json"));
}

/// Test: `jin import` with symlink file should error
#[test]
fn test_import_symlink() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    StdCommand::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        // Create a real file
        fs::write(temp.path().join("real_file.txt"), "content").unwrap();

        // Create symlink
        symlink("real_file.txt", temp.path().join("link.txt")).unwrap();

        // Add symlink to Git
        StdCommand::new("git")
            .args(["add", "link.txt"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["commit", "-m", "Add symlink"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Try to import symlink - should fail
        jin()
            .arg("import")
            .arg("link.txt")
            .current_dir(temp.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .failure()
            .stderr(contains("Symlinks are not supported"));
    }
}

/// Test: `jin import` updates .gitignore
#[test]
fn test_import_gitignore_update() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit file
    fs::write(temp.path().join("config.json"), r#"{"key": "value"}"#).unwrap();
    StdCommand::new("git")
        .args(["add", "config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import the file
    jin()
        .arg("import")
        .arg("config.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify .gitignore updated
    let gitignore_content = fs::read_to_string(temp.path().join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("# --- JIN MANAGED START ---"));
    assert!(gitignore_content.contains("config.json"));
}

/// Test: `jin import` with no files specified
#[test]
fn test_import_no_files() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Try to import with no files - should fail
    jin()
        .arg("import")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("No files specified"));
}

/// Test: `jin import` with non-existent file
#[test]
fn test_import_file_not_found() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Try to import non-existent file - should fail
    jin()
        .arg("import")
        .arg("nonexistent.json")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// Test: `jin import --local` routes to Layer 8 (UserLocal)
#[test]
fn test_import_with_local_flag() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit a file to Git
    let config_path = temp.path().join("local_config.json");
    fs::write(&config_path, r#"{"local": "setting"}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "local_config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import with --local flag
    jin()
        .args(["import", "--local", "local_config.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 1 file(s)"));

    // Verify file removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("local_config.json"));

    // Verify file still exists in workspace
    assert!(config_path.exists());

    // Verify staging index exists at JIN_DIR-aware path
    let staging_index_path = jin_dir.join("staging").join("index.json");
    assert!(staging_index_path.exists());
    let staging_content = fs::read_to_string(&staging_index_path).unwrap();

    // Verify file in staging index
    assert!(staging_content.contains("local_config.json"));

    // Verify target_layer is user_local (Layer 8)
    assert!(
        staging_content.contains("UserLocal") || staging_content.contains("user_local"),
        "Staging index should contain UserLocal layer reference. Content:\n{}",
        staging_content
    );
}

/// Test: `jin import --global` routes to Layer 1 (GlobalBase)
#[test]
fn test_import_with_global_flag() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Create and commit a file to Git
    let config_path = temp.path().join("global_config.json");
    fs::write(&config_path, r#"{"global": "setting"}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "global_config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Import with --global flag
    jin()
        .args(["import", "--global", "global_config.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 1 file(s)"));

    // Verify file removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("global_config.json"));

    // Verify file still exists in workspace
    assert!(config_path.exists());

    // Verify staging index exists at JIN_DIR-aware path
    let staging_index_path = jin_dir.join("staging").join("index.json");
    assert!(staging_index_path.exists());
    let staging_content = fs::read_to_string(&staging_index_path).unwrap();

    // Verify file in staging index
    assert!(staging_content.contains("global_config.json"));

    // Verify target_layer is global_base (Layer 1)
    assert!(
        staging_content.contains("GlobalBase") || staging_content.contains("global_base"),
        "Staging index should contain GlobalBase layer reference. Content:\n{}",
        staging_content
    );
}

/// Test: `jin import --mode` routes to Layer 2 (ModeBase) with active mode
#[test]
fn test_import_with_mode_flag() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Git repo
    git_init_with_config(&temp);

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a mode
    jin()
        .args(["mode", "create", "testmode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode (required for --mode flag)
    jin()
        .args(["mode", "use", "testmode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and commit a file to Git
    let config_path = temp.path().join("mode_config.json");
    fs::write(&config_path, r#"{"mode": "setting"}"#).unwrap();

    StdCommand::new("git")
        .args(["add", "mode_config.json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Import with --mode flag (routes to Layer 2: ModeBase)
    jin()
        .args(["import", "--mode", "mode_config.json"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("Imported 1 file(s)"));

    // Verify file removed from Git index
    let git_files_output = StdCommand::new("git")
        .args(["ls-files"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let git_files = String::from_utf8_lossy(&git_files_output.stdout);
    assert!(!git_files.contains("mode_config.json"));

    // Verify file still exists in workspace
    assert!(config_path.exists());

    // Verify staging index exists at JIN_DIR-aware path
    let staging_index_path = jin_dir.join("staging").join("index.json");
    assert!(staging_index_path.exists());
    let staging_content = fs::read_to_string(&staging_index_path).unwrap();

    // Verify file in staging index
    assert!(staging_content.contains("mode_config.json"));

    // Verify target_layer is mode_base (Layer 2)
    assert!(
        staging_content.contains("ModeBase") || staging_content.contains("mode_base"),
        "Staging index should contain ModeBase layer reference. Content:\n{}",
        staging_content
    );
}
