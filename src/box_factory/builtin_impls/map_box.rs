/*!
 * Builtin MapBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-map-plugin (exists?)
 * 🎯 Phase 2.5: Delete this file to remove builtin MapBox support
 */

use crate::box_trait::NyashBox;
use crate::box_factory::RuntimeError;

/// Create builtin MapBox instance
///
/// ⚠️ DEPRECATED: Check if nyash-map-plugin exists
pub fn create(_args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    let verbose = std::env::var("NYASH_DEPRECATED_BUILTIN_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
    if verbose {
        eprintln!(
            "⚠️ [DEPRECATED] Using builtin MapBox - check nyash-map-plugin!\n\
            📋 Phase 15.5: Everything is Plugin!\n\
            🔧 Check: plugins/nyash-map-plugin"
        );
    }

    Ok(Box::new(crate::boxes::map_box::MapBox::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boxes::map_box::MapBox;

    #[test]
    fn test_builtin_map_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<MapBox>().is_some());
    }
}
