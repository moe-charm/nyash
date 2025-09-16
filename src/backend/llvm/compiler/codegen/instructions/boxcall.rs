use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use crate::backend::llvm::context::CodegenContext;
mod fields;
pub(crate) mod invoke;
mod marshal;
use self::invoke as invoke_mod;
use self::marshal as marshal_mod;
use super::builder_cursor::BuilderCursor;
use super::ctx::{BlockCtx, LowerFnCtx};
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

// BoxCall lowering (large): mirrors existing logic; kept in one function for now
pub(in super::super) fn lower_boxcall<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    method_id: &Option<u16>,
    args: &[ValueId],
    box_type_ids: &HashMap<String, i64>,
    entry_builder: &inkwell::builder::Builder<'ctx>,
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
    use super::super::types::classify_tag;
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    let i64t = codegen.context.i64_type();
    // Resolve receiver as handle and pointer (i8*) via Resolver to ensure dominance safety
    let recv_h = resolver.resolve_i64(
        codegen,
        cursor,
        cur_bid,
        *box_val,
        bb_map,
        preds,
        block_end_values,
        vmap,
    )?;
    let recv_p = resolver.resolve_ptr(
        codegen,
        cursor,
        cur_bid,
        *box_val,
        bb_map,
        preds,
        block_end_values,
        vmap,
    )?;
    let recv_v: BVE = recv_p.into();

    // Resolve type_id
    let type_id: i64 =
        if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
            *box_type_ids.get(bname).unwrap_or(&0)
        } else if let Some(crate::mir::MirType::String) = func.metadata.value_types.get(box_val) {
            *box_type_ids.get("StringBox").unwrap_or(&0)
        } else {
            0
        };

    // Delegate String methods
    if super::strings::try_handle_string_method(
        codegen,
        cursor,
        resolver,
        cur_bid,
        func,
        vmap,
        dst,
        box_val,
        method,
        args,
        bb_map,
        preds,
        block_end_values,
    )? {
        return Ok(());
    }

    // Delegate Map methods first (to avoid Array fallback catching get/set ambiguously)
    if super::maps::try_handle_map_method(
        codegen, cursor, resolver, cur_bid, func, vmap, dst, box_val, method, args, recv_h,
    )? {
        return Ok(());
    }

    // Delegate Array methods
    if super::arrays::try_handle_array_method(
        codegen,
        cursor,
        resolver,
        cur_bid,
        func,
        vmap,
        dst,
        box_val,
        method,
        args,
        recv_h,
        bb_map,
        preds,
        block_end_values,
    )? {
        return Ok(());
    }

    // Console convenience: treat println as env.console.log
    if method == "println" {
        return super::externcall::lower_externcall(
            codegen,
            cursor,
            resolver,
            cur_bid,
            func,
            vmap,
            dst,
            &"env.console".to_string(),
            &"log".to_string(),
            args,
            bb_map,
            preds,
            block_end_values,
        );
    }

    // getField/setField
    if fields::try_handle_field_method(
        codegen,
        cursor,
        cur_bid,
        vmap,
        dst,
        method,
        args,
        recv_h,
        resolver,
        bb_map,
        preds,
        block_end_values,
    )? {
        return Ok(());
    }

    // Minimal untyped fallback: Array.length with missing annotations
    if method == "length" && args.is_empty() {
        let fnty = i64t.fn_type(&[i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash_array_length_h")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash_array_length_h", fnty, None)
            });
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(callee, &[recv_h.into()], "alen_fallback")
            })
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("array_length_h returned void".to_string())?;
            vmap.insert(*d, rv);
        }
        return Ok(());
    }

    if let Some(mid) = method_id {
        invoke::try_handle_tagged_invoke(
            codegen,
            func,
            cursor,
            resolver,
            vmap,
            dst,
            *mid,
            type_id,
            recv_h,
            args,
            entry_builder,
            cur_bid,
            bb_map,
            preds,
            block_end_values,
        )?;
        return Ok(());
    } else {
        // Fallback: treat as direct call to a user function in the same module, if present.
        // Compose candidate name like "<Module>.<method>/<arity>" (e.g., Main.esc_json/1)
        let arity = args.len();
        let module_name = func
            .signature
            .name
            .split('.')
            .next()
            .unwrap_or("")
            .to_string();
        if !module_name.is_empty() {
            let candidate = format!("{}.{}{}", module_name, method, format!("/{}", arity));
            // Sanitize symbol the same way as codegen/mod.rs does
            let sym: String = {
                let mut s = String::from("ny_f_");
                s.push_str(
                    &candidate
                        .replace('.', "_")
                        .replace('/', "_")
                        .replace('-', "_"),
                );
                s
            };
            if let Some(callee) = codegen.module.get_function(&sym) {
                // Coerce arguments to callee parameter types
                let exp_tys = callee.get_type().get_param_types();
                if exp_tys.len() != args.len() {
                    return Err("boxcall direct-call: arg count mismatch".to_string());
                }
                let mut call_args: Vec<inkwell::values::BasicMetadataValueEnum> =
                    Vec::with_capacity(args.len());
                for (i, a) in args.iter().enumerate() {
                    use inkwell::types::BasicMetadataTypeEnum as BMTy;
                    let coerced: BVE<'ctx> = match exp_tys[i] {
                        BMTy::IntType(it) => {
                            // Use Resolver via our surrounding lowering
                            let iv = resolver.resolve_i64(
                                codegen,
                                cursor,
                                cur_bid,
                                *a,
                                bb_map,
                                preds,
                                block_end_values,
                                vmap,
                            )?;
                            let bw_dst = it.get_bit_width();
                            let bw_src = iv.get_type().get_bit_width();
                            if bw_src == bw_dst {
                                iv.into()
                            } else if bw_src < bw_dst {
                                cursor
                                    .emit_instr(cur_bid, |b| {
                                        b.build_int_z_extend(iv, it, "boxcall_arg_zext")
                                    })
                                    .map_err(|e| e.to_string())?
                                    .into()
                            } else if bw_dst == 1 {
                                super::super::types::to_bool(
                                    codegen.context,
                                    iv.into(),
                                    &codegen.builder,
                                )?
                                .into()
                            } else {
                                cursor
                                    .emit_instr(cur_bid, |b| {
                                        b.build_int_truncate(iv, it, "boxcall_arg_trunc")
                                    })
                                    .map_err(|e| e.to_string())?
                                    .into()
                            }
                        }
                        BMTy::PointerType(pt) => {
                            let iv = resolver.resolve_i64(
                                codegen,
                                cursor,
                                cur_bid,
                                *a,
                                bb_map,
                                preds,
                                block_end_values,
                                vmap,
                            )?;
                            let p = cursor
                                .emit_instr(cur_bid, |b| {
                                    b.build_int_to_ptr(iv, pt, "boxcall_arg_i2p")
                                })
                                .map_err(|e| e.to_string())?;
                            p.into()
                        }
                        BMTy::FloatType(ft) => {
                            let fv = resolver.resolve_f64(
                                codegen,
                                cursor,
                                cur_bid,
                                *a,
                                bb_map,
                                preds,
                                block_end_values,
                                vmap,
                            )?;
                            if fv.get_type() == ft {
                                fv.into()
                            } else {
                                cursor
                                    .emit_instr(cur_bid, |b| {
                                        b.build_float_cast(fv, ft, "boxcall_arg_fcast")
                                    })
                                    .map_err(|e| e.to_string())?
                                    .into()
                            }
                        }
                        _ => {
                            return Err(
                                "boxcall direct-call: unsupported parameter type".to_string()
                            )
                        }
                    };
                    call_args.push(coerced.into());
                }
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &call_args, "user_meth_call")
                    })
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    if let Some(rv) = call.try_as_basic_value().left() {
                        vmap.insert(*d, rv);
                    }
                }
                return Ok(());
            }
        }
        // Last resort: invoke plugin by name (host resolves method_id)
        {
            let i64t = codegen.context.i64_type();
            let argc = i64t.const_int(args.len() as u64, false);
            let mname = cursor
                .emit_instr(cur_bid, |b| b.build_global_string_ptr(method, "meth_name"))
                .map_err(|e| e.to_string())?;
            // up to 2 args for this minimal path
            let a1 = if let Some(v0) = args.get(0) {
                resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *v0,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?
            } else {
                i64t.const_zero()
            };
            let a2 = if let Some(v1) = args.get(1) {
                resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *v1,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?
            } else {
                i64t.const_zero()
            };
            let fnty = i64t.fn_type(
                &[
                    i64t.into(),                                            // recv handle
                    codegen.context.ptr_type(AddressSpace::from(0)).into(), // method cstr
                    i64t.into(),
                    i64t.into(),
                    i64t.into(), // argc, a1, a2
                ],
                false,
            );
            let callee = codegen
                .module
                .get_function("nyash.plugin.invoke_by_name_i64")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.plugin.invoke_by_name_i64", fnty, None)
                });
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(
                        callee,
                        &[
                            recv_h.into(),
                            mname.as_pointer_value().into(),
                            argc.into(),
                            a1.into(),
                            a2.into(),
                        ],
                        "pinvoke_by_name",
                    )
                })
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("invoke_by_name returned void".to_string())?;
                // Inline minimal return normalization similar to store_invoke_return()
                if let Some(mt) = func.metadata.value_types.get(d) {
                    match mt {
                        crate::mir::MirType::Integer => {
                            vmap.insert(*d, rv);
                        }
                        crate::mir::MirType::Bool => {
                            if let BVE::IntValue(iv) = rv {
                                let i64t = codegen.context.i64_type();
                                let zero = i64t.const_zero();
                                let b1 = cursor
                                    .emit_instr(cur_bid, |bd| {
                                        bd.build_int_compare(
                                            inkwell::IntPredicate::NE,
                                            iv,
                                            zero,
                                            "bool_i64_to_i1",
                                        )
                                    })
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*d, b1.into());
                            } else {
                                vmap.insert(*d, rv);
                            }
                        }
                        crate::mir::MirType::String => {
                            if let BVE::IntValue(iv) = rv {
                                let p = cursor
                                    .emit_instr(cur_bid, |bd| {
                                        bd.build_int_to_ptr(
                                            iv,
                                            codegen.context.ptr_type(AddressSpace::from(0)),
                                            "str_h2p_ret",
                                        )
                                    })
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*d, p.into());
                            } else {
                                vmap.insert(*d, rv);
                            }
                        }
                        crate::mir::MirType::Box(_)
                        | crate::mir::MirType::Array(_)
                        | crate::mir::MirType::Future(_)
                        | crate::mir::MirType::Unknown => {
                            if let BVE::IntValue(iv) = rv {
                                let p = cursor
                                    .emit_instr(cur_bid, |bd| {
                                        bd.build_int_to_ptr(
                                            iv,
                                            codegen.context.ptr_type(AddressSpace::from(0)),
                                            "h2p_ret",
                                        )
                                    })
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*d, p.into());
                            } else {
                                vmap.insert(*d, rv);
                            }
                        }
                        _ => {
                            vmap.insert(*d, rv);
                        }
                    }
                } else {
                    vmap.insert(*d, rv);
                }
            }
            return Ok(());
        }
        Err(format!("BoxCall requires method_id for method '{}'. The method_id should be automatically injected during MIR compilation.", method))
    }
}

