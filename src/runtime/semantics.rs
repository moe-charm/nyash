/*!
 * Shared semantics for coercions and basic ops
 *
 * Goal: Unify Script→MIR→VM→AOT semantics by centralizing
 * string/number coercions and common operation ordering.
 */

use crate::box_trait::{IntegerBox, NyashBox, StringBox};

/// Try to unwrap InstanceBox and return inner if present
fn maybe_unwrap_instance(b: &dyn NyashBox) -> &dyn NyashBox {
    if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
        if let Some(ref inner) = inst.inner_content {
            return inner.as_ref();
        }
    }
    b
}

/// Result.Ok(inner) → recurse helper
fn maybe_unwrap_result_ok(b: &dyn NyashBox) -> &dyn NyashBox {
    if let Some(res) = b
        .as_any()
        .downcast_ref::<crate::boxes::result::NyashResultBox>()
    {
        if let crate::boxes::result::NyashResultBox::Ok(inner) = res {
            return inner.as_ref();
        }
    }
    b
}

/// Best-effort string coercion used by all backends.
pub fn coerce_to_string(b: &dyn NyashBox) -> Option<String> {
    let b = maybe_unwrap_instance(b);
    // Internal StringBox
    if let Some(s) = b.as_any().downcast_ref::<StringBox>() {
        return Some(s.value.clone());
    }
    // Result.Ok recursion
    let b2 = maybe_unwrap_result_ok(b);
    if !std::ptr::eq(b2 as *const _, b as *const _) {
        if let Some(s) = coerce_to_string(b2) {
            return Some(s);
        }
    }

    // Plugin StringBox: prefer toUtf8; fallback to toString
    if let Some(pb) = b
        .as_any()
        .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
    {
        // StringBox.toUtf8
        if pb.box_type == "StringBox" {
            let host = crate::runtime::get_global_plugin_host();
            let read_res = host.read();
            if let Ok(ro) = read_res {
                if let Ok(ret) =
                    ro.invoke_instance_method("StringBox", "toUtf8", pb.instance_id(), &[])
                {
                    if let Some(vb) = ret {
                        if let Some(sb2) = vb.as_any().downcast_ref::<StringBox>() {
                            return Some(sb2.value.clone());
                        }
                    }
                }
            }
        }
        // AnyBox.toString
        let host = crate::runtime::get_global_plugin_host();
        let read_res = host.read();
        if let Ok(ro) = read_res {
            if let Ok(ret) =
                ro.invoke_instance_method(&pb.box_type, "toString", pb.instance_id(), &[])
            {
                if let Some(vb) = ret {
                    if let Some(s) = coerce_to_string(vb.as_ref()) {
                        return Some(s);
                    }
                }
            }
        }
    }
    None
}

/// Best-effort integer coercion used by all backends.
pub fn coerce_to_i64(b: &dyn NyashBox) -> Option<i64> {
    let b = maybe_unwrap_instance(b);
    if let Some(i) = b.as_any().downcast_ref::<IntegerBox>() {
        return Some(i.value);
    }

    // Plugin numeric getters
    if let Some(pb) = b
        .as_any()
        .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
    {
        // IntegerBox.get -> IntegerBox
        if pb.box_type == "IntegerBox" {
            let host = crate::runtime::get_global_plugin_host();
            let read_res = host.read();
            if let Ok(ro) = read_res {
                if let Ok(ret) =
                    ro.invoke_instance_method("IntegerBox", "get", pb.instance_id(), &[])
                {
                    if let Some(vb) = ret {
                        if let Some(ii) = vb.as_any().downcast_ref::<IntegerBox>() {
                            return Some(ii.value);
                        }
                    }
                }
            }
        }
        // FloatBox.toDouble -> FloatBox
        if pb.box_type == "FloatBox" {
            let host = crate::runtime::get_global_plugin_host();
            let read_res = host.read();
            if let Ok(ro) = read_res {
                if let Ok(ret) =
                    ro.invoke_instance_method("FloatBox", "toDouble", pb.instance_id(), &[])
                {
                    if let Some(vb) = ret {
                        if let Some(fb) = vb.as_any().downcast_ref::<crate::boxes::FloatBox>() {
                            return Some(fb.value as i64);
                        }
                    }
                }
            }
        }
    }

    // Fallback via string coercion -> parse
    if let Some(s) = coerce_to_string(b) {
        return s.trim().parse::<i64>().ok();
    }
    None
}
