use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, IntValue, PhiValue};
use std::collections::HashMap;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

use super::super::types::{to_bool, map_mirtype_to_basic};
use super::builder_cursor::BuilderCursor;

pub(in super::super) fn emit_return<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    _bid: BasicBlockId,
    func: &MirFunction,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    value: &Option<ValueId>,
) -> Result<(), String> {
    match (&func.signature.return_type, value) {
        (crate::mir::MirType::Void, _) => {
            cursor.emit_term(_bid, |b| { b.build_return(None).unwrap(); });
            Ok(())
        }
        (_t, Some(vid)) => {
            let v = *vmap.get(vid).ok_or("ret value missing")?;
            // If function expects a pointer but we have an integer handle, convert i64 -> ptr
            let expected = map_mirtype_to_basic(codegen.context, &func.signature.return_type);
            use inkwell::types::BasicTypeEnum as BT;
            let v_adj = match (expected, v) {
                (BT::PointerType(pt), BasicValueEnum::IntValue(iv)) => {
                    cursor.emit_instr(_bid, |b| b
                        .build_int_to_ptr(iv, pt, "ret_i2p"))
                        .map_err(|e| e.to_string())?
                        .into()
                }
                _ => v,
            };
            cursor.emit_term(_bid, |b| {
                b.build_return(Some(&v_adj)).map_err(|e| e.to_string()).unwrap();
            });
            Ok(())
        }
        (_t, None) => Err("non-void function missing return value".to_string()),
    }
}

pub(in super::super) fn emit_jump<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    bid: BasicBlockId,
    target: &BasicBlockId,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    // Non-sealed incoming wiring removed: rely on sealed snapshots and resolver-driven PHIs.
    let tbb = *bb_map.get(target).ok_or("target bb missing")?;
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[LLVM] emit_jump: {} -> {}", bid.as_u32(), target.as_u32());
    }
    cursor.emit_term(bid, |b| {
        b.build_unconditional_branch(tbb).map_err(|e| e.to_string()).unwrap();
    });
    Ok(())
}

pub(in super::super) fn emit_branch<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    resolver: &mut super::Resolver<'ctx>,
    bid: BasicBlockId,
    condition: &ValueId,
    then_bb: &BasicBlockId,
    else_bb: &BasicBlockId,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    preds: &HashMap<BasicBlockId, Vec<BasicBlockId>>,
    block_end_values: &HashMap<BasicBlockId, HashMap<ValueId, BasicValueEnum<'ctx>>>,
) -> Result<(), String> {
    // Localize condition as i64 and convert to i1 via != 0
    let cond_v = *vmap.get(condition).ok_or("cond missing")?;
    let b = match cond_v {
        BasicValueEnum::IntValue(_) | BasicValueEnum::PointerValue(_) | BasicValueEnum::FloatValue(_) => {
            let ci = resolver.resolve_i64(codegen, cursor, bid, *condition, bb_map, preds, block_end_values, vmap)?;
            let zero = codegen.context.i64_type().const_zero();
            codegen.builder.build_int_compare(inkwell::IntPredicate::NE, ci, zero, "cond_nez").map_err(|e| e.to_string())?
        }
        _ => to_bool(codegen.context, cond_v, &codegen.builder)?,
    };
    // Non-sealed incoming wiring removed: rely on sealed snapshots and resolver-driven PHIs.
    let tbb = *bb_map.get(then_bb).ok_or("then bb missing")?;
    let ebb = *bb_map.get(else_bb).ok_or("else bb missing")?;
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[LLVM] emit_branch: {} -> then {} / else {}", bid.as_u32(), then_bb.as_u32(), else_bb.as_u32());
    }
    cursor.emit_term(bid, |bd| {
        bd.build_conditional_branch(b, tbb, ebb).map_err(|e| e.to_string()).unwrap();
    });
    Ok(())
}

