/*!
 * MIR Basic Block - Control Flow Graph Building Block
 * 
 * SSA-form basic blocks with phi functions and terminator instructions
 */

use super::{MirInstruction, ValueId, EffectMask};
use std::collections::HashSet;
use std::fmt;

/// Unique identifier for basic blocks within a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BasicBlockId(pub u32);

impl BasicBlockId {
    /// Create a new BasicBlockId
    pub fn new(id: u32) -> Self {
        BasicBlockId(id)
    }
    
    /// Get the raw ID value
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    /// Create BasicBlockId from usize (for array indexing)
    pub fn from_usize(id: usize) -> Self {
        BasicBlockId(id as u32)
    }
    
    /// Convert to usize (for array indexing)
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A basic block in SSA form
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: BasicBlockId,
    
    /// Instructions in this block (excluding terminator)
    pub instructions: Vec<MirInstruction>,
    
    /// Terminator instruction (branch, jump, or return)
    pub terminator: Option<MirInstruction>,
    
    /// Predecessors in the control flow graph
    pub predecessors: HashSet<BasicBlockId>,
    
    /// Successors in the control flow graph
    pub successors: HashSet<BasicBlockId>,
    
    /// Combined effect mask for all instructions in this block
    pub effects: EffectMask,
    
    /// Whether this block is reachable from the entry block
    pub reachable: bool,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: None,
            predecessors: HashSet::new(),
            successors: HashSet::new(),
            effects: EffectMask::PURE,
            reachable: false,
        }
    }
    
    /// Add an instruction to this block
    pub fn add_instruction(&mut self, instruction: MirInstruction) {
        // Update effect mask
        self.effects = self.effects | instruction.effects();
        
        // Check if this is a terminator instruction
        if self.is_terminator(&instruction) {
            if self.terminator.is_some() {
                panic!("Basic block {} already has a terminator", self.id);
            }
            self.terminator = Some(instruction);
            
            // Update successors based on terminator
            self.update_successors_from_terminator();
        } else {
            self.instructions.push(instruction);
        }
    }
    
    /// Check if an instruction is a terminator
    fn is_terminator(&self, instruction: &MirInstruction) -> bool {
        matches!(instruction, 
            MirInstruction::Branch { .. } |
            MirInstruction::Jump { .. } |
            MirInstruction::Return { .. } |
            MirInstruction::Throw { .. }
        )
    }
    
    /// Update successors based on the terminator instruction
    fn update_successors_from_terminator(&mut self) {
        self.successors.clear();
        
        if let Some(ref terminator) = self.terminator {
            match terminator {
                MirInstruction::Branch { then_bb, else_bb, .. } => {
                    self.successors.insert(*then_bb);
                    self.successors.insert(*else_bb);
                },
                MirInstruction::Jump { target } => {
                    self.successors.insert(*target);
                },
                MirInstruction::Return { .. } => {
                    // No successors for return
                },
                MirInstruction::Throw { .. } => {
                    // No normal successors for throw - control goes to exception handlers
                    // Exception edges are handled separately from normal control flow
                },
                _ => unreachable!("Non-terminator instruction in terminator position"),
            }
        }
    }
    
    /// Add a predecessor
    pub fn add_predecessor(&mut self, pred: BasicBlockId) {
        self.predecessors.insert(pred);
    }
    
    /// Remove a predecessor
    pub fn remove_predecessor(&mut self, pred: BasicBlockId) {
        self.predecessors.remove(&pred);
    }
    
    /// Get all instructions including terminator
    pub fn all_instructions(&self) -> impl Iterator<Item = &MirInstruction> {
        self.instructions.iter().chain(self.terminator.iter())
    }
    
    /// Get all values defined in this block
    pub fn defined_values(&self) -> Vec<ValueId> {
        self.all_instructions()
            .filter_map(|inst| inst.dst_value())
            .collect()
    }
    
    /// Get all values used in this block
    pub fn used_values(&self) -> Vec<ValueId> {
        self.all_instructions()
            .flat_map(|inst| inst.used_values())
            .collect()
    }
    
    /// Check if this block is empty (no instructions)
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty() && self.terminator.is_none()
    }
    
    /// Check if this block has a terminator
    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }
    
    /// Check if this block ends with a return
    pub fn ends_with_return(&self) -> bool {
        matches!(self.terminator, Some(MirInstruction::Return { .. }))
    }
    
    /// Get the phi instructions at the beginning of this block
    pub fn phi_instructions(&self) -> impl Iterator<Item = &MirInstruction> {
        self.instructions.iter()
            .take_while(|inst| matches!(inst, MirInstruction::Phi { .. }))
    }
    
    /// Get non-phi instructions
    pub fn non_phi_instructions(&self) -> impl Iterator<Item = &MirInstruction> {
        self.instructions.iter()
            .skip_while(|inst| matches!(inst, MirInstruction::Phi { .. }))
    }
    
    /// Insert instruction at the beginning (after phi instructions)
    pub fn insert_instruction_after_phis(&mut self, instruction: MirInstruction) {
        let phi_count = self.phi_instructions().count();
        self.effects = self.effects | instruction.effects();
        self.instructions.insert(phi_count, instruction);
    }
    
    /// Replace terminator instruction
    pub fn set_terminator(&mut self, terminator: MirInstruction) {
        if !self.is_terminator(&terminator) {
            panic!("Instruction is not a valid terminator: {:?}", terminator);
        }
        
        self.effects = self.effects | terminator.effects();
        self.terminator = Some(terminator);
        self.update_successors_from_terminator();
    }
    
    /// Mark this block as reachable
    pub fn mark_reachable(&mut self) {
        self.reachable = true;
    }
    
    /// Check if this block dominates another block (simplified check)
    pub fn dominates(&self, other: BasicBlockId, dominators: &[HashSet<BasicBlockId>]) -> bool {
        if let Some(dom_set) = dominators.get(other.to_usize()) {
            dom_set.contains(&self.id)
        } else {
            false
        }
    }
}

