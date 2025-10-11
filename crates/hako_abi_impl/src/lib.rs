//! Hakorune ABI Implementation
//!
//! Shared implementation used by both nyash_kernel and plugins.
//! NO dependency on nyash-rust to avoid circular dependency.

pub mod tlv;
pub mod array_impl;

// Re-exports
pub use hako_abi;
pub use array_impl::ArrayRegistry;
