//! Git layer integration for Jin phantom repository.
//!
//! This module provides:
//! - [`JinRepo`]: Wrapper for Jin's dedicated bare Git repository
//! - [`RefOps`]: Reference operations under `refs/jin/layers/*` namespace
//! - [`ObjectOps`]: Object creation (blobs, trees, commits)
//! - [`TreeOps`]: Tree walking utilities
//! - [`JinTransaction`]: Transaction wrapper for atomic reference updates
//! - [`remote`]: Remote operation utilities for fetch, pull, push

pub mod merge;
pub mod objects;
pub mod refs;
pub mod remote;
pub mod repo;
pub mod transaction;
pub mod tree;

pub use merge::{detect_merge_type, find_merge_base, MergeType};
pub use objects::{EntryMode, ObjectOps, TreeEntry};
pub use refs::RefOps;
pub use repo::JinRepo;
pub use transaction::{
    IncompleteTransaction, JinTransaction, LayerTransaction, LayerUpdate, RecoveryManager,
    TransactionLog, TransactionState,
};
pub use tree::TreeOps;

// Re-export git2 types commonly used
pub use git2::{ObjectType, Oid, TreeWalkMode, TreeWalkResult};
