/*!
 * VM Methods Glue
 *
 * Extracted wrappers for Box method dispatch to keep vm.rs slim.
 * These delegate to the real implementation in vm_boxcall.rs, preserving
 * the existing VM API surface.
 */

use super::vm::{VMError, VM};
use crate::box_trait::NyashBox;

impl VM {
    /// Unified method dispatch entry. Currently delegates to `call_box_method_impl`.
    fn call_unified_method(
        &self,
        box_value: Box<dyn NyashBox>,
        method: &str,
        args: Vec<Box<dyn NyashBox>>,
    ) -> Result<Box<dyn NyashBox>, VMError> {
        self.call_box_method_impl(box_value, method, args)
    }

    /// Public-facing method call used by vm_instructions::boxcall.
    /// Kept as a thin wrapper to the implementation in vm_boxcall.rs.
    pub(super) fn call_box_method(
        &self,
        box_value: Box<dyn NyashBox>,
        method: &str,
        args: Vec<Box<dyn NyashBox>>,
    ) -> Result<Box<dyn NyashBox>, VMError> {
        self.call_box_method_impl(box_value, method, args)
    }
}