// Coerce a value to the PHI node's type, inserting casts in the current block if necessary.
fn coerce_to_type<'ctx>(
    codegen: &CodegenContext<'ctx>,
    phi: &PhiValue<'ctx>,
    val: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    use inkwell::types::BasicTypeEnum as BT;
    match (phi.as_basic_value().get_type(), val) {
        (BT::IntType(it), BasicValueEnum::IntValue(iv)) => {
            let bw_src = iv.get_type().get_bit_width();
            let bw_dst = it.get_bit_width();
            if bw_src == bw_dst {
                Ok(iv.into())
            } else if bw_src < bw_dst {
                Ok(codegen
                    .builder
                    .build_int_z_extend(iv, it, "phi_zext")
                    .map_err(|e| e.to_string())?
                    .into())
            } else if bw_dst == 1 {
                // Narrow to i1 via != 0
                Ok(super::super::types::to_bool(codegen.context, iv.into(), &codegen.builder)?.into())
            } else {
                Ok(codegen
                    .builder
                    .build_int_truncate(iv, it, "phi_trunc")
                    .map_err(|e| e.to_string())?
                    .into())
            }
        }
        (BT::IntType(it), BasicValueEnum::PointerValue(pv)) => Ok(codegen
            .builder
            .build_ptr_to_int(pv, it, "phi_p2i")
            .map_err(|e| e.to_string())?
            .into()),
        (BT::IntType(it), BasicValueEnum::FloatValue(fv)) => Ok(codegen
            .builder
            .build_float_to_signed_int(fv, it, "phi_f2i")
            .map_err(|e| e.to_string())?
            .into()),
        (BT::PointerType(pt), BasicValueEnum::IntValue(iv)) => Ok(codegen
            .builder
            .build_int_to_ptr(iv, pt, "phi_i2p")
            .map_err(|e| e.to_string())?
            .into()),
        (BT::PointerType(_), BasicValueEnum::PointerValue(pv)) => Ok(pv.into()),
        (BT::FloatType(ft), BasicValueEnum::IntValue(iv)) => Ok(codegen
            .builder
            .build_signed_int_to_float(iv, ft, "phi_i2f")
            .map_err(|e| e.to_string())?
            .into()),
        (BT::FloatType(_), BasicValueEnum::FloatValue(fv)) => Ok(fv.into()),
        // Already matching or unsupported combination
        (_, v) => Ok(v),
    }
}

