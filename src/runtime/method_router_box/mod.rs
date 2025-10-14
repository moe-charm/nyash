//! MethodRouterBox — Single entry to route method calls
//!
//! Phase‑1 implementation:
//! - Resolver: classify receiver into builtin/core vs plugin box
//! - Invoker:
//!   - BuiltinInvoker → delegate to core semantics (String/Array/Map)
//!   - PluginInvoker  → invoke v2 TypeBox FFI through plugin host

mod map_callable;
mod method_ref;
mod plugin;
mod builtin;

use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::vm_ops::boxcall;
use method_ref::MethodRefBox;


#[inline]
fn maybe_arity_guard(type_name: &str, method: &str, arity: usize) -> Result<(), crate::backend::vm_types::VMError> {
    if method == "birth" { return Ok(()); }
    if crate::runtime::type_registry::resolve_typebox_by_name(type_name).is_some() {
        if crate::runtime::type_registry::resolve_slot_by_name(type_name, method, arity).is_none() {
            if let Some(known) = crate::runtime::type_registry::known_arities_for(type_name, method) {
                if !known.is_empty() {
                    return Err(crate::backend::vm_types::VMError::InvalidInstruction(
                        crate::common::diagnostics::msg::no_method_arity_short(type_name, method, arity),
                    ));
                }
            }
        }
    }
    Ok(())
}
pub fn route(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    if let Some(result) = MethodRefBox::try_route(receiver, method, args) {
        return result;
    }

    if let VMValue::BoxRef(bx) = receiver {
        if let Some(hh) = bx.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
            if let Some(real) = crate::runtime::host_handles::get(hh.id) {
                let real_vm = VMValue::BoxRef(real.clone());
                return route(_interp, &real_vm, method, args);
            } else {
                return Err(VMError::InvalidInstruction(format!("Unknown HostHandle:{}", hh.id)));
            }
        }
    }

    // Primitive String → delegate to builtin handler
    if let Some(res) = builtin::try_route_string_primitive(_interp, receiver, method, args)? { return Ok(res); }
    // BoxRef
    if let VMValue::BoxRef(bx) = receiver {
        // Phase 15.75 split（delegated routing only）
        if let Some(res) = plugin::try_route_plugin_box(_interp, bx, method, args)? { return Ok(res); }

        // Plugin strict policy: forbid builtin fallback when policy=force and plugins are ON
        // If a plugin provider exists for this box type (per registry), but the receiver is not a PluginBoxV2
        // or plugin routing did not handle it, fail fast instead of delegating to builtin.
        {
            let policy = crate::runtime::env_gate_box::string_alias_or(
                "NYASH_PLUGIN_POLICY", "HAKO_PLUGIN_POLICY", "auto",
            );
            let strict = policy.to_ascii_lowercase() == "force";
            let plugin_on = crate::runtime::env_gate_box::plugin_policy_on()
                && !crate::runtime::env_gate_box::plugins_disabled();
            if strict && plugin_on {
                let reg = crate::runtime::get_global_registry();
                if let Some(p) = reg.get_provider(bx.type_name()) {
                    if matches!(p, crate::runtime::box_registry::BoxProvider::Plugin(_)) {
                        return Err(VMError::InvalidInstruction(format!(
                            "plugin strict: builtin fallback disabled for {}.{}({} args)",
                            bx.type_name(), method, args.len()
                        )));
                    }
                }
            }
        }

        if let Some(res) = builtin::try_route_builtin_box(_interp, bx, method, args)? { return Ok(res); }
        // Any remaining BoxRef is unsupported here
        return Err(boxcall::method_not_supported(method, receiver));
    }
    Err(boxcall::method_not_supported(method, receiver))
}
