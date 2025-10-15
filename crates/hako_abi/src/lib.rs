//! Hakorune ABI Definitions
//!
//! Pure trait-based ABI contracts with ZERO dependencies.
//! Used by both core and plugins.

pub mod array;
pub mod handles;
pub mod types;

// Re-exports for convenience
pub use array::ArrayAbi;
pub use handles::{HAKO_INVALID_HANDLE, HakoHandle};
pub use types::*;
