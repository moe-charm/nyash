use std::collections::HashMap;

use inkwell::values::BasicValueEnum;

use super::builder_cursor::BuilderCursor;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, CompareOp, ValueId};

/// Compare lowering: return the resulting BasicValueEnum (i1)
pub(in super::super) fn lower_compare<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    cur_bid: BasicBlockId,
    func: &MirFunction,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    op: &CompareOp,
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
) -> Result<BasicValueEnum<'ctx>, String> {
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    // Synthesize proxy values via Resolver according to metadata
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
    // String equality/inequality by content when annotated as String/StringBox
    if matches!(op, CompareOp::Eq | CompareOp::Ne) {
        let l_is_str = match func.metadata.value_types.get(lhs) {
            Some(crate::mir::MirType::String) => true,
            Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
            _ => false,
        };
        let r_is_str = match func.metadata.value_types.get(rhs) {
            Some(crate::mir::MirType::String) => true,
            Some(crate::mir::MirType::Box(b)) if b == "StringBox" => true,
            _ => false,
        };
        if l_is_str && r_is_str {
            let i64t = codegen.context.i64_type();
            // Convert both sides to handles if needed
            let mut to_handle =
                |v: BasicValueEnum<'ctx>| -> Result<inkwell::values::IntValue<'ctx>, String> {
                    match v {
                        BasicValueEnum::IntValue(iv) => {
                            if iv.get_type() == i64t {
                                Ok(iv)
                            } else {
                                cursor
                                    .emit_instr(cur_bid, |b| {
                                        b.build_int_s_extend(iv, i64t, "i2i64")
                                    })
                                    .map_err(|e| e.to_string())
                            }
                        }
                        BasicValueEnum::PointerValue(pv) => {
                            let fnty = i64t.fn_type(
                                &[codegen
                                    .context
                                    .ptr_type(inkwell::AddressSpace::from(0))
                                    .into()],
                                false,
                            );
                            let callee = codegen
                                .module
                                .get_function("nyash.box.from_i8_string")
                                .unwrap_or_else(|| {
                                    codegen.module.add_function(
                                        "nyash.box.from_i8_string",
                                        fnty,
                                        None,
                                    )
                                });
                            let call = cursor
                                .emit_instr(cur_bid, |b| {
                                    b.build_call(callee, &[pv.into()], "str_ptr_to_handle_cmp")
                                })
                                .map_err(|e| e.to_string())?;
                            let rv = call
                                .try_as_basic_value()
                                .left()
                                .ok_or("from_i8_string returned void".to_string())?;
                            Ok(rv.into_int_value())
                        }
                        _ => Err("unsupported value for string compare".to_string()),
                    }
                };
            let lh = to_handle(lv)?;
            let rh = to_handle(rv)?;
            let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
            let callee = codegen
                .module
                .get_function("nyash.string.eq_hh")
                .unwrap_or_else(|| {
                    codegen
                        .module
                        .add_function("nyash.string.eq_hh", fnty, None)
                });
            let call = cursor
                .emit_instr(cur_bid, |b| {
                    b.build_call(callee, &[lh.into(), rh.into()], "str_eq_hh")
                })
                .map_err(|e| e.to_string())?;
            let iv = call
                .try_as_basic_value()
                .left()
                .ok_or("eq_hh returned void".to_string())?
                .into_int_value();
            let zero = i64t.const_zero();
            let pred = if matches!(op, CompareOp::Eq) {
                inkwell::IntPredicate::NE
            } else {
                inkwell::IntPredicate::EQ
            };
            let b = cursor
                .emit_instr(cur_bid, |bd| {
                    bd.build_int_compare(pred, iv, zero, "str_eq_to_bool")
                })
                .map_err(|e| e.to_string())?;
            return Ok(b.into());
        }
    }
    let out = if let (Some(_li0), Some(_ri0)) = (as_int(lv), as_int(rv)) {
        // Localize integer operands into current block to satisfy dominance
        let mut li = resolver
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
            .unwrap_or_else(|_| as_int(lv).unwrap());
        let mut ri = resolver
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
            .unwrap_or_else(|_| as_int(rv).unwrap());
        // Normalize integer widths: extend the narrower to match the wider to satisfy LLVM
        let lw = li.get_type().get_bit_width();
        let rw = ri.get_type().get_bit_width();
        if lw != rw {
            if lw < rw {
                li = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_int_z_extend(li, ri.get_type(), "icmp_zext_l")
                    })
                    .map_err(|e| e.to_string())?;
            } else {
                ri = cursor
                    .emit_instr(cur_bid, |b| {
                        b.build_int_z_extend(ri, li.get_type(), "icmp_zext_r")
                    })
                    .map_err(|e| e.to_string())?;
            }
        }
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::IntPredicate::EQ,
            C::Ne => inkwell::IntPredicate::NE,
            C::Lt => inkwell::IntPredicate::SLT,
            C::Le => inkwell::IntPredicate::SLE,
            C::Gt => inkwell::IntPredicate::SGT,
            C::Ge => inkwell::IntPredicate::SGE,
        };
        cursor
            .emit_instr(cur_bid, |b| b.build_int_compare(pred, li, ri, "icmp"))
            .map_err(|e| e.to_string())?
            .into()
    } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::FloatPredicate::OEQ,
            C::Ne => inkwell::FloatPredicate::ONE,
            C::Lt => inkwell::FloatPredicate::OLT,
            C::Le => inkwell::FloatPredicate::OLE,
            C::Gt => inkwell::FloatPredicate::OGT,
            C::Ge => inkwell::FloatPredicate::OGE,
        };
        cursor
            .emit_instr(cur_bid, |b| b.build_float_compare(pred, lf, rf, "fcmp"))
            .map_err(|e| e.to_string())?
            .into()
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        // Support pointer equality/inequality comparisons
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let li = cursor
                    .emit_instr(cur_bid, |b| b.build_ptr_to_int(lp, i64t, "pi_l"))
                    .map_err(|e| e.to_string())?;
                let ri = cursor
                    .emit_instr(cur_bid, |b| b.build_ptr_to_int(rp, i64t, "pi_r"))
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                cursor
                    .emit_instr(cur_bid, |b| b.build_int_compare(pred, li, ri, "pcmp"))
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported pointer comparison (only Eq/Ne)".to_string()),
        }
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::IntValue(ri)) = (lv, rv) {
        use CompareOp as C;
        let i64t = codegen.context.i64_type();
        let li = cursor
            .emit_instr(cur_bid, |b| b.build_ptr_to_int(lp, i64t, "pi_l"))
            .map_err(|e| e.to_string())?;
        let pred = match op {
            C::Eq => inkwell::IntPredicate::EQ,
            C::Ne => inkwell::IntPredicate::NE,
            C::Lt => inkwell::IntPredicate::SLT,
            C::Le => inkwell::IntPredicate::SLE,
            C::Gt => inkwell::IntPredicate::SGT,
            C::Ge => inkwell::IntPredicate::SGE,
        };
        cursor
            .emit_instr(cur_bid, |b| b.build_int_compare(pred, li, ri, "pcmpi"))
            .map_err(|e| e.to_string())?
            .into()
    } else if let (BasicValueEnum::IntValue(li), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        use CompareOp as C;
        let i64t = codegen.context.i64_type();
        let ri = cursor
            .emit_instr(cur_bid, |b| b.build_ptr_to_int(rp, i64t, "pi_r"))
            .map_err(|e| e.to_string())?;
        let pred = match op {
            C::Eq => inkwell::IntPredicate::EQ,
            C::Ne => inkwell::IntPredicate::NE,
            C::Lt => inkwell::IntPredicate::SLT,
            C::Le => inkwell::IntPredicate::SLE,
            C::Gt => inkwell::IntPredicate::SGT,
            C::Ge => inkwell::IntPredicate::SGE,
        };
        cursor
            .emit_instr(cur_bid, |b| b.build_int_compare(pred, li, ri, "pcmpi"))
            .map_err(|e| e.to_string())?
            .into()
    } else {
        return Err("compare type mismatch".to_string());
    };
    Ok(out)
}

fn guessed_zero<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vid: &crate::mir::ValueId,
) -> BasicValueEnum<'ctx> {
    use crate::mir::MirType as MT;
    match func.metadata.value_types.get(vid) {
        Some(MT::Bool) => codegen.context.bool_type().const_zero().into(),
        Some(MT::Integer) => codegen.context.i64_type().const_zero().into(),
        Some(MT::Float) => codegen.context.f64_type().const_zero().into(),
        Some(MT::String) | Some(MT::Box(_)) | Some(MT::Array(_)) | Some(MT::Future(_))
        | Some(MT::Unknown) | Some(MT::Void) | None => codegen
            .context
            .ptr_type(inkwell::AddressSpace::from(0))
            .const_zero()
            .into(),
    }
}
