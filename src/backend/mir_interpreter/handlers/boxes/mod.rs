//! Boxes handler split (Phase 1)
//!
//! Non-breaking structural split of handlers/boxes.rs into submodules.
//! - legacy.rs keeps existing behavior.
//! - newbox.rs extracts NewBox handler.
//! - fields.rs/methods.rs/lifecycle.rs are prepared for stepwise extraction.

pub(super) mod legacy;
pub(super) mod newbox;
pub(super) mod fields;
pub(super) mod methods;
pub(super) mod lifecycle;

// Preserve current API surface via legacy re-export while extracting gradually.
#[allow(unused_imports)]
pub(super) use legacy::*;