/// Sealed-SSA style: when a block is finalized, add PHI incoming for all successor blocks.
pub(in super::super) fn seal_block<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    func: &MirFunction,
    bid: BasicBlockId,
    succs: &HashMap<BasicBlockId, Vec<BasicBlockId>>,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    // Snapshot of value map at end of each predecessor block
    block_end_values: &HashMap<BasicBlockId, HashMap<ValueId, BasicValueEnum<'ctx>>>,
    // Fallback: current vmap (used only if snapshot missing)
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    if let Some(slist) = succs.get(&bid) {
        for sb in slist {
            if let Some(pl) = phis_by_block.get(sb) {
                for (_dst, phi, inputs) in pl {
                    // Handle only the current predecessor (bid)
                    if let Some((_, in_vid)) = inputs.iter().find(|(p, _)| p == &bid) {
                        // Prefer the predecessor's block-end snapshot; fall back to current vmap
                        let snap_opt = block_end_values
                                .get(&bid)
                                .and_then(|m| m.get(in_vid).copied());
                        let mut val = if let Some(sv) = snap_opt {
                            sv
                        } else {
                            // Trust vmap only when the value is a function parameter (dominates all paths)
                            if func.params.contains(in_vid) {
                                vmap.get(in_vid).copied().unwrap_or_else(|| {
                                    let bt = phi.as_basic_value().get_type();
                                    use inkwell::types::BasicTypeEnum as BT;
                                    match bt {
                                        BT::IntType(it) => it.const_zero().into(),
                                        BT::FloatType(ft) => ft.const_zero().into(),
                                        BT::PointerType(pt) => pt.const_zero().into(),
                                        _ => unreachable!(),
                                    }
                                })
                            } else {
                                // Synthesize zero to avoid dominance violations
                                let bt = phi.as_basic_value().get_type();
                                use inkwell::types::BasicTypeEnum as BT;
                                match bt {
                                    BT::IntType(it) => it.const_zero().into(),
                                    BT::FloatType(ft) => ft.const_zero().into(),
                                    BT::PointerType(pt) => pt.const_zero().into(),
                                    _ => return Err(format!(
                                            "phi incoming (seal) missing: pred={} succ_bb={} in_vid={} (no snapshot)",
                                            bid.as_u32(), sb.as_u32(), in_vid.as_u32()
                                        )),
                                }
                            }
                        };
                            // Insert any required casts in the predecessor block, right before its terminator
                            let saved_block = codegen.builder.get_insert_block();
                            if let Some(pred_llbb) = bb_map.get(&bid) {
                                let term = unsafe { pred_llbb.get_terminator() };
                                if let Some(t) = term {
                                    codegen.builder.position_before(&t);
                                } else {
                                    codegen.builder.position_at_end(*pred_llbb);
                                }
                            }
                            val = coerce_to_type(codegen, phi, val)?;
                            if let Some(bb) = saved_block { codegen.builder.position_at_end(bb); }
                            let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                let tys = phi
                                    .as_basic_value()
                                    .get_type()
                                    .print_to_string()
                                    .to_string();
                                eprintln!(
                                    "[PHI] sealed add pred_bb={} val={} ty={}{}",
                                    bid.as_u32(),
                                    in_vid.as_u32(),
                                    tys,
                                    if snap_opt.is_some() { " (snapshot)" } else { " (vmap)" }
                                );
                            }
                            match val {
                                BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                                BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                                BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                                _ => return Err("unsupported phi incoming value (seal)".to_string()),
                            }
                    } else {
                        // Missing mapping for this predecessor: synthesize a typed zero
                            let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                            use inkwell::types::BasicTypeEnum as BT;
                            let bt = phi.as_basic_value().get_type();
                            let z: BasicValueEnum = match bt {
                                BT::IntType(it) => it.const_zero().into(),
                                BT::FloatType(ft) => ft.const_zero().into(),
                                BT::PointerType(pt) => pt.const_zero().into(),
                                _ => return Err("unsupported phi type for zero synth (seal)".to_string()),
                            };
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!(
                                    "[PHI] sealed add (synth) pred_bb={} zero-ty={}",
                                    bid.as_u32(),
                                    bt.print_to_string().to_string()
                                );
                            }
                            match z {
                                BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                                BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                                BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                                _ => return Err("unsupported phi incoming (synth)".to_string()),
                            }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Normalize PHI incoming entries for a successor block, ensuring exactly
/// one entry per predecessor. This runs once all preds have been sealed.
pub(in super::super) fn finalize_phis<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    func: &MirFunction,
    succ_bb: BasicBlockId,
    preds: &HashMap<BasicBlockId, Vec<BasicBlockId>>,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    block_end_values: &HashMap<BasicBlockId, HashMap<ValueId, BasicValueEnum<'ctx>>>,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    let pred_list = preds.get(&succ_bb).cloned().unwrap_or_default();
    if pred_list.is_empty() { return Ok(()); }
    if let Some(phis) = phis_by_block.get(&succ_bb) {
        for (_dst, phi, inputs) in phis {
            for pred in &pred_list {
                // If this phi expects a value from pred, find the associated Mir ValueId
                if let Some((_, in_vid)) = inputs.iter().find(|(p, _)| p == pred) {
                    // If an incoming from this pred already exists, skip
                    // Note: inkwell does not expose an iterator over incoming; rely on the fact
                    // we add at most once per pred in seal_block. If duplicates occurred earlier,
                    // adding again is harmlessly ignored by verifier if identical; otherwise rely on our new regime.
                    // Fetch value snapshot at end of pred; fallback per our policy
                    let snap_opt = block_end_values.get(pred).and_then(|m| m.get(in_vid).copied());
                    let mut val = if let Some(sv) = snap_opt {
                        sv
                    } else if func.params.contains(in_vid) {
                        vmap.get(in_vid).copied().unwrap_or_else(|| {
                            let bt = phi.as_basic_value().get_type();
                            use inkwell::types::BasicTypeEnum as BT;
                            match bt {
                                BT::IntType(it) => it.const_zero().into(),
                                BT::FloatType(ft) => ft.const_zero().into(),
                                BT::PointerType(pt) => pt.const_zero().into(),
                                _ => unreachable!(),
                            }
                        })
                    } else {
                        let bt = phi.as_basic_value().get_type();
                        use inkwell::types::BasicTypeEnum as BT;
                        match bt {
                            BT::IntType(it) => it.const_zero().into(),
                            BT::FloatType(ft) => ft.const_zero().into(),
                            BT::PointerType(pt) => pt.const_zero().into(),
                            _ => return Err(format!(
                                "phi incoming (finalize) missing: pred={} succ_bb={} in_vid={} (no snapshot)",
                                pred.as_u32(), succ_bb.as_u32(), in_vid.as_u32()
                            )),
                        }
                    };
                    // Insert casts in pred block, just before its terminator
                    let saved_block = codegen.builder.get_insert_block();
                    if let Some(pred_llbb) = bb_map.get(pred) {
                        let term = unsafe { pred_llbb.get_terminator() };
                        if let Some(t) = term { codegen.builder.position_before(&t); }
                        else { codegen.builder.position_at_end(*pred_llbb); }
                    }
                    val = coerce_to_type(codegen, phi, val)?;
                    if let Some(bb) = saved_block { codegen.builder.position_at_end(bb); }
                    let pred_bb = *bb_map.get(pred).ok_or("pred bb missing")?;
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!(
                            "[PHI] finalize add pred_bb={} val={} ty={}",
                            pred.as_u32(), in_vid.as_u32(),
                            phi.as_basic_value().get_type().print_to_string().to_string()
                        );
                    }
                    match val {
                        BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                        BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                        BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                        _ => return Err("unsupported phi incoming value (finalize)".to_string()),
                    }
                } else {
                    // This PHI lacks a mapping for this predecessor entirely; synthesize zero
                    let pred_bb = *bb_map.get(pred).ok_or("pred bb missing")?;
                    use inkwell::types::BasicTypeEnum as BT;
                    let bt = phi.as_basic_value().get_type();
                    let z: BasicValueEnum = match bt {
                        BT::IntType(it) => it.const_zero().into(),
                        BT::FloatType(ft) => ft.const_zero().into(),
                        BT::PointerType(pt) => pt.const_zero().into(),
                        _ => return Err("unsupported phi type for zero synth (finalize)".to_string()),
                    };
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!(
                            "[PHI] finalize add (synth) pred_bb={} zero-ty={}",
                            pred.as_u32(), bt.print_to_string().to_string()
                        );
                    }
                    match z {
                        BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                        BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                        BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                        _ => return Err("unsupported phi incoming (synth finalize)".to_string()),
                    }
                }
            }
        }
    }
    Ok(())
}

