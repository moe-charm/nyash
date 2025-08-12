/*!
 * MIR Function and Module - High-level MIR organization
 * 
 * Functions contain basic blocks and SSA values, modules contain functions
 */

use super::{BasicBlock, BasicBlockId, ValueId, EffectMask, MirType};
use std::collections::HashMap;
use std::fmt;

/// Function signature for MIR functions
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    
    /// Parameter types
    pub params: Vec<MirType>,
    
    /// Return type
    pub return_type: MirType,
    
    /// Overall effect mask for the function
    pub effects: EffectMask,
}

/// A MIR function in SSA form
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Function signature
    pub signature: FunctionSignature,
    
    /// Basic blocks indexed by ID
    pub blocks: HashMap<BasicBlockId, BasicBlock>,
    
    /// Entry basic block ID
    pub entry_block: BasicBlockId,
    
    /// Local variable declarations (before SSA conversion)
    pub locals: Vec<MirType>,
    
    /// Parameter value IDs
    pub params: Vec<ValueId>,
    
    /// Next available value ID
    pub next_value_id: u32,
    
    /// Function-level metadata
    pub metadata: FunctionMetadata,
}

/// Metadata for MIR functions
#[derive(Debug, Clone, Default)]
pub struct FunctionMetadata {
    /// Source file location
    pub source_file: Option<String>,
    
    /// Line number in source
    pub line_number: Option<u32>,
    
    /// Whether this function is an entry point
    pub is_entry_point: bool,
    
    /// Whether this function is pure (no side effects)
    pub is_pure: bool,
    
    /// Optimization hints
    pub optimization_hints: Vec<String>,
}

impl MirFunction {
    /// Create a new MIR function
    pub fn new(signature: FunctionSignature, entry_block: BasicBlockId) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(entry_block, BasicBlock::new(entry_block));
        
