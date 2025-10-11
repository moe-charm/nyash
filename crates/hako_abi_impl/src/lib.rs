//! Hakorune ABI Implementation
//!
//! Shared implementation used by both nyash_kernel and plugins.
//! NO dependency on nyash-rust to avoid circular dependency.
//!
//! ## Instance Manager Macros
//!
//! Plugins can use the instance manager macros for standardized instance storage:
//!
//! ```rust
//! use hako_abi_impl::define_instance_storage;
//!
//! struct MyInstance { data: Vec<i32> }
//! define_instance_storage!(MyInstance);
//!
//! // Then use with_instance! and with_instance_mut! macros
//! ```
//!
//! See [`instance_manager`] module for details.

pub mod array_impl;
pub mod instance_manager;
pub mod tlv;

// Re-exports
pub use array_impl::ArrayRegistry;
pub use hako_abi;

// Note: Macros from instance_manager are automatically exported at crate root
// via #[macro_export], so they can be used as hako_abi_impl::macro_name!
