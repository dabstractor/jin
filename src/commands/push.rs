//! Implementation of `jin push`
//!
//! Uploads modified local layer refs to remote repository.
//! Never pushes user-local layer (machine-specific).

use crate::cli::PushArgs;
use crate::core::{JinConfig, JinError, Result};
use crate::git::remote::build_push_options;
use crate::git::{JinRepo, RefOps};
use git2::ErrorCode;
use std::collections::HashMap;

/// Execute the push command
///
/// Uploads modified layers to remote repository.
/// Validates remote configuration exists and filters out user-local layer.
pub fn execute(args: PushArgs) -> Result<()> {
    // 1. Validate remote configuration
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(JinError::Config(
        "No remote configured. Run 'jin link <url>'.".into(),
    ))?;

    // 2. Open repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 3. Capture pre-fetch local refs (fetch will overwrite them)
    let pre_fetch_refs = capture_local_refs(&jin_repo)?;

    // 4. Capture what refs exist on remote BEFORE fetch (to detect new refs)
    let pre_fetch_remote_refs = capture_remote_refs(&jin_repo)?;

    // 5. Fetch remote state
    super::fetch::execute()?;

    // 6. Find the remote
    let mut remote = repo.find_remote("origin").map_err(|e| {
        if e.code() == ErrorCode::NotFound {
            JinError::Config(
                "Remote 'origin' not found in repository. Run 'jin link <url>'.".into(),
            )
        } else {
            e.into()
        }
    })?;

    // 7. Detect modified layers (exclude user-local)
    let modified_refs =
        detect_modified_layers(&jin_repo, &pre_fetch_refs, &pre_fetch_remote_refs, &args)?;

    if modified_refs.is_empty() {
        println!("Nothing to push");
        return Ok(());
    }

    // 8. Build refspecs for push
    let refspecs: Vec<String> = modified_refs
        .iter()
        .map(|ref_name| {
            if args.force {
                format!("+{}:{}", ref_name, ref_name) // Force push
            } else {
                format!("{}:{}", ref_name, ref_name) // Normal push
            }
        })
        .collect();

    // 9. Warn on force push
    if args.force {
        println!("WARNING: Force push will overwrite remote changes!");
        println!("This may cause data loss for other team members.");
    }

    // 10. Setup push options
    let mut push_opts = build_push_options()?;

    // 11. Perform push
    println!("Pushing to origin ({})...", remote_config.url);

    let refspec_refs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();

    match remote.push(&refspec_refs, Some(&mut push_opts)) {
        Ok(()) => {
            println!("\nSuccessfully pushed {} layer(s)", modified_refs.len());
            Ok(())
        }
        Err(e) => {
            println!(); // New line after push attempt
            match e.code() {
                ErrorCode::Auth => Err(JinError::Config(
                    "Authentication failed. Check your SSH keys or credentials.\n\
                    Try: ssh -T git@github.com (for GitHub)"
                        .into(),
                )),
                _ if e.message().contains("non-fast-forward") => Err(JinError::Config(
                    "Push rejected: non-fast-forward update.\n\
                    The remote contains commits you don't have locally.\n\
                    Run 'jin pull' to merge remote changes, or use '--force' to overwrite.\n\
                    WARNING: --force may cause data loss!"
                        .into(),
                )),
                _ => Err(e.into()),
            }
        }
    }
}

/// Capture local refs before fetch (fetch will overwrite them with remote refs)
///
/// We need to store the pre-fetch local OIDs so we can compare them against
/// the post-fetch state (which contains remote OIDs) to detect if local is behind.
fn capture_local_refs(jin_repo: &JinRepo) -> Result<HashMap<String, git2::Oid>> {
    let mut local_refs = HashMap::new();
    let all_refs = jin_repo.list_refs("refs/jin/layers/*")?;

    for ref_name in all_refs {
        // Skip user-local layer
        if ref_name.contains("/local") {
            continue;
        }

        // Store the OID of each local ref
        if let Ok(oid) = jin_repo.resolve_ref(&ref_name) {
            local_refs.insert(ref_name, oid);
        }
    }

    Ok(local_refs)
}

