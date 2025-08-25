/*!
 * MIR Builder Core - Core builder functionality
 * 
 * Contains the MirBuilder struct and core instruction emission functionality
 */

use super::*;
use crate::ast::ASTNode;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn builder_debug_enabled() -> bool {
    std::env::var("NYASH_BUILDER_DEBUG").is_ok()
}

pub fn builder_debug_log(msg: &str) {
    if builder_debug_enabled() {
        eprintln!("[BUILDER] {}", msg);
    }
}

/// MIR builder for converting AST to SSA form
pub struct MirBuilder {
    /// Current module being built
    pub(super) current_module: Option<MirModule>,
    
    /// Current function being built
    pub(super) current_function: Option<MirFunction>,
    
    /// Current basic block being built
    pub(super) current_block: Option<BasicBlockId>,
    
    /// Value ID generator
    pub(super) value_gen: ValueIdGenerator,
    
    /// Basic block ID generator
    pub(super) block_gen: BasicBlockIdGenerator,
    
    /// Variable name to ValueId mapping (for SSA conversion)
    pub(super) variable_map: HashMap<String, ValueId>,
    
    /// Pending phi functions to be inserted
    #[allow(dead_code)]
    pub(super) pending_phis: Vec<(BasicBlockId, ValueId, String)>,

    /// Origin tracking for simple optimizations (e.g., object.method after new)
    /// Maps a ValueId to the class name if it was produced by NewBox of that class
    pub(super) value_origin_newbox: HashMap<ValueId, String>,

    /// Names of user-defined boxes declared in the current module
    pub(super) user_defined_boxes: HashSet<String>,

    /// Weak field registry: BoxName -> {weak field names}
    pub(super) weak_fields_by_box: HashMap<String, HashSet<String>>,

    /// Remember class of object fields after assignments: (base_id, field) -> class_name
    pub(super) field_origin_class: HashMap<(ValueId, String), String>,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self {
            current_module: None,
            current_function: None,
            current_block: None,
            value_gen: ValueIdGenerator::new(),
            block_gen: BasicBlockIdGenerator::new(),
            variable_map: HashMap::new(),
            pending_phis: Vec::new(),
            value_origin_newbox: HashMap::new(),
            user_defined_boxes: HashSet::new(),
            weak_fields_by_box: HashMap::new(),
            field_origin_class: HashMap::new(),
        }
    }

    pub(super) fn emit_type_check(&mut self, value: ValueId, expected_type: String) -> Result<ValueId, String> {
        let target_value = self.value_gen.next_value_id();
        
        let instruction = MirInstruction::TypeOp {
            dst: target_value,
            operation: super::TypeOpKind::Check,
            operand: value,
            type_info: expected_type,
            effects: EffectMask::new(Effect::ReadOnly),
        };
        
        self.emit_instruction(instruction)?;
        Ok(target_value)
    }

    pub(super) fn emit_cast(&mut self, value: ValueId, target_type: super::MirType) -> Result<ValueId, String> {
        let target_value = self.value_gen.next_value_id();
        
        let instruction = MirInstruction::TypeOp {
            dst: target_value,
            operation: super::TypeOpKind::Cast,
            operand: value,
            type_info: format!("{:?}", target_type),
            effects: EffectMask::new(Effect::ReadOnly),
        };
        
        self.emit_instruction(instruction)?;
        Ok(target_value)
    }

    pub(super) fn emit_weak_new(&mut self, box_val: ValueId) -> Result<ValueId, String> {
        let weak_ref = self.value_gen.next_value_id();
        
        let instruction = MirInstruction::WeakNew {
            dst: weak_ref,
            source: box_val,
            effects: EffectMask::new(Effect::Pure),
        };
        
        self.emit_instruction(instruction)?;
        Ok(weak_ref)
    }

    pub(super) fn emit_weak_load(&mut self, weak_ref: ValueId) -> Result<ValueId, String> {
        let loaded_value = self.value_gen.next_value_id();
        
        let instruction = MirInstruction::WeakLoad {
            dst: loaded_value,
            weak_ref,
            effects: EffectMask::new(Effect::ReadOnly),
        };
        
        self.emit_instruction(instruction)?;
        Ok(loaded_value)
    }

    pub(super) fn emit_barrier_read(&mut self, ptr: ValueId) -> Result<(), String> {
        let instruction = MirInstruction::BarrierRead {
            ptr,
            effects: EffectMask::new(Effect::SideEffect),
        };
        
        self.emit_instruction(instruction)?;
        Ok(())
    }

    pub(super) fn emit_barrier_write(&mut self, ptr: ValueId) -> Result<(), String> {
        let instruction = MirInstruction::BarrierWrite {
            ptr,
            effects: EffectMask::new(Effect::SideEffect),
        };
        
        self.emit_instruction(instruction)?;
        Ok(())
    }

    pub(super) fn emit_instruction(&mut self, instruction: MirInstruction) -> Result<(), String> {
        // Ensure we have a current function to emit into
        if self.current_function.is_none() {
            return Err("Cannot emit instruction without current function".to_string());
        }

        // Ensure we have a current block to emit into
        if self.current_block.is_none() {
            return Err("Cannot emit instruction without current block".to_string());
        }

        let current_block_id = self.current_block.unwrap();
        
        // Get a mutable reference to the current function
        let current_function = self.current_function.as_mut().unwrap();
        
        // Ensure the block exists
        self.ensure_block_exists(current_block_id)?;
        
        // Add instruction to current block
        if let Some(block) = current_function.basic_blocks.get_mut(&current_block_id) {
            block.instructions.push(instruction);
        } else {
            return Err(format!("Block {:?} not found in current function", current_block_id));
        }

        Ok(())
    }

    pub(super) fn ensure_block_exists(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        let current_function = self.current_function.as_mut()
            .ok_or("No current function")?;
        
        if !current_function.basic_blocks.contains_key(&block_id) {
            current_function.basic_blocks.insert(block_id, BasicBlock {
                id: block_id,
                instructions: Vec::new(),
            });
        }
        
        Ok(())
    }

    pub(super) fn start_new_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        // Ensure the block exists in the current function
        self.ensure_block_exists(block_id)?;
        
        // Set as current block
        self.current_block = Some(block_id);
        
        Ok(())
    }
}

