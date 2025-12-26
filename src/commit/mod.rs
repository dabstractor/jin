//! Commit pipeline for atomic commits of staged files to Jin layers.
//!
//! This module provides the commit pipeline infrastructure that:
//! - Validates staged entries before commit
//! - Manages the `.jinmap` file for layer-to-file tracking
//! - Orchestrates atomic multi-layer Git commits
//! - Logs audit trail for all commits
//!
//! # Modules
//!
//! - [`validate`] - Pre-commit validation logic
//! - [`jinmap`] - Jinmap file management
//! - [`audit`] - Audit trail logging
//! - [`pipeline`] - Commit pipeline orchestration

pub mod audit;
pub mod jinmap;
pub mod pipeline;
pub mod validate;

// Re-exports for convenience
pub use audit::{format_timestamp, get_audit_dir, AuditContext, AuditEntry};
pub use jinmap::{Jinmap, JinmapMeta};
pub use pipeline::{CommitPipeline, CommitResult};
pub use validate::{
    validate_staging_index, ValidationError, ValidationErrorType, ValidationResult,
    ValidationWarning,
};
