use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{BasicBlockId, MirInstruction, MirModule, ValueId};

/// Diagnostic: detect unlowered is/as/isType/asType after Builder
pub fn diagnose_unlowered_type_ops(
    opt: &mut MirOptimizer,
    module: &MirModule,
) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    let diag_on = opt.debug_enabled() || crate::config::env::opt_diag();
    for (fname, function) in &module.functions {
        let mut def_map: std::collections::HashMap<ValueId, (BasicBlockId, usize)> =
            std::collections::HashMap::new();
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
                                        value: crate::mir::instruction::ConstValue::String(s),
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
pub fn diagnose_legacy_instructions(
    opt: &mut MirOptimizer,
    module: &MirModule,
) -> OptimizationStats {
    let mut stats = OptimizationStats::new();
    let diag_on = opt.debug_enabled()
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
