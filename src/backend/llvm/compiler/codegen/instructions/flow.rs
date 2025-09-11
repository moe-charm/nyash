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
    if let Some(list) = phis_by_block.get(target) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap.get(in_vid).ok_or("phi incoming value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value".to_string()),
                }
            }
        }
    }
    let tbb = *bb_map.get(target).ok_or("target bb missing")?;
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
    // then
    if let Some(list) = phis_by_block.get(then_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (then) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (then)".to_string()),
                }
            }
        }
    }
    // else
    if let Some(list) = phis_by_block.get(else_bb) {
        for (_dst, phi, inputs) in list {
            if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                let val = *vmap
                    .get(in_vid)
                    .ok_or("phi incoming (else) value missing")?;
                let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                match val {
                    BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                    BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                    BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                    _ => return Err("unsupported phi incoming value (else)".to_string()),
                }
            }
        }
    }
    let tbb = *bb_map.get(then_bb).ok_or("then bb missing")?;
    let ebb = *bb_map.get(else_bb).ok_or("else bb missing")?;
    codegen
        .builder
        .build_conditional_branch(b, tbb, ebb)
        .map_err(|e| e.to_string())?;
    Ok(())
}
