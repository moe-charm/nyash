use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, PhiValue};
use std::collections::HashMap;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

use super::super::types::{to_bool, map_mirtype_to_basic};

pub(in super::super) fn emit_return<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    value: &Option<ValueId>,
) -> Result<(), String> {
    match (&func.signature.return_type, value) {
        (crate::mir::MirType::Void, _) => {
            codegen.builder.build_return(None).unwrap();
            Ok(())
        }
        (_t, Some(vid)) => {
            let v = *vmap.get(vid).ok_or("ret value missing")?;
            // If function expects a pointer but we have an integer handle, convert i64 -> ptr
            let expected = map_mirtype_to_basic(codegen.context, &func.signature.return_type);
            use inkwell::types::BasicTypeEnum as BT;
            let v_adj = match (expected, v) {
                (BT::PointerType(pt), BasicValueEnum::IntValue(iv)) => {
                    codegen
                        .builder
                        .build_int_to_ptr(iv, pt, "ret_i2p")
                        .map_err(|e| e.to_string())?
                        .into()
                }
                _ => v,
            };
            codegen
                .builder
                .build_return(Some(&v_adj))
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        (_t, None) => Err("non-void function missing return value".to_string()),
    }
}

pub(in super::super) fn emit_jump<'ctx>(
    codegen: &CodegenContext<'ctx>,
    bid: BasicBlockId,
    target: &BasicBlockId,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    phis_by_block: &HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    >,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<(), String> {
    let sealed = std::env::var("NYASH_LLVM_PHI_SEALED").ok().as_deref() == Some("1");
    if !sealed {
    if let Some(list) = phis_by_block.get(target) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let mut val = *vmap.get(in_vid).ok_or("phi incoming value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                // Coerce incoming to PHI type when needed
                val = coerce_to_type(codegen, phi, val)?;
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    let tys = phi
                        .as_basic_value()
                        .get_type()
                        .print_to_string()
                        .to_string();
                    eprintln!(
                        "[PHI] incoming add pred_bb={} val={} ty={}",
                        bid.as_u32(),
                        in_vid.as_u32(),
                        tys
                    );
                }
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value".to_string()),
                }
            }
        }
    }
    }
    let tbb = *bb_map.get(target).ok_or("target bb missing")?;
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[LLVM] emit_jump: {} -> {}", bid.as_u32(), target.as_u32());
    }
    codegen
        .builder
        .build_unconditional_branch(tbb)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(in super::super) fn emit_branch<'ctx>(
    codegen: &CodegenContext<'ctx>,
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
) -> Result<(), String> {
    let cond_v = *vmap.get(condition).ok_or("cond missing")?;
    let b = to_bool(codegen.context, cond_v, &codegen.builder)?;
    let sealed = std::env::var("NYASH_LLVM_PHI_SEALED").ok().as_deref() == Some("1");
    // then
    if !sealed {
    if let Some(list) = phis_by_block.get(then_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let mut val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (then) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                val = coerce_to_type(codegen, phi, val)?;
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    let tys = phi
                        .as_basic_value()
                        .get_type()
                        .print_to_string()
                        .to_string();
                    eprintln!(
                        "[PHI] incoming add (then) pred_bb={} val={} ty={}",
                        bid.as_u32(),
                        in_vid.as_u32(),
                        tys
                    );
                }
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (then)".to_string()),
                }
            }
        }
    }
    }
    // else
    if !sealed {
    if let Some(list) = phis_by_block.get(else_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let mut val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (else) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                val = coerce_to_type(codegen, phi, val)?;
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    let tys = phi
                        .as_basic_value()
                        .get_type()
                        .print_to_string()
                        .to_string();
                    eprintln!(
                        "[PHI] incoming add (else) pred_bb={} val={} ty={}",
                        bid.as_u32(),
                        in_vid.as_u32(),
                        tys
                    );
                }
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (else)".to_string()),
                }
            }
        }
    }
    }
    let tbb = *bb_map.get(then_bb).ok_or("then bb missing")?;
    let ebb = *bb_map.get(else_bb).ok_or("else bb missing")?;
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[LLVM] emit_branch: {} -> then {} / else {}", bid.as_u32(), then_bb.as_u32(), else_bb.as_u32());
    }
    codegen
        .builder
        .build_conditional_branch(b, tbb, ebb)
        .map_err(|e| e.to_string())?;
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
pub(in super::super) fn seal_block<'ctx>(
    codegen: &CodegenContext<'ctx>,
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
                    if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                        // Prefer the predecessor's block-end snapshot; fall back to current vmap
                        let snap_opt = block_end_values
                            .get(&bid)
                            .and_then(|m| m.get(in_vid).copied());
                        let mut val = if let Some(sv) = snap_opt {
                            sv
                        } else {
                            match vmap.get(in_vid).copied() {
                                Some(v) => v,
                                None => {
                                    let msg = format!(
                                        "phi incoming (seal) missing: pred={} succ_bb={} in_vid={} (no snapshot)",
                                        bid.as_u32(), sb.as_u32(), in_vid.as_u32()
                                    );
                                    return Err(msg);
                                }
                            }
                        };
                        let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                        val = coerce_to_type(codegen, phi, val)?;
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
                    }
                }
            }
        }
    }
    Ok(())
}
