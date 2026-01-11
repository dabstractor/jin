//! Integration tests for `jin list` command
//!
//! Tests the list command's ability to enumerate modes, scopes, and projects
//! from the Jin repository's layer refs.

use predicates::prelude::*;

mod common;
use common::fixtures::*;

/// Test list with empty repository
#[test]
fn test_list_empty_repository() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize the Jin repository with the same JIN_DIR that list will use
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Available in Jin repository:"))
        .stdout(predicate::str::contains(
            "(no modes, scopes, or projects found)",
        ))
        .stdout(predicate::str::contains(
            "Use 'jin mode use <mode>' to activate a mode",
        ));
}

/// Test list after creating a mode with files (creates layer ref)
#[test]
fn test_list_with_modes() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize the Jin repository with the same JIN_DIR
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // Create and activate a mode
    jin()
        .args(["mode", "create", "development"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", "development"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add and commit a file to create the layer ref
    let test_file = fixture.path().join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // List should show the mode
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Modes:"))
        .stdout(predicate::str::contains("- development"));
}

/// Test list with all categories
#[test]
fn test_list_with_all_categories() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize project context with the same JIN_DIR
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // Create a mode and add files to it
    jin()
        .args(["mode", "create", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let editor_config = fixture.path().join("editor.conf");
    std::fs::write(&editor_config, "editor settings").unwrap();

    jin()
        .args(["add", "editor.conf"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add editor config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a scope and add files to it
    jin()
        .args(["mode", "unset"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "create", "config:vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", "config:vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let vim_config = fixture.path().join("vim.vim");
    std::fs::write(&vim_config, "vim settings").unwrap();

    jin()
        .args(["add", "vim.vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add vim config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Unset scope and add project file
    jin()
        .args(["scope", "unset"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let readme_file = fixture.path().join("README.md");
    std::fs::write(&readme_file, "# Test Project").unwrap();

    jin()
        .args(["add", "README.md"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add README"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // List should show all categories
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Modes:"))
        .stdout(predicate::str::contains("- editor"))
        .stdout(predicate::str::contains("Scopes:"))
        .stdout(predicate::str::contains("- config:vim"))
        .stdout(predicate::str::contains("Projects:"));
}

/// Test list not initialized error
#[test]
fn test_list_not_initialized() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // List without Jin repository should fail
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

/// Test list shows usage hints at the end
#[test]
fn test_list_shows_usage_hints() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize the Jin repository with the same JIN_DIR
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // Create a mode with files
    jin()
        .args(["mode", "create", "vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", "vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let vim_file = fixture.path().join("vimrc");
    std::fs::write(&vim_file, "vim config").unwrap();

    jin()
        .args(["add", "vimrc"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add vimrc"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // List should show usage hints at the end
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use 'jin mode use <mode>' to activate a mode",
        ))
        .stdout(predicate::str::contains(
            "Use 'jin scope use <scope>' to activate a scope",
        ));
}

/// Test list with mode-bound scopes
#[test]
fn test_list_with_mode_bound_scopes() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize the Jin repository with the same JIN_DIR
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // Create a mode
    jin()
        .args(["mode", "create", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a scope bound to the mode
    jin()
        .args(["scope", "create", "config:vim", "--mode", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate mode and scope, then commit files
    jin()
        .args(["mode", "use", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let mode_file = fixture.path().join("mode.conf");
    std::fs::write(&mode_file, "mode config").unwrap();

    jin()
        .args(["add", "mode.conf"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add mode config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode-bound scope and add files
    jin()
        .args(["scope", "use", "config:vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let scope_file = fixture.path().join("scope.vim");
    std::fs::write(&scope_file, "scope config").unwrap();

    jin()
        .args(["add", "scope.vim"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add scope config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // List should show both mode and scope
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Modes:"))
        .stdout(predicate::str::contains("- editor"))
        .stdout(predicate::str::contains("Scopes:"))
        .stdout(predicate::str::contains("- config:vim"));
}

/// Test list empty categories are hidden
#[test]
fn test_list_empty_categories_hidden() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.path().join(".jin");

    // Initialize the Jin repository with the same JIN_DIR
    jin_init(fixture.path(), Some(&jin_dir)).unwrap();

    // Create only modes (no scopes or projects)
    jin()
        .args(["mode", "create", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", "editor"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let mode_file = fixture.path().join("config.txt");
    std::fs::write(&mode_file, "config").unwrap();

    // Use --mode flag to add file to mode layer, not project layer
    jin()
        .args(["add", "config.txt", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // List should show only Modes section, not Scopes or Projects
    jin()
        .arg("list")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Modes:"))
        .stdout(predicate::str::contains("Scopes:").not())
        .stdout(predicate::str::contains("Projects:").not());
}
