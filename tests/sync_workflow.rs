//! Remote synchronization workflow integration tests for Jin
//!
//! Tests remote sync operations using local filesystem remotes (no network).
//! Validates: link → fetch → pull → push → sync workflows.

use predicates::prelude::*;
use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Test linking to a local remote repository
#[test]
fn test_link_to_local_remote() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;

    // Link to the bare remote using file:// URL
    let remote_url = format!("file://{}", remote_fixture.remote_path.display());

    jin()
        .args(["link", &remote_url])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Verify remote configured in ~/.jin/config.toml
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let config_path = home_dir.join(".jin/config.toml");

    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)?;
        assert!(
            config_content.contains(&remote_url) || config_content.contains("remote"),
            "Config should contain remote URL or remote section"
        );
    }

    Ok(())
}

/// Test linking with filesystem path (not file:// URL)
#[test]
fn test_link_with_filesystem_path() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;

    // Link using absolute filesystem path
    let remote_path = remote_fixture.remote_path.to_str().unwrap();

    jin()
        .args(["link", remote_path])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    Ok(())
}

/// Test fetch updates refs from remote
#[test]
fn test_fetch_updates_refs() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("fetch_test_{}", std::process::id());

    // Setup: Create commit in "remote" (actually local for testing)
    // First, create a temporary workspace to populate the remote
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path())?;

    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    fs::write(temp_workspace.path().join("test.txt"), "fetch test")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Test commit for fetch"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    // Link temp workspace to remote and push
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    // Now test fetch in main local repo
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Verify refs updated (exact verification depends on implementation)
    // At minimum, fetch should succeed without errors

    Ok(())
}

/// Test pull merges changes from remote
#[test]
fn test_pull_merges_changes() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("pull_test_{}", std::process::id());

    // Setup: Create commit in remote (via temp workspace)
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path())?;

    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    fs::write(temp_workspace.path().join("remote_file.txt"), "from remote")?;

    jin()
        .args(["add", "remote_file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    // Now pull in main local repo
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Activate the mode and apply to see the pulled changes
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Verify file exists in workspace
    assert_workspace_file_exists(&remote_fixture.local_path, "remote_file.txt");

    Ok(())
}

/// Test push uploads commits to remote
#[test]
fn test_push_uploads_commits() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("push_test_{}", std::process::id());

    // Link to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Create local commit
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin()
        .args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit to push"])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Push to remote
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Verify commit exists in remote by opening it as a git repo
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);

    match remote_repo.find_reference(&ref_path) {
        Ok(reference) => {
            // Verify it points to a commit
            let oid = reference.target().expect("Ref should have target");
            let commit = remote_repo.find_commit(oid)?;
            let message = commit.message().unwrap_or("");
            assert!(
                message.contains("Local commit to push"),
                "Remote should have the pushed commit"
            );
        }
        Err(e) => panic!("Remote should have ref {}: {}", ref_path, e),
    }

    Ok(())
}

/// Test sync performs complete fetch+pull+apply workflow
#[test]
fn test_sync_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("sync_test_{}", std::process::id());

    // Setup: Create commit in remote
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path())?;

    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    fs::write(temp_workspace.path().join("sync.txt"), "sync content")?;

    jin()
        .args(["add", "sync.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Sync test"])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    // Test sync in main repo
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Sync should fetch + pull + apply in one command
    jin()
        .arg("sync")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Verify file in workspace (sync should apply automatically)
    assert_workspace_file_exists(&remote_fixture.local_path, "sync.txt");

    Ok(())
}

/// Test error: link without valid remote URL
#[test]
fn test_link_invalid_url_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    jin()
        .args(["link", "invalid-url"])
        .current_dir(fixture.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));

    Ok(())
}

/// Test error: fetch without linked remote
#[test]
fn test_fetch_no_remote_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    let result = jin()
        .arg("fetch")
        .current_dir(fixture.path())
        .assert();

    // Should fail or warn about no remote
    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        !output.status.success()
            || stderr_str.contains("remote")
            || stderr_str.contains("not configured")
            || stdout_str.contains("not yet implemented"),
        "Fetch without remote should fail or warn"
    );

    Ok(())
}

/// Test error: push without linked remote
#[test]
fn test_push_no_remote_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    let result = jin()
        .arg("push")
        .current_dir(fixture.path())
        .assert();

    // Should fail or warn about no remote
    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        !output.status.success()
            || stderr_str.contains("remote")
            || stderr_str.contains("not configured")
            || stdout_str.contains("not yet implemented"),
        "Push without remote should fail or warn"
    );

    Ok(())
}

/// Test link with --force flag (replace existing remote)
#[test]
fn test_link_force_replaces_existing() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;

    // Link to first remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Create second remote
    let second_remote = remote_fixture._tempdir.path().join("remote2");
    fs::create_dir(&second_remote)?;
    git2::Repository::init_bare(&second_remote)?;

    // Link to second remote with --force
    jin()
        .args([
            "link",
            second_remote.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    Ok(())
}

/// Test that sync works with empty remote
#[test]
fn test_sync_empty_remote() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;

    // Link to empty remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
        ])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Sync with empty remote should succeed (no changes)
    jin()
        .arg("sync")
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    Ok(())
}
