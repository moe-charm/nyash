use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

use super::marshal::{get_i64, get_tag_const};

/// Handle method_id-tagged plugin invoke path; returns Ok(()) if handled.
pub(super) fn try_handle_tagged_invoke<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    mid: u16,
    type_id: i64,
    recv_h: inkwell::values::IntValue<'ctx>,
    args: &[ValueId],
    entry_builder: &inkwell::builder::Builder<'ctx>,
) -> Result<(), String> {
    let i64t = codegen.context.i64_type();
    let argc_val = i64t.const_int(args.len() as u64, false);

    // Fast path: <= 4 fixed args
    if args.len() <= 4 {
        let mut a = [i64t.const_zero(); 4];
        for (i, vid) in args.iter().enumerate() {
            a[i] = get_i64(codegen, vmap, *vid)?;
        }
        let mut tags = [i64t.const_int(3, false); 4];
        for (i, vid) in args.iter().enumerate() {
            tags[i] = get_tag_const(codegen, vmap, *vid);
        }
        let fnty = i64t.fn_type(
            &[
                i64t.into(), i64t.into(), i64t.into(), i64t.into(),
                i64t.into(), i64t.into(), i64t.into(), i64t.into(),
                i64t.into(), i64t.into(), i64t.into(), i64t.into(),
            ],
            false,
        );
        let callee = codegen
            .module
            .get_function("nyash_plugin_invoke3_tagged_i64")
            .unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_tagged_i64", fnty, None));
        let tid = i64t.const_int(type_id as u64, true);
        let midv = i64t.const_int(mid as u64, false);
        let call = codegen
            .builder
            .build_call(
                callee,
                &[
                    tid.into(), midv.into(), argc_val.into(), recv_h.into(),
                    a[0].into(), tags[0].into(), a[1].into(), tags[1].into(),
                    a[2].into(), tags[2].into(), a[3].into(), tags[3].into(),
                ],
                "pinvoke_tagged",
            )
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("invoke3_i64 returned void".to_string())?;
            store_invoke_return(codegen, func, vmap, *d, rv)?;
        }
        return Ok(());
    }

    // Variable length path: build i64 arrays for vals and tags
    let n = args.len() as u32;
    let arr_ty = i64t.array_type(n);
    let vals_arr = entry_builder
        .build_alloca(arr_ty, "vals_arr")
        .map_err(|e| e.to_string())?;
    let tags_arr = entry_builder
        .build_alloca(arr_ty, "tags_arr")
        .map_err(|e| e.to_string())?;
    for (i, vid) in args.iter().enumerate() {
        let idx = [
            codegen.context.i32_type().const_zero(),
            codegen.context.i32_type().const_int(i as u64, false),
        ];
        let gep_v = unsafe {
            codegen
                .builder
                .build_in_bounds_gep(arr_ty, vals_arr, &idx, &format!("v_gep_{}", i))
                .map_err(|e| e.to_string())?
        };
        let gep_t = unsafe {
            codegen
                .builder
                .build_in_bounds_gep(arr_ty, tags_arr, &idx, &format!("t_gep_{}", i))
                .map_err(|e| e.to_string())?
        };
        let vi = get_i64(codegen, vmap, *vid)?;
        let ti = get_tag_const(codegen, vmap, *vid);
        codegen.builder.build_store(gep_v, vi).map_err(|e| e.to_string())?;
        codegen.builder.build_store(gep_t, ti).map_err(|e| e.to_string())?;
    }
    let vals_ptr = codegen
        .builder
        .build_pointer_cast(
            vals_arr,
            codegen.context.ptr_type(AddressSpace::from(0)),
            "vals_arr_i8p",
        )
        .map_err(|e| e.to_string())?;
    let tags_ptr = codegen
        .builder
        .build_pointer_cast(
            tags_arr,
            codegen.context.ptr_type(AddressSpace::from(0)),
            "tags_arr_i8p",
        )
        .map_err(|e| e.to_string())?;
    let fnty = i64t.fn_type(
        &[
            i64t.into(),
            i64t.into(),
            i64t.into(),
            i64t.into(),
            codegen.context.ptr_type(AddressSpace::from(0)).into(),
            codegen.context.ptr_type(AddressSpace::from(0)).into(),
        ],
        false,
    );
    let callee = codegen
        .module
        .get_function("nyash.plugin.invoke_tagged_v_i64")
        .unwrap_or_else(|| codegen.module.add_function("nyash.plugin.invoke_tagged_v_i64", fnty, None));
    let tid = i64t.const_int(type_id as u64, true);
    let midv = i64t.const_int(mid as u64, false);
    let call = codegen
        .builder
        .build_call(
            callee,
            &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), vals_ptr.into(), tags_ptr.into()],
            "pinvoke_tagged_v",
        )
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("invoke_v returned void".to_string())?;
        store_invoke_return(codegen, func, vmap, *d, rv)?;
    }
    Ok(())
}

fn store_invoke_return<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: ValueId,
    rv: inkwell::values::BasicValueEnum<'ctx>,
) -> Result<(), String> {
    if let Some(mt) = func.metadata.value_types.get(&dst) {
        match mt {
            crate::mir::MirType::Integer | crate::mir::MirType::Bool => {
                vmap.insert(dst, rv);
            }
            crate::mir::MirType::String => {
                // keep as i64 handle
                vmap.insert(dst, rv);
            }
            crate::mir::MirType::Box(_)
            | crate::mir::MirType::Array(_)
            | crate::mir::MirType::Future(_)
            | crate::mir::MirType::Unknown => {
                let h = if let BVE::IntValue(iv) = rv {
                    iv
                } else {
                    return Err("invoke ret expected i64".to_string());
                };
                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                let ptr = codegen
                    .builder
                    .build_int_to_ptr(h, pty, "ret_handle_to_ptr")
                    .map_err(|e| e.to_string())?;
                vmap.insert(dst, ptr.into());
            }
            _ => {
                vmap.insert(dst, rv);
            }
        }
    } else {
        vmap.insert(dst, rv);
    }
    Ok(())
}

