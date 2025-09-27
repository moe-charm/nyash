/*!
 * Method router for MirInterpreter — centralized cross-class reroute and
 * narrow special-method fallbacks. Phase 1: minimal extraction from exec.rs
 * to keep behavior unchanged while making execution flow easier to reason about.
 */

use super::{MirFunction, MirInterpreter};
use crate::backend::vm::{VMError, VMValue};

#[derive(Debug, Clone)]
struct ParsedSig<'a> {
    class: &'a str,
    method: &'a str,
    arity_str: &'a str,
}

fn parse_method_signature(name: &str) -> Option<ParsedSig<'_>> {
    let dot = name.find('.')?;
    let slash = name.rfind('/')?;
    if dot >= slash { return None; }
    let class = &name[..dot];
    let method = &name[dot + 1..slash];
    let arity_str = &name[slash + 1..];
    Some(ParsedSig { class, method, arity_str })
}

fn extract_instance_box_class(arg0: &VMValue) -> Option<String> {
    if let VMValue::BoxRef(bx) = arg0 {
        if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
            return Some(inst.class_name.clone());
        }
    }
    None
}

fn reroute_to_correct_method(
    interp: &mut MirInterpreter,
    recv_cls: &str,
    parsed: &ParsedSig<'_>,
    arg_vals: Option<&[VMValue]>,
) -> Option<Result<VMValue, VMError>> {
    let target = format!("{}.{}{}", recv_cls, parsed.method, format!("/{}", parsed.arity_str));
    if let Some(f) = interp.functions.get(&target).cloned() {
        return Some(interp.exec_function_inner(&f, arg_vals));
    }
    None
}

/// Try mapping special methods to canonical targets (table-driven).
/// Example: toString/0 → stringify/0 (prefer instance class, then base class without "Instance" suffix).
fn try_special_reroute(
    interp: &mut MirInterpreter,
    recv_cls: &str,
    parsed: &ParsedSig<'_>,
    arg_vals: Option<&[VMValue]>,
) -> Option<Result<VMValue, VMError>> {
    // toString → stringify
    if parsed.method == "toString" && parsed.arity_str == "0" {
        // Prefer instance class stringify first, then base (strip trailing "Instance")
        let base = recv_cls.strip_suffix("Instance").unwrap_or(recv_cls);
        let candidates = [
            format!("{}.stringify/0", recv_cls),
            format!("{}.stringify/0", base),
        ];
        for name in candidates.iter() {
            if let Some(f) = interp.functions.get(name).cloned() {
                return Some(interp.exec_function_inner(&f, arg_vals));
            }
        }
    }

    // equals passthrough (instance/base)
    // In some user setups, only base class provides equals(other).
    // Try instance first, then base (strip trailing "Instance").
    if parsed.method == "equals" && parsed.arity_str == "1" {
        let base = recv_cls.strip_suffix("Instance").unwrap_or(recv_cls);
        let candidates = [
            format!("{}.equals/1", recv_cls),
            format!("{}.equals/1", base),
        ];
        for name in candidates.iter() {
            if let Some(f) = interp.functions.get(name).cloned() {
                return Some(interp.exec_function_inner(&f, arg_vals));
            }
        }
    }
    None
}

fn try_special_method(
    recv_cls: &str,
    parsed: &ParsedSig<'_>,
    arg_vals: Option<&[VMValue]>,
) -> Option<Result<VMValue, VMError>> {
    // Keep narrow fallbacks minimal, deterministic, and cheap.
    if parsed.method == "is_eof" && parsed.arity_str == "0" {
        if let Some(args) = arg_vals {
            if let VMValue::BoxRef(bx) = &args[0] {
                if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    if recv_cls == "JsonToken" {
                        let is = match inst.get_field_ng("type") {
                            Some(crate::value::NyashValue::String(ref s)) => s == "EOF",
                            _ => false,
                        };
                        return Some(Ok(VMValue::Bool(is)));
                    }
                    if recv_cls == "JsonScanner" {
                        let pos = match inst.get_field_ng("position") { Some(crate::value::NyashValue::Integer(i)) => i, _ => 0 };
                        let len = match inst.get_field_ng("length")   { Some(crate::value::NyashValue::Integer(i)) => i, _ => 0 };
                        return Some(Ok(VMValue::Bool(pos >= len)));
                    }
                }
            }
        }
    }
    None
}

/// Pre-execution reroute/short-circuit.
///
/// When a direct Call to "Class.method/N" is about to execute, verify that the
/// first argument ('me') actually belongs to the same InstanceBox class. If it
/// does not, try rerouting to the matching class method. If no matching method
/// exists, apply a very narrow fallback for well-known methods (dev-oriented,
/// but safe and deterministic) and return a value. Returning Some(Result<..>)
/// indicates that the router handled the call (rerouted or short-circuited).
/// Returning None means normal execution should continue.
pub(super) fn pre_exec_reroute(
    interp: &mut MirInterpreter,
    func: &MirFunction,
    arg_vals: Option<&[VMValue]>,
) -> Option<Result<VMValue, VMError>> {
    let args = match arg_vals { Some(a) => a, None => return None };
    if args.is_empty() { return None; }
    let parsed = match parse_method_signature(func.signature.name.as_str()) { Some(p) => p, None => return None };
    let recv_cls = match extract_instance_box_class(&args[0]) { Some(c) => c, None => return None };
    // Always consider special re-routes (e.g., toString→stringify) even when class matches
    if let Some(r) = try_special_reroute(interp, &recv_cls, &parsed, arg_vals) { return Some(r); }
    if recv_cls == parsed.class { return None; }
    // Class mismatch: reroute to same method on the receiver's class
    if let Some(r) = reroute_to_correct_method(interp, &recv_cls, &parsed, arg_vals) { return Some(r); }
    // Narrow special fallback (e.g., is_eof)
    if let Some(r) = try_special_method(&recv_cls, &parsed, arg_vals) { return Some(r); }
    None
}
