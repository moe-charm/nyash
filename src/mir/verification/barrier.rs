use crate::mir::{function::MirFunction, MirInstruction};
use crate::mir::verification_types::VerificationError;

/// Verify WeakRef/Barrier minimal semantics
pub fn check_weakref_and_barrier(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    use crate::mir::{BasicBlockId, ValueId};
    let mut errors = Vec::new();
    // Build def map value -> (block, idx, &inst)
    let mut def_map: std::collections::HashMap<ValueId, (BasicBlockId, usize, &MirInstruction)> = std::collections::HashMap::new();
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.all_instructions().enumerate() {
            if let Some(dst) = inst.dst_value() { def_map.insert(dst, (*bid, idx, inst)); }
        }
    }
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.all_instructions().enumerate() {
            match inst {
                MirInstruction::WeakRef { op: crate::mir::WeakRefOp::Load, value, .. } => {
                    match def_map.get(value) {
                        Some((_db, _di, def_inst)) => match def_inst {
                            MirInstruction::WeakRef { op: crate::mir::WeakRefOp::New, .. } | MirInstruction::WeakNew { .. } => {}
                            _ => errors.push(VerificationError::InvalidWeakRefSource {
                                weak_ref: *value, block: *bid, instruction_index: idx,
                                reason: "weakref.load source is not a weakref.new/weak_new".to_string(),
                            }),
                        },
                        None => errors.push(VerificationError::InvalidWeakRefSource {
                            weak_ref: *value, block: *bid, instruction_index: idx,
                            reason: "weakref.load source is undefined".to_string(),
                        }),
                    }
                }
                MirInstruction::WeakLoad { weak_ref, .. } => {
                    match def_map.get(weak_ref) {
                        Some((_db, _di, def_inst)) => match def_inst {
                            MirInstruction::WeakNew { .. } | MirInstruction::WeakRef { op: crate::mir::WeakRefOp::New, .. } => {}
                            _ => errors.push(VerificationError::InvalidWeakRefSource {
                                weak_ref: *weak_ref, block: *bid, instruction_index: idx,
                                reason: "weak_load source is not a weak_new/weakref.new".to_string(),
                            }),
                        },
                        None => errors.push(VerificationError::InvalidWeakRefSource {
                            weak_ref: *weak_ref, block: *bid, instruction_index: idx,
                            reason: "weak_load source is undefined".to_string(),
                        }),
                    }
                }
                MirInstruction::Barrier { ptr, .. } | MirInstruction::BarrierRead { ptr } | MirInstruction::BarrierWrite { ptr } => {
                    if let Some((_db, _di, def_inst)) = def_map.get(ptr) {
                        if let MirInstruction::Const { value: crate::mir::instruction::ConstValue::Void, .. } = def_inst {
                            errors.push(VerificationError::InvalidBarrierPointer {
                                ptr: *ptr, block: *bid, instruction_index: idx,
                                reason: "barrier pointer is void".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Light diagnostic: Barrier should be near memory ops in the same block (best-effort)
/// Enabled only when NYASH_VERIFY_BARRIER_STRICT=1
pub fn check_barrier_context(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let strict = std::env::var("NYASH_VERIFY_BARRIER_STRICT").ok().as_deref() == Some("1");
    if !strict { return Ok(()); }

    let mut errors = Vec::new();
    for (bid, block) in &function.blocks {
        let mut insts: Vec<(usize, &MirInstruction)> = block.instructions.iter().enumerate().collect();
        if let Some(term) = &block.terminator { insts.push((usize::MAX, term)); }
        for (idx, inst) in &insts {
            let is_barrier = matches!(inst,
                MirInstruction::Barrier { .. } |
                MirInstruction::BarrierRead { .. } |
                MirInstruction::BarrierWrite { .. }
            );
            if !is_barrier { continue; }

            // Look around +-2 instructions for a memory op hint
            let mut has_mem_neighbor = false;
            for (j, other) in &insts {
                if *j == *idx { continue; }
                let dist = if *idx == usize::MAX || *j == usize::MAX { 99 } else { idx.max(j) - idx.min(j) };
                if dist > 2 { continue; }
                if matches!(other,
                    MirInstruction::Load { .. } |
                    MirInstruction::Store { .. } |
                    MirInstruction::ArrayGet { .. } |
                    MirInstruction::ArraySet { .. } |
                    MirInstruction::RefGet { .. } |
                    MirInstruction::RefSet { .. }
                ) { has_mem_neighbor = true; break; }
            }
            if !has_mem_neighbor {
                errors.push(VerificationError::SuspiciousBarrierContext {
                    block: *bid,
                    instruction_index: *idx,
                    note: "barrier without nearby memory op (±2 inst)".to_string(),
                });
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
