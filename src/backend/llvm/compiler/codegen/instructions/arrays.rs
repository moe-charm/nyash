use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

/// Handle ArrayBox fast-paths. Returns true if handled.
pub(super) fn try_handle_array_method<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    args: &[ValueId],
    recv_h: inkwell::values::IntValue<'ctx>,
) -> Result<bool, String> {
    // Only when receiver is ArrayBox
    let is_array = matches!(func.metadata.value_types.get(box_val), Some(crate::mir::MirType::Box(b)) if b == "ArrayBox")
        || matches!(method, "get" | "set" | "push" | "length");
    if !is_array {
        return Ok(false);
    }
    let i64t = codegen.context.i64_type();
    match method {
        "get" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Array.get (core)");
            }
            if args.len() != 1 {
                return Err("ArrayBox.get expects 1 arg".to_string());
            }
            let idx_v = *vmap.get(&args[0]).ok_or("array.get index missing")?;
            let idx_i = if let BVE::IntValue(iv) = idx_v {
                iv
            } else {
                return Err("array.get index must be int".to_string());
            };
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash_array_get_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash_array_get_h", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[recv_h.into(), idx_i.into()], "aget")
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("array_get_h returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            Ok(true)
        }
        "set" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Array.set (core)");
            }
            if args.len() != 2 {
                return Err("ArrayBox.set expects 2 arg".to_string());
            }
            let idx_v = *vmap.get(&args[0]).ok_or("array.set index missing")?;
            let val_v = *vmap.get(&args[1]).ok_or("array.set value missing")?;
            let idx_i = if let BVE::IntValue(iv) = idx_v {
                iv
            } else {
                return Err("array.set index must be int".to_string());
            };
            let val_i = if let BVE::IntValue(iv) = val_v {
                iv
            } else {
                return Err("array.set value must be int".to_string());
            };
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash_array_set_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash_array_set_h", fnty, None));
            let _ = codegen
                .builder
                .build_call(callee, &[recv_h.into(), idx_i.into(), val_i.into()], "aset")
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        "push" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Array.push (core)");
            }
            if args.len() != 1 {
                return Err("ArrayBox.push expects 1 arg".to_string());
            }
            let val_v = *vmap.get(&args[0]).ok_or("array.push value missing")?;
            let val_i = match val_v {
                BVE::IntValue(iv) => iv,
                BVE::PointerValue(pv) => codegen
                    .builder
                    .build_ptr_to_int(pv, i64t, "val_p2i")
                    .map_err(|e| e.to_string())?,
                _ => return Err("array.push value must be int or handle ptr".to_string()),
            };
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash_array_push_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash_array_push_h", fnty, None));
            let _ = codegen
                .builder
                .build_call(callee, &[recv_h.into(), val_i.into()], "apush")
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        "length" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[LLVM] lower Array.length (core)");
            }
            if !args.is_empty() {
                return Err("ArrayBox.length expects 0 arg".to_string());
            }
            let fnty = i64t.fn_type(&[i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash_array_length_h")
                .unwrap_or_else(|| codegen.module.add_function("nyash_array_length_h", fnty, None));
            let call = codegen
                .builder
                .build_call(callee, &[recv_h.into()], "alen")
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("array_length_h returned void".to_string())?;
                vmap.insert(*d, rv);
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