        Self {
            signature,
            blocks,
            entry_block,
            locals: Vec::new(),
            params: Vec::new(),
            next_value_id: 0,
            metadata: FunctionMetadata::default(),
        }
    }
    
    /// Get the next available ValueId
    pub fn next_value_id(&mut self) -> ValueId {
        let id = ValueId::new(self.next_value_id);
        self.next_value_id += 1;
        id
    }
    
    /// Add a new basic block
    pub fn add_block(&mut self, block: BasicBlock) -> BasicBlockId {
        let id = block.id;
        self.blocks.insert(id, block);
        id
    }
    
    /// Get a basic block by ID
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    /// Get a mutable basic block by ID
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
    
    /// Get the entry block
    pub fn entry_block(&self) -> &BasicBlock {
        self.blocks.get(&self.entry_block)
            .expect("Entry block must exist")
    }
    
    /// Get all basic block IDs in insertion order
    pub fn block_ids(&self) -> Vec<BasicBlockId> {
        let mut ids: Vec<_> = self.blocks.keys().copied().collect();
        ids.sort();
        ids
    }
    
    /// Get all values defined in this function
    pub fn defined_values(&self) -> Vec<ValueId> {
        let mut values = Vec::new();
        values.extend(&self.params);
        
        for block in self.blocks.values() {
            values.extend(block.defined_values());
        }
        
        values
    }
    
    /// Verify function integrity (basic checks)
    pub fn verify(&self) -> Result<(), String> {
        // Check entry block exists
        if !self.blocks.contains_key(&self.entry_block) {
            return Err("Entry block does not exist".to_string());
        }
        
        // Check all blocks are reachable from entry
        let reachable = self.compute_reachable_blocks();
        for (id, block) in &self.blocks {
            if !reachable.contains(id) {
                eprintln!("Warning: Block {} is unreachable", id);
            }
        }
        
        // Check terminator consistency
        for block in self.blocks.values() {
            if !block.is_terminated() && !block.is_empty() {
                return Err(format!("Block {} is not properly terminated", block.id));
            }
            
            // Check successor/predecessor consistency
            for successor_id in &block.successors {
                if let Some(successor) = self.blocks.get(successor_id) {
                    if !successor.predecessors.contains(&block.id) {
                        return Err(format!(
                            "Inconsistent CFG: {} -> {} but {} doesn't have {} as predecessor",
                            block.id, successor_id, successor_id, block.id
                        ));
                    }
                } else {
                    return Err(format!("Block {} references non-existent successor {}", 
                                     block.id, successor_id));
                }
            }
        }
        
        Ok(())
    }
    
    /// Compute reachable blocks from entry
    fn compute_reachable_blocks(&self) -> std::collections::HashSet<BasicBlockId> {
        let mut reachable = std::collections::HashSet::new();
        let mut worklist = vec![self.entry_block];
        
        while let Some(current) = worklist.pop() {
            if reachable.insert(current) {
                if let Some(block) = self.blocks.get(&current) {
                    worklist.extend(block.successors.iter());
                }
            }
        }
        
        reachable
    }
    
    /// Update predecessor/successor relationships
    pub fn update_cfg(&mut self) {
        // Clear all predecessors
        for block in self.blocks.values_mut() {
            block.predecessors.clear();
        }
        
        // Rebuild predecessors from successors
        let edges: Vec<(BasicBlockId, BasicBlockId)> = self.blocks.values()
            .flat_map(|block| {
                block.successors.iter().map(move |&succ| (block.id, succ))
            })
            .collect();
        
        for (pred, succ) in edges {
            if let Some(successor_block) = self.blocks.get_mut(&succ) {
                successor_block.add_predecessor(pred);
            }
        }
    }
    
    /// Mark reachable blocks
    pub fn mark_reachable_blocks(&mut self) {
        let reachable = self.compute_reachable_blocks();
        for (id, block) in &mut self.blocks {
            if reachable.contains(id) {
                block.mark_reachable();
            }
        }
    }
    
    /// Get function statistics
    pub fn stats(&self) -> FunctionStats {
        let instruction_count = self.blocks.values()
            .map(|block| block.instructions.len() + if block.terminator.is_some() { 1 } else { 0 })
            .sum();
            
        let phi_count = self.blocks.values()
            .map(|block| block.phi_instructions().count())
            .sum();
            
        FunctionStats {
            block_count: self.blocks.len(),
            instruction_count,
            phi_count,
            value_count: self.next_value_id as usize,
            is_pure: self.signature.effects.is_pure(),
        }
    }
}

/// Function statistics for profiling and optimization
#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub block_count: usize,
    pub instruction_count: usize,
    pub phi_count: usize,
    pub value_count: usize,
    pub is_pure: bool,
}

/// A MIR module containing multiple functions
#[derive(Debug, Clone)]
pub struct MirModule {
    /// Module name
    pub name: String,
    
    /// Functions in this module
    pub functions: HashMap<String, MirFunction>,
    
    /// Global constants/statics
    pub globals: HashMap<String, super::ConstValue>,
    
    /// Module metadata
    pub metadata: ModuleMetadata,
}

/// Metadata for MIR modules
#[derive(Debug, Clone, Default)]
pub struct ModuleMetadata {
    /// Source file this module was compiled from
    pub source_file: Option<String>,
    
    /// Compilation timestamp
    pub compiled_at: Option<String>,
    
    /// Compiler version
    pub compiler_version: Option<String>,
    
    /// Optimization level used
    pub optimization_level: u32,
}

