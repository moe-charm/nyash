use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PhiValue};
use std::collections::HashMap;

use super::super::types::map_mirtype_to_basic;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

// Small, safe extraction: create LLVM basic blocks for a MIR function and
// return the block map together with the entry block.
pub(in super::super) fn create_basic_blocks<'ctx>(
    codegen: &CodegenContext<'ctx>,
    llvm_func: FunctionValue<'ctx>,
    func: &MirFunction,
    fn_label: &str,
) -> (HashMap<BasicBlockId, BasicBlock<'ctx>>, BasicBlock<'ctx>) {
    let mut bb_map: HashMap<BasicBlockId, BasicBlock> = HashMap::new();
    let entry_first = func.entry_block;
    let entry_bb = codegen.context.append_basic_block(
        llvm_func,
        &format!("{}_bb{}", fn_label, entry_first.as_u32()),
    );
    bb_map.insert(entry_first, entry_bb);
    for bid in func.block_ids() {
        if bid == entry_first {
            continue;
        }
        let name = format!("{}_bb{}", fn_label, bid.as_u32());
        let bb = codegen.context.append_basic_block(llvm_func, &name);
        bb_map.insert(bid, bb);
    }
    (bb_map, entry_bb)
}

// Pre-create PHI nodes for all blocks; also inserts placeholder values into vmap.
pub(in super::super) fn precreate_phis<'ctx>(
    codegen: &CodegenContext<'ctx>,
    func: &MirFunction,
    bb_map: &HashMap<BasicBlockId, BasicBlock<'ctx>>,
    vmap: &mut HashMap<ValueId, BasicValueEnum<'ctx>>,
) -> Result<
    HashMap<BasicBlockId, Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>>,
    String,
> {
    use super::super::types::map_mirtype_to_basic;
    let mut phis_by_block: HashMap<
        BasicBlockId,
        Vec<(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)>,
    > = HashMap::new();
    for bid in func.block_ids() {
        let bb = *bb_map.get(&bid).ok_or("missing bb in map")?;
        codegen.builder.position_at_end(bb);
        let block = func.blocks.get(&bid).unwrap();
        for inst in block
            .instructions
            .iter()
            .take_while(|i| matches!(i, crate::mir::instruction::MirInstruction::Phi { .. }))
        {
            if let crate::mir::instruction::MirInstruction::Phi { dst, inputs } = inst {
                let mut phi_ty: Option<inkwell::types::BasicTypeEnum> = None;
                // Prefer pointer when any input (or dst) is String/Box/Array/Future/Unknown
                let mut wants_ptr = false;
                if let Some(mt) = func.metadata.value_types.get(dst) {
                    wants_ptr |= matches!(
                        mt,
                        crate::mir::MirType::String
                            | crate::mir::MirType::Box(_)
                            | crate::mir::MirType::Array(_)
                            | crate::mir::MirType::Future(_)
                            | crate::mir::MirType::Unknown
                    );
                }
                for (_, iv) in inputs.iter() {
                    if let Some(mt) = func.metadata.value_types.get(iv) {
                        wants_ptr |= matches!(
                            mt,
                            crate::mir::MirType::String
                                | crate::mir::MirType::Box(_)
                                | crate::mir::MirType::Array(_)
                                | crate::mir::MirType::Future(_)
                                | crate::mir::MirType::Unknown
                        );
                    }
                }
                if wants_ptr {
                    phi_ty = Some(
                        codegen
                            .context
                            .ptr_type(inkwell::AddressSpace::from(0))
                            .into(),
                    );
                } else if let Some(mt) = func.metadata.value_types.get(dst) {
                    phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                } else if let Some((_, iv)) = inputs.first() {
                    if let Some(mt) = func.metadata.value_types.get(iv) {
                        phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                    }
                }
                let phi_ty = phi_ty.unwrap_or_else(|| codegen.context.i64_type().into());
                let phi = codegen
                    .builder
                    .build_phi(phi_ty, &format!("phi_{}", dst.as_u32()))
                    .map_err(|e| e.to_string())?;
                vmap.insert(*dst, phi.as_basic_value());
                phis_by_block
                    .entry(bid)
                    .or_default()
                    .push((*dst, phi, inputs.clone()));
            }
        }
    }
    Ok(phis_by_block)
}
