//! Staging system for Jin
//!
//! Manages the staging area where files are prepared before committing
//! to their target layers.

pub mod entry;
pub mod gitignore;
pub mod index;
pub mod router;
pub mod workspace;

pub use entry::{StagedEntry, StagedOperation};
pub use gitignore::ensure_in_managed_block;
pub use index::StagingIndex;
pub use router::{route_to_layer, validate_routing_options, RoutingOptions};
pub use workspace::{get_file_mode, is_git_tracked, is_symlink, read_file, walk_directory};
