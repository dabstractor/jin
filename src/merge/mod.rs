//! Merge engine for Jin
//!
//! Handles deterministic merging of structured configuration files (JSON, YAML, TOML)
//! and 3-way text merging for plain text files.

pub mod deep;
pub mod layer;
pub mod text;
pub mod value;

pub use deep::deep_merge;
pub use layer::merge_layers;
pub use text::text_merge;
pub use value::MergeValue;