/// Capture remote refs before fetch (to detect new local refs)
///
/// Since our refspec fetches directly into local refs (+refs/jin/layers/*:refs/jin/layers/*),
/// we cannot rely on remote-tracking refs. Instead, we open the remote repository
/// directly (via its URL or filesystem path) and check which refs exist there.
fn capture_remote_refs(jin_repo: &JinRepo) -> Result<std::collections::HashSet<String>> {
    let mut remote_refs = std::collections::HashSet::new();

    // Get the remote URL from the git config
    let repo = jin_repo.inner();
    let remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => return Ok(remote_refs), // No remote configured
    };

    let remote_url = match remote.url() {
        Some(url) => url,
        None => return Ok(remote_refs),
    };

    // For file:// URLs, we can open the bare repo directly
    if remote_url.starts_with("file://") || remote_url.starts_with('/') {
        // Strip the file:// prefix if present
        let remote_path = if remote_url.starts_with("file://") {
            remote_url.trim_start_matches("file://")
        } else {
            remote_url
        };

        // Try to open the remote repository
        if let Ok(remote_repo) = git2::Repository::open(remote_path) {
            // List all refs in the remote repository
            let all_refs = match remote_repo.references_glob("refs/jin/layers/*") {
                Ok(refs) => refs,
                Err(_) => return Ok(remote_refs),
            };

            for reference in all_refs.flatten() {
                if let Some(name) = reference.name() {
                    // Skip user-local layer
                    if !name.contains("/local") {
                        remote_refs.insert(name.to_string());
                    }
                }
            }
        }
    }

    // For non-file URLs (network remotes), we return empty set
    // In this case, we'll rely on fetch behavior to determine if ref exists

    Ok(remote_refs)
}

/// Detect modified layers that need to be pushed
///
/// Compares pre-fetch local refs with remote refs and post-fetch local refs to determine:
/// - New local refs (not on remote) -> push
/// - Local refs ahead of remote -> push
/// - Local refs behind remote -> reject (unless --force)
/// - Local refs diverged from remote -> reject (unless --force)
/// - Local refs equal to remote -> skip
///
/// NOTE: Since our refspec fetches directly into local refs (+refs/jin/layers/*:refs/jin/layers/*),
/// after fetch the local ref contains the remote OID. We use pre_fetch_remote_refs (captured
/// by directly opening the remote repo) to determine if a ref exists on remote.
fn detect_modified_layers(
    jin_repo: &JinRepo,
    pre_fetch_local_refs: &HashMap<String, git2::Oid>,
    pre_fetch_remote_refs: &std::collections::HashSet<String>,
    args: &PushArgs,
) -> Result<Vec<String>> {
    let mut modified = Vec::new();

    for (ref_name, pre_fetch_local_oid) in pre_fetch_local_refs {
        // Check if this ref exists on remote
        let remote_has_ref = pre_fetch_remote_refs.contains(ref_name);

        if !remote_has_ref {
            // Ref doesn't exist on remote - it's new, push it
            modified.push(ref_name.clone());
            continue;
        }

        // Ref exists on both local and remote - compare OIDs
        // Note: after fetch, the local ref now points to the remote OID
        let remote_oid = match jin_repo.resolve_ref(ref_name) {
            Ok(oid) => oid,
            Err(_) => {
                // Ref was deleted by fetch - shouldn't happen but handle gracefully
                continue;
            }
        };

        // Compare pre-fetch local OID with remote OID
        if args.force {
            // Force flag bypasses safety checks - push if different
            if pre_fetch_local_oid != &remote_oid {
                modified.push(ref_name.clone());
            }
        } else {
            // Use graph comparison to determine if push is safe
            match crate::git::refs::compare_refs(jin_repo, *pre_fetch_local_oid, remote_oid)? {
                crate::git::refs::RefComparison::Ahead => {
                    // Local is ahead - safe to push
                    modified.push(ref_name.clone());
                }
                crate::git::refs::RefComparison::Equal => {
                    // Refs are equal - nothing to push
                }
                crate::git::refs::RefComparison::Behind
                | crate::git::refs::RefComparison::Diverged => {
                    // REJECT: Local is behind or diverged from remote
                    return Err(JinError::BehindRemote {
                        layer: ref_name.clone(),
                    });
                }
            }
        }
    }

    Ok(modified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_args_force() {
        let args = PushArgs { force: true };
        assert!(args.force);

        let args = PushArgs { force: false };
        assert!(!args.force);
    }
}
