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

    // Primitive String — table-driven via TypeRegistry slots
    if let VMValue::String(s) = receiver {
        let _ = maybe_arity_guard("StringBox", method, args.len());
        if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("StringBox", method, args.len()) {
            let res = match slot as u32 {
                300 => {
                    // Dev/test: optionally force HostHandleRouter path for String.len/size
                    if std::env::var("NYASH_STRING_SIZE_FORCE_HOST").ok().as_deref() == Some("1") {
                        let sb = Box::new(crate::box_trait::StringBox::new(s.clone())) as Box<dyn crate::box_trait::NyashBox>;
                        let hh = crate::runtime::host_handles::to_handle_box(sb);
                        let mut out_len: usize = 64;
                        let mut out_buf = vec![0u8; out_len];
                        let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 300, std::ptr::null(), 0, out_buf.as_mut_ptr(), &mut out_len);
                        if rc == 0 {
                            if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                                if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(v); }
                            }
                        }
                        // Fallback to builtin on decode/rc failure
                    }
                    Ok(VMValue::Integer(hako_core_string::length_bytes(s)))
                },
                301 => { // substring(start,end)
                    let start = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    let end = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(VMValue::String(hako_core_string::substring_bytes(s, start, end)))
                }
                303 => { // indexOf(needle[, from])
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(VMValue::Integer(hako_core_string::index_of(s, &needle, from)))
                }
                313 => { // lastIndexOf(needle[, from])
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(VMValue::Integer(hako_core_string::last_index_of(s, &needle, from)))
                }
                314 => { // charAt(idx)
                    let idx = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(VMValue::String(hako_core_string::char_at_byte(s, idx)))
                }

                302 => { // concat(rhs)
                    let rhs = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let mut out = String::with_capacity(s.len() + rhs.len());
                    out.push_str(s);
                    out.push_str(&rhs);
                    Ok(VMValue::String(out))
                }
                304 => { // replace(from, to)
                    let from = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let to = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::String(hako_core_string::replace_all(s, &from, &to)))
                }
                305 => { // trim()
                    Ok(VMValue::String(s.trim().to_string()))
                }
                306 => { // toUpper()
                    Ok(VMValue::String(s.to_uppercase()))
                }
                307 => { // toLower()
                    Ok(VMValue::String(s.to_lowercase()))
                }
                308 => { // toString()
                    Ok(VMValue::String(s.clone()))
                }
                309 => { // stringify() — keep identity for now (JSON.stringify exists as module)
                    Ok(VMValue::String(s.clone()))
                }
                310 => { // startsWith(needle)
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::Bool(s.starts_with(&needle)))
                }
                311 => { // endsWith(needle)
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::Bool(s.ends_with(&needle)))
                }
                312 => Ok(VMValue::Bool(hako_core_string::is_empty(s))),
                _ => Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::unknown_slot("StringBox", method, slot))),
            };
            return res;
        } else {
            return Err(crate::vm_ops::boxcall::unknown_method_err("StringBox", method, args.len()));
        }
    }
    // BoxRef
    if let VMValue::BoxRef(bx) = receiver {
        // Phase 15.75 split（delegated routing only）
        if let Some(res) = plugin::try_route_plugin_box(_interp, bx, method, args)? { return Ok(res); }
        if let Some(res) = builtin::try_route_builtin_box(_interp, bx, method, args)? { return Ok(res); }
        // Any remaining BoxRef is unsupported here
        return Err(boxcall::method_not_supported(method, receiver));
    }
    Err(boxcall::method_not_supported(method, receiver))
}
