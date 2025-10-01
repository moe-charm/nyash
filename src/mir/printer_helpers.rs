use super::{MirInstruction, MirType, ValueId};
use std::collections::HashMap;

pub fn format_type(mir_type: &MirType) -> String {
    match mir_type {
        MirType::Integer => "i64".to_string(),
        MirType::Float => "f64".to_string(),
        MirType::Bool => "i1".to_string(),
        MirType::String => "str".to_string(),
        MirType::Box(name) => format!("box<{}>", name),
        MirType::Array(elem_type) => format!("[{}]", format_type(elem_type)),
        MirType::Future(inner_type) => {
            format!("future<{}>", format_type(inner_type))
        }
        MirType::Void => "void".to_string(),
        MirType::Unknown => "?".to_string(),
    }
}

pub fn format_dst(dst: &ValueId, types: &HashMap<ValueId, MirType>) -> String {
    if let Some(ty) = types.get(dst) {
        format!("{}: {:?} =", dst, ty)
    } else {
        format!("{} =", dst)
    }
}

pub fn format_instruction(
    instruction: &MirInstruction,
    types: &HashMap<ValueId, MirType>,
) -> String {
    match instruction {
        MirInstruction::Const { dst, value } => {
            format!("{} const {}", format_dst(dst, types), value)
        }

        MirInstruction::BinOp { dst, op, lhs, rhs } => {
            format!("{} {} {:?} {}", format_dst(dst, types), lhs, op, rhs)
        }

        MirInstruction::UnaryOp { dst, op, operand } => {
            format!("{} {:?} {}", format_dst(dst, types), op, operand)
        }

        MirInstruction::Compare { dst, op, lhs, rhs } => {
            format!(
                "{} icmp {:?} {}, {}",
                format_dst(dst, types),
                op,
                lhs,
                rhs
            )
        }

        MirInstruction::Load { dst, ptr } => {
            format!("{} load {}", format_dst(dst, types), ptr)
        }

        MirInstruction::Store { value, ptr } => {
            format!("store {} -> {}", value, ptr)
        }

        MirInstruction::Call {
            dst,
            func,
            callee,
            args,
            effects: _,
        } => {
            let args_str = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");

            // ✅ MIRダンプCallee表示改良 - ChatGPT5 Pro革命！
            let call_display = if let Some(callee_info) = callee {
                match callee_info {
                    super::Callee::Global(name) => {
                        format!("call_global {}({})", name, args_str)
                    }
                    super::Callee::Method { box_name, method, receiver, certainty } => {
                        if let Some(recv) = receiver {
                            format!(
                                "call_method {}.{}({}) [recv: {}] [{}]",
                                box_name,
                                method,
                                args_str,
                                recv,
                                match certainty { crate::mir::definitions::call_unified::TypeCertainty::Known => "Known", crate::mir::definitions::call_unified::TypeCertainty::Union => "Union" }
                            )
                        } else {
                            format!(
                                "call_method {}.{}({}) [{}]",
                                box_name,
                                method,
                                args_str,
                                match certainty { crate::mir::definitions::call_unified::TypeCertainty::Known => "Known", crate::mir::definitions::call_unified::TypeCertainty::Union => "Union" }
                            )
                        }
                    }
                    super::Callee::Constructor { box_type } => {
                        format!("call_constructor {}({})", box_type, args_str)
                    }
                    super::Callee::Closure { params, captures, me_capture } => {
                        let params_str = params.join(", ");
                        let captures_str = captures.iter()
                            .map(|(name, val)| format!("{}={}", name, val))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let me_str = me_capture.map_or(String::new(), |v| format!(" [me={}]", v));
                        format!("call_closure ({}) [captures: {}]{}",
                            params_str, captures_str, me_str)
                    }
                    super::Callee::Value(func_val) => {
                        format!("call_value {}({})", func_val, args_str)
                    }
                    super::Callee::Extern(extern_name) => {
                        format!("call_extern {}({})", extern_name, args_str)
                    }
                    super::Callee::ModuleFunction(name) => {
                        format!("call_module_fn {}({})", name, args_str)
                    }
                }
            } else {
                // LEGACY: 従来の表示（後方互換性）
                format!("call_legacy {}({})", func, args_str)
            };

            if let Some(dst) = dst {
                format!("{} {}", format_dst(dst, types), call_display)
            } else {
                call_display
            }
        }
        MirInstruction::NewClosure {
            dst,
            params,
            body,
            captures,
            me,
        } => {
            let p = params.join(", ");
            let c = captures
                .iter()
                .map(|(n, v)| format!("{}={}", n, v))
                .collect::<Vec<_>>()
                .join(", ");
            let me_s = me.map(|m| format!(" me={}", m)).unwrap_or_default();
            let cap_s = if c.is_empty() { String::new() } else { format!(" [{}]", c) };
            format!(
                "{} new_closure ({}) {{...{}}}{}{}",
                format_dst(dst, types),
                p,
                body.len(),
                cap_s,
                me_s
            )
        }

        MirInstruction::BoxCall {
            dst,
            box_val,
            method,
            method_id,
            args,
            effects: _,
        } => {
            let args_str = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");
            let id_suffix = method_id.map(|id| format!("[#{}]", id)).unwrap_or_default();
            if let Some(dst) = dst {
                format!(
                    "{} call {}.{}{}({})",
                    format_dst(dst, types),
                    box_val,
                    method,
                    id_suffix,
                    args_str
                )
            } else {
                format!("call {}.{}{}({})", box_val, method, id_suffix, args_str)
            }
        }
        MirInstruction::PluginInvoke {
            dst,
            box_val,
            method,
            args,
            effects: _,
        } => {
            let args_str = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(dst) = dst {
                format!(
                    "{} plugin_invoke {}.{}({})",
                    format_dst(dst, types),
                    box_val,
                    method,
                    args_str
                )
            } else {
                format!("plugin_invoke {}.{}({})", box_val, method, args_str)
            }
        }

        MirInstruction::Branch {
            condition,
            then_bb,
            else_bb,
        } => {
            format!("br {}, label {}, label {}", condition, then_bb, else_bb)
        }

        MirInstruction::Jump { target } => {
            format!("br label {}", target)
        }

        MirInstruction::Return { value } => {
            if let Some(value) = value {
                format!("ret {}", value)
            } else {
                "ret void".to_string()
            }
        }

        MirInstruction::Phi { dst, inputs } => {
            let inputs_str = inputs
                .iter()
                .map(|(bb, val)| format!("[{}, {}]", val, bb))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} phi {}", format_dst(dst, types), inputs_str)
        }

        MirInstruction::NewBox { dst, box_type, args } => {
            let args_str = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{} new {}({})",
                format_dst(dst, types),
                box_type,
                args_str
            )
        }

        // Legacy -> Unified print: TypeCheck as TypeOp(check)
        MirInstruction::TypeCheck { dst, value, expected_type } => {
            format!(
                "{} typeop check {} {}",
                format_dst(dst, types),
                value,
                expected_type
            )
        }

        MirInstruction::Cast { dst, value, target_type } => {
            format!(
                "{} cast {} to {:?}",
                format_dst(dst, types),
                value,
                target_type
            )
        }

        MirInstruction::TypeOp { dst, op, value, ty } => {
            let op_str = match op {
                super::TypeOpKind::Check => "check",
                super::TypeOpKind::Cast => "cast",
            };
            format!(
                "{} typeop {} {} {:?}",
                format_dst(dst, types),
                op_str,
                value,
                ty
            )
        }

        MirInstruction::ArrayGet { dst, array, index } => {
            format!("{} {}[{}]", format_dst(dst, types), array, index)
        }

        MirInstruction::ArraySet { array, index, value } => {
            format!("{}[{}] = {}", array, index, value)
        }

        MirInstruction::Copy { dst, src } => {
            format!("{} copy {}", format_dst(dst, types), src)
        }

        MirInstruction::Debug { value, message } => {
            format!("debug {} \"{}\"", value, message)
        }

        MirInstruction::Print { value, effects: _ } => {
            format!("print {}", value)
        }

        MirInstruction::Nop => "nop".to_string(),

        // Phase 5: Control flow & exception handling
        MirInstruction::Throw { exception, effects: _ } => {
            format!("throw {}", exception)
        }

        MirInstruction::Catch { exception_type, exception_value, handler_bb } => {
            if let Some(ref exc_type) = exception_type {
                format!("catch {} {} -> {}", exc_type, exception_value, handler_bb)
            } else {
                format!("catch * {} -> {}", exception_value, handler_bb)
            }
        }

        MirInstruction::Safepoint => "safepoint".to_string(),

        // Phase 6: Box reference operations
        MirInstruction::RefNew { dst, box_val } => {
            format!("{} ref_new {}", format_dst(dst, types), box_val)
        }

        MirInstruction::RefGet { dst, reference, field } => {
            format!(
                "{} ref_get {}.{}",
                format_dst(dst, types),
                reference,
                field
            )
        }

        MirInstruction::RefSet { reference, field, value } => {
            format!("ref_set {}.{} = {}", reference, field, value)
        }

        // Legacy -> Unified print: WeakNew/WeakLoad/BarrierRead/BarrierWrite
        MirInstruction::WeakNew { dst, box_val } => {
            format!("{} weakref new {}", format_dst(dst, types), box_val)
        }
        MirInstruction::WeakLoad { dst, weak_ref } => {
            format!("{} weakref load {}", format_dst(dst, types), weak_ref)
        }
        MirInstruction::BarrierRead { ptr } => {
            format!("barrier read {}", ptr)
        }
        MirInstruction::BarrierWrite { ptr } => {
            format!("barrier write {}", ptr)
        }

        // Phase 6: WeakRef/Barrier unified
        MirInstruction::WeakRef { dst, op, value } => {
            let op_str = match op {
                super::WeakRefOp::New => "new",
                super::WeakRefOp::Load => "load",
            };
            format!(
                "{} weakref {} {}",
                format_dst(dst, types),
                op_str,
                value
            )
        }

        MirInstruction::Barrier { op, ptr } => {
            let op_str = match op {
                super::BarrierOp::Read => "read",
                super::BarrierOp::Write => "write",
            };
            format!("barrier {} {}", op_str, ptr)
        }

        // Phase 7: Async/Future Operations
        MirInstruction::FutureNew { dst, value } => {
            format!("{} future_new {}", format_dst(dst, types), value)
        }

        MirInstruction::FutureSet { future, value } => {
            format!("future_set {} = {}", future, value)
        }

        MirInstruction::Await { dst, future } => {
            format!("{} await {}", format_dst(dst, types), future)
        }

        // Phase 9.7: External Function Calls
        MirInstruction::ExternCall { dst, iface_name, method_name, args, effects } => {
            let args_str = args
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(dst) = dst {
                format!(
                    "{} extern_call {}.{}({}) [effects: {}]",
                    format_dst(dst, types),
                    iface_name,
                    method_name,
                    args_str,
                    effects
                )
            } else {
                format!(
                    "extern_call {}.{}({}) [effects: {}]",
                    iface_name, method_name, args_str, effects
                )
            }
        }
    }
}
