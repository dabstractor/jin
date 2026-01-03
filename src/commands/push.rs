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

    // 4. Fetch remote state
    super::fetch::execute()?;

    // 5. Find the remote
    let mut remote = repo.find_remote("origin").map_err(|e| {
        if e.code() == ErrorCode::NotFound {
            JinError::Config(
                "Remote 'origin' not found in repository. Run 'jin link <url>'.".into(),
            )
        } else {
            e.into()
        }
    })?;

    // 6. Detect modified layers (exclude user-local)
    let modified_refs = detect_modified_layers(&jin_repo, &pre_fetch_refs, &args)?;

    if modified_refs.is_empty() {
        println!("Nothing to push");
        return Ok(());
    }

    // 7. Build refspecs for push
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

    // 8. Warn on force push
    if args.force {
        println!("WARNING: Force push will overwrite remote changes!");
        println!("This may cause data loss for other team members.");
    }

    // 9. Setup push options
    let mut push_opts = build_push_options()?;

    // 10. Perform push
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

/// Detect modified layers that need to be pushed
///
/// Compares pre-fetch local refs with post-fetch refs (which contain remote state) to determine:
/// - New local refs (not on remote) -> push
/// - Local refs ahead of remote -> push
/// - Local refs behind remote -> reject (unless --force)
/// - Local refs diverged from remote -> reject (unless --force)
/// - Local refs equal to remote -> skip
fn detect_modified_layers(
    jin_repo: &JinRepo,
    pre_fetch_refs: &HashMap<String, git2::Oid>,
    args: &PushArgs,
) -> Result<Vec<String>> {
    let mut modified = Vec::new();

    for (ref_name, pre_fetch_oid) in pre_fetch_refs {
        // Resolve current ref state (after fetch, this contains remote OID)
        let current_oid = match jin_repo.resolve_ref(ref_name) {
            Ok(oid) => oid,
            Err(_) => {
                // Ref was deleted by fetch - shouldn't happen but handle gracefully
                continue;
            }
        };

        // Compare pre-fetch local OID with current (remote) OID
        if args.force {
            // Force flag bypasses safety checks - push if different
            if pre_fetch_oid != &current_oid {
                modified.push(ref_name.clone());
            }
        } else {
            // Use graph comparison to determine if push is safe
            match crate::git::refs::compare_refs(jin_repo, *pre_fetch_oid, current_oid)? {
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

    // Check for new local refs that didn't exist before fetch
    let current_refs = jin_repo.list_refs("refs/jin/layers/*")?;
    for ref_name in current_refs {
        if ref_name.contains("/local") {
            continue;
        }

        // If this ref wasn't in our pre-fetch map, it's new - push it
        if !pre_fetch_refs.contains_key(&ref_name) {
            modified.push(ref_name);
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
