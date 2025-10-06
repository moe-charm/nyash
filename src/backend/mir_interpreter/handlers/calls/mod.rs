//! Call handlers — structured module
//!
//! Phase 1: non-breaking structural split.
//! - Keep legacy implementation in `legacy.rs` (unchanged semantics)
//! - Prepare submodules for stepwise extraction: function/method/extern_call/box_call
//! - Re-export legacy so existing call sites remain valid.

pub(super) mod legacy;

// Planned stepwise extractions (placeholders for upcoming moves)
pub(super) mod function;
pub(super) mod method;
pub(super) mod extern_call;
pub(super) mod box_call;

// Re-export legacy impl to preserve current API surface
#[allow(unused_imports)]
pub(super) use legacy::*;
