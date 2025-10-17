use crate::mir::{MirInstruction, ValueId};

/// ParameterGuardBox — prevents overwriting function parameter registers (v%0..v%N).
///
/// Enabled by default; can be disabled with ENV `NYASH_BUILDER_PARAM_GUARD=0` (or `false`/`off`).
pub struct ParameterGuardBox {
    enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn with_guard_env<F: FnOnce()>(nyash: Option<&str>, hako: Option<&str>, f: F)
    where
        F: FnOnce(),
    {
        let _lock = ENV_GUARD.lock().unwrap();
        match nyash {
            Some(v) => std::env::set_var("NYASH_BUILDER_PARAM_GUARD", v),
            None => std::env::remove_var("NYASH_BUILDER_PARAM_GUARD"),
        }
        match hako {
            Some(v) => std::env::set_var("HAKO_BUILDER_PARAM_GUARD", v),
            None => std::env::remove_var("HAKO_BUILDER_PARAM_GUARD"),
        }

        f();

        std::env::remove_var("NYASH_BUILDER_PARAM_GUARD");
        std::env::remove_var("HAKO_BUILDER_PARAM_GUARD");
    }

    #[test]
    fn guard_blocks_parameter_overwrite_when_enabled() {
        with_guard_env(Some("1"), None, || {
            let guard = ParameterGuardBox::new();
            let inst = MirInstruction::Copy {
                dst: ValueId::new(0),
                src: ValueId::new(2),
            };
            let params = [ValueId::new(0), ValueId::new(1)];
            assert!(
                guard.check(&inst, &params).is_err(),
                "parameter overwrite should fail when guard is enabled"
            );
        });
    }

    #[test]
    fn guard_allows_overwrite_when_disabled() {
        with_guard_env(Some("0"), Some("0"), || {
            let guard = ParameterGuardBox::new();
            let inst = MirInstruction::Copy {
                dst: ValueId::new(0),
                src: ValueId::new(2),
            };
            let params = [ValueId::new(0), ValueId::new(1)];
            assert!(
                guard.check(&inst, &params).is_ok(),
                "guard disabled via env should permit overwrite"
            );
        });
    }
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
