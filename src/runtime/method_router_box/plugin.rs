//! plugin.rs — Plugin(TypeBox v2) routing adapter (Phase 15.75 split scaffold)
use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;
use super::map_callable::MapCallableBox;
use super::tables::{ARRAY_HOST_ROUTES, MAP_HOST_ROUTES, STRING_HOST_ROUTES};

fn plugin_normalize_enabled() -> bool {
    crate::runtime::env_gate_box::bool_alias_or("HAKO_PLUGIN_NORMALIZE", "NYASH_PLUGIN_NORMALIZE", false)
}

fn normalize_plugin_value(v: VMValue) -> VMValue {
    if !plugin_normalize_enabled() { return v; }
    match v {
        // Normalization hook (phase-in): keep Void stable, unwrap VoidBox if any leaked
        VMValue::BoxRef(ref arc) => {
            if arc.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                VMValue::Void
            } else {
                v
            }
        }
        _ => v,
    }
}

/// Try routing a PluginBoxV2 receiver.
/// Phase 0-mini: scaffold only. Keep behavior in mod.rs for now.
/// Returns Ok(None) when not handled.
pub fn try_route_plugin_box(
    _interp: &mut MirInterpreter,
    bx: &std::sync::Arc<dyn NyashBox>,
    method: &str,
    args: &[VMValue],
) -> Result<Option<VMValue>, VMError> {
    // Plugin TypeBox v2
    if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
        // StringBox — early host path for len/size/length (dev/test)
        if p.box_type == "StringBox" {
            if let Some(route) = STRING_HOST_ROUTES.pick(method, args.len()) {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                if let Some(v) = super::host_slot::invoke(hh, route, args) {
                    return Ok(Some(normalize_plugin_value(v)));
                }
            }
        }
        // SetBox — route to unified extern path to avoid path ambiguity
        if p.box_type == "SetBox" {
            if crate::runtime::env_gate_box::bool_any(&["NYASH_DEBUG_SET_ROUTER"]) {
                eprintln!("[router] SetBox.{} arity={} via extern", method, args.len());
            }
            let iface = "nyrt.set";
            let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
            argv.push(VMValue::BoxRef(bx.clone()));
            match method {
                "size" | "clear" | "toArray" if args.is_empty() => {
                    if let Some(res) = _interp.extern_call_public(iface, method, &argv) { return res.map(Some); }
                }
                "add" | "remove" | "has" if args.len() == 1 => {
                    argv.push(args[0].clone());
                    if let Some(res) = _interp.extern_call_public(iface, method, &argv) { return res.map(Some); }
                }
                _ => {}
            }
            // Fallback: plugin host invocation (kept for compatibility)
            let mut argv_boxes: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for v in args { argv_boxes.push(v.to_nyash_box()); }
            let out = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", method, p.inner.instance_id, &argv_boxes);
            return match out {
                Ok(Some(ret)) => Ok(Some(VMValue::from_nyash_box(ret))),
                Ok(None) => Ok(Some(VMValue::Void)),
                Err(e) => Err(VMError::InvalidInstruction(format!("Plugin method SetBox.{} failed: {:?}", method, e))),
            };
        }

        // Central arity guard (vm_ops/boxcall)
        crate::vm_ops::boxcall::arity_guard_for(&p.box_type, method, args.len())?;
        if let Some(result) = MapCallableBox::try_route(_interp, &VMValue::BoxRef(bx.clone()), method, args) {
            return result.map(Some);
        }
        // Dev/test: optionally force HostHandleRouter path for Map.size/has/get/set (table-driven)
        if p.box_type == "MapBox" {
            if let Some(route) = MAP_HOST_ROUTES.pick(method, args.len()) {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                if let Some(v) = super::host_slot::invoke(hh, route, args) {
                    return Ok(Some(if route.returns_value {
                        normalize_plugin_value(v)
                    } else {
                        v
                    }));
                }
            }
        }

        // Stage-1 fallback STUB: convert VMValue args to NyashBox (core types→HostHandle)
        let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for v in args {
            if let VMValue::BoxRef(bx) = v {
                if crate::runtime::type_registry::is_core_box(bx.type_name()) {
                    let h = crate::runtime::host_handles::to_handle_arc(bx.clone());
                    argv.push(Box::new(crate::runtime::host_handle_box::HostHandleBox::new(h)));
                    continue;
                }
            }
            argv.push(v.to_nyash_box());
        }

        // Optional fast path: ArrayBox.size via slot 102
        if p.box_type == "ArrayBox" && (method == "size" || method == "len" || method == "length")
        {
            if let Some(route) = ARRAY_HOST_ROUTES.pick(method, args.len()) {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                if let Some(v) = super::host_slot::invoke(hh, route, args) {
                    return Ok(Some(normalize_plugin_value(v)));
                }
            }
        }

        // (duplicate Map.* fallback removed; early-host branches above cover size/has/get/set)

        // Delegate to plugin host
        let out = crate::runtime::plugin_host_box::invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv);
        return match out {
            Ok(Some(ret)) => Ok(Some(normalize_plugin_value(VMValue::from_nyash_box(ret)))),
            Ok(None) => Ok(Some(VMValue::Void)),
            Err(e) => Err(VMError::InvalidInstruction(format!("Plugin method {}.{} failed: {:?}", p.box_type, method, e))),
        };
    }
    Ok(None)
}
