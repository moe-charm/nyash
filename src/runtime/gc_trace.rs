//! Minimal GC tracing helpers (skeleton)
//!
//! Downcast-based child edge enumeration for builtin containers.
//! This is a non-invasive helper to support diagnostics and future collectors.

use std::sync::Arc;

use crate::box_trait::NyashBox;

/// Visit child boxes of a given object and invoke `visit(child)` for each.
/// This function recognizes builtin containers (ArrayBox/MapBox) and is a no-op otherwise.
pub fn trace_children(obj: &dyn NyashBox, _visit: &mut dyn FnMut(Arc<dyn NyashBox>)) {
    // ArrayBox
    #[cfg(feature = "legacy-boxes")]
    if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
        if let Ok(items) = arr.items.read() {
            for it in items.iter() {
                // Convert Box<dyn NyashBox> to Arc<dyn NyashBox>
                let arc: Arc<dyn NyashBox> = Arc::from(it.clone_box());
                visit(arc);
            }
        }
        return;
    }
    #[cfg(not(feature = "legacy-boxes"))]
    if obj.type_name() == "ArrayBox" {
        // Plugin-only: internal iteration is opaque; skip.
        return;
    }
    // MapBox
    #[cfg(feature = "legacy-boxes")]
    if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
        if let Ok(data) = map.get_data().read() {
            for (_k, v) in data.iter() {
                let arc: Arc<dyn NyashBox> = Arc::from(v.clone_box());
                visit(arc);
            }
        }
        return;
    }
    #[cfg(not(feature = "legacy-boxes"))]
    if obj.type_name() == "MapBox" {
        // Plugin-only: internal iteration is opaque; skip.
        return;
    }
}
