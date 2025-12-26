// Core type exports
// This module contains fundamental types: error, layer, config

pub mod error;
pub mod layer;

// Re-exports for convenience
pub use error::{JinError, Result};
pub use layer::Layer;

// Future modules
// pub mod config;