/// Localize a MIR value as an i64 in the current block by creating a PHI that merges
/// predecessor snapshots. This avoids using values defined in non-dominating blocks.
/// Sealed SSA mode is assumed; when a predecessor snapshot is missing, synthesize zero.
pub(in super::super) fn localize_to_i64<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    vid: ValueId,
    bb_map: &std::collections::HashMap<BasicBlockId, BasicBlock<'ctx>>,
    preds: &std::collections::HashMap<BasicBlockId, Vec<BasicBlockId>>,
    block_end_values: &std::collections::HashMap<BasicBlockId, std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>>,
    vmap: &std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<IntValue<'ctx>, String> {
    let i64t = codegen.context.i64_type();
    let cur_llbb = *bb_map.get(&cur_bid).ok_or("cur bb missing")?;
    // If no predecessors, fallback to current vmap or zero
    let pred_list = preds.get(&cur_bid).cloned().unwrap_or_default();
    if pred_list.is_empty() {
        if let Some(v) = vmap.get(&vid).copied() {
            return Ok(match v {
                BasicValueEnum::IntValue(iv) => {
                    if iv.get_type() == i64t { iv }
                    else { codegen.builder.build_int_z_extend(iv, i64t, "loc_zext").map_err(|e| e.to_string())? }
                }
                BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "loc_p2i").map_err(|e| e.to_string())?,
                BasicValueEnum::FloatValue(fv) => codegen.builder.build_float_to_signed_int(fv, i64t, "loc_f2i").map_err(|e| e.to_string())?,
                _ => i64t.const_zero(),
            });
        }
        return Ok(i64t.const_zero());
    }
    // Build PHI at the top of current block (before any non-PHI), then restore insertion point
    let saved_ip = codegen.builder.get_insert_block();
    if let Some(first) = cur_llbb.get_first_instruction() {
        codegen.builder.position_before(&first);
    } else {
        codegen.builder.position_at_end(cur_llbb);
    }
    let phi = codegen.builder.build_phi(i64t, &format!("loc_i64_{}", vid.as_u32())).map_err(|e| e.to_string())?;
    for p in &pred_list {
        let pred_bb = *bb_map.get(p).ok_or("pred bb missing")?;
        // Fetch snapshot at end of pred; if missing, synthesize zero
        let mut val = block_end_values
            .get(p)
            .and_then(|m| m.get(&vid).copied())
            .unwrap_or_else(|| i64t.const_zero().into());
        // Coerce to i64
        use inkwell::types::BasicTypeEnum as BT;
        val = match val {
            BasicValueEnum::IntValue(iv) => {
                if iv.get_type() == i64t { iv.into() }
                else { codegen.builder.build_int_z_extend(iv, i64t, "loc_zext_p").map_err(|e| e.to_string())?.into() }
            }
            BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "loc_p2i_p").map_err(|e| e.to_string())?.into(),
            BasicValueEnum::FloatValue(fv) => codegen.builder.build_float_to_signed_int(fv, i64t, "loc_f2i_p").map_err(|e| e.to_string())?.into(),
            _ => i64t.const_zero().into(),
        };
        match val {
            BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
            _ => unreachable!(),
        }
    }
    // Restore insertion point
    if let Some(bb) = saved_ip { codegen.builder.position_at_end(bb); }
    Ok(phi.as_basic_value().into_int_value())
}
