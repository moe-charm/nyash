/*!
 * Builtin IntegerBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-integer-plugin
 * 🎯 Phase 2.2: Delete this file to remove builtin IntegerBox support
 */

use crate::box_trait::{NyashBox, IntegerBox};
use crate::box_factory::RuntimeError;

/// Create builtin IntegerBox instance
///
/// ⚠️ DEPRECATED: Install nyash-integer-plugin instead
pub fn create(args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    eprintln!(
        "⚠️ [DEPRECATED] Using builtin IntegerBox - install nyash-integer-plugin!\n\
        📋 Phase 15.5: Everything is Plugin!\n\
        🔧 Command: cargo build -p nyash-integer-plugin --release"
    );

    if let Some(arg0) = args.get(0) {
        if let Some(ib) = arg0.as_any().downcast_ref::<IntegerBox>() {
            return Ok(Box::new(IntegerBox::new(ib.value)));
        }
    }
    Ok(Box::new(IntegerBox::new(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_integer_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<IntegerBox>().is_some());
    }
}