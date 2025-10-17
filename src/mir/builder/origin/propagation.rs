//! Method-specific origin propagation rules
//!
//! Purpose: Propagate origin metadata through method calls
//! - substring/indexOf/lastIndexOf: StringBox → StringBox
//! - slice: ArrayBox → ArrayBox
//! - keys/values: MapBox → ArrayBox
//!
//! Design: Explicit method whitelist (fail-safe)

use crate::mir::ValueId;
use crate::mir::builder::MirBuilder;

/// Propagate origin from receiver to result for specific methods
///
/// # Arguments
/// * `builder` - MIR builder with origin tracking
/// * `receiver` - Method receiver ValueId
/// * `method` - Method name
/// * `result` - Result ValueId to propagate to
///
/// # Returns
/// * `true` if propagation occurred, `false` if method not in whitelist
pub fn propagate_method_result(
    builder: &mut MirBuilder,
    receiver: ValueId,
    method: &str,
    result: ValueId,
) -> bool {
    // Get receiver origin (if any)
    let recv_origin = match builder.origin_get(receiver) {
        Some(o) => o.to_string(),
        None => return false, // No origin to propagate
    };

    // Method-specific propagation rules
    let result_origin = match method {
        // StringBox → StringBox
        "substring" | "indexOf" | "lastIndexOf" | "trim" | "toUpperCase" | "toLowerCase" => {
            if recv_origin == "StringBox" {
                Some("StringBox".to_string())
            } else {
                None
            }
        }

        // ArrayBox → ArrayBox
        "slice" => {
            if recv_origin == "ArrayBox" {
                Some("ArrayBox".to_string())
            } else {
                None
            }
        }

        // MapBox → ArrayBox
        "keys" | "values" => {
            if recv_origin == "MapBox" {
                Some("ArrayBox".to_string())
            } else {
                None
            }
        }

        // Not in whitelist
        _ => None,
    };

    // Apply propagation if rule matched
    if let Some(origin) = result_origin {
        builder.origin_register(result, origin);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirBuilder, ValueId};

    #[test]
    fn test_substring_propagation() {
        let mut builder = MirBuilder::new();
        let recv = builder.safe_next_value();
        let result = builder.safe_next_value();

        // Register receiver as StringBox
        builder.origin_register(recv, "StringBox".to_string());

        // Propagate through substring
        assert!(propagate_method_result(&mut builder, recv, "substring", result));
        assert_eq!(builder.origin_get(result), Some("StringBox"));
    }

    #[test]
    fn test_keys_propagation() {
        let mut builder = MirBuilder::new();
        let recv = builder.safe_next_value();
        let result = builder.safe_next_value();

        // Register receiver as MapBox
        builder.origin_register(recv, "MapBox".to_string());

        // Propagate through keys (MapBox → ArrayBox)
        assert!(propagate_method_result(&mut builder, recv, "keys", result));
        assert_eq!(builder.origin_get(result), Some("ArrayBox"));
    }

    #[test]
    fn test_no_origin_no_propagation() {
        let mut builder = MirBuilder::new();
        let recv = builder.safe_next_value();
        let result = builder.safe_next_value();

        // No origin registered for receiver
        assert!(!propagate_method_result(&mut builder, recv, "substring", result));
        assert_eq!(builder.origin_get(result), None);
    }

    #[test]
    fn test_unknown_method_no_propagation() {
        let mut builder = MirBuilder::new();
        let recv = builder.safe_next_value();
        let result = builder.safe_next_value();

        // Register receiver as StringBox
        builder.origin_register(recv, "StringBox".to_string());

        // Unknown method - no propagation
        assert!(!propagate_method_result(&mut builder, recv, "unknown_method", result));
        assert_eq!(builder.origin_get(result), None);
    }
}
