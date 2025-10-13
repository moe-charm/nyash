#![cfg(feature = "legacy-boxes")]
/*!
 * Builtin ConsoleBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-console-plugin (exists!)
 * 🎯 Phase 2.6: Delete this file to remove builtin ConsoleBox support (LAST)
 */

use crate::box_trait::NyashBox;
use crate::box_factory::RuntimeError;

/// Create builtin ConsoleBox instance
///
/// ⚠️ DEPRECATED: ConsoleBox plugin should replace this (check plugins/nyash-console-plugin)
pub fn create(_args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    eprintln!(
        "⚠️ [DEPRECATED] Using builtin ConsoleBox - use nyash-console-plugin!\n\
        📋 Phase 15.5: Everything is Plugin!\n\
        🔧 Check: plugins/nyash-console-plugin\n\
        ⚠️ WARNING: ConsoleBox is critical for logging - remove LAST!"
    );

    Ok(Box::new(crate::boxes::console_box::ConsoleBox::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boxes::console_box::ConsoleBox;

    #[test]
    fn test_builtin_console_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<ConsoleBox>().is_some());
    }
}
