/*!
 * Builtin StringBox Implementation (Phase 15.5: Scheduled for Removal)
 *
 * ⚠️ DEPRECATED: This will be replaced by nyash-string-plugin
 * 🎯 Phase 2.1: Delete this file to remove builtin StringBox support
 */

use crate::box_trait::{NyashBox, StringBox};
use crate::box_factory::RuntimeError;

/// Create builtin StringBox instance
///
/// ⚠️ DEPRECATED: Install nyash-string-plugin instead
pub fn create(args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    eprintln!(
        "⚠️ [DEPRECATED] Using builtin StringBox - install nyash-string-plugin!\n\
        📋 Phase 15.5: Everything is Plugin!\n\
        🔧 Command: cargo build -p nyash-string-plugin --release"
    );

    if let Some(arg0) = args.get(0) {
        if let Some(sb) = arg0.as_any().downcast_ref::<StringBox>() {
            return Ok(Box::new(StringBox::new(&sb.value)));
        }
    }
    Ok(Box::new(StringBox::new("")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_string_box_creation() {
        let result = create(&[]).unwrap();
        assert!(result.as_any().downcast_ref::<StringBox>().is_some());
    }
}