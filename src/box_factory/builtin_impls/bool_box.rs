/*!
 * Builtin BoolBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-bool-plugin (to be created)
 * 🎯 Phase 2.3: Delete this file to remove builtin BoolBox support
 */

use crate::box_trait::{NyashBox, BoolBox};
use crate::box_factory::RuntimeError;

/// Create builtin BoolBox instance
///
/// ⚠️ DEPRECATED: BoolBox plugin needs to be created
pub fn create(args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    eprintln!(
        "⚠️ [DEPRECATED] Using builtin BoolBox - BoolBox plugin needed!\n\
        📋 Phase 15.5: Everything is Plugin!\n\
        🔧 TODO: Create nyash-bool-plugin"
    );

    if let Some(arg0) = args.get(0) {
        if let Some(bb) = arg0.as_any().downcast_ref::<BoolBox>() {
            return Ok(Box::new(BoolBox::new(bb.value)));
        }
    }
    Ok(Box::new(BoolBox::new(false)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_bool_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<BoolBox>().is_some());
    }
}