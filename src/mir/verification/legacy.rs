use crate::mir::verification_types::VerificationError;
use crate::mir::{function::MirFunction, MirInstruction};

/// Reject legacy instructions that should be rewritten to Core-15 equivalents
/// Skips check when NYASH_VERIFY_ALLOW_LEGACY=1
pub fn check_no_legacy_ops(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    if std::env::var("NYASH_VERIFY_ALLOW_LEGACY").ok().as_deref() == Some("1") {
        return Ok(());
    }
    let mut errors = Vec::new();
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.all_instructions().enumerate() {
            let legacy_name = match inst {
                MirInstruction::TypeCheck { .. } => Some("TypeCheck"), // -> TypeOp(Check)
                MirInstruction::Cast { .. } => Some("Cast"),           // -> TypeOp(Cast)
                MirInstruction::WeakNew { .. } => Some("WeakNew"),     // -> WeakRef(New)
                MirInstruction::WeakLoad { .. } => Some("WeakLoad"),   // -> WeakRef(Load)
                MirInstruction::BarrierRead { .. } => Some("BarrierRead"), // -> Barrier(Read)
                MirInstruction::BarrierWrite { .. } => Some("BarrierWrite"), // -> Barrier(Write)
                MirInstruction::Print { .. } => Some("Print"), // -> ExternCall(env.console.log)
                MirInstruction::ArrayGet { .. } => Some("ArrayGet"), // -> BoxCall("get")
                MirInstruction::ArraySet { .. } => Some("ArraySet"), // -> BoxCall("set")
                MirInstruction::RefGet { .. } => Some("RefGet"), // -> BoxCall("getField")
                MirInstruction::RefSet { .. } => Some("RefSet"), // -> BoxCall("setField")
                MirInstruction::PluginInvoke { .. } => Some("PluginInvoke"), // -> BoxCall
                _ => None,
            };
            if let Some(name) = legacy_name {
                errors.push(VerificationError::UnsupportedLegacyInstruction {
                    block: *bid,
                    instruction_index: idx,
                    name: name.to_string(),
                });
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
