/*!
 * extern_registry – thin re-export helpers around MIR externs registry
 *
 * Provides a stable facade for builder/VM without exposing internal layout.
 */

use crate::mir::externs::registry::{effects_for as reg_effects_for, registry as reg_registry};

pub fn exists(iface: &str, method: &str) -> bool {
    reg_registry().get(iface, method).is_some()
}

pub fn effects_for(iface: &str, method: &str) -> Option<crate::mir::EffectMask> {
    reg_effects_for(iface, method)
}
