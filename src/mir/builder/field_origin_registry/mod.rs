/*!
 * FieldOriginRegistryBox - Centralized field origin tracking
 *
 * Tracks field origins for optimizations and type inference:
 * - Value-level: (base_id, field) -> origin_class
 * - Box-level: (base_box, field) -> origin_class
 */

use crate::mir::ValueId;
use std::collections::HashMap;

/// Field origin registry for centralized tracking
#[derive(Debug, Clone, Default)]
pub struct FieldOriginRegistryBox {
    /// Value-level field origins: (base_id, field) -> origin_class
    value_field_origins: HashMap<(ValueId, String), String>,

    /// Box-level field origins: (base_box, field) -> origin_class
    box_field_origins: HashMap<(String, String), String>,
}

impl FieldOriginRegistryBox {
    /// Create a new field origin registry
    pub fn new() -> Self {
        Self {
            value_field_origins: HashMap::new(),
            box_field_origins: HashMap::new(),
        }
    }

    /// Register a value-level field origin
    pub fn register_value_field(&mut self, base: ValueId, field: String, origin_class: String) {
        self.value_field_origins.insert((base, field), origin_class);
    }

    /// Register a box-level field origin
    pub fn register_box_field(&mut self, base_box: String, field: String, origin_class: String) {
        self.box_field_origins.insert((base_box, field), origin_class);
    }

    /// Infer field origin from value-level or box-level registry
    ///
    /// Lookup order:
    /// 1. Value-level: (base_id, field) -> origin_class
    /// 2. Box-level: (base_box_hint, field) -> origin_class
    pub fn infer_field_origin(
        &self,
        base: ValueId,
        field: &str,
        base_box_hint: Option<&str>,
    ) -> Option<String> {
        // 1. Try value-level lookup
        if let Some(origin) = self.value_field_origins.get(&(base, field.to_string())) {
            return Some(origin.clone());
        }

        // 2. Try box-level lookup with hint
        if let Some(base_box) = base_box_hint {
            if let Some(origin) = self.box_field_origins.get(&(base_box.to_string(), field.to_string())) {
                return Some(origin.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_origin_registry_value_level() {
        let mut registry = FieldOriginRegistryBox::new();
        let base = ValueId::new(1);

        registry.register_value_field(base, "name".to_string(), "StringBox".to_string());

        assert_eq!(
            registry.infer_field_origin(base, "name", None),
            Some("StringBox".to_string())
        );
        assert_eq!(registry.infer_field_origin(base, "age", None), None);
    }

    #[test]
    fn test_field_origin_registry_box_level() {
        let mut registry = FieldOriginRegistryBox::new();

        registry.register_box_field("Person".to_string(), "name".to_string(), "StringBox".to_string());

        let base = ValueId::new(1);
        assert_eq!(
            registry.infer_field_origin(base, "name", Some("Person")),
            Some("StringBox".to_string())
        );
        assert_eq!(registry.infer_field_origin(base, "name", Some("Animal")), None);
    }

    #[test]
    fn test_field_origin_registry_priority() {
        let mut registry = FieldOriginRegistryBox::new();
        let base = ValueId::new(1);

        // Register both value-level and box-level
        registry.register_value_field(base, "name".to_string(), "StringBox".to_string());
        registry.register_box_field("Person".to_string(), "name".to_string(), "TextBox".to_string());

        // Value-level should take priority
        assert_eq!(
            registry.infer_field_origin(base, "name", Some("Person")),
            Some("StringBox".to_string())
        );
    }
}
