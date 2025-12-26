//! Staging system for Jin.
//!
//! This module provides the staging infrastructure that tracks files to be
//! committed to specific layers in Jin's multi-layer configuration system.
//!
//! # Components
//!
//! - [`FileStatus`] - Bitflags for tracking file state
//! - [`StagedEntry`] - Represents a single staged file with metadata
//! - [`StagingIndex`] - Manages the collection of staged entries
//! - [`LayerRouter`] - Routes files to appropriate layers based on CLI flags
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::staging::{StagingIndex, LayerRouter, StagedEntry};
//! use jin_glm::core::Layer;
//! use std::path::PathBuf;
//!
//! // Create a router to determine target layers
//! let router = LayerRouter::new("myapp".to_string());
//! let layer = router.route(Some("claude"), None, false, false)?;
//!
//! // Stage a file
//! let entry = StagedEntry::new(
//!     PathBuf::from("config.json"),
//!     layer,
//!     b"{\"key\": \"value\"}"
//! )?;
//!
//! let mut index = StagingIndex::new();
//! index.add_entry(entry)?;
//! ```

pub mod entry;
pub mod index;
pub mod router;

pub use entry::{FileStatus, StagedEntry};
pub use index::StagingIndex;
pub use router::LayerRouter;
