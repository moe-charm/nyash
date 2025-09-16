use std::collections::HashMap;

use inkwell::{values::BasicValueEnum as BVE, AddressSpace};

use super::builder_cursor::BuilderCursor;
use super::Resolver;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

/// Handle String-specific methods. Returns true if handled, false to let caller continue.
pub(super) fn try_handle_string_method<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    dst: &Option<ValueId>,
    box_val: &ValueId,
    method: &str,
    args: &[ValueId],
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, inkwell::values::BasicValueEnum<'ctx>>,
    >,
) -> Result<bool, String> {
    // Receiver annotation check (kept for future diagnostics)
    let _is_string_recv = match func.metadata.value_types.get(box_val) {
        Some(crate::mir::MirType::String) => true,
        Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
        _ => false,
    };
    // Do not early-return; method-specific checksで型検証を行う

    // concat fast-paths
    if method == "concat" {
        if args.len() != 1 {
            return Err("String.concat expects 1 arg".to_string());
        }
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        // Resolve rhs as either pointer (string) or i64 (handle/int)
        let rhs_val = match func.metadata.value_types.get(&args[0]) {
            Some(crate::mir::MirType::String) => {
                let p = resolver.resolve_ptr(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                BVE::PointerValue(p)
            }
            _ => {
                // Default to integer form for non-String metadata
                let iv = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                BVE::IntValue(iv)
            }
        };
        let lp = resolver.resolve_ptr(
            codegen,
            cursor,
            cur_bid,
            *box_val,
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        match (BVE::PointerValue(lp), rhs_val) {
            (BVE::PointerValue(lp), BVE::PointerValue(rp)) => {
                let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_ss")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.string.concat_ss", fnty, None)
                    });
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &[lp.into(), rp.into()], "concat_ss_call")
                    })
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_ss returned void".to_string())?;
                    // return as handle (i64) across blocks
                    let i64t = codegen.context.i64_type();
                    let h = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_ptr_to_int(rv.into_pointer_value(), i64t, "str_ptr2i")
                        })
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*d, h.into());
                }
                return Ok(true);
            }
            (BVE::PointerValue(lp), BVE::IntValue(_ri)) => {
                let i64t = codegen.context.i64_type();
                // Localize rhs integer in current block via Resolver
                let ri = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    args[0],
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_si")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.string.concat_si", fnty, None)
                    });
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &[lp.into(), ri.into()], "concat_si_call")
                    })
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_si returned void".to_string())?;
                    let i64t = codegen.context.i64_type();
                    let h = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_ptr_to_int(rv.into_pointer_value(), i64t, "str_ptr2i")
                        })
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*d, h.into());
                }
                return Ok(true);
            }
            (BVE::PointerValue(_li_as_p), BVE::PointerValue(rp)) => {
                let i64t = codegen.context.i64_type();
                // Localize receiver integer in current block (box_val)
                let li = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *box_val,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                let callee = codegen
                    .module
                    .get_function("nyash.string.concat_is")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.string.concat_is", fnty, None)
                    });
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &[li.into(), rp.into()], "concat_is_call")
                    })
                    .map_err(|e| e.to_string())?;
                if let Some(d) = dst {
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_is returned void".to_string())?;
                    let i64t = codegen.context.i64_type();
                    let h = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_ptr_to_int(rv.into_pointer_value(), i64t, "str_ptr2i")
                        })
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*d, h.into());
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
        let recv_h = {
            // Prefer i64 handle from resolver; if metadata says String but actual is i8*, box it
            if let Some(crate::mir::MirType::String) = func.metadata.value_types.get(box_val) {
                // Receiver is a String: resolve pointer then box to i64
                let p = resolver.resolve_ptr(
                    codegen,
                    cursor,
                    cur_bid,
                    *box_val,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                let fnty = i64t.fn_type(
                    &[codegen.context.ptr_type(AddressSpace::from(0)).into()],
                    false,
                );
                let callee = codegen
                    .module
                    .get_function("nyash.box.from_i8_string")
                    .unwrap_or_else(|| {
                        codegen
                            .module
                            .add_function("nyash.box.from_i8_string", fnty, None)
                    });
                let call = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_call(callee, &[p.into()], "str_ptr_to_handle")
                    })
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
            } else {
                resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *box_val,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?
            }
        };
        // call i64 @nyash.string.len_h(i64)
        let fnty = i64t.fn_type(&[i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.len_h")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash.string.len_h", fnty, None)
            });
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(callee, &[recv_h.into()], "strlen_h")
            })
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
        // receiver pointer via Resolver
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
        // Localize start/end indices to current block via sealed snapshots (i64)
        let s = resolver.resolve_i64(
            codegen,
            cursor,
            cur_bid,
            args[0],
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let e = resolver.resolve_i64(
            codegen,
            cursor,
            cur_bid,
            args[1],
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let fnty = i8p.fn_type(&[i8p.into(), i64t.into(), i64t.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.substring_sii")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash.string.substring_sii", fnty, None)
            });
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(
                    callee,
                    &[recv_p.into(), s.into(), e.into()],
                    "substring_call",
                )
            })
            .map_err(|e| e.to_string())?;
        if let Some(d) = dst {
            let rv = call
                .try_as_basic_value()
                .left()
                .ok_or("substring returned void".to_string())?;
            let i64t = codegen.context.i64_type();
            let h = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_ptr_to_int(rv.into_pointer_value(), i64t, "str_ptr2i_sub")
                })
                .map_err(|e| e.to_string())?;
            vmap.insert(*d, h.into());
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
        // receiver pointer via Resolver (String fast path)
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
        let needle_p = resolver.resolve_ptr(
            codegen,
            cursor,
            cur_bid,
            args[0],
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let fnty = i64t.fn_type(&[i8p.into(), i8p.into()], false);
        let callee = codegen
            .module
            .get_function("nyash.string.lastIndexOf_ss")
            .unwrap_or_else(|| {
                codegen
                    .module
                    .add_function("nyash.string.lastIndexOf_ss", fnty, None)
            });
        let call = cursor
            .emit_instr(cur_bid, |b| {
                b.build_call(
                    callee,
                    &[recv_p.into(), needle_p.into()],
                    "lastindexof_call",
                )
            })
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