// Boxed API: thin shim adapting LowerFnCtx/BlockCtx to the existing implementation.
pub(in super::super) fn lower_boxcall_boxed<'ctx, 'b>(
    ctx: &mut LowerFnCtx<'ctx, 'b>,
    blk: &BlockCtx<'ctx>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    method_id: &Option<u16>,
    args: &[ValueId],
    entry_builder: &inkwell::builder::Builder<'ctx>,
) -> Result<(), String> {
    // Optional dev check: ensure block is open for insertion
    if ctx.dev_checks {
        ctx.cursor.assert_open(blk.cur_bid);
    }
    lower_boxcall(
        ctx.codegen,
        ctx.cursor,
        ctx.resolver,
        blk.cur_bid,
        ctx.func,
        ctx.vmap,
        dst,
        box_val,
        method,
        method_id,
        args,
        ctx.box_type_ids
            .ok_or_else(|| "LowerFnCtx.box_type_ids missing".to_string())?,
        entry_builder,
        ctx.bb_map,
        ctx.preds,
        ctx.block_end_values,
    )
}

// Convenience wrapper: construct LowerFnCtx/BlockCtx inside to keep caller borrow scopes short.
pub(in super::super) fn lower_boxcall_via_ctx<'ctx, 'b>(
    codegen: &'ctx CodegenContext<'ctx>,
    cursor: &'b mut BuilderCursor<'ctx, 'b>,
    resolver: &'b mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &'b MirFunction,
    vmap: &'b mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    method_id: &Option<u16>,
    args: &[ValueId],
    box_type_ids: &'b HashMap<String, i64>,
    entry_builder: &inkwell::builder::Builder<'ctx>,
    bb_map: &'b std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &'b std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &'b std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    let llbb = *bb_map.get(&cur_bid).ok_or("missing cur bb")?;
    let blkctx = BlockCtx::new(cur_bid, llbb);
    let mut fnctx = LowerFnCtx::new(
        codegen,
        func,
        cursor,
        resolver,
        vmap,
        bb_map,
        preds,
        block_end_values,
    )
    .with_box_type_ids(box_type_ids);
    lower_boxcall_boxed(
        &mut fnctx,
        &blkctx,
        dst,
        box_val,
        method,
        method_id,
        args,
        entry_builder,
    )
}

