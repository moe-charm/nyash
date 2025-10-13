#![cfg(feature = "legacy-boxes")]
/*!
 * Builtin NullBox Implementation (Phase 15.5: Consider Keeping?)
 *
 * 🤔 CONSIDERATION: NullBox might be fundamental enough to remain builtin
 * 📋 Discussion needed: Is null a language primitive or plugin concern?
 */

use crate::box_trait::NyashBox;
use crate::box_factory::RuntimeError;

/// Create builtin NullBox instance
///
/// 🤔 DISCUSSION: Should null remain as builtin language primitive?
pub fn create(_args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    // Note: No deprecation warning - null might remain builtin
    Ok(Box::new(crate::boxes::null_box::NullBox::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boxes::null_box::NullBox;

    #[test]
    fn test_builtin_null_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<NullBox>().is_some());
    }
}
