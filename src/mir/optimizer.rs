/*!
 * MIR Optimizer - Phase 3 Implementation
 *
 * Implements Effect System based optimizations for the new 26-instruction MIR
 * - Pure instruction reordering and CSE (Common Subexpression Elimination)
 * - BoxFieldLoad/Store dependency analysis
 * - Intrinsic function optimization
 * - Dead code elimination
 */

use super::{MirFunction, MirInstruction, MirModule, MirType, ValueId};
use crate::mir::optimizer_stats::OptimizationStats;
// std::collections imports removed (local DCE/CSE impls deleted)

/// MIR optimization passes
pub struct MirOptimizer {
    /// Enable debug output for optimization passes
    debug: bool,
}

impl MirOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        Self { debug: false }
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

        // Env toggles for phased MIR cleanup
        let core13 = crate::config::env::mir_core13();
        let mut ref_to_boxcall = crate::config::env::mir_ref_boxcall();
        if core13 {
            ref_to_boxcall = true;
        }

        // Pass 0: Normalize legacy instructions to unified forms
        //  - Includes optional Array→BoxCall guarded by env (inside the pass)
        stats.merge(
            crate::mir::optimizer_passes::normalize::normalize_legacy_instructions(self, module),
        );
        // Pass 0.1: RefGet/RefSet → BoxCall(getField/setField) (guarded)
        if ref_to_boxcall {
            stats.merge(
                crate::mir::optimizer_passes::normalize::normalize_ref_field_access(self, module),
            );
        }

        // Option: Force BoxCall → PluginInvoke (env)
        if crate::config::env::mir_plugin_invoke() || crate::config::env::plugin_only() {
            stats.merge(crate::mir::optimizer_passes::normalize::force_plugin_invoke(self, module));
        }

        // Normalize Python helper form: py.getattr(obj, name) → obj.getattr(name)
        stats.merge(
            crate::mir::optimizer_passes::normalize::normalize_python_helper_calls(self, module),
        );

        // Pass 1: Dead code elimination (modularized pass)
        {
            let eliminated = crate::mir::passes::dce::eliminate_dead_code(module);
            stats.dead_code_eliminated += eliminated;
        }

        // Pass 2: Pure instruction CSE (modularized)
        {
            let eliminated = crate::mir::passes::cse::eliminate_common_subexpressions(module);
            stats.cse_eliminated += eliminated;
        }

        // Pass 3: Pure instruction reordering for better locality
        stats.merge(crate::mir::optimizer_passes::reorder::reorder_pure_instructions(self, module));

        // Pass 4: Intrinsic function optimization
        stats.merge(
            crate::mir::optimizer_passes::intrinsics::optimize_intrinsic_calls(self, module),
        );

        // Safety-net passesは削除（Phase 2: 変換の一本化）。診断のみ後段で実施。

        // Pass 5: BoxField dependency optimization
        stats.merge(
            crate::mir::optimizer_passes::boxfield::optimize_boxfield_operations(self, module),
        );

        // Pass 6: 受け手型ヒントの伝搬（callsite→callee）
        // 目的: helper(arr){ return arr.length() } のようなケースで、
        //       呼び出し元の引数型（String/Integer/Bool/Float）を callee の params に反映し、
        //       Lowererがより正確にBox種別を選べるようにする。
        let updates = crate::mir::passes::type_hints::propagate_param_type_hints(module);
        if updates > 0 {
            stats.intrinsic_optimizations += updates as usize;
        }

        // Pass 7 (optional): Core-13 pure normalization
        if crate::config::env::mir_core13_pure() {
            stats.merge(crate::mir::optimizer_passes::normalize_core13_pure::normalize_pure_core13(self, module));
        }

        if self.debug {
            println!("✅ Optimization complete: {}", stats);
        }
        // Diagnostics (informational): report unlowered patterns
        let diag1 =
            crate::mir::optimizer_passes::diagnostics::diagnose_unlowered_type_ops(self, module);
        stats.merge(diag1);
        // Diagnostics (policy): detect legacy (pre-unified) instructions when requested
        let diag2 =
            crate::mir::optimizer_passes::diagnostics::diagnose_legacy_instructions(self, module);
        stats.merge(diag2);

        stats
    }

    /// Core-13 "pure" normalization notes moved to optimizer_passes::normalize_core13_pure


    /// Convert instruction to string key for CSE
    #[allow(dead_code)]
    fn instruction_to_key(&self, instruction: &MirInstruction) -> String {
        match instruction {
            MirInstruction::Const { value, .. } => format!("const_{:?}", value),
            MirInstruction::BinOp { op, lhs, rhs, .. } => {
                format!("binop_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32())
            }
            MirInstruction::Compare { op, lhs, rhs, .. } => {
                format!("cmp_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32())
            }
            // BoxFieldLoad removed from instruction set
            // MirInstruction::BoxFieldLoad { box_val, field, .. } => format!("boxload_{}_{}", box_val.as_u32(), field),
            MirInstruction::Call { func, args, .. } => {
                let args_str = args
                    .iter()
                    .map(|v| v.as_u32().to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("call_{}_{}", func.as_u32(), args_str)
            }
            _ => format!("other_{:?}", instruction),
        }
    }

    // Reorder/Intrinsics/BoxField passes moved to optimizer_passes/* modules
}

impl MirOptimizer {
    /// Expose debug flag for helper modules
    pub(crate) fn debug_enabled(&self) -> bool {
        self.debug
    }
}

impl MirOptimizer {
    /// Rewrite all BoxCall to PluginInvoke to force plugin path (no builtin fallback)
    #[allow(dead_code)]
    fn force_plugin_invoke(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::force_plugin_invoke(self, module)
    }

    /// Normalize Python helper calls that route via PyRuntimeBox into proper receiver form.
    ///
    /// Rewrites: PluginInvoke { box_val=py (PyRuntimeBox), method="getattr"|"call", args=[obj, rest...] }
    ///        →  PluginInvoke { box_val=obj, method, args=[rest...] }
    #[allow(dead_code)]
    fn normalize_python_helper_calls(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::normalize_python_helper_calls(self, module)
    }
    /// Normalize legacy instructions into unified MIR26 forms.
    /// - TypeCheck/Cast → TypeOp(Check/Cast)
    /// - WeakNew/WeakLoad → WeakRef(New/Load)
    /// - BarrierRead/BarrierWrite → Barrier(Read/Write)
    /// - Print → ExternCall(env.console.log)
    #[allow(dead_code)]
    fn normalize_legacy_instructions(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::normalize_legacy_instructions(self, module)
    }

    /// Normalize RefGet/RefSet to BoxCall("getField"/"setField") with Const String field argument.
    #[allow(dead_code)]
    fn normalize_ref_field_access(&mut self, module: &mut MirModule) -> OptimizationStats {
        crate::mir::optimizer_passes::normalize::normalize_ref_field_access(self, module)
    }
}

/// Map string type name to MIR type (optimizer-level helper)
#[allow(dead_code)]
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

#[allow(dead_code)]
fn opt_debug_enabled() -> bool {
    crate::config::env::opt_debug()
}
#[allow(dead_code)]
fn opt_debug(msg: &str) {
    if opt_debug_enabled() {
        eprintln!("[OPT] {}", msg);
    }
}

/// Resolve a MIR type from a value id that should represent a type name
/// Supports: Const String("T") and NewBox(StringBox, Const String("T"))
#[allow(dead_code)]
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
                    MirInstruction::Const {
                        value: ConstValue::String(s),
                        ..
                    } => {
                        return Some(map_type_name(s));
                    }
                    MirInstruction::NewBox { box_type, args, .. }
                        if box_type == "StringBox" && args.len() == 1 =>
                    {
                        let inner = args[0];
                        if let Some((sbb, sidx)) = def_map.get(&inner).copied() {
                            if let Some(sblock) = function.blocks.get(&sbb) {
                                if sidx < sblock.instructions.len() {
                                    if let MirInstruction::Const {
                                        value: ConstValue::String(s),
                                        ..
                                    } = &sblock.instructions[sidx]
                                    {
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

// OptimizationStats moved to crate::mir::optimizer_stats

/// Diagnostics: identify unlowered type-ops embedded as strings in Call/BoxCall
#[allow(dead_code)]
fn diagnose_unlowered_type_ops(
    optimizer: &MirOptimizer,
    module: &MirModule,
) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    let diag_on = optimizer.debug || crate::config::env::opt_diag();
    for (fname, function) in &module.functions {
        // def map for resolving constants
        let mut def_map: std::collections::HashMap<
            ValueId,
            (super::basic_block::BasicBlockId, usize),
        > = std::collections::HashMap::new();
        for (bb_id, block) in &function.blocks {
            for (i, inst) in block.instructions.iter().enumerate() {
                if let Some(dst) = inst.dst_value() {
                    def_map.insert(dst, (*bb_id, i));
                }
            }
            if let Some(term) = &block.terminator {
                if let Some(dst) = term.dst_value() {
                    def_map.insert(dst, (*bb_id, usize::MAX));
                }
            }
        }
        let mut count = 0usize;
        for (_bb, block) in &function.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInstruction::BoxCall { method, .. }
                        if method == "is"
                            || method == "as"
                            || method == "isType"
                            || method == "asType" =>
                    {
                        count += 1;
                    }
                    MirInstruction::Call { func, .. } => {
                        if let Some((bb, idx)) = def_map.get(func).copied() {
                            if let Some(b) = function.blocks.get(&bb) {
                                if idx < b.instructions.len() {
                                    if let MirInstruction::Const {
                                        value: super::instruction::ConstValue::String(s),
                                        ..
                                    } = &b.instructions[idx]
                                    {
                                        if s == "isType" || s == "asType" {
                                            count += 1;
                                        }
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
                eprintln!(
                    "[OPT][DIAG] Function '{}' has {} unlowered type-op calls",
                    fname, count
                );
            }
        }
    }
    stats
}

/// Diagnostic: detect legacy instructions that should be unified
/// Legacy set: TypeCheck/Cast/WeakNew/WeakLoad/BarrierRead/BarrierWrite/ArrayGet/ArraySet/RefGet/RefSet/PluginInvoke
/// When NYASH_OPT_DIAG or NYASH_OPT_DIAG_FORBID_LEGACY is set, prints diagnostics.
#[allow(dead_code)]
fn diagnose_legacy_instructions(module: &MirModule, debug: bool) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    let diag_on = debug
        || crate::config::env::opt_diag()
        || crate::config::env::opt_diag_forbid_legacy();
    for (fname, function) in &module.functions {
        let mut count = 0usize;
        for (_bb, block) in &function.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInstruction::TypeCheck { .. }
                    | MirInstruction::Cast { .. }
                    | MirInstruction::WeakNew { .. }
                    | MirInstruction::WeakLoad { .. }
                    | MirInstruction::BarrierRead { .. }
                    | MirInstruction::BarrierWrite { .. }
                    | MirInstruction::ArrayGet { .. }
                    | MirInstruction::ArraySet { .. }
                    | MirInstruction::RefGet { .. }
                    | MirInstruction::RefSet { .. }
                    | MirInstruction::PluginInvoke { .. } => {
                        count += 1;
                    }
                    _ => {}
                }
            }
            if let Some(term) = &block.terminator {
                match term {
                    MirInstruction::TypeCheck { .. }
                    | MirInstruction::Cast { .. }
                    | MirInstruction::WeakNew { .. }
                    | MirInstruction::WeakLoad { .. }
                    | MirInstruction::BarrierRead { .. }
                    | MirInstruction::BarrierWrite { .. }
                    | MirInstruction::ArrayGet { .. }
                    | MirInstruction::ArraySet { .. }
                    | MirInstruction::RefGet { .. }
                    | MirInstruction::RefSet { .. }
                    | MirInstruction::PluginInvoke { .. } => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
        if count > 0 {
            stats.diagnostics_reported += count;
            if diag_on {
                eprintln!(
                    "[OPT][DIAG] Function '{}' has {} legacy MIR ops: unify to Core‑13 (TypeOp/WeakRef/Barrier/BoxCall)",
                    fname, count
                );
                if crate::config::env::opt_diag_forbid_legacy() {
                    panic!(
                        "NYASH_OPT_DIAG_FORBID_LEGACY=1: legacy MIR ops detected in '{}': {}",
                        fname, count
                    );
                }
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        BasicBlock, BasicBlockId, ConstValue, FunctionSignature, MirFunction, MirModule, MirType,
        TypeOpKind, ValueId,
    };

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
    fn test_dce_does_not_drop_typeop_used_by_console_log() {
        // Build: %v=TypeOp(check); extern_call env.console.log(%v); ensure TypeOp remains after optimize
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
        b0.add_instruction(MirInstruction::NewBox {
            dst: v0,
            box_type: "IntegerBox".to_string(),
            args: vec![],
        });
        b0.add_instruction(MirInstruction::TypeOp {
            dst: v1,
            op: TypeOpKind::Check,
            value: v0,
            ty: MirType::Integer,
        });
        b0.add_instruction(MirInstruction::ExternCall {
            dst: None,
            iface_name: "env.console".to_string(),
            method_name: "log".to_string(),
            args: vec![v1],
            effects: super::super::effect::EffectMask::IO,
        });
        b0.add_instruction(MirInstruction::Return { value: None });
        func.add_block(b0);
        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut opt = MirOptimizer::new();
        let _stats = opt.optimize_module(&mut module);

        // Ensure TypeOp remains in bb0
        let f = module.get_function("main").unwrap();
        let block = f.get_block(bb0).unwrap();
        let has_typeop = block
            .all_instructions()
            .any(|i| matches!(i, MirInstruction::TypeOp { .. }));
        assert!(
            has_typeop,
            "TypeOp should not be dropped by DCE when used by console.log (ExternCall)"
        );
    }
}