impl MirModule {
    /// Create a new MIR module
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: HashMap::new(),
            globals: HashMap::new(),
            metadata: ModuleMetadata::default(),
        }
    }
    
    /// Add a function to the module
    pub fn add_function(&mut self, function: MirFunction) {
        let name = function.signature.name.clone();
        self.functions.insert(name, function);
    }
    
    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Option<&MirFunction> {
        self.functions.get(name)
    }
    
    /// Get a mutable function by name
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut MirFunction> {
        self.functions.get_mut(name)
    }
    
    /// Get all function names
    pub fn function_names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
    
    /// Add a global constant
    pub fn add_global(&mut self, name: String, value: super::ConstValue) {
        self.globals.insert(name, value);
    }
    
    /// Verify entire module
    pub fn verify(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for (name, function) in &self.functions {
            if let Err(e) = function.verify() {
                errors.push(format!("Function '{}': {}", name, e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get module statistics
    pub fn stats(&self) -> ModuleStats {
        let function_stats: Vec<_> = self.functions.values()
            .map(|f| f.stats())
            .collect();
            
        ModuleStats {
            function_count: self.functions.len(),
            global_count: self.globals.len(),
            total_blocks: function_stats.iter().map(|s| s.block_count).sum(),
            total_instructions: function_stats.iter().map(|s| s.instruction_count).sum(),
            total_values: function_stats.iter().map(|s| s.value_count).sum(),
            pure_functions: function_stats.iter().filter(|s| s.is_pure).count(),
        }
    }
}

/// Module statistics
#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub function_count: usize,
    pub global_count: usize,
    pub total_blocks: usize,
    pub total_instructions: usize,
    pub total_values: usize,
    pub pure_functions: usize,
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "function {}({}) -> {} {{", 
                self.signature.name,
                self.signature.params.iter()
                    .enumerate()
                    .map(|(i, ty)| format!("%{}: {:?}", i, ty))
                    .collect::<Vec<_>>()
                    .join(", "),
                format!("{:?}", self.signature.return_type))?;
        
        // Show effects if not pure
        if !self.signature.effects.is_pure() {
            writeln!(f, "  ; effects: {}", self.signature.effects)?;
        }
        
        // Show blocks in order
        let mut block_ids: Vec<_> = self.blocks.keys().copied().collect();
        block_ids.sort();
        
        for block_id in block_ids {
            if let Some(block) = self.blocks.get(&block_id) {
                write!(f, "{}", block)?;
            }
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Display for MirModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "module {} {{", self.name)?;
        
        // Show globals
        if !self.globals.is_empty() {
            writeln!(f, "  ; globals:")?;
            for (name, value) in &self.globals {
                writeln!(f, "  global {} = {}", name, value)?;
            }
            writeln!(f)?;
        }
        
        // Show functions
        for function in self.functions.values() {
            writeln!(f, "{}", function)?;
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirType, EffectMask};
    
    #[test]
    fn test_function_creation() {
        let signature = FunctionSignature {
            name: "test_func".to_string(),
            params: vec![MirType::Integer, MirType::Float],
            return_type: MirType::Integer,
            effects: EffectMask::PURE,
        };
        
        let entry_block = BasicBlockId::new(0);
        let function = MirFunction::new(signature.clone(), entry_block);
        
        assert_eq!(function.signature.name, "test_func");
        assert_eq!(function.entry_block, entry_block);
        assert!(function.blocks.contains_key(&entry_block));
    }
    
    #[test]
    fn test_module_creation() {
        let mut module = MirModule::new("test_module".to_string());
        
        let signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let function = MirFunction::new(signature, BasicBlockId::new(0));
        module.add_function(function);
        
        assert_eq!(module.name, "test_module");
        assert!(module.get_function("main").is_some());
        assert_eq!(module.function_names().len(), 1);
    }
    
    #[test]
    fn test_value_id_generation() {
        let signature = FunctionSignature {
            name: "test".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let mut function = MirFunction::new(signature, BasicBlockId::new(0));
        
        let val1 = function.next_value_id();
        let val2 = function.next_value_id();
        let val3 = function.next_value_id();
        
        assert_eq!(val1, ValueId::new(0));
        assert_eq!(val2, ValueId::new(1));
        assert_eq!(val3, ValueId::new(2));
    }
    
    #[test]
    fn test_function_stats() {
        let signature = FunctionSignature {
            name: "test".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let function = MirFunction::new(signature, BasicBlockId::new(0));
        let stats = function.stats();
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.instruction_count, 0);
        assert_eq!(stats.value_count, 0);
        assert!(stats.is_pure);
    }
}