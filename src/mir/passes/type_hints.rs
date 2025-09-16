//! Propagate simple param type hints from callsites to callees.
//!
//! This is a minimal, no-op scaffold used to keep the build green
//! while the full pass is being implemented. It returns 0 updates.

use crate::mir::MirModule;

/// Walks the module and would propagate basic type hints when implemented.
/// Returns the number of updates applied.
pub fn propagate_param_type_hints(_module: &mut MirModule) -> u32 {
    0
}
