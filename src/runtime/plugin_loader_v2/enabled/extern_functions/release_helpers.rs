//! Release helper functions for external function handlers
//!
//! This module provides unified `release` and `release_many` implementations
//! to avoid code duplication across all env.* extern function handlers.

use crate::bid::BidResult;
use crate::box_trait::NyashBox;

/// Handle single box finalization (best-effort)
///
/// Attempts to finalize InstanceBox or PluginBoxV2 if present.
/// This is a best-effort operation - failures are silently ignored.
pub(super) fn handle_release_single(
    args: &[Box<dyn NyashBox>]
) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some(b) = args.get(0) {
        // Try InstanceBox finalization
        if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
            let _ = inst.fini();
            return Ok(None);
        }
        // Try PluginBoxV2 finalization (plugins feature only, non-WASM)
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        if let Some(p) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            p.finalize_now();
            return Ok(None);
        }
    }
    Ok(None)
}

/// Handle array of boxes finalization (best-effort)
///
/// Iterates through an ArrayBox and attempts to finalize each element.
/// This is a best-effort operation - failures are silently ignored.
pub(super) fn handle_release_many(
    args: &[Box<dyn NyashBox>]
) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some(b) = args.get(0) {
        #[cfg(feature = "legacy-boxes")]
        if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let n = arr.len();
            for i in 0..n {
                let idx = Box::new(crate::box_trait::IntegerBox::new(i as i64))
                    as Box<dyn crate::box_trait::NyashBox>;
                let elem = arr.get(idx);

                // Try InstanceBox finalization
                if let Some(inst) = elem.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let _ = inst.fini();
                    continue;
                }

                // Try PluginBoxV2 finalization (plugins feature only, non-WASM)
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if let Some(p) = elem.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    p.finalize_now();
                    continue;
                }
            }
        }
    }
    Ok(None)
}
