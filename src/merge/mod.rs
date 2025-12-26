// Merge engine exports
// This module will contain the universal MergeValue type and merge algorithms

pub mod layer;
pub mod text;
pub mod value;
pub use layer::{FileFormat, LayerMerge, MergeContext};
pub use text::{MergeResult, TextMerge};
pub use value::{ArrayMergeStrategy, MergeConfig, MergeValue};
