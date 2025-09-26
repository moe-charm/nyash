use std::collections::HashMap;

use crate::backend::llvm::compiler::codegen::types;
use inkwell::{values::BasicValueEnum, AddressSpace};

use super::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, instruction::UnaryOp, BasicBlockId, BinaryOp, ValueId};

/// Lower UnaryOp and store into vmap (0-diff)
pub(in super::super) fn lower_unary<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: ValueId,
    op: &UnaryOp,
    operand: &ValueId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    use crate::mir::MirType as MT;
    let out = match op {
        UnaryOp::Neg => match func.metadata.value_types.get(operand) {
            Some(MT::Float) => {
                let fv = resolver.resolve_f64(
                    codegen,
                    cursor,
                    cur_bid,
                    *operand,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                cursor
                    .emit_instr(cur_bid, |b| b.build_float_neg(fv, "fneg"))
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => {
                let iv = resolver.resolve_i64(
                    codegen,
                    cursor,
                    cur_bid,
                    *operand,
                    bb_map,
                    preds,
                    block_end_values,
                    vmap,
                )?;
                cursor
                    .emit_instr(cur_bid, |b| b.build_int_neg(iv, "ineg"))
                    .map_err(|e| e.to_string())?
                    .into()
            }
        },
        UnaryOp::Not | UnaryOp::BitNot => {
            let iv = resolver.resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *operand,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?;
            cursor
                .emit_instr(cur_bid, |b| b.build_not(iv, "inot"))
                .map_err(|e| e.to_string())?
                .into()
        }
    };
    vmap.insert(dst, out);
    Ok(())
}

/// Lower BinOp and store into vmap (includes concat fallback)
pub(in super::super) fn lower_binop<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
    dst: ValueId,
    op: &BinaryOp,
    lhs: &ValueId,
    rhs: &ValueId,
    bb_map: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        inkwell::basic_block::BasicBlock<'ctx>,
    >,
    preds: &std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>>,
    block_end_values: &std::collections::HashMap<
        crate::mir::BasicBlockId,
        std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
    >,
) -> Result<(), String> {
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    use inkwell::values::BasicValueEnum as BVE;
    use inkwell::IntPredicate;
    // Construct lhs/rhs proxy values via Resolver according to metadata (no vmap direct access)
    let lv: BasicValueEnum<'ctx> = match func.metadata.value_types.get(lhs) {
        Some(crate::mir::MirType::Float) => resolver
            .resolve_f64(
                codegen,
                cursor,
                cur_bid,
                *lhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
        Some(crate::mir::MirType::String) | Some(crate::mir::MirType::Box(_)) => resolver
            .resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                *lhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
        _ => resolver
            .resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *lhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
    };
    let rv: BasicValueEnum<'ctx> = match func.metadata.value_types.get(rhs) {
        Some(crate::mir::MirType::Float) => resolver
            .resolve_f64(
                codegen,
                cursor,
                cur_bid,
                *rhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
        Some(crate::mir::MirType::String) | Some(crate::mir::MirType::Box(_)) => resolver
            .resolve_ptr(
                codegen,
                cursor,
                cur_bid,
                *rhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
        _ => resolver
            .resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *rhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )?
            .into(),
    };
    let mut handled_concat = false;
    if let BinaryOp::Add = op {
        let i8p = codegen.context.ptr_type(AddressSpace::from(0));
        let is_stringish = |vid: &ValueId| -> bool {
            match func.metadata.value_types.get(vid) {
                Some(crate::mir::MirType::String) => true,
                Some(crate::mir::MirType::Box(_)) => true,
                _ => false,
            }
        };
        match (lv, rv) {
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
                        b.build_call(callee, &[lp.into(), rp.into()], "concat_ss")
                    })
                    .map_err(|e| e.to_string())?;
                let rv = call
                    .try_as_basic_value()
                    .left()
                    .ok_or("concat_ss returned void".to_string())?;
                // store as handle (i64) across blocks
                let i64t = codegen.context.i64_type();
                let h = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_ptr_to_int(rv.into_pointer_value(), i64t, "str_ptr2i")
                    })
                    .map_err(|e| e.to_string())?;
                vmap.insert(dst, h.into());
                handled_concat = true;
            }
            (BVE::PointerValue(lp), BVE::IntValue(ri)) => {
                if is_stringish(lhs) && is_stringish(rhs) {
                    let i64t = codegen.context.i64_type();
                    let fnty_conv = i64t.fn_type(&[i8p.into()], false);
                    let conv = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty_conv, None)
                        });
                    let call_c = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(conv, &[lp.into()], "lhs_i8_to_handle")
                        })
                        .map_err(|e| e.to_string())?;
                    let lh = call_c
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value();
                    let fnty_hh = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.string.concat_hh")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.string.concat_hh", fnty_hh, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[lh.into(), ri.into()], "concat_hh")
                        })
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_hh returned void".to_string())?;
                    vmap.insert(dst, rv);
                    handled_concat = true;
                } else {
                    let i64t = codegen.context.i64_type();
                    let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.string.concat_si")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.string.concat_si", fnty, None)
                        });
                    let call = codegen
                        .builder
                        .build_call(callee, &[lp.into(), ri.into()], "concat_si")
                        .map_err(|e| e.to_string())?;
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
                    vmap.insert(dst, h.into());
                    handled_concat = true;
                }
            }
            (BVE::IntValue(li), BVE::PointerValue(rp)) => {
                if is_stringish(lhs) && is_stringish(rhs) {
                    let i64t = codegen.context.i64_type();
                    let fnty_conv = i64t.fn_type(&[i8p.into()], false);
                    let conv = codegen
                        .module
                        .get_function("nyash.box.from_i8_string")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.box.from_i8_string", fnty_conv, None)
                        });
                    let call_c = codegen
                        .builder
                        .build_call(conv, &[rp.into()], "rhs_i8_to_handle")
                        .map_err(|e| e.to_string())?;
                    let rh = call_c
                        .try_as_basic_value()
                        .left()
                        .ok_or("from_i8_string returned void".to_string())?
                        .into_int_value();
                    let fnty_hh = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                    let callee = codegen
                        .module
                        .get_function("nyash.string.concat_hh")
                        .unwrap_or_else(|| {
                            codegen
                                .module
                                .add_function("nyash.string.concat_hh", fnty_hh, None)
                        });
                    let call = cursor
                        .emit_instr(cur_bid, |b| {
                            b.build_call(callee, &[li.into(), rh.into()], "concat_hh")
                        })
                        .map_err(|e| e.to_string())?;
                    let rv = call
                        .try_as_basic_value()
                        .left()
                        .ok_or("concat_hh returned void".to_string())?;
                    vmap.insert(dst, rv);
                    handled_concat = true;
                } else {
                    let i64t = codegen.context.i64_type();
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
                            b.build_call(callee, &[li.into(), rp.into()], "concat_is")
                        })
                        .map_err(|e| e.to_string())?;
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
                    vmap.insert(dst, h.into());
                    handled_concat = true;
                }
            }
            _ => {}
        }
    }
    if handled_concat {
        return Ok(());
    }

    let out = if let (Some(_li0), Some(_ri0)) = (as_int(lv), as_int(rv)) {
        // Localize integer operands into current block to satisfy dominance (normalize to i64)
        let li = resolver
            .resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *lhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )
            .unwrap_or_else(|_| codegen.context.i64_type().const_zero());
        let ri = resolver
            .resolve_i64(
                codegen,
                cursor,
                cur_bid,
                *rhs,
                bb_map,
                preds,
                block_end_values,
                vmap,
            )
            .unwrap_or_else(|_| codegen.context.i64_type().const_zero());
        use BinaryOp as B;
        match op {
            B::Add => cursor
                .emit_instr(cur_bid, |b| b.build_int_add(li, ri, "iadd"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Sub => cursor
                .emit_instr(cur_bid, |b| b.build_int_sub(li, ri, "isub"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Mul => cursor
                .emit_instr(cur_bid, |b| b.build_int_mul(li, ri, "imul"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Div => cursor
                .emit_instr(cur_bid, |b| b.build_int_signed_div(li, ri, "idiv"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Mod => cursor
                .emit_instr(cur_bid, |b| b.build_int_signed_rem(li, ri, "imod"))
                .map_err(|e| e.to_string())?
                .into(),
            B::BitAnd => cursor
                .emit_instr(cur_bid, |b| b.build_and(li, ri, "iand"))
                .map_err(|e| e.to_string())?
                .into(),
            B::BitOr => cursor
                .emit_instr(cur_bid, |b| b.build_or(li, ri, "ior"))
                .map_err(|e| e.to_string())?
                .into(),
            B::BitXor => cursor
                .emit_instr(cur_bid, |b| b.build_xor(li, ri, "ixor"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Shl => cursor
                .emit_instr(cur_bid, |b| b.build_left_shift(li, ri, "ishl"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Shr => cursor
                .emit_instr(cur_bid, |b| b.build_right_shift(li, ri, false, "ishr"))
                .map_err(|e| e.to_string())?
                .into(),
            B::And | B::Or => {
                // Treat as logical on integers: convert to i1 and and/or
                let lb = types::to_bool(codegen.context, li.into(), &codegen.builder)?;
                let rb = types::to_bool(codegen.context, ri.into(), &codegen.builder)?;
                match op {
                    B::And => cursor
                        .emit_instr(cur_bid, |b| b.build_and(lb, rb, "land"))
                        .map_err(|e| e.to_string())?
                        .into(),
                    _ => cursor
                        .emit_instr(cur_bid, |b| b.build_or(lb, rb, "lor"))
                        .map_err(|e| e.to_string())?
                        .into(),
                }
            }
        }
    } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
        use BinaryOp as B;
        match op {
            B::Add => cursor
                .emit_instr(cur_bid, |b| b.build_float_add(lf, rf, "fadd"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Sub => cursor
                .emit_instr(cur_bid, |b| b.build_float_sub(lf, rf, "fsub"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Mul => cursor
                .emit_instr(cur_bid, |b| b.build_float_mul(lf, rf, "fmul"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Div => cursor
                .emit_instr(cur_bid, |b| b.build_float_div(lf, rf, "fdiv"))
                .map_err(|e| e.to_string())?
                .into(),
            B::Mod => return Err("fmod not supported yet".to_string()),
            _ => return Err("bit/logic ops on float".to_string()),
        }
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        // Support pointer addition/subtraction if needed? For now, only equality is in compare.
        return Err("unsupported pointer binop".to_string());
    } else {
        return Err("binop type mismatch".to_string());
    };
    vmap.insert(dst, out);
    Ok(())
}

fn guessed_zero<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vid: &ValueId,
) -> BasicValueEnum<'ctx> {
    use crate::mir::MirType as MT;
    match func.metadata.value_types.get(vid) {
        Some(MT::Bool) => codegen.context.bool_type().const_zero().into(),
        Some(MT::Integer) => codegen.context.i64_type().const_zero().into(),
        Some(MT::Float) => codegen.context.f64_type().const_zero().into(),
        Some(MT::String) | Some(MT::Box(_)) | Some(MT::Array(_)) | Some(MT::Future(_))
        | Some(MT::Unknown) | Some(MT::Void) | None => codegen
            .context
            .ptr_type(AddressSpace::from(0))
            .const_zero()
            .into(),
    }
}
