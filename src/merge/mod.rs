// Merge engine exports
// This module will contain the universal MergeValue type and merge algorithms

pub mod value;
pub mod layer;
pub mod text;
pub use value::{MergeValue, ArrayMergeStrategy, MergeConfig};
pub use layer::{LayerMerge, FileFormat, MergeContext};
pub use text::{TextMerge, MergeResult};
