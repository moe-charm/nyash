// MIR optimization subpasses module
// Minimal scaffold to unblock builds when type hint propagation is not yet implemented.

pub mod cse;
pub mod dce;
pub mod escape;
pub mod method_id_inject;
pub mod type_hints;

/// Minimal pass trait for future expansion. Currently unused by the main
/// optimizer pipeline but provided to guide modularization.
pub trait MirPass {
    fn name(&self) -> &'static str;
    fn run(&mut self, module: &mut crate::mir::MirModule);
}
