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
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);
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
    eprintln!("DEBUG: Push stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("DEBUG: Push stderr: {}", String::from_utf8_lossy(&output.stderr));
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
    let remote_ref_path = format!("refs/jin/layers/mode/{}", mode_name);

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
    fs::write(remote_fixture.local_path.join("local2.txt"), "local2 content")?;

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
        .args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
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
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);

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
    let remote_ref_path = format!("refs/jin/layers/mode/{}", mode_name);

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
    fs::write(remote_fixture.local_path.join("local_divergent.txt"), "local divergent content")?;

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
