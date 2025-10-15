//! ModuleFunction trampolines for core/plugin boxes.
//!
//! These helpers adapt canonical `ModuleFunction("Box.method/arity")` calls
//! into runtime `Method` dispatches, keeping legacy call sites working while
//! Phase‑31 normalizes static boxes to singleton receivers.

use crate::backend::mir_interpreter::{MirInterpreter, VMError, VMValue};
use crate::mir::{self, ValueId};

/// Try dispatching a ModuleFunction call via a trampoline.
/// Returns Some(Result) when handled, otherwise None to continue with the normal path.
pub fn try_module_function_trampoline(
    interp: &mut MirInterpreter,
    class: &str,
    method: &str,
    args: &[ValueId],
) -> Option<Result<VMValue, VMError>> {
    if args.is_empty() {
        return None;
    }
    match class {
        // Core boxes with legacy static-call shims
        "StringBox" => {
            if method == "birth" {
                return None;
            }
            // Early extern alias for length/size/len
            if method == "size" || method == "len" || method == "length" {
                let recv = args[0];
                return Some(interp.handle_callee_extern("nyrt.string.length", &[recv]));
            }
            Some(dispatch_method(interp, class, method, args))
        }
        "ArrayBox" | "MapBox" | "ConsoleBox" => {
            if method == "birth" {
                return None;
            }
            Some(dispatch_method(interp, class, method, args))
        }
        _ => None,
    }
}

fn dispatch_method(
    interp: &mut MirInterpreter,
    class_hint: &str,
    method: &str,
    args: &[ValueId],
) -> Result<VMValue, VMError> {
    let recv = args[0];
    let rest: Vec<ValueId> = args.iter().copied().skip(1).collect();
    let recv_val = interp.reg_load(recv)?;
    let box_name = match &recv_val {
        VMValue::String(_) if class_hint == "StringBox" => "StringBox".to_string(),
        VMValue::BoxRef(bx) => {
            if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                inst.class_name.clone()
            } else {
                bx.type_name().to_string()
            }
        }
        _ => class_hint.to_string(),
    };
    let callee = mir::Callee::Method {
        box_name,
        method: method.to_string(),
        receiver: Some(recv),
        certainty: mir::definitions::call_unified::TypeCertainty::Known,
    };
    interp.execute_callee_call(&callee, &rest)
}

