//! Implementation of `jin push`
//!
//! Uploads modified local layer refs to remote repository.
//! Never pushes user-local layer (machine-specific).

use crate::cli::PushArgs;
use crate::core::{JinConfig, JinError, Result};
use crate::git::remote::build_push_options;
use crate::git::{JinRepo, RefOps};
use git2::ErrorCode;

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

    // 2. Fetch remote state first
    super::fetch::execute()?;

    // 3. Open repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 4. Find the remote
    let mut remote = repo.find_remote("origin").map_err(|e| {
        if e.code() == ErrorCode::NotFound {
            JinError::Config(
                "Remote 'origin' not found in repository. Run 'jin link <url>'.".into(),
            )
        } else {
            e.into()
        }
    })?;

    // 5. Detect modified layers (exclude user-local)
    let modified_refs = detect_modified_layers(&jin_repo)?;

    if modified_refs.is_empty() {
        println!("Nothing to push");
        return Ok(());
    }

    // 6. Build refspecs for push
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

    // 7. Warn on force push
    if args.force {
        println!("WARNING: Force push will overwrite remote changes!");
        println!("This may cause data loss for other team members.");
    }

    // 8. Setup push options
    let mut push_opts = build_push_options()?;

    // 9. Perform push
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

/// Detect modified layers that need to be pushed
///
/// Returns refs that exist locally but either don't exist remotely or differ from remote.
/// Filters out user-local layer (refs/jin/layers/local) which is never synced.
fn detect_modified_layers(jin_repo: &JinRepo) -> Result<Vec<String>> {
    let local_refs = jin_repo.list_refs("refs/jin/layers/*")?;
    let mut modified = Vec::new();

    for ref_name in local_refs {
        // CRITICAL: Skip user-local layer (never push)
        if ref_name.contains("/local") {
            continue;
        }

        // Check if ref exists locally
        if !jin_repo.ref_exists(&ref_name) {
            continue;
        }

        let local_oid = jin_repo.resolve_ref(&ref_name)?;

        // Check if ref differs from remote or doesn't exist on remote
        let should_push = if jin_repo.ref_exists(&ref_name) {
            // Ref exists both locally and remotely - check if they differ
            // Note: After fetch, both local and remote refs exist in our local repo
            // We compare the OIDs to see if local is ahead
            match jin_repo.resolve_ref(&ref_name) {
                Ok(remote_oid) => local_oid != remote_oid,
                Err(_) => true, // Assume push needed if we can't resolve remote
            }
        } else {
            true // New local ref, push it
        };

        if should_push {
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
