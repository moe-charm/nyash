use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;

/// Delegate: legacy normalization (moved from optimizer.rs)
pub fn normalize_legacy_instructions(opt: &mut MirOptimizer, module: &mut crate::mir::MirModule) -> OptimizationStats {
    crate::mir::optimizer_passes::normalize::normalize_legacy_instructions(opt, module)
}

