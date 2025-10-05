/*!
 * Unified Call System
 *
 * ChatGPT5 Pro A++ design for complete call unification
 * Replaces 6 different call instructions with a single unified system
 */

use crate::mir::{Callee, Effect, EffectMask, ValueId};
use crate::mir::definitions::call_unified::{CallFlags, MirCall};
use super::call_target::CallTarget;
use super::method_resolution;
use super::extern_calls;

/// Check if unified call system is enabled
pub fn is_unified_call_enabled() -> bool {
    match std::env::var("NYASH_MIR_UNIFIED_CALL").ok().as_deref().map(|s| s.to_ascii_lowercase()) {
        Some(s) if s == "1" || s == "true" || s == "on" => true,
        _ => false, // default OFF (Unified Call has bugs with recursion - see tmp/bench_results/CRITICAL_NEW_DEFAULTS_BROKEN_2025-10-05.md)
    }
}

/// Convert CallTarget to Callee
/// Main translation layer between builder and MIR representations
pub fn convert_target_to_callee<OriginLookup>(
    target: CallTarget,
    origin_lookup: OriginLookup,
    value_types: &std::collections::HashMap<ValueId, crate::mir::MirType>,
) -> Result<Callee, String>
where
    OriginLookup: Fn(ValueId) -> Option<String>,
{
    match target {
        CallTarget::Global(name) => {
            // Check if it's a built-in function
            if method_resolution::is_builtin_function(&name) {
                Ok(Callee::Global(name))
            } else if method_resolution::is_extern_function(&name) {
                Ok(Callee::Extern(name))
            } else {
                Err(format!("Unknown global function: {}", name))
            }
        },

        CallTarget::Method { box_type, method, receiver } => {
            let (box_name, certainty) = crate::mir::builder::infer::receiver::infer_receiver(
                box_type.as_deref(),
                method.as_str(),
                receiver,
                |vid| origin_lookup(vid),
                value_types,
            );
            Ok(Callee::Method { box_name, method, receiver: Some(receiver), certainty })
        },

        CallTarget::Constructor(box_type) => {
            Ok(Callee::Constructor { box_type })
        },

        CallTarget::Extern(name) => {
            Ok(Callee::Extern(name))
        },

        CallTarget::Value(func_val) => {
            Ok(Callee::Value(func_val))
        },

        CallTarget::Closure { params, captures, me_capture } => {
            Ok(Callee::Closure { params, captures, me_capture })
        },
    }
}

/// Compute effects for a call based on its callee
pub fn compute_call_effects(callee: &Callee) -> EffectMask {
    // Phase‑in: try resolver when enabled; fallback to legacy logic
    if let Some(eff) = crate::mir::builder::effects::resolve_effects_for_callee(callee) {
        return eff;
    }
    match callee {
        Callee::Global(name) => {
            match name.as_str() {
                "print" | "error" => EffectMask::IO,
                "panic" | "exit" => EffectMask::IO.add(Effect::Control),
                "gc_collect" => EffectMask::IO.add(Effect::Alloc),
                _ => EffectMask::IO,
            }
        },
        Callee::ModuleFunction(_name) => {
            // Conservative default for user/module functions
            // Assume they may read heap; refined by future inference
            EffectMask::READ.add(Effect::ReadHeap)
        },

        Callee::Method { method, box_name, .. } => {
            match method.as_str() {
                "birth" => EffectMask::PURE.add(Effect::Alloc),
                "get" | "length" | "size" => EffectMask::READ,
                "set" | "push" | "pop" => EffectMask::READ.add(Effect::WriteHeap),
                _ => {
                    // Check if it's a known pure method
                    if is_pure_method(box_name, method) {
                        EffectMask::PURE
                    } else {
                        EffectMask::READ
                    }
                }
            }
        },

        Callee::Constructor { .. } => EffectMask::PURE.add(Effect::Alloc),

        Callee::Closure { .. } => EffectMask::PURE.add(Effect::Alloc),

        Callee::Extern(name) => {
            let (iface, method) = extern_calls::parse_extern_name(name);
            extern_calls::compute_extern_effects(&iface, &method)
        },

        Callee::Value(_) => EffectMask::IO, // Conservative for dynamic calls
    }
}

/// Check if a method is known to be pure (no side effects)
fn is_pure_method(box_name: &str, method: &str) -> bool {
    match (box_name, method) {
        ("StringBox", m) => matches!(m, "upper" | "lower" | "trim" | "length"),
        ("IntegerBox", m) => matches!(m, "abs" | "toString"),
        ("FloatBox", m) => matches!(m, "round" | "floor" | "ceil"),
        ("BoolBox", "not") => true,
        _ => false,
    }
}

/// Create CallFlags based on callee type
pub fn create_call_flags(callee: &Callee) -> CallFlags {
    if callee.is_constructor() {
        CallFlags::constructor()
    } else if matches!(callee, Callee::Closure { .. }) {
        CallFlags::constructor() // Closures are also constructors
    } else {
        CallFlags::new()
    }
}

/// Create a MirCall instruction from components
pub fn create_mir_call(
    dst: Option<ValueId>,
    callee: Callee,
    args: Vec<ValueId>,
) -> MirCall {
    let effects = compute_call_effects(&callee);
    let flags = create_call_flags(&callee);

    MirCall {
        dst,
        callee,
        args,
        flags,
        effects,
    }
}

/// Validate call arguments match expected signature
pub fn validate_call_args(
    callee: &Callee,
    args: &[ValueId],
) -> Result<(), String> {
    match callee {
        Callee::Global(name) => {
            // Check known global functions
            match name.as_str() {
                "print" | "error" | "panic" => {
                    if args.is_empty() {
                        return Err(format!("{} requires at least one argument", name));
                    }
                }
                "exit" => {
                    if args.len() != 1 {
                        return Err("exit requires exactly one argument (exit code)".to_string());
                    }
                }
                _ => {} // Unknown functions pass through
            }
        },

        Callee::Method { box_name, method, .. } => {
            // Validate known methods
            match (box_name.as_str(), method.as_str()) {
                ("ArrayBox", "get") | ("ArrayBox", "set") => {
                    if args.is_empty() {
                        return Err(format!("ArrayBox.{} requires an index", method));
                    }
                }
                _ => {} // Unknown methods pass through
            }
        },

        _ => {} // Other callee types don't have validation yet
    }

    Ok(())
}
