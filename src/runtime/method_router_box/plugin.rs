//! plugin.rs — Plugin(TypeBox v2) routing adapter (Phase 15.75 split scaffold)
use super::map_callable::MapCallableBox;
use super::tables::{ARRAY_HOST_ROUTES, MAP_HOST_ROUTES, STRING_HOST_ROUTES};
use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;

fn plugin_normalize_enabled() -> bool {
    crate::runtime::env_gate_box::bool_alias_or(
        "HAKO_PLUGIN_NORMALIZE",
        "NYASH_PLUGIN_NORMALIZE",
        false,
    )
}

fn normalize_plugin_value(v: VMValue) -> VMValue {
    if !plugin_normalize_enabled() {
        return v;
    }
    match v {
        // Normalization hook (phase-in): keep Void stable, unwrap VoidBox if any leaked
        VMValue::BoxRef(ref arc) => {
            if arc
                .as_any()
                .downcast_ref::<crate::box_trait::VoidBox>()
                .is_some()
            {
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
    if let Some(p) = bx
        .as_any()
        .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
    {
        let method_ref: &str = method;
        let args_ref: &[VMValue] = args;
        // StringBox — early host path for len/size/length (dev/test)
        if p.box_type == "StringBox" {
            if crate::runtime::env_gate_box::bool_any(&[
                "HAKO_HOSTHANDLE_TEST_RET_MISMATCH",
                "NYASH_HOSTHANDLE_TEST_RET_MISMATCH",
            ]) && method_ref == "size" && args_ref.is_empty() {
                return Ok(Some(VMValue::Integer(
                    crate::runtime::host_handle_router::consts::ERR_BAD_RETURN as i64,
                )));
            }
            if let Some((_route, value)) =
                super::host_slot::try_invoke_arc(bx, STRING_HOST_ROUTES, method_ref, args_ref)
            {
                if crate::runtime::env_gate_box::debug_host_slot() {
                    eprintln!(
                        "[debug:plugin host] {}.{} -> {:?}",
                        p.box_type, method_ref, value
                    );
                }
                return Ok(Some(normalize_plugin_value(value)));
            }
        }
        // SetBox — route to unified extern pathを使用（早期に arity ガードを適用）
        if p.box_type == "SetBox" {
            // Manual guard until SetBox metadata lands in TypeRegistry.
            let expected = match method_ref {
                "size" | "clear" | "toArray" => Some(0),
                "add" | "remove" | "has" => Some(1),
                _ => None,
            };
            if let Some(exp) = expected {
                if args_ref.len() != exp {
                    return Err(crate::backend::vm_types::VMError::InvalidInstruction(
                        crate::common::diagnostics::msg::no_method_arity("SetBox", method_ref, args_ref.len(), &[exp]),
                    ));
                }
            }
            // Centralized arity guard for SetBox (fail-fast for bad arity)
            crate::vm_ops::boxcall::arity_guard_for("SetBox", method_ref, args_ref.len())?;
            if crate::runtime::env_gate_box::bool_any(&["NYASH_DEBUG_SET_ROUTER"]) {
                eprintln!("[router] SetBox.{} arity={} via extern", method, args.len());
            }
            let iface = "nyrt.set";
            let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
            argv.push(VMValue::BoxRef(bx.clone()));
            match method_ref {
                "size" | "clear" | "toArray" if args.is_empty() => {
                    if let Some(res) = _interp.extern_call_public(iface, method, &argv) {
                        return res.map(Some);
                    }
                }
                "add" | "remove" | "has" if args_ref.len() == 1 => {
                    argv.push(args_ref[0].clone());
                    if let Some(res) = _interp.extern_call_public(iface, method, &argv) {
                        return res.map(Some);
                    }
                }
                _ => {}
            }
            // Fallback: plugin host invocation (kept for compatibility)
            let mut argv_boxes: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for v in args {
                argv_boxes.push(v.to_nyash_box());
            }
            let out = crate::runtime::plugin_host_box::invoke_instance_method(
                "SetBox",
                method,
                p.inner.instance_id,
                &argv_boxes,
            );
            return match out {
                Ok(Some(ret)) => Ok(Some(VMValue::from_nyash_box(ret))),
                Ok(None) => Ok(Some(VMValue::Void)),
                Err(e) => Err(VMError::InvalidInstruction(format!(
                    "Plugin method SetBox.{} failed: {:?}",
                    method, e
                ))),
            };
        }

        // Central arity guard (vm_ops/boxcall)
        crate::vm_ops::boxcall::arity_guard_for(&p.box_type, method_ref, args_ref.len())?;
        if let Some(result) =
            MapCallableBox::try_route(_interp, &VMValue::BoxRef(bx.clone()), method_ref, args_ref)
        {
            return result.map(Some);
        }
        // Dev/test: optionally force HostHandleRouter path for Map.size/has/get/set (table-driven)
        if p.box_type == "MapBox" {
            if let Some((route, value)) =
                super::host_slot::try_invoke_arc(bx, MAP_HOST_ROUTES, method_ref, args_ref)
            {
                if crate::runtime::env_gate_box::debug_host_slot() {
                    eprintln!(
                        "[debug:plugin host] {}.{} -> {:?}",
                        p.box_type, method_ref, value
                    );
                }
                let out = if route.returns_value {
                    normalize_plugin_value(value)
                } else {
                    value
                };
                return Ok(Some(out));
            }
        }

        // Stage-1 fallback STUB: convert VMValue args to NyashBox (core types→HostHandle)
        let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for v in args {
            if let VMValue::BoxRef(bx) = v {
                if crate::runtime::type_registry::is_core_box(bx.type_name()) {
                    let h = crate::runtime::host_handles::to_handle_arc(bx.clone());
                    argv.push(Box::new(
                        crate::runtime::host_handle_box::HostHandleBox::new(h),
                    ));
                    continue;
                }
            }
            argv.push(v.to_nyash_box());
        }

        if p.box_type == "ArrayBox" {
            if let Some((route, value)) =
                super::host_slot::try_invoke_arc(bx, ARRAY_HOST_ROUTES, method_ref, args_ref)
            {
                if crate::runtime::env_gate_box::debug_host_slot() {
                    eprintln!(
                        "[debug:plugin host] {}.{} -> {:?}",
                        p.box_type, method_ref, value
                    );
                }
                let out = if route.returns_value {
                    normalize_plugin_value(value)
                } else {
                    value
                };
                return Ok(Some(out));
            }
        }

        // Delegate to plugin host
        let out = crate::runtime::plugin_host_box::invoke_instance_method(&p.box_type, method_ref, p.inner.instance_id, &argv);
        return match out {
            Ok(Some(ret)) => Ok(Some(normalize_plugin_value(VMValue::from_nyash_box(ret)))),
            Ok(None) => Ok(Some(VMValue::Void)),
            Err(e) => Err(VMError::InvalidInstruction(format!(
                "Plugin method {}.{} failed: {:?}", p.box_type, method_ref, e
            ))),
        };
    }
    Ok(None)
}
