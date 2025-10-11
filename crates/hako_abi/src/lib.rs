//! Hakorune ABI Definitions
//!
//! Pure trait-based ABI contracts with ZERO dependencies.
//! Used by both core and plugins.

pub mod handles;
pub mod types;
pub mod array;

// Re-exports for convenience
pub use handles::{HakoHandle, HAKO_INVALID_HANDLE};
pub use types::*;
pub use array::ArrayAbi;
