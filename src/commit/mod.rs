//! Commit pipeline for Jin
//!
//! Handles atomic commits across multiple layers.

pub mod pipeline;

pub use pipeline::{CommitConfig, CommitPipeline, CommitResult};