// Adapter: Implement LoopBuilderApi for modularized MirBuilder to enable shared helpers
impl crate::mir::loop_api::LoopBuilderApi for MirBuilder {
    fn new_block(&mut self) -> super::BasicBlockId { self.block_gen.next() }
    fn current_block(&self) -> Result<super::BasicBlockId, String> {
        self.current_block.ok_or_else(|| "No current block".to_string())
    }
    fn start_new_block(&mut self, block: super::BasicBlockId) -> Result<(), String> {
        self.start_new_block(block)
    }
    fn emit(&mut self, inst: super::MirInstruction) -> Result<(), String> {
        self.emit_instruction(inst)
    }
    fn new_value(&mut self) -> super::ValueId { self.value_gen.next() }

    fn add_predecessor(&mut self, block: super::BasicBlockId, pred: super::BasicBlockId) -> Result<(), String> {
        if let Some(ref mut f) = self.current_function {
            if let Some(bb) = f.get_block_mut(block) {
                bb.add_predecessor(pred);
                Ok(())
            } else { Err(format!("Block {} not found", block.as_u32())) }
        } else { Err("No current function".into()) }
    }

    fn seal_block(&mut self, block: super::BasicBlockId) -> Result<(), String> {
        if let Some(ref mut f) = self.current_function {
            if let Some(bb) = f.get_block_mut(block) {
                bb.seal();
                Ok(())
            } else { Err(format!("Block {} not found", block.as_u32())) }
        } else { Err("No current function".into()) }
    }

    fn insert_phi_at_block_start(&mut self, block: super::BasicBlockId, dst: super::ValueId, inputs: Vec<(super::BasicBlockId, super::ValueId)>) -> Result<(), String> {
        if let Some(ref mut f) = self.current_function {
            if let Some(bb) = f.get_block_mut(block) {
                let inst = super::MirInstruction::Phi { dst, inputs };
                bb.effects = bb.effects | inst.effects();
                bb.instructions.insert(0, inst);
                Ok(())
            } else { Err(format!("Block {} not found", block.as_u32())) }
        } else { Err("No current function".into()) }
    }
}
