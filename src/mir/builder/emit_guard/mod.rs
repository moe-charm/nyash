use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

/// Finalize call operands (receiver/args) using LocalSSA; thin wrapper to centralize usage.
pub fn finalize_call_operands(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}

/// Verify block schedule invariants after emitting a call (dev-only WARNs inside).
pub fn verify_after_call(builder: &mut MirBuilder) {
    crate::mir::builder::verify::call_order::verify_after_call(builder);
}
