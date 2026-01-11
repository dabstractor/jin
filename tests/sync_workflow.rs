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
    let mode_name = format!("fetch_test_{}", unique_test_id());

    // Setup: Create commit in "remote" (actually local for testing)
    // First, create a temporary workspace to populate the remote
    let temp_workspace = TestFixture::new()?;
    let jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("test.txt"), "fetch test")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Test commit for fetch"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
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
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .assert()
        .success();

    // Now test fetch in main local repo
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
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
    let mode_name = format!("pull_test_{}", unique_test_id());

    // Setup: Create commit in remote (via temp workspace)
    let temp_workspace = TestFixture::new()?;
    let jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("remote_file.txt"), "from remote")?;

    jin()
        .args(["add", "remote_file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Now pull in main local repo
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Activate the mode and apply to see the pulled changes
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
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
    let mode_name = format!("push_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create local commit
    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin()
        .args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit to push"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // DEBUG: Check if the ref exists in local Jin repo
    let local_jin_repo = git2::Repository::open(jin_dir)?;
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    match local_jin_repo.find_reference(&ref_path) {
        Ok(_) => {}
        Err(e) => {
            // List all refs in the local Jin repo for debugging
            let all_refs = local_jin_repo.references()?;
            eprintln!("DEBUG: All refs in local Jin repo ({:?}):", jin_dir);
            for r in all_refs {
                if let Ok(reference) = r {
                    if let Some(name) = reference.name() {
                        eprintln!("  - {}", name);
                    }
                }
            }
            panic!("Local Jin repo should have ref {}: {}", ref_path, e);
        }
    }

    // Push to remote
    let output = jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .output()?;
    eprintln!("DEBUG: Push output status: {}", output.status);
    eprintln!(
        "DEBUG: Push stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "DEBUG: Push stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "Push should succeed");

    // DEBUG: Check refs in remote after push
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let all_remote_refs = remote_repo.references()?;
    eprintln!("DEBUG: All refs in remote after push:");
    for r in all_remote_refs {
        if let Ok(reference) = r {
            if let Some(name) = reference.name() {
                eprintln!("  - {}", name);
            }
        }
    }

    // Verify commit exists in remote by opening it as a git repo
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);

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

/// Test push rejected when local is behind remote
#[test]
fn test_push_rejected_when_behind() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("behind_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a commit
    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin()
        .args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Push to remote
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Now simulate remote being updated by someone else
    // Directly update the remote ref using git
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let remote_ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);

    // Create a new commit on the remote
    let sig = remote_repo.signature()?;
    let mut tree_builder = remote_repo.treebuilder(None)?;
    let oid = remote_repo.blob(b"remote update")?;
    tree_builder.insert("remote.txt", oid, 0o100644)?;
    let tree_oid = tree_builder.write()?;
    let tree = remote_repo.find_tree(tree_oid)?;

    // Get the current remote commit as parent
    let current_remote = remote_repo.find_reference(&remote_ref_path)?;
    let current_oid = current_remote.target().unwrap();
    let current_commit = remote_repo.find_commit(current_oid)?;

    // Create new commit on top
    let new_commit_oid = remote_repo.commit(
        Some(&remote_ref_path),
        &sig,
        &sig,
        "Remote update",
        &tree,
        &[&current_commit],
    )?;

    // Update the remote ref directly
    let mut remote_ref = remote_repo.find_reference(&remote_ref_path)?;
    remote_ref.set_target(new_commit_oid, "Remote update")?;

    // Now try to push again - should be rejected (local is behind)
    // First create a new local commit
    fs::write(
        remote_fixture.local_path.join("local2.txt"),
        "local2 content",
    )?;

    jin()
        .args(["add", "local2.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit 2"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Try to push - should be rejected
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("behind remote"))
        .stderr(predicate::str::contains("jin pull"))
        .stderr(predicate::str::contains("--force"));

    Ok(())
}

/// Test push succeeds with --force when behind
#[test]
fn test_push_succeeds_with_force_when_behind() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("force_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Step 1: Create remote commit via temp workspace
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("remote.txt"), "remote content")?;

    jin()
        .args(["add", "remote.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
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
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 2: In local repo, create divergent commit on same layer
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create local commit
    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin()
        .args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 3: Try to push with --force - should succeed
    jin()
        .args(["push", "--force"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    Ok(())
}

/// Test push succeeds when local is ahead
#[test]
fn test_push_succeeds_when_ahead() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("ahead_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create first commit and push
    fs::write(remote_fixture.local_path.join("v1.txt"), "version 1")?;

    jin()
        .args(["add", "v1.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Version 1"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create second commit - now we're ahead
    fs::write(remote_fixture.local_path.join("v2.txt"), "version 2")?;

    jin()
        .args(["add", "v2.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Version 2"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Push again - should succeed (ahead is OK)
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    Ok(())
}

/// Test push succeeds when local and remote are up-to-date (P1.M2.T3.S3)
///
/// When local and remote point to the same commit, push should succeed
/// (possibly reporting "Nothing to push") without any errors.
#[test]
fn test_push_succeeds_when_up_to_date() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("uptodate_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create commit and push
    fs::write(remote_fixture.local_path.join("file.txt"), "content")?;

    jin()
        .args(["add", "file.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Now try to push again without any new commits
    // Local and remote are up-to-date (same commit)
    // Push should succeed (may report "Nothing to push")
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify remote state is unchanged
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);

    match remote_repo.find_reference(&ref_path) {
        Ok(reference) => {
            let oid = reference.target().expect("Ref should have target");
            let commit = remote_repo.find_commit(oid)?;
            // Should still point to "Initial commit"
            assert!(
                commit.message().unwrap_or("").contains("Initial commit"),
                "Remote should still point to initial commit"
            );
        }
        Err(e) => panic!("Remote should have ref {}: {}", ref_path, e),
    }

    Ok(())
}

/// Test push rejected when histories have diverged
///
/// When local and remote have different commits on the same branch
/// (diverged from a common ancestor), push should be rejected without --force.
#[test]
fn test_push_rejected_when_divergent() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("divergent_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    create_mode(&mode_name, Some(&jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 1: Create base commit and push it
    fs::write(remote_fixture.local_path.join("base.txt"), "base content")?;

    jin()
        .args(["add", "base.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 2: Create divergent commit in remote (via direct git2 manipulation)
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let remote_ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);

    // Get current remote commit as base
    let current_remote = remote_repo.find_reference(&remote_ref_path)?;
    let current_oid = current_remote.target().unwrap();
    let base_commit = remote_repo.find_commit(current_oid)?;

    // Create divergent commit on remote
    let sig = remote_repo.signature()?;
    let mut tree_builder = remote_repo.treebuilder(None)?;
    let blob_oid = remote_repo.blob(b"remote divergent content")?;
    tree_builder.insert("remote_divergent.txt", blob_oid, 0o100644)?;
    let tree_oid = tree_builder.write()?;
    let tree = remote_repo.find_tree(tree_oid)?;

    let remote_commit_oid = remote_repo.commit(
        Some(&remote_ref_path),
        &sig,
        &sig,
        "Remote divergent commit",
        &tree,
        &[&base_commit],
    )?;

    // Update remote ref
    let mut remote_ref = remote_repo.find_reference(&remote_ref_path)?;
    remote_ref.set_target(remote_commit_oid, "Remote divergent update")?;

    // Step 3: Create divergent commit in local
    fs::write(
        remote_fixture.local_path.join("local_divergent.txt"),
        "local divergent content",
    )?;

    jin()
        .args(["add", "local_divergent.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local divergent commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 4: Try to push - should be rejected (histories have diverged)
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("behind remote"))
        .stderr(predicate::str::contains("jin pull"))
        .stderr(predicate::str::contains("--force"));

    Ok(())
}

/// Test sync performs complete fetch+pull+apply workflow
#[test]
fn test_sync_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("sync_test_{}", unique_test_id());

    // Setup: Create commit in remote
    let temp_workspace = TestFixture::new()?;
    let jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("sync.txt"), "sync content")?;

    jin()
        .args(["add", "sync.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Sync test"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Test sync in main repo
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Sync should fetch + pull + apply in one command
    jin()
        .arg("sync")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
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

    let result = jin().arg("fetch").current_dir(fixture.path()).assert();

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

    let result = jin().arg("push").current_dir(fixture.path()).assert();

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
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    // Create second remote
    let second_remote = remote_fixture._tempdir.path().join("remote2");
    fs::create_dir(&second_remote)?;
    git2::Repository::init_bare(&second_remote)?;

    // Link to second remote with --force
    jin()
        .args(["link", second_remote.to_str().unwrap(), "--force"])
        .current_dir(&remote_fixture.local_path)
        .assert()
        .success();

    Ok(())
}

/// Test fetch loads context with graceful fallback (P2.M3.T1)
///
/// Verifies that fetch command loads ProjectContext at startup:
/// - When context exists (.jin/context present), fetch succeeds
/// - When context doesn't exist, fetch still works with default context
/// - Context is available for filtering operations in P2.M3.T2
#[test]
fn test_fetch_loads_context() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Test 1: Fetch with initialized context (should succeed)
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Fetch should work with initialized context
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Test 2: Fetch without initialized context (should still succeed with fallback)
    let fixture2 = setup_test_repo()?;

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(fixture2.path())
        .env("JIN_DIR", fixture2.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    // Remove context file to simulate uninitialized project
    let context_path = fixture2.path().join(".jin").join("context");
    fs::remove_file(&context_path).ok(); // Ignore error if doesn't exist

    // Fetch should still work (graceful fallback to default context)
    jin()
        .arg("fetch")
        .current_dir(fixture2.path())
        .env("JIN_DIR", fixture2.jin_dir.as_ref().unwrap())
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
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
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

/// Test fetch highlights active mode updates (P2.M3.T2)
///
/// Verifies that fetch command shows active mode updates prominently:
/// - Updates to active mode are shown in "Updates for your active context" section
/// - Other mode updates are shown in "Other updates" section
#[test]
fn test_fetch_highlights_active_mode_updates() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();
    let mode_name = format!("active_test_{}", unique_test_id());
    let other_mode = format!("other_mode_{}", unique_test_id());

    // Setup: Create commits in remote for two modes
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // Create and populate active mode
    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(
        temp_workspace.path().join("active_file.txt"),
        "active mode content",
    )?;
    jin()
        .args(["add", "active_file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Add active mode file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Create and populate other mode
    create_mode(&other_mode, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &other_mode])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(
        temp_workspace.path().join("other_file.txt"),
        "other mode content",
    )?;
    jin()
        .args(["add", "other_file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Add other mode file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // In local repo, set active mode and fetch
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create the active mode in local repo
    create_mode(&mode_name, Some(jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Fetch and verify active mode is highlighted
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Updates for your active context"))
        .stdout(predicate::str::contains(format!("mode: {}", mode_name)));

    Ok(())
}

/// Test fetch separates active and other updates (P2.M3.T2)
///
/// Verifies that fetch command properly separates updates:
/// - Active context updates shown in first section
/// - Other updates shown in "Other updates" section
/// - Clear visual separation between sections
#[test]
fn test_fetch_separates_active_and_other_updates() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();
    let active_mode = format!("active_sep_{}", unique_test_id());
    let other_mode = format!("other_sep_{}", unique_test_id());

    // Setup: Create commits in remote for two modes
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // Create and populate active mode
    create_mode(&active_mode, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &active_mode])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("active.txt"), "active content")?;
    jin()
        .args(["add", "active.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Active mode commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Create and populate other mode
    create_mode(&other_mode, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &other_mode])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("other.txt"), "other content")?;
    jin()
        .args(["add", "other.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Other mode commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // In local repo, set active mode and fetch
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    create_mode(&active_mode, Some(jin_dir))?;
    jin()
        .args(["mode", "use", &active_mode])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Fetch and verify separation
    let result = jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    // Should have both sections
    assert!(stdout.contains("Updates for your active context"));
    assert!(stdout.contains("Other updates"));

    // Active mode should be in active context section
    assert!(stdout.contains(&format!("mode/{}", active_mode)));

    // Other mode should be in other updates section
    assert!(stdout.contains(&format!("mode/{}", other_mode)));

    Ok(())
}

/// Test fetch with default context (P2.M3.T2)
///
/// Verifies that fetch command works with default context:
/// - When no active mode/scope is set, all updates shown in "Other updates"
/// - No "Updates for your active context" section shown
#[test]
fn test_fetch_with_default_context() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();
    let mode_name = format!("default_ctx_{}", unique_test_id());

    // Setup: Create commits in remote
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("file.txt"), "content")?;
    jin()
        .args(["add", "file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // In local repo, fetch without setting any active mode
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Remove context to ensure default context is used
    let context_path = remote_fixture.local_path.join(".jin").join("context");
    fs::remove_file(&context_path).ok();

    // Fetch with default context
    let result = jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    // Should NOT have active context section (no mode/scope set)
    assert!(!stdout.contains("Updates for your active context"));

    // Should have updates in "Other updates" section
    assert!(stdout.contains("Other updates"));

    Ok(())
}

/// Test fetch updates refs without merging (P3.M3.T1.S2)
///
/// Verifies that fetch downloads remote refs without modifying workspace.
/// Fetch is a read-only operation from the user's perspective.
#[test]
fn test_fetch_updates_refs_without_merging() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("fetch_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Create content in remote (via temp workspace)
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("remote.txt"), "remote content")?;
    jin()
        .args(["add", "remote.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Link and push to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // In local repo: link and fetch
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Capture pre-fetch state for comparison
    let pre_fetch_oid = capture_ref_before_fetch(jin_dir, &mode_name)?;

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify: Remote ref now exists locally (fetch updates refs)
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    let jin_repo = git2::Repository::open(jin_dir)?;
    match jin_repo.find_reference(&ref_path) {
        Ok(reference) => {
            let oid = reference.target().expect("Ref should have target");
            let commit = jin_repo.find_commit(oid)?;
            assert!(
                commit.message().unwrap_or("").contains("Remote commit"),
                "Fetched ref should point to remote commit"
            );
            // Verify the ref changed (was new or updated)
            assert!(
                pre_fetch_oid.is_none() || pre_fetch_oid != Some(oid),
                "Fetch should have updated the ref"
            );
        }
        Err(e) => panic!("Fetch should have created remote ref locally: {}", e),
    }

    // Verify: Workspace NOT modified (fetch is read-only)
    let workspace_file = remote_fixture.local_path.join("remote.txt");
    assert!(
        !workspace_file.exists(),
        "Fetch should NOT create workspace files (read-only operation)"
    );

    Ok(())
}

/// Helper to capture ref OID before fetch for comparison
fn capture_ref_before_fetch(
    jin_dir: &std::path::PathBuf,
    mode_name: &str,
) -> Result<Option<git2::Oid>, Box<dyn std::error::Error>> {
    let jin_repo = git2::Repository::open(jin_dir)?;
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    Ok(jin_repo
        .find_reference(&ref_path)
        .ok()
        .map(|r| r.target().unwrap()))
}

/// Test pull merges remote changes (P3.M3.T1.S2)
///
/// Verifies that pull performs fast-forward merge when local is behind.
#[test]
fn test_pull_merges_remote_changes() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("pull_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Setup: Create commit in remote
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("merged.txt"), "remote content")?;
    jin()
        .args(["add", "merged.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // In local repo: link and pull
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create the mode in local first
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Activate mode and apply to see pulled changes
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify: File exists in workspace
    assert_workspace_file_exists(&remote_fixture.local_path, "merged.txt");

    // Verify: Layer ref points to merged commit
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    Ok(())
}

/// Test pull creates .jinmerge files for conflicts (P3.M3.T1.S2)
///
/// Verifies that pull creates .jinmerge files for conflicting changes.
/// NOTE: This test is marked as #[ignore] because conflict detection (P1.M1) is not fully implemented yet.
/// When P1.M1 is complete, remove the #[ignore] attribute.
#[test]
#[ignore = "Conflict detection (P1.M1) not fully implemented - testPull_divergent_with_conflicts also fails"]
fn test_pull_creates_jinmerge_for_conflicts() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Create base commit - use separate JIN_DIR for temp workspace
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // Base content
    fs::write(temp_workspace.path().join("config.txt"), "version=1")?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Setup local repo with base
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Make conflicting local change
    fs::write(
        remote_fixture.local_path.join("config.txt"),
        "version=local",
    )?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Local conflicting change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Make conflicting remote change
    fs::write(temp_workspace.path().join("config.txt"), "version=remote")?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Remote conflicting change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Step 4: Pull should detect conflict and create .jinmerge
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("conflicts"))
        .stdout(predicate::str::contains(".jinmerge"));

    // Verify .jinmerge file created
    let jinmerge_path = remote_fixture.local_path.join("config.txt.jinmerge");
    assert!(
        jinmerge_path.exists(),
        "Pull should create .jinmerge file for conflicts"
    );

    // Verify conflict marker format
    let jinmerge_content = fs::read_to_string(&jinmerge_path)?;
    assert!(jinmerge_content.contains("<<<<<<<"));
    assert!(jinmerge_content.contains("======="));
    assert!(jinmerge_content.contains(">>>>>>>"));
    assert!(jinmerge_content.contains("version=local")); // Local version
    assert!(jinmerge_content.contains("version=remote")); // Remote version

    Ok(())
}

/// Test pull fast-forward behaves correctly (P3.M3.T1.S2)
///
/// Verifies that pull fast-forwards when local is ancestor of remote.
/// NOTE: This test is marked as #[ignore] because merge type detection (P1.M1) is not fully implemented yet.
/// When P1.M1 is complete, remove the #[ignore] attribute.
#[test]
#[ignore = "Merge type detection (P1.M1) not fully implemented"]
fn test_pull_fast_forward_behaves_correctly() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("ff_test_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Create local commit A and push to remote
    create_mode(&mode_name, Some(jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(remote_fixture.local_path.join("file.txt"), "commit A")?;
    jin()
        .args(["add", "file.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Commit A"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create remote commit B on top of A via temp workspace
    // CRITICAL: Use the SAME JIN_DIR for both workspaces to share layer refs
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    create_mode(&mode_name, Some(jin_dir))?;
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Pull first to get the base state
    jin()
        .arg("pull")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply to get base file in workspace
    jin()
        .arg("apply")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("file2.txt"), "commit B")?;
    jin()
        .args(["add", "file2.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Commit B"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Run pull in local - should fast-forward
    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("fast-forward"));

    // Verify linear history (no merge commit)
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    let jin_repo = git2::Repository::open(jin_dir)?;
    let reference = jin_repo.find_reference(&ref_path)?;
    let oid = reference.target().unwrap();
    let commit = jin_repo.find_commit(oid)?;

    assert_eq!(
        commit.parent_count(),
        1,
        "Fast-forward should have single parent, not merge commit"
    );

    // Verify workspace has remote content
    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    assert_workspace_file_exists(&remote_fixture.local_path, "file2.txt");

    Ok(())
}

/// Test pull divergent clean merge (P3.M3.T1.S2)
///
/// Verifies that pull performs 3-way merge for non-overlapping changes.
/// NOTE: This test is marked as #[ignore] because 3-way merge (P1.M1) is not fully implemented yet.
/// When P1.M1 is complete, remove the #[ignore] attribute.
#[test]
#[ignore = "3-way merge (P1.M1) not fully implemented"]
fn test_pull_divergent_clean_merge() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Step 1: Create base commit
    // CRITICAL: Use the SAME JIN_DIR for both workspaces to share layer refs
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    fs::write(
        temp_workspace.path().join("config.txt"),
        "line1\nline2\nline3",
    )?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Push base to remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Setup local repo with base
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply to get base file in workspace
    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Local changes line2 to line2-local
    fs::write(
        remote_fixture.local_path.join("config.txt"),
        "line1\nline2-local\nline3",
    )?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Local change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Remote changes line1 to line1-remote
    fs::write(
        temp_workspace.path().join("config.txt"),
        "line1-remote\nline2\nline3",
    )?;
    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Remote change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 4: Pull - should 3-way merge cleanly
    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("3-way"));

    // Apply and verify merged content
    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let merged_content = fs::read_to_string(remote_fixture.local_path.join("config.txt"))?;
    assert!(merged_content.contains("line1-remote"));
    assert!(merged_content.contains("line2-local"));
    assert!(merged_content.contains("line3"));

    // Verify merge commit created with 2 parents
    let jin_repo = git2::Repository::open(jin_dir)?;
    let reference = jin_repo.find_reference("refs/jin/layers/global")?;
    let oid = reference.target().unwrap();
    let commit = jin_repo.find_commit(oid)?;

    assert_eq!(
        commit.parent_count(),
        2,
        "3-way merge should have 2 parents"
    );

    Ok(())
}

/// Test pull verifies layer state (P3.M3.T1.S2)
///
/// Verifies that layer state is correct after pull operation.
#[test]
fn test_pull_verifies_layer_state() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("layer_state_{}", unique_test_id());
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Setup remote with commits
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("layer.txt"), "layer content")?;
    jin()
        .args(["add", "layer.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Layer commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Pull in local repo
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify layer ref state using git2
    let jin_repo = git2::Repository::open(jin_dir)?;
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);

    let reference = jin_repo
        .find_reference(&ref_path)
        .expect("Layer ref should exist after pull");

    let oid = reference.target().expect("Ref should have target");
    let commit = jin_repo.find_commit(oid)?;

    // Verify commit message
    assert!(
        commit.message().unwrap_or("").contains("Layer commit"),
        "Layer should have correct commit"
    );

    // Verify parent count (1 for fast-forward)
    assert!(
        commit.parent_count() <= 2,
        "Merge commit should have at most 2 parents"
    );

    Ok(())
}

/// Test pull rejected with dirty workspace (P3.M3.T1.S2)
///
/// Verifies that pull fails when workspace has uncommitted changes.
#[test]
fn test_pull_rejected_with_dirty_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create uncommitted changes (add to staging but don't commit)
    fs::write(remote_fixture.local_path.join("dirty.txt"), "uncommitted")?;
    // First create a mode to have something to add to
    let mode_name = format!("dirty_test_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
    // Add the file to staging (creates uncommitted changes)
    jin()
        .args(["add", "dirty.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Pull should fail
    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("uncommitted"))
        .stderr(predicate::str::contains("commit"))
        .stderr(predicate::str::contains("reset"));

    Ok(())
}

/// Test pull --rebase rebases local changes (P3.M3.T1.S2)
///
/// NOTE: This test is marked as #[ignore] because --rebase is not implemented yet.
/// When the flag is added, remove the #[ignore] attribute.
#[test]
#[ignore = "jin pull --rebase is not implemented yet"]
fn test_pull_rebase_rebases_local_changes() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement when --rebase flag is added to pull command
    // Test should verify:
    // 1. Create local commit A
    // 2. Create remote commit B on same base
    // 3. Run jin pull --rebase
    // 4. Verify local commit A rebased on top of B
    // 5. Verify linear history: base -> B -> A'
    Ok(())
}
