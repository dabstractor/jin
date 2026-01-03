//! Merge detection for determining merge strategy
//!
//! Provides [`MergeType`] enum and [`detect_merge_type()`] function for
//! determining whether a pull operation requires fast-forward or 3-way merge.

use crate::core::Result;
use crate::git::JinRepo;
use git2::Oid;

/// Type of merge required to integrate remote changes
///
/// Determines the merge strategy based on the relationship between
/// local and remote commit histories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeType {
    /// Local and remote are at the same commit (no action needed)
    UpToDate,

    /// Local commit is an ancestor of remote (simple fast-forward possible)
    FastForward,

    /// Remote commit is an ancestor of local (local is ahead)
    LocalAhead,

    /// Local and remote have diverged (requires 3-way merge)
    Divergent,
}

/// Detect the merge type required to integrate remote changes
///
/// # Algorithm
///
/// 1. Check if OIDs are equal → `UpToDate`
/// 2. Use `graph_ahead_behind` to determine relationship:
///    - (0, 0) → Already equal (redundant check)
///    - (n, 0) → Local is ancestor of remote → `FastForward`
///    - (0, n) → Remote is ancestor of local → `LocalAhead`
///    - (m, n) → Both have unique commits → `Divergent`
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `local_oid` - OID of local commit
/// * `remote_oid` - OID of remote commit
///
/// # Returns
///
/// `MergeType` indicating the merge strategy required
///
/// # Errors
///
/// Returns `JinError::Git` if graph traversal fails
///
/// # Example
///
/// ```ignore
/// use jin::git::{JinRepo, merge::{detect_merge_type, MergeType}};
///
/// let repo = JinRepo::open_or_create()?;
/// let local_oid = repo.resolve_ref("refs/jin/layers/global")?;
/// let remote_oid = repo.resolve_ref("refs/remotes/origin/layers/global")?;
///
/// match detect_merge_type(&repo, local_oid, remote_oid)? {
///     MergeType::FastForward => println!("Fast-forward merge"),
///     MergeType::Divergent => println!("3-way merge needed"),
///     _ => println!("No merge needed"),
/// }
/// ```
pub fn detect_merge_type(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<MergeType> {
    // PATTERN: Check equality first - graph_ahead_behind is expensive
    // This check also handles the case where both OIDs are the same
    if local_oid == remote_oid {
        return Ok(MergeType::UpToDate);
    }

    // PATTERN: Use graph_ahead_behind for accurate state detection
    // This matches the existing RefComparison pattern in src/git/refs.rs
    // graph_ahead_behind returns (ahead_count, behind_count)
    let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;

    match (ahead, behind) {
        (0, 0) => Ok(MergeType::UpToDate),    // Same commit (redundant due to equality check above)
        (0, _) => Ok(MergeType::FastForward), // Remote is ahead (we're behind, can fast-forward)
        (_, 0) => Ok(MergeType::LocalAhead),  // Local is ahead (we're ahead of remote)
        (_, _) => Ok(MergeType::Divergent),   // Both have unique commits
    }
}

/// Alternative implementation using merge_base and descendant_of
///
/// This approach uses git2's merge_base and descendant_of methods
/// instead of graph_ahead_behind. Either approach is valid.
#[allow(dead_code)]
pub fn detect_merge_type_with_base(
    repo: &JinRepo,
    local_oid: Oid,
    remote_oid: Oid,
) -> Result<MergeType> {
    // Check equality first
    if local_oid == remote_oid {
        return Ok(MergeType::UpToDate);
    }

    // Check fast-forward: is local an ancestor of remote?
    if repo.inner().graph_descendant_of(remote_oid, local_oid)? {
        return Ok(MergeType::FastForward);
    }

    // Check if remote is ancestor of local (local is ahead)
    if repo.inner().graph_descendant_of(local_oid, remote_oid)? {
        return Ok(MergeType::LocalAhead);
    }

    // Find merge base to confirm divergence
    match repo.inner().merge_base(local_oid, remote_oid) {
        Ok(_) => Ok(MergeType::Divergent),
        // No merge base means unrelated histories - treat as divergent
        Err(_) => Ok(MergeType::Divergent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test commit chain
    fn create_test_commit_chain(repo: &git2::Repository, count: usize) -> Vec<Oid> {
        let sig = repo.signature().unwrap();
        let mut commit_oids = Vec::new();

        for i in 0..count {
            let mut tree_builder = repo.treebuilder(None).unwrap();
            let blob_oid = repo.blob(format!("content{}", i).as_bytes()).unwrap();
            tree_builder.insert("file.txt", blob_oid, 0o100644).unwrap();
            let tree_oid = tree_builder.write().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();

            // Build parent slice using local array
            // This creates a linear chain: commit_0 <- commit_1 <- commit_2
            let parent_array: [&git2::Commit<'_>; 1] = match commit_oids.last() {
                Some(oid) => {
                    let commit = repo.find_commit(*oid).unwrap();
                    // Store reference in local array
                    // SAFETY: The commit lives until the end of this scope,
                    // and we immediately use it in the commit call below
                    unsafe { std::mem::transmute([&commit]) }
                }
                None => unsafe { std::mem::zeroed() },
            };

            let parent_refs: &[&git2::Commit<'_>] = if commit_oids.is_empty() {
                &[]
            } else {
                &parent_array
            };

            let oid = repo
                .commit(
                    None,
                    &sig,
                    &sig,
                    &format!("Commit {}", i),
                    &tree,
                    parent_refs,
                )
                .unwrap();

            commit_oids.push(oid);
        }

        commit_oids
    }

    #[test]
    fn test_detect_merge_type_equal() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 1);
        let oid = commits[0];

        let result = detect_merge_type(&jin_repo, oid, oid).unwrap();
        assert_eq!(result, MergeType::UpToDate);
    }

    #[test]
    fn test_detect_merge_type_fast_forward() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        // local is ancestor of remote (local is at commit 0, remote is at commit 2)
        // For fast-forward: local should be ancestor of remote, meaning remote has more commits
        // So (ahead=0, behind>0) when calling graph_ahead_behind(local, remote)
        let local = commits[0];
        let remote = commits[2];

        let result = detect_merge_type(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::FastForward);
    }

    #[test]
    fn test_detect_merge_type_local_ahead() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        // remote is ancestor of local (remote is at commit 0, local is at commit 2)
        // For local ahead: local has more commits than remote
        // So (ahead>0, behind=0) when calling graph_ahead_behind(local, remote)
        let local = commits[2];
        let remote = commits[0];

        let result = detect_merge_type(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::LocalAhead);
    }

    #[test]
    fn test_detect_merge_type_divergent() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let sig = jin_repo.inner().signature().unwrap();

        // Create base commit
        let mut tree_builder = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid = jin_repo.inner().blob(b"base").unwrap();
        tree_builder.insert("file.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = jin_repo.inner().find_tree(tree_oid).unwrap();
        let base_oid = jin_repo
            .inner()
            .commit(None, &sig, &sig, "base", &tree, &[])
            .unwrap();
        let base_commit = jin_repo.inner().find_commit(base_oid).unwrap();

        // Create divergent commit 1
        let mut tree_builder1 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid1 = jin_repo.inner().blob(b"divergent1").unwrap();
        tree_builder1
            .insert("file1.txt", blob_oid1, 0o100644)
            .unwrap();
        let tree_oid1 = tree_builder1.write().unwrap();
        let tree1 = jin_repo.inner().find_tree(tree_oid1).unwrap();
        let divergent1 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent1", &tree1, &[&base_commit])
            .unwrap();

        // Create divergent commit 2 (same parent, different content)
        let mut tree_builder2 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid2 = jin_repo.inner().blob(b"divergent2").unwrap();
        tree_builder2
            .insert("file2.txt", blob_oid2, 0o100644)
            .unwrap();
        let tree_oid2 = tree_builder2.write().unwrap();
        let tree2 = jin_repo.inner().find_tree(tree_oid2).unwrap();
        let divergent2 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent2", &tree2, &[&base_commit])
            .unwrap();

        let result = detect_merge_type(&jin_repo, divergent1, divergent2).unwrap();
        assert_eq!(result, MergeType::Divergent);
    }

    #[test]
    fn test_detect_merge_type_with_base_equal() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 1);
        let oid = commits[0];

        let result = detect_merge_type_with_base(&jin_repo, oid, oid).unwrap();
        assert_eq!(result, MergeType::UpToDate);
    }

    #[test]
    fn test_detect_merge_type_with_base_fast_forward() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        let local = commits[0];
        let remote = commits[2];

        let result = detect_merge_type_with_base(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::FastForward);
    }

    #[test]
    fn test_detect_merge_type_with_base_local_ahead() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        let local = commits[2];
        let remote = commits[0];

        let result = detect_merge_type_with_base(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::LocalAhead);
    }

    #[test]
    fn test_detect_merge_type_with_base_divergent() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let jin_repo = JinRepo::create_at(&repo_path).unwrap();

        let sig = jin_repo.inner().signature().unwrap();

        // Create base commit
        let mut tree_builder = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid = jin_repo.inner().blob(b"base").unwrap();
        tree_builder.insert("file.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = jin_repo.inner().find_tree(tree_oid).unwrap();
        let base_oid = jin_repo
            .inner()
            .commit(None, &sig, &sig, "base", &tree, &[])
            .unwrap();
        let base_commit = jin_repo.inner().find_commit(base_oid).unwrap();

        // Create divergent commit 1
        let mut tree_builder1 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid1 = jin_repo.inner().blob(b"divergent1").unwrap();
        tree_builder1
            .insert("file1.txt", blob_oid1, 0o100644)
            .unwrap();
        let tree_oid1 = tree_builder1.write().unwrap();
        let tree1 = jin_repo.inner().find_tree(tree_oid1).unwrap();
        let divergent1 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent1", &tree1, &[&base_commit])
            .unwrap();

        // Create divergent commit 2
        let mut tree_builder2 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid2 = jin_repo.inner().blob(b"divergent2").unwrap();
        tree_builder2
            .insert("file2.txt", blob_oid2, 0o100644)
            .unwrap();
        let tree_oid2 = tree_builder2.write().unwrap();
        let tree2 = jin_repo.inner().find_tree(tree_oid2).unwrap();
        let divergent2 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent2", &tree2, &[&base_commit])
            .unwrap();

        let result = detect_merge_type_with_base(&jin_repo, divergent1, divergent2).unwrap();
        assert_eq!(result, MergeType::Divergent);
    }

    #[test]
    fn test_merge_type_derive_traits() {
        // Verify MergeType has the expected derives
        let mt = MergeType::FastForward;

        // Debug
        assert_eq!(format!("{:?}", mt), "FastForward");

        // Clone
        let mt_clone = mt.clone();
        assert_eq!(mt, mt_clone);

        // Copy
        let mt_copy = mt;
        assert_eq!(mt, mt_copy);

        // PartialEq
        assert_eq!(MergeType::UpToDate, MergeType::UpToDate);
        assert_ne!(MergeType::FastForward, MergeType::Divergent);
    }
}
