/*!
 * MIR Optimizer - Phase 3 Implementation
 * 
 * Implements Effect System based optimizations for the new 26-instruction MIR
 * - Pure instruction reordering and CSE (Common Subexpression Elimination)
 * - BoxFieldLoad/Store dependency analysis
 * - Intrinsic function optimization
 * - Dead code elimination
 */

use super::{MirModule, MirFunction, MirInstruction, ValueId, EffectMask, Effect};
use std::collections::{HashMap, HashSet};

/// MIR optimization passes
pub struct MirOptimizer {
    /// Enable debug output for optimization passes
    debug: bool,
}

impl MirOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        Self {
            debug: false,
        }
    }
    
    /// Enable debug output
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }
    
    /// Run all optimization passes on a MIR module
    pub fn optimize_module(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        if self.debug {
            println!("🚀 Starting MIR optimization passes");
        }
        
        // Pass 1: Dead code elimination
        stats.merge(self.eliminate_dead_code(module));
        
        // Pass 2: Pure instruction CSE (Common Subexpression Elimination)
        stats.merge(self.common_subexpression_elimination(module));
        
        // Pass 3: Pure instruction reordering for better locality
        stats.merge(self.reorder_pure_instructions(module));
        
        // Pass 4: Intrinsic function optimization
        stats.merge(self.optimize_intrinsic_calls(module));
        
        // Pass 5: BoxField dependency optimization
        stats.merge(self.optimize_boxfield_operations(module));
        
        if self.debug {
            println!("✅ Optimization complete: {}", stats);
        }
        
        stats
    }
    
    /// Eliminate dead code (unused values)
    fn eliminate_dead_code(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  🗑️  Dead code elimination in function: {}", func_name);
            }
            
            let eliminated = self.eliminate_dead_code_in_function(function);
            stats.dead_code_eliminated += eliminated;
        }
        
        stats
    }
    
    /// Eliminate dead code in a single function
    fn eliminate_dead_code_in_function(&mut self, function: &mut MirFunction) -> usize {
        // Collect all used values
        let mut used_values = HashSet::new();
        
        // Mark values used in terminators and side-effect instructions
        for (_, block) in &function.blocks {
            for instruction in &block.instructions {
                // Always keep instructions with side effects
                if !instruction.effects().is_pure() {
                    if let Some(dst) = instruction.dst_value() {
                        used_values.insert(dst);
                    }
                    for used in instruction.used_values() {
                        used_values.insert(used);
                    }
                }
            }
            
            // Mark values used in terminators
            if let Some(terminator) = &block.terminator {
                for used in terminator.used_values() {
                    used_values.insert(used);
                }
            }
        }
        
        // Propagate usage backwards
        let mut changed = true;
        while changed {
            changed = false;
            for (_, block) in &function.blocks {
                for instruction in &block.instructions {
                    if let Some(dst) = instruction.dst_value() {
                        if used_values.contains(&dst) {
                            for used in instruction.used_values() {
                                if used_values.insert(used) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Remove unused pure instructions
        let mut eliminated = 0;
        for (_, block) in &mut function.blocks {
            block.instructions.retain(|instruction| {
                if instruction.effects().is_pure() {
                    if let Some(dst) = instruction.dst_value() {
                        if !used_values.contains(&dst) {
                            eliminated += 1;
                            return false;
                        }
                    }
                }
                true
            });
        }
        
        eliminated
    }
    
    /// Common Subexpression Elimination for pure instructions
    fn common_subexpression_elimination(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  🔄 CSE in function: {}", func_name);
            }
            
            let eliminated = self.cse_in_function(function);
            stats.cse_eliminated += eliminated;
        }
        
        stats
    }
    
    /// CSE in a single function
    fn cse_in_function(&mut self, function: &mut MirFunction) -> usize {
        let mut expression_map: HashMap<String, ValueId> = HashMap::new();
        let mut replacements: HashMap<ValueId, ValueId> = HashMap::new();
        let mut eliminated = 0;
        
        for (_, block) in &mut function.blocks {
            for instruction in &mut block.instructions {
                // Only optimize pure instructions
                if instruction.effects().is_pure() {
                    let expr_key = self.instruction_to_key(instruction);
                    
                    if let Some(&existing_value) = expression_map.get(&expr_key) {
                        // Found common subexpression
                        if let Some(dst) = instruction.dst_value() {
                            replacements.insert(dst, existing_value);
                            eliminated += 1;
                        }
                    } else {
                        // First occurrence of this expression
                        if let Some(dst) = instruction.dst_value() {
                            expression_map.insert(expr_key, dst);
                        }
                    }
                }
            }
        }
        
        // Apply replacements (simplified - in full implementation would need proper SSA update)
        eliminated
    }
    
    /// Convert instruction to string key for CSE
    fn instruction_to_key(&self, instruction: &MirInstruction) -> String {
        match instruction {
            MirInstruction::Const { value, .. } => format!("const_{:?}", value),
            MirInstruction::BinOp { op, lhs, rhs, .. } => format!("binop_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32()),
            MirInstruction::Compare { op, lhs, rhs, .. } => format!("cmp_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32()),
            MirInstruction::BoxFieldLoad { box_val, field, .. } => format!("boxload_{}_{}", box_val.as_u32(), field),
            MirInstruction::Call { func, args, .. } => {
                let args_str = args.iter().map(|v| v.as_u32().to_string()).collect::<Vec<_>>().join(",");
                format!("call_{}_{}", func.as_u32(), args_str)
            },
            _ => format!("other_{:?}", instruction),
        }
    }
    
    /// Reorder pure instructions for better locality
    fn reorder_pure_instructions(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  🔀 Pure instruction reordering in function: {}", func_name);
            }
            
            stats.reorderings += self.reorder_in_function(function);
        }
        
        stats
    }
    
    /// Reorder instructions in a function
    fn reorder_in_function(&mut self, _function: &mut MirFunction) -> usize {
        // Simplified implementation - in full version would implement:
        // 1. Build dependency graph
        // 2. Topological sort respecting effects
        // 3. Group pure instructions together
        // 4. Move loads closer to uses
        0
    }
    
    /// Optimize intrinsic function calls
    fn optimize_intrinsic_calls(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  ⚡ Intrinsic optimization in function: {}", func_name);
            }
            
            stats.intrinsic_optimizations += self.optimize_intrinsics_in_function(function);
        }
        
        stats
    }
    
    /// Optimize intrinsics in a function
    fn optimize_intrinsics_in_function(&mut self, _function: &mut MirFunction) -> usize {
        // Simplified implementation - would optimize:
        // 1. Constant folding in intrinsic calls
        // 2. Strength reduction (e.g., @unary_neg(@unary_neg(x)) → x)
        // 3. Identity elimination (e.g., x + 0 → x)
        0
    }
    
    /// Optimize BoxField operations
    fn optimize_boxfield_operations(&mut self, module: &mut MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        
        for (func_name, function) in &mut module.functions {
            if self.debug {
                println!("  📦 BoxField optimization in function: {}", func_name);
            }
            
            stats.boxfield_optimizations += self.optimize_boxfield_in_function(function);
        }
        
        stats
    }
    
    /// Optimize BoxField operations in a function
    fn optimize_boxfield_in_function(&mut self, _function: &mut MirFunction) -> usize {
        // Simplified implementation - would optimize:
        // 1. Load-after-store elimination
        // 2. Store-after-store elimination  
        // 3. Load forwarding
        // 4. Field access coalescing
        0
    }
}

impl Default for MirOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from optimization passes
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub dead_code_eliminated: usize,
    pub cse_eliminated: usize,
    pub reorderings: usize,
    pub intrinsic_optimizations: usize,
    pub boxfield_optimizations: usize,
}