fn coerce_to_type<'ctx>(
    codegen: &CodegenContext<'ctx>,
    val: inkwell::values::BasicValueEnum<'ctx>,
    target: inkwell::types::BasicMetadataTypeEnum<'ctx>,
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    use inkwell::types::BasicMetadataTypeEnum as BT;
    match (val, target) {
        (inkwell::values::BasicValueEnum::IntValue(iv), BT::IntType(it)) => {
            let bw_src = iv.get_type().get_bit_width();
            let bw_dst = it.get_bit_width();
            if bw_src == bw_dst {
                Ok(iv.into())
            } else if bw_src < bw_dst {
                Ok(codegen
                    .builder
                    .build_int_z_extend(iv, it, "bc_zext")
                    .map_err(|e| e.to_string())?
                    .into())
            } else if bw_dst == 1 {
                Ok(
                    super::super::types::to_bool(codegen.context, iv.into(), &codegen.builder)?
                        .into(),
                )
            } else {
                Ok(codegen
                    .builder
                    .build_int_truncate(iv, it, "bc_trunc")
                    .map_err(|e| e.to_string())?
                    .into())
            }
        }
        (inkwell::values::BasicValueEnum::PointerValue(pv), BT::IntType(it)) => Ok(codegen
            .builder
            .build_ptr_to_int(pv, it, "bc_p2i")
            .map_err(|e| e.to_string())?
            .into()),
        (inkwell::values::BasicValueEnum::FloatValue(fv), BT::IntType(it)) => Ok(codegen
            .builder
            .build_float_to_signed_int(fv, it, "bc_f2i")
            .map_err(|e| e.to_string())?
            .into()),
        (inkwell::values::BasicValueEnum::IntValue(iv), BT::PointerType(pt)) => Ok(codegen
            .builder
            .build_int_to_ptr(iv, pt, "bc_i2p")
            .map_err(|e| e.to_string())?
            .into()),
        (inkwell::values::BasicValueEnum::PointerValue(pv), BT::PointerType(_)) => Ok(pv.into()),
        (inkwell::values::BasicValueEnum::IntValue(iv), BT::FloatType(ft)) => Ok(codegen
            .builder
            .build_signed_int_to_float(iv, ft, "bc_i2f")
            .map_err(|e| e.to_string())?
            .into()),
        (inkwell::values::BasicValueEnum::FloatValue(fv), BT::FloatType(_)) => Ok(fv.into()),
        (v, _) => Ok(v),
    }
}
