//! Hakorune ABI Implementation
//!
//! Shared implementation used by both nyash_kernel and plugins.
//! NO dependency on nyash-rust to avoid circular dependency.

pub mod array_impl;
pub mod tlv;

// Re-exports
pub use array_impl::ArrayRegistry;
pub use hako_abi;
