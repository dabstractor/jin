//! Merge engine for Jin
//!
//! Handles deterministic merging of structured configuration files (JSON, YAML, TOML, INI)
//! and 3-way text merging for plain text files.
//!
//! # Core Components
//!
//! - [`MergeValue`]: Universal representation for structured data
//! - [`deep_merge`]: RFC 7396 compliant deep merge with keyed array support
//! - [`merge_layers`]: Multi-layer merge orchestration for Jin's 9-layer system
//! - [`text_merge`]: 3-way text merge for plain text files
//!
//! # Example
//!
//! ```ignore
//! use jin::merge::{MergeValue, deep_merge};
//!
//! let base = MergeValue::from_json(r#"{"name": "base"}"#)?;
//! let overlay = MergeValue::from_json(r#"{"name": "override"}"#)?;
//! let merged = deep_merge(base, overlay)?;
//! ```

pub mod deep;
pub mod layer;
pub mod text;
pub mod value;

// Core deep merge
pub use deep::{deep_merge, deep_merge_with_config, MergeConfig};

// Layer merge orchestration
pub use layer::{
    detect_format, get_applicable_layers, merge_layers, parse_content, FileFormat,
    LayerMergeConfig, LayerMergeResult, MergedFile,
};

// Text merge
pub use text::{
    has_conflict_markers, parse_conflicts, text_merge, text_merge_with_config, ConflictRegion,
    TextMergeConfig, TextMergeResult,
};

// Value type
pub use value::MergeValue;