/// Basic block ID generator
#[derive(Debug, Clone)]
pub struct BasicBlockIdGenerator {
    next_id: u32,
}

impl BasicBlockIdGenerator {
    /// Create a new generator starting from 0
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    /// Generate the next unique BasicBlockId
    pub fn next(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Peek at the next ID without consuming it
    pub fn peek_next(&self) -> BasicBlockId {
        BasicBlockId(self.next_id)
    }
    
    /// Reset the generator (for testing)
    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

impl Default for BasicBlockIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.id)?;
        
        // Show predecessors
        if !self.predecessors.is_empty() {
            let preds: Vec<String> = self.predecessors.iter()
                .map(|p| format!("{}", p))
                .collect();
            writeln!(f, "  ; preds: {}", preds.join(", "))?;
        }
        
        // Show instructions
        for instruction in &self.instructions {
            writeln!(f, "  {}", instruction)?;
        }
        
        // Show terminator
        if let Some(ref terminator) = self.terminator {
            writeln!(f, "  {}", terminator)?;
        }
        
        // Show effects if not pure
        if !self.effects.is_pure() {
            writeln!(f, "  ; effects: {}", self.effects)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{ConstValue, BinaryOp};
    
    #[test]
    fn test_basic_block_creation() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        assert_eq!(bb.id, bb_id);
        assert!(bb.is_empty());
        assert!(!bb.is_terminated());
        assert!(bb.effects.is_pure());
    }
    
    #[test]
    fn test_instruction_addition() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        let const_inst = MirInstruction::Const {
            dst: ValueId::new(0),
            value: ConstValue::Integer(42),
        };
        
        bb.add_instruction(const_inst);
        
        assert_eq!(bb.instructions.len(), 1);
        assert!(!bb.is_empty());
        assert!(bb.effects.is_pure());
    }
    
    #[test]
    fn test_terminator_addition() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        let return_inst = MirInstruction::Return {
            value: Some(ValueId::new(0)),
        };
        
        bb.add_instruction(return_inst);
        
        assert!(bb.is_terminated());
        assert!(bb.ends_with_return());
        assert_eq!(bb.instructions.len(), 0); // Terminator not in instructions
        assert!(bb.terminator.is_some());
    }
    
    #[test]
    fn test_branch_successors() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        let then_bb = BasicBlockId::new(1);
        let else_bb = BasicBlockId::new(2);
        
        let branch_inst = MirInstruction::Branch {
            condition: ValueId::new(0),
            then_bb,
            else_bb,
        };
        
        bb.add_instruction(branch_inst);
        
        assert_eq!(bb.successors.len(), 2);
        assert!(bb.successors.contains(&then_bb));
        assert!(bb.successors.contains(&else_bb));
    }
    
    #[test]
    fn test_basic_block_id_generator() {
        let mut gen = BasicBlockIdGenerator::new();
        
        let bb1 = gen.next();
        let bb2 = gen.next();
        let bb3 = gen.next();
        
        assert_eq!(bb1, BasicBlockId(0));
        assert_eq!(bb2, BasicBlockId(1));
        assert_eq!(bb3, BasicBlockId(2));
        
        assert_eq!(gen.peek_next(), BasicBlockId(3));
    }
    
    #[test]
    fn test_value_tracking() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        let val1 = ValueId::new(1);
        let val2 = ValueId::new(2);
        let val3 = ValueId::new(3);
        
        // Add instruction that defines val3 and uses val1, val2
        bb.add_instruction(MirInstruction::BinOp {
            dst: val3,
            op: BinaryOp::Add,
            lhs: val1,
            rhs: val2,
        });
        
        let defined = bb.defined_values();
        let used = bb.used_values();
        
        assert_eq!(defined, vec![val3]);
        assert_eq!(used, vec![val1, val2]);
    }
    
    #[test]
    fn test_phi_instruction_ordering() {
        let bb_id = BasicBlockId::new(0);
        let mut bb = BasicBlock::new(bb_id);
        
        // Add phi instruction
        let phi_inst = MirInstruction::Phi {
            dst: ValueId::new(0),
            inputs: vec![(BasicBlockId::new(1), ValueId::new(1))],
        };
        bb.add_instruction(phi_inst);
        
        // Add regular instruction
        let const_inst = MirInstruction::Const {
            dst: ValueId::new(2),
            value: ConstValue::Integer(42),
        };
        bb.add_instruction(const_inst);
        
        // Phi instructions should come first
        let phi_count = bb.phi_instructions().count();
        assert_eq!(phi_count, 1);
        
        let non_phi_count = bb.non_phi_instructions().count();
        assert_eq!(non_phi_count, 1);
    }
}