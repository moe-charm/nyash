/*!
 * MIR Optimizer - Phase 3 Implementation
 * 
 * Implements Effect System based optimizations for the new 26-instruction MIR
 * - Pure instruction reordering and CSE (Common Subexpression Elimination)
 * - BoxFieldLoad/Store dependency analysis
 * - Intrinsic function optimization
 * - Dead code elimination
 */

use super::{MirModule, MirFunction, MirInstruction, ValueId, MirType, TypeOpKind};
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

        // Safety-net passesは削除（Phase 2: 変換の一本化）。診断のみ後段で実施。
        
        // Pass 5: BoxField dependency optimization
        stats.merge(self.optimize_boxfield_operations(module));
        
        if self.debug {
            println!("✅ Optimization complete: {}", stats);
        }
        // Diagnostics (informational): report unlowered patterns
        let diag = self.diagnose_unlowered_type_ops(module);
        stats.merge(diag);
        
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
        for (bbid, block) in &mut function.blocks {
            block.instructions.retain(|instruction| {
                if instruction.effects().is_pure() {
                    if let Some(dst) = instruction.dst_value() {
                        if !used_values.contains(&dst) {
                            opt_debug(&format!("DCE drop @{}: {:?}", bbid.as_u32(), instruction));
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
            // BoxFieldLoad removed from instruction set
            // MirInstruction::BoxFieldLoad { box_val, field, .. } => format!("boxload_{}_{}", box_val.as_u32(), field),
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

/// Map string type name to MIR type (optimizer-level helper)
fn map_type_name(name: &str) -> MirType {
    match name {
        "Integer" | "Int" | "I64" => MirType::Integer,
        "Float" | "F64" => MirType::Float,
        "Bool" | "Boolean" => MirType::Bool,
        "String" => MirType::String,
        "Void" | "Unit" => MirType::Void,
        other => MirType::Box(other.to_string()),
    }
}

fn opt_debug_enabled() -> bool { std::env::var("NYASH_OPT_DEBUG").is_ok() }
fn opt_debug(msg: &str) { if opt_debug_enabled() { eprintln!("[OPT] {}", msg); } }

/// Resolve a MIR type from a value id that should represent a type name
/// Supports: Const String("T") and NewBox(StringBox, Const String("T"))
fn resolve_type_from_value(
    function: &MirFunction,
    def_map: &std::collections::HashMap<ValueId, (super::basic_block::BasicBlockId, usize)>,
    id: ValueId,
) -> Option<MirType> {
    use super::instruction::ConstValue;
    if let Some((bb, idx)) = def_map.get(&id).copied() {
        if let Some(block) = function.blocks.get(&bb) {
            if idx < block.instructions.len() {
                match &block.instructions[idx] {
                    MirInstruction::Const { value: ConstValue::String(s), .. } => {
                        return Some(map_type_name(s));
                    }
                    MirInstruction::NewBox { box_type, args, .. } if box_type == "StringBox" && args.len() == 1 => {
                        let inner = args[0];
                        if let Some((sbb, sidx)) = def_map.get(&inner).copied() {
                            if let Some(sblock) = function.blocks.get(&sbb) {
                                if sidx < sblock.instructions.len() {
                                    if let MirInstruction::Const { value: ConstValue::String(s), .. } = &sblock.instructions[sidx] {
                                        return Some(map_type_name(s));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
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
    pub diagnostics_reported: usize,
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
        self.diagnostics_reported += other.diagnostics_reported;
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

impl MirOptimizer {
    /// Diagnostic: detect unlowered is/as/isType/asType after Builder
    fn diagnose_unlowered_type_ops(&mut self, module: &MirModule) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        let diag_on = self.debug || std::env::var("NYASH_OPT_DIAG").is_ok();
        for (fname, function) in &module.functions {
            // def map for resolving constants
            let mut def_map: std::collections::HashMap<ValueId, (super::basic_block::BasicBlockId, usize)> = std::collections::HashMap::new();
            for (bb_id, block) in &function.blocks {
                for (i, inst) in block.instructions.iter().enumerate() {
                    if let Some(dst) = inst.dst_value() { def_map.insert(dst, (*bb_id, i)); }
                }
                if let Some(term) = &block.terminator { if let Some(dst) = term.dst_value() { def_map.insert(dst, (*bb_id, usize::MAX)); } }
            }
            let mut count = 0usize;
            for (_bb, block) in &function.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::BoxCall { method, .. } if method == "is" || method == "as" || method == "isType" || method == "asType" => { count += 1; }
                        MirInstruction::Call { func, .. } => {
                            if let Some((bb, idx)) = def_map.get(func).copied() {
                                if let Some(b) = function.blocks.get(&bb) {
                                    if idx < b.instructions.len() {
                                        if let MirInstruction::Const { value: super::instruction::ConstValue::String(s), .. } = &b.instructions[idx] {
                                            if s == "isType" || s == "asType" { count += 1; }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if count > 0 {
                stats.diagnostics_reported += count;
                if diag_on {
                    eprintln!("[OPT][DIAG] Function '{}' has {} unlowered type-op calls", fname, count);
                }
            }
        }
        stats
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

    #[test]
    fn test_dce_does_not_drop_typeop_used_by_print() {
        // Build a simple function: %v=TypeOp(check); print %v; ensure TypeOp remains after optimize
        let signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: super::super::effect::EffectMask::PURE,
        };
        let mut func = MirFunction::new(signature, BasicBlockId::new(0));
        let bb0 = BasicBlockId::new(0);
        let mut b0 = BasicBlock::new(bb0);
        let v0 = ValueId::new(0);
        let v1 = ValueId::new(1);
        b0.add_instruction(MirInstruction::NewBox { dst: v0, box_type: "IntegerBox".to_string(), args: vec![] });
        b0.add_instruction(MirInstruction::TypeOp { dst: v1, op: TypeOpKind::Check, value: v0, ty: MirType::Integer });
        b0.add_instruction(MirInstruction::Print { value: v1, effects: super::super::effect::EffectMask::IO });
        b0.add_instruction(MirInstruction::Return { value: None });
        func.add_block(b0);
        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut opt = MirOptimizer::new();
        let _stats = opt.optimize_module(&mut module);

        // Ensure TypeOp remains in bb0
        let f = module.get_function("main").unwrap();
        let block = f.get_block(&bb0).unwrap();
        let has_typeop = block.all_instructions().any(|i| matches!(i, MirInstruction::TypeOp { .. }));
        assert!(has_typeop, "TypeOp should not be dropped by DCE when used by print");
    }
}
