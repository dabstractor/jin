//! Staging system for Jin
//!
//! Manages the staging area where files are prepared before committing
//! to their target layers.

pub mod entry;
pub mod index;
pub mod router;

pub use entry::StagedEntry;
pub use index::StagingIndex;
pub use router::route_to_layer;
