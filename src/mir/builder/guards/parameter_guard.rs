use crate::mir::{MirInstruction, ValueId};

/// ParameterGuardBox — prevents overwriting function parameter registers (v%0..v%N).
///
/// Enabled by default; can be disabled with ENV `NYASH_BUILDER_PARAM_GUARD=0` (or `false`/`off`).
pub struct ParameterGuardBox {
    enabled: bool,
}

impl ParameterGuardBox {
    pub fn new() -> Self {
        let enabled = crate::config::env::builder_param_guard_enabled();
        Self { enabled }
    }

    #[inline]
    pub fn check(&self, inst: &MirInstruction, params: &[ValueId]) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        if let Some(dst) = Self::extract_destination(inst) {
            if params.contains(&dst) {
                return Err(format!(
                    "Cannot overwrite parameter register {} (function has {} params)",
                    dst,
                    params.len()
                ));
            }
        }
        Ok(())
    }

    #[inline]
    fn extract_destination(inst: &MirInstruction) -> Option<ValueId> {
        match inst {
            MirInstruction::Const { dst, .. }
            | MirInstruction::BinOp { dst, .. }
            | MirInstruction::UnaryOp { dst, .. }
            | MirInstruction::Compare { dst, .. }
            | MirInstruction::Load { dst, .. }
            | MirInstruction::NewClosure { dst, .. }
            | MirInstruction::Phi { dst, .. }
            | MirInstruction::NewBox { dst, .. }
            | MirInstruction::TypeCheck { dst, .. }
            | MirInstruction::Cast { dst, .. }
            | MirInstruction::TypeOp { dst, .. }
            | MirInstruction::Copy { dst, .. }
            | MirInstruction::RefNew { dst, .. }
            | MirInstruction::WeakNew { dst, .. }
            | MirInstruction::WeakLoad { dst, .. } => Some(*dst),
            MirInstruction::Call { dst, .. } | MirInstruction::BoxCall { dst, .. } => dst.clone(),
            _ => None,
        }
    }
}

