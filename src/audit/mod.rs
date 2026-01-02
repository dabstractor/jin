//! Audit logging for Jin operations
//!
//! Tracks all Jin commit operations in PRD-compliant JSON format,
//! stored in `.jin/audit/` directory for compliance and debugging.

pub mod entry;
pub mod logger;

pub use entry::{AuditContext, AuditEntry};
pub use logger::AuditLogger;
