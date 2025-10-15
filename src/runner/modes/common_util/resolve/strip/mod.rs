//! Using statement collection and stripping functionality.
//!
//! This module provides the core functionality for handling `using` statements
//! in Nyash source code. It strips `using` statements from source files and
//! collects the targets for later resolution and AST merging.
//!
//! ## Organization
//!
//! - `collect` - Main collection and stripping logic
//! - `resolver` - Prelude resolution, parsing, merging, and expansion utilities

mod collect;
mod resolver;

// Re-export public APIs
pub use collect::collect_using_and_strip;
pub use resolver::{
    merge_prelude_asts_with_main,
    parse_preludes_to_asts,
    preexpand_at_local,
    resolve_prelude_paths_profiled,
};