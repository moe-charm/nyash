use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

/// Handle String-specific methods. Returns true if handled, false to let caller continue.
pub(super) fn try_handle_string_method<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    args: &[ValueId],
    recv_v: BVE<'ctx>,
) -> Result<bool, String> {
    // Act if receiver is annotated as String/StringBox, or if the actual value is an i8* (string literal path)
    let is_string_recv = match func.metadata.value_types.get(box_val) {
        Some(crate::mir::MirType::String) => true,
        Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
        _ => matches!(recv_v, BVE::PointerValue(_)),
    };
    // Do not early-return; allow method-specific checks below to validate types

    // concat fast-paths
    if method == "concat" {
        if args.len() != 1 {
            return Err("String.concat expects 1 arg".to_string());
        }
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let rhs_v = *vmap.get(&args[0]).ok_or("concat arg missing")?;
        match (recv_v, rhs_v) {
            (BVE::PointerValue(lp), BVE::PointerValue(rp)) => {
                let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_ss")
                    .unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_ss", fnty, None));
                let call = codegen
                    .builder
                    .build_call(callee, &[lp.into(), rp.into()], "concat_ss_call")
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_ss returned void".to_string())?;
                    vmap.insert(*d, rv);
                }
                return Ok(true);
            }
            (BVE::PointerValue(lp), BVE::IntValue(ri)) => {
                let i64t = codegen.context.i64_type();
                let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_si")
                    .unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_si", fnty, None));
                let call = codegen
                    .builder
                    .build_call(callee, &[lp.into(), ri.into()], "concat_si_call")
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_si returned void".to_string())?;
                    vmap.insert(*d, rv);
                }
                return Ok(true);
            }
            (BVE::IntValue(li), BVE::PointerValue(rp)) => {
                let i64t = codegen.context.i64_type();
                let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_is")
                    .unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_is", fnty, None));
                let call = codegen
                    .builder
                    .build_call(callee, &[li.into(), rp.into()], "concat_is_call")
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_is returned void".to_string())?;
                    vmap.insert(*d, rv);
                }
                return Ok(true);
            }
            _ => { /* fall through */ }
        }
    }

    // length/len fast-path
    if method == "length" || method == "len" {
        let i64t = codegen.context.i64_type();
        // Ensure handle for receiver (i8* -> i64 via from_i8_string)
        let recv_h = match recv_v {
            BVE::IntValue(h) => h,
            BVE::PointerValue(p) => {
                let fnty = i64t.fn_type(&[codegen.context.ptr_type(AddressSpace::from(0)).into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.box.from_i8_string")
                    .unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                let call = codegen
                    .builder
                    .build_call(callee, &[p.into()], "str_ptr_to_handle")
                    .map_err(|e| e.to_string())?;
        
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("from_i8_string returned void".to_string())?;
                if let BVE::IntValue(iv) = rv {
                    iv
                } else {
                    return Err("from_i8_string ret expected i64".to_string());
                }
            }
            _ => return Err("String.length receiver type unsupported".to_string()),
        };
        // call i64 @nyash.string.len_h(i64)
        let fnty = i64t.fn_type(&[i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.len_h")
            .unwrap_or_else(|| codegen.module.add_function("nyash.string.len_h", fnty, None));
        let call = codegen
            .builder
            .build_call(callee, &[recv_h.into()], "strlen_h")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("len_h returned void".to_string())?;
            vmap.insert(*d, rv);
        }
        return Ok(true);
    }

    // substring(start, end) -> i8*
    if method == "substring" {
        if args.len() != 2 {
            return Err("String.substring expects 2 args (start, end)".to_string());
        }
        let i64t = codegen.context.i64_type();
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        // receiver preferably i8*; if it's a handle (i64), conservatively cast to i8*
        let recv_p = match recv_v {
            BVE::PointerValue(p) => p,
            BVE::IntValue(iv) => codegen
                .builder
                .build_int_to_ptr(iv, codegen.context.ptr_type(AddressSpace::from(0)), "str_h2p_sub")
                .map_err(|e| e.to_string())?,
            _ => return Ok(false),
        };
        let a0 = *vmap.get(&args[0]).ok_or("substring start arg missing")?;
        let a1 = *vmap.get(&args[1]).ok_or("substring end arg missing")?;
        let s = match a0 {
            BVE::IntValue(iv) => iv,
            _ => return Err("substring start must be integer".to_string()),
        };
        let e = match a1 {
            BVE::IntValue(iv) => iv,
            _ => return Err("substring end must be integer".to_string()),
        };
        let fnty = i8p.fn_type(&[i8p.into(), i64t.into(), i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.substring_sii")
            .unwrap_or_else(|| codegen.module.add_function("nyash.string.substring_sii", fnty, None));
        let call = codegen
            .builder
            .build_call(callee, &[recv_p.into(), s.into(), e.into()], "substring_call")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("substring returned void".to_string())?;
            vmap.insert(*d, rv);
        }
        return Ok(true);
    }

    // lastIndexOf(needle) -> i64
    if method == "lastIndexOf" {
        if args.len() != 1 {
            return Err("String.lastIndexOf expects 1 arg".to_string());
        }
        let i64t = codegen.context.i64_type();
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        // receiver must be i8* for this fast path
        let recv_p = match recv_v {
            BVE::PointerValue(p) => p,
            _ => return Ok(false),
        };
        let a0 = *vmap.get(&args[0]).ok_or("lastIndexOf arg missing")?;
        let needle_p = match a0 {
            BVE::PointerValue(p) => p,
            _ => return Err("lastIndexOf arg must be i8*".to_string()),
        };
        let fnty = i64t.fn_type(&[i8p.into(), i8p.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.lastIndexOf_ss")
            .unwrap_or_else(|| codegen.module.add_function("nyash.string.lastIndexOf_ss", fnty, None));
        let call = codegen
            .builder
            .build_call(callee, &[recv_p.into(), needle_p.into()], "lastindexof_call")
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("lastIndexOf returned void".to_string())?;
            vmap.insert(*d, rv);
        }
        return Ok(true);
    }

    Ok(false)
}