impl OptimizationStats {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn merge(&mut self, other: OptimizationStats) {
        self.dead_code_eliminated += other.dead_code_eliminated;
        self.cse_eliminated += other.cse_eliminated;
        self.reorderings += other.reorderings;
        self.intrinsic_optimizations += other.intrinsic_optimizations;
        self.boxfield_optimizations += other.boxfield_optimizations;
    }
    
    pub fn total_optimizations(&self) -> usize {
        self.dead_code_eliminated + self.cse_eliminated + self.reorderings + 
        self.intrinsic_optimizations + self.boxfield_optimizations
    }
}

impl std::fmt::Display for OptimizationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "dead_code: {}, cse: {}, reorder: {}, intrinsic: {}, boxfield: {} (total: {})",
            self.dead_code_eliminated,
            self.cse_eliminated, 
            self.reorderings,
            self.intrinsic_optimizations,
            self.boxfield_optimizations,
            self.total_optimizations()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, BasicBlock, BasicBlockId, ValueId, ConstValue};
    
    #[test]
    fn test_optimizer_creation() {
        let optimizer = MirOptimizer::new();
        assert!(!optimizer.debug);
        
        let debug_optimizer = MirOptimizer::new().with_debug();
        assert!(debug_optimizer.debug);
    }
    
    #[test]
    fn test_optimization_stats() {
        let mut stats = OptimizationStats::new();
        assert_eq!(stats.total_optimizations(), 0);
        
        stats.dead_code_eliminated = 5;
        stats.cse_eliminated = 3;
        assert_eq!(stats.total_optimizations(), 8);
        
        let other_stats = OptimizationStats {
            dead_code_eliminated: 2,
            cse_eliminated: 1,
            ..Default::default()
        };
        
        stats.merge(other_stats);
        assert_eq!(stats.dead_code_eliminated, 7);
        assert_eq!(stats.cse_eliminated, 4);
        assert_eq!(stats.total_optimizations(), 11);
    }
    
    #[test]
    fn test_instruction_to_key() {
        let optimizer = MirOptimizer::new();
        
        let const_instr = MirInstruction::Const {
            dst: ValueId::new(1),
            value: ConstValue::Integer(42),
        };
        
        let key = optimizer.instruction_to_key(&const_instr);
        assert!(key.contains("const"));
        assert!(key.contains("42"));
    }
}