use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use super::super::ctx::{BlockCtx, LowerFnCtx};
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

// use super::marshal::{get_i64, get_tag_const};

/// Handle method_id-tagged plugin invoke path; returns Ok(()) if handled.
pub(super) fn try_handle_tagged_invoke<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    cursor: &mut crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor<'ctx, 'b>,
    resolver: &mut super::super::Resolver<'ctx>,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    mid: u16,
    type_id: i64,
    recv_h: inkwell::values::IntValue<'ctx>,
    args: &[ValueId],
    entry_builder: &inkwell::builder::Builder<'ctx>,
    cur_bid: crate::mir::BasicBlockId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    let i64t = codegen.context.i64_type();
    let argc_val = i64t.const_int(args.len() as u64, false);

    // Fast path: <= 4 fixed args
    if args.len() <= 4 {
        let mut a = [i64t.const_zero(); 4];
        for (i, vid) in args.iter().enumerate() {
            let iv = match func.metadata.value_types.get(vid) {
                Some(crate::mir::MirType::Float) => {
                    let fv = resolver.resolve_f64(
                        codegen,
                        cursor,
                        cur_bid,
                        *vid,
                        bb_map,
                        preds,
                        block_end_values,
                        vmap,
                    )?;
                    let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.box.from_f64")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_f64", fnty, None)
                        });
                    let call = codegen
                        .builder
                        .build_call(callee, &[fv.into()], "arg_f2h")
                        .map_err(|e| e.to_string())?;
                    call.try_as_basic_value()
                        .left()
                        .ok_or("from_f64 returned void".to_string())?
                        .into_int_value()
                }
                _ => resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *vid,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?,
            };
            a[i] = iv;
        }
        let mut tags = [i64t.const_int(3, false); 4];
        for (i, vid) in args.iter().enumerate() {
            let tag = match func.metadata.value_types.get(vid) {
                Some(crate::mir::MirType::Float) => 5,
                Some(crate::mir::MirType::String)
                | Some(crate::mir::MirType::Box(_))
                | Some(crate::mir::MirType::Array(_))
                | Some(crate::mir::MirType::Future(_))
                | Some(crate::mir::MirType::Unknown) => 8,
                _ => 3,
            };
            tags[i] = i64t.const_int(tag as u64, false);
        }
        let fnty = i64t.fn_type(
            &[
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
            ],
            false,
        );
        let callee = codegen
            .module
            .get_function("nyash_plugin_invoke3_tagged_i64")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash_plugin_invoke3_tagged_i64", fnty, None)
            });
        let tid = i64t.const_int(type_id as u64, true);
        let midv = i64t.const_int(mid as u64, false);
        let call = codegen
            .builder
            .build_call(
                callee,
                &[
                    tid.into(),
                    midv.into(),
                    argc_val.into(),
                    recv_h.into(),
                    a[0].into(),
                    tags[0].into(),
                    a[1].into(),
                    tags[1].into(),
                    a[2].into(),
                    tags[2].into(),
                    a[3].into(),
                    tags[3].into(),
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
        let vi = match func.metadata.value_types.get(vid) {
            Some(crate::mir::MirType::Float) => {
                let fv = resolver.resolve_f64(
                    codegen,
                    cursor,
                    cur_bid,
                    *vid,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.box.from_f64")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.box.from_f64", fnty, None)
                    });
                let call = codegen
                    .builder
                    .build_call(callee, &[fv.into()], "arg_f2h")
                    .map_err(|e| e.to_string())?;
                call.try_as_basic_value()
                    .left()
                    .ok_or("from_f64 returned void".to_string())?
                    .into_int_value()
            }
            _ => resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *vid,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?,
        };
        let ti = match func.metadata.value_types.get(vid) {
            Some(crate::mir::MirType::Float) => i64t.const_int(5, false),
            Some(crate::mir::MirType::String)
            | Some(crate::mir::MirType::Box(_))
            | Some(crate::mir::MirType::Array(_))
            | Some(crate::mir::MirType::Future(_))
            | Some(crate::mir::MirType::Unknown) => i64t.const_int(8, false),
            _ => i64t.const_int(3, false),
        };
        codegen
            .builder
            .build_store(gep_v, vi)
            .map_err(|e| e.to_string())?;
        codegen
            .builder
            .build_store(gep_t, ti)
            .map_err(|e| e.to_string())?;
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
        .unwrap_or_else(|| {
            codegen
                .module
                .add_function("nyash.plugin.invoke_tagged_v_i64", fnty, None)
        });
    let tid = i64t.const_int(type_id as u64, true);
    let midv = i64t.const_int(mid as u64, false);
    let call = codegen
        .builder
        .build_call(
            callee,
            &[
                tid.into(),
                midv.into(),
                argc_val.into(),
                recv_h.into(),
                vals_ptr.into(),
                tags_ptr.into(),
            ],
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
            crate::mir::MirType::Integer => {
                vmap.insert(dst, rv);
            }
            crate::mir::MirType::Bool => {
                // Normalize i64 bool (0/1) to i1
                if let BVE::IntValue(iv) = rv {
                    let i64t = codegen.context.i64_type();
                    let zero = i64t.const_zero();
                    let b1 = codegen
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, iv, zero, "bool_i64_to_i1")
                        .map_err(|e| e.to_string())?;
                    vmap.insert(dst, b1.into());
                } else {
                    vmap.insert(dst, rv);
                }
            }
            crate::mir::MirType::String => {
                // Keep as i64 handle across blocks (pointer is produced on demand via Resolver)
                if let BVE::IntValue(iv) = rv {
                    vmap.insert(dst, iv.into());
                } else {
                    return Err("invoke ret expected i64 for String".to_string());
                }
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

// Boxed wrapper delegating to the existing implementation
pub(super) fn try_handle_tagged_invoke_boxed<'ctx, 'b>(
    ctx: &mut LowerFnCtx<'ctx, 'b>,
    blk: &BlockCtx<'ctx>,
    dst: &Option<ValueId>,
    mid: u16,
    type_id: i64,
    recv_h: inkwell::values::IntValue<'ctx>,
    args: &[ValueId],
    entry_builder: &inkwell::builder::Builder<'ctx>,
) -> Result<(), String> {
    try_handle_tagged_invoke(
        ctx.codegen,
        ctx.func,
        ctx.cursor,
        ctx.resolver,
        ctx.vmap,
        dst,
        mid,
        type_id,
        recv_h,
        args,
        entry_builder,
        blk.cur_bid,
        ctx.bb_map,
        ctx.preds,
        ctx.block_end_values,
    )
}
