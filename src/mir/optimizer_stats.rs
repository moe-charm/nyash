/*!
 * Optimizer statistics (extracted from optimizer.rs)
 */

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
        self.dead_code_eliminated
            + self.cse_eliminated
            + self.reorderings
            + self.intrinsic_optimizations
            + self.boxfield_optimizations
    }
}

impl std::fmt::Display for OptimizationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
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
