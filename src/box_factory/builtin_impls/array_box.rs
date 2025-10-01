/*!
 * Builtin ArrayBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-array-plugin (exists?)
 * 🎯 Phase 2.4: Delete this file to remove builtin ArrayBox support
 */

use crate::box_trait::NyashBox;
use crate::box_factory::RuntimeError;

/// Create builtin ArrayBox instance
///
/// ⚠️ DEPRECATED: Check if nyash-array-plugin exists
pub fn create(_args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    let verbose = std::env::var("NYASH_DEPRECATED_BUILTIN_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
    if verbose {
        eprintln!(
            "⚠️ [DEPRECATED] Using builtin ArrayBox - check nyash-array-plugin!\n\
            📋 Phase 15.5: Everything is Plugin!\n\
            🔧 Check: plugins/nyash-array-plugin"
        );
    }

    Ok(Box::new(crate::boxes::array::ArrayBox::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boxes::array::ArrayBox;

    #[test]
    fn test_builtin_array_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<ArrayBox>().is_some());
    }
}
