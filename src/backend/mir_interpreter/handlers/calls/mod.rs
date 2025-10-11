//! Call handlers — structured module
//!
//! Phase 1: non-breaking structural split.
//! - Keep legacy implementation in `legacy.rs` (unchanged semantics)
//! - Active submodules: function, method
//! - Re-export legacy so existing call sites remain valid.
//!
//! Phase 1 Cleanup (2025-10-11):
//! - ✅ extern_call module removed (8 lines) - skeleton only, ExternCall instruction retired
//! - ✅ box_call module removed (11 lines) - empty impl, fast-paths removed in Phase 15.7

pub(super) mod legacy;

// Active submodules
pub(super) mod function;
pub(super) mod method;

// Re-export legacy impl to preserve current API surface
#[allow(unused_imports)]
pub(super) use legacy::*;
