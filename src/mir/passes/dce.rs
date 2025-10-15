//! Dead Code Elimination (pure instruction DCE)
//!
//! Extracted from the monolithic optimizer to enable modular pass composition.

use crate::mir::{MirFunction, MirModule, ValueId};
use std::collections::HashSet;

/// Eliminate dead code (unused results of pure instructions) across the module.
/// Returns the number of eliminated instructions.
pub fn eliminate_dead_code(module: &mut MirModule) -> usize {
    let mut eliminated_total = 0usize;
    for (_func_name, func) in &mut module.functions {
        eliminated_total += eliminate_dead_code_in_function(func);
    }
    eliminated_total
}

fn eliminate_dead_code_in_function(function: &mut MirFunction) -> usize {
    // Collect values that must be kept (used results + effects)
    let mut used_values: HashSet<ValueId> = HashSet::new();

    // Mark values used by side-effecting instructions and terminators
    for (_bid, block) in &function.blocks {
        for instruction in &block.instructions {
            if !instruction.effects().is_pure() {
                if let Some(dst) = instruction.dst_value() {
                    used_values.insert(dst);
                }
                for u in instruction.used_values() {
                    used_values.insert(u);
                }
            }
        }
        if let Some(term) = &block.terminator {
            for u in term.used_values() {
                used_values.insert(u);
            }
        }
    }

    // Backward propagation: if a value is used, mark its operands as used
    let mut changed = true;
    while changed {
        changed = false;
        for (_bid, block) in &function.blocks {
            for instruction in &block.instructions {
                if let Some(dst) = instruction.dst_value() {
                    if used_values.contains(&dst) {
                        for u in instruction.used_values() {
                            if used_values.insert(u) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove unused pure instructions
    let mut eliminated = 0usize;
    for (_bbid, block) in &mut function.blocks {
        block.instructions.retain(|inst| {
            if inst.effects().is_pure() {
                if let Some(dst) = inst.dst_value() {
                    if !used_values.contains(&dst) {
                        // Keep indices stable is not required here; remove entirely
                        // Logging is suppressed to keep pass quiet by default
                        eliminated += 1;
                        return false;
                    }
                }
            }
            true
        });
    }
    if eliminated > 0 {
        function.update_cfg();
    }
    eliminated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        definitions::call_unified::{Callee, TypeCertainty},
        EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType,
    };
    use crate::mir::basic_block::BasicBlock;
    use crate::mir::types::ConstValue;

    #[test]
    fn receiver_copy_survives_single_block() {
        let mut module = MirModule::new("single".to_string());
        let entry = crate::mir::basic_block::BasicBlockId(0);
        let signature = FunctionSignature {
            name: "f".into(),
            params: Vec::new(),
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        let mut func = MirFunction::new(signature, entry);

        let recv_src = func.next_value_id();
        let func_sym = func.next_value_id();
        let copy_val = func.next_value_id();
        let call_dst = func.next_value_id();

        {
            let block = func.get_block_mut(entry).unwrap();
            block.add_instruction(MirInstruction::Const {
                dst: recv_src,
                value: ConstValue::String("hello".into()),
            });
            block.add_instruction(MirInstruction::Const {
                dst: func_sym,
                value: ConstValue::String("StringBox.len/0".into()),
            });
            block.add_instruction(MirInstruction::Copy { dst: copy_val, src: recv_src });
            block.add_instruction(MirInstruction::Call {
                dst: Some(call_dst),
                func: func_sym,
                callee: Some(Callee::Method {
                    box_name: "StringBox".into(),
                    method: "len".into(),
                    receiver: Some(copy_val),
                    certainty: TypeCertainty::Known,
                }),
                args: Vec::new(),
                effects: EffectMask::READ,
            });
            block.add_instruction(MirInstruction::Return { value: Some(call_dst) });
        }

        func.update_cfg();
        module.add_function(func);

        let removed = eliminate_dead_code(&mut module);
        assert_eq!(removed, 0);

        let block = module.get_function("f").unwrap().get_block(entry).unwrap();
        assert!(block.instructions.iter().any(|inst| {
            matches!(inst, MirInstruction::Copy { dst, src } if *dst == copy_val && *src == recv_src)
        }));
    }

    #[test]
    fn receiver_copy_survives_branch_flow() {
        let mut module = MirModule::new("branch".to_string());
        let entry = crate::mir::basic_block::BasicBlockId(0);
        let then_bb = crate::mir::basic_block::BasicBlockId(1);
        let else_bb = crate::mir::basic_block::BasicBlockId(2);
        let signature = FunctionSignature {
            name: "g".into(),
            params: Vec::new(),
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        let mut func = MirFunction::new(signature, entry);
        func.add_block(BasicBlock::new(then_bb));
        func.add_block(BasicBlock::new(else_bb));

        let cond_val = func.next_value_id();
        let recv_src = func.next_value_id();
        let copy_val = func.next_value_id();

        {
            let entry_block = func.get_block_mut(entry).unwrap();
            entry_block.add_instruction(MirInstruction::Const {
                dst: cond_val,
                value: ConstValue::Bool(true),
            });
            entry_block.add_instruction(MirInstruction::Const {
                dst: recv_src,
                value: ConstValue::String("branch".into()),
            });
            entry_block.add_instruction(MirInstruction::Copy { dst: copy_val, src: recv_src });
            entry_block.add_instruction(MirInstruction::Branch {
                condition: cond_val,
                then_bb,
                else_bb,
            });
        }

        {
            let func_sym = func.next_value_id();
            let call_dst = func.next_value_id();
            let then_block = func.get_block_mut(then_bb).unwrap();
            then_block.add_instruction(MirInstruction::Const {
                dst: func_sym,
                value: ConstValue::String("StringBox.len/0".into()),
            });
            then_block.add_instruction(MirInstruction::Call {
                dst: Some(call_dst),
                func: func_sym,
                callee: Some(Callee::Method {
                    box_name: "StringBox".into(),
                    method: "len".into(),
                    receiver: Some(copy_val),
                    certainty: TypeCertainty::Known,
                }),
                args: Vec::new(),
                effects: EffectMask::READ,
            });
            then_block.add_instruction(MirInstruction::Return { value: Some(call_dst) });
        }

        {
            let else_block = func.get_block_mut(else_bb).unwrap();
            else_block.add_instruction(MirInstruction::Return { value: None });
        }

        func.update_cfg();
        module.add_function(func);

        let removed = eliminate_dead_code(&mut module);
        assert_eq!(removed, 0);

        let entry_block = module.get_function("g").unwrap().get_block(entry).unwrap();
        assert!(entry_block.instructions.iter().any(|inst| {
            matches!(inst, MirInstruction::Copy { dst, src } if *dst == copy_val && *src == recv_src)
        }));
    }

    #[test]
    fn receiver_copy_survives_loop_header() {
        let mut module = MirModule::new("loop".to_string());
        let entry = crate::mir::basic_block::BasicBlockId(0);
        let header = crate::mir::basic_block::BasicBlockId(1);
        let body = crate::mir::basic_block::BasicBlockId(2);
        let exit = crate::mir::basic_block::BasicBlockId(3);
        let signature = FunctionSignature {
            name: "h".into(),
            params: Vec::new(),
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        let mut func = MirFunction::new(signature, entry);
        func.add_block(BasicBlock::new(header));
        func.add_block(BasicBlock::new(body));
        func.add_block(BasicBlock::new(exit));

        let recv_src = func.next_value_id();
        let entry_flag = func.next_value_id();
        let loop_flag = func.next_value_id();
        let body_flag = func.next_value_id();
        let copy_val = func.next_value_id();
        let func_sym = func.next_value_id();
        let call_dst = func.next_value_id();

        {
            let entry_block = func.get_block_mut(entry).unwrap();
            entry_block.add_instruction(MirInstruction::Const {
                dst: recv_src,
                value: ConstValue::String("loop".into()),
            });
            entry_block.add_instruction(MirInstruction::Const {
                dst: entry_flag,
                value: ConstValue::Bool(true),
            });
            entry_block.add_instruction(MirInstruction::Jump { target: header });
        }

        {
            let header_block = func.get_block_mut(header).unwrap();
            header_block.add_instruction(MirInstruction::Phi {
                dst: loop_flag,
                inputs: vec![(entry, entry_flag), (body, body_flag)],
            });
            header_block.add_instruction(MirInstruction::Copy { dst: copy_val, src: recv_src });
            header_block.add_instruction(MirInstruction::Branch {
                condition: loop_flag,
                then_bb: body,
                else_bb: exit,
            });
        }

        {
            let body_block = func.get_block_mut(body).unwrap();
            body_block.add_instruction(MirInstruction::Const {
                dst: func_sym,
                value: ConstValue::String("StringBox.len/0".into()),
            });
            body_block.add_instruction(MirInstruction::Call {
                dst: Some(call_dst),
                func: func_sym,
                callee: Some(Callee::Method {
                    box_name: "StringBox".into(),
                    method: "len".into(),
                    receiver: Some(copy_val),
                    certainty: TypeCertainty::Known,
                }),
                args: Vec::new(),
                effects: EffectMask::READ,
            });
            body_block.add_instruction(MirInstruction::Const {
                dst: body_flag,
                value: ConstValue::Bool(false),
            });
            body_block.add_instruction(MirInstruction::Jump { target: header });
        }

        {
            let exit_block = func.get_block_mut(exit).unwrap();
            exit_block.add_instruction(MirInstruction::Return { value: Some(call_dst) });
        }

        func.update_cfg();
        module.add_function(func);

        let removed = eliminate_dead_code(&mut module);
        assert_eq!(removed, 0);

        let header_block = module.get_function("h").unwrap().get_block(header).unwrap();
        assert!(header_block.instructions.iter().any(|inst| {
            matches!(inst, MirInstruction::Copy { dst, src } if *dst == copy_val && *src == recv_src)
        }));
    }
}
