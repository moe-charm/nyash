/*!
 * WeakFieldRegistryBox - Centralized weak field tracking
 *
 * Tracks which fields are weak references for each Box type:
 * - Simplifies weak field checks (3-line nesting → 1-line call)
 * - Centralized registration during box declaration lowering
 */

use std::collections::{HashMap, HashSet};

/// Weak field registry for centralized tracking
#[derive(Debug, Clone, Default)]
pub struct WeakFieldRegistryBox {
    /// BoxName -> {weak field names}
    weak_fields: HashMap<String, HashSet<String>>,
}

impl WeakFieldRegistryBox {
    /// Create a new weak field registry
    pub fn new() -> Self {
        Self {
            weak_fields: HashMap::new(),
        }
    }

    /// Register weak fields for a box type
    pub fn register(&mut self, box_name: String, weak_fields: HashSet<String>) {
        if !weak_fields.is_empty() {
            self.weak_fields.insert(box_name, weak_fields);
        }
    }

    /// Check if a field is weak for a given box type
    pub fn is_weak_field(&self, box_name: &str, field_name: &str) -> bool {
        self.weak_fields
            .get(box_name)
            .map(|set| set.contains(field_name))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_field_registry_basic() {
        let mut registry = WeakFieldRegistryBox::new();

        let mut weak_fields = HashSet::new();
        weak_fields.insert("child".to_string());
        weak_fields.insert("next".to_string());

        registry.register("TreeNode".to_string(), weak_fields);

        assert!(registry.is_weak_field("TreeNode", "child"));
        assert!(registry.is_weak_field("TreeNode", "next"));
        assert!(!registry.is_weak_field("TreeNode", "value"));
        assert!(!registry.is_weak_field("OtherBox", "child"));
    }

    #[test]
    fn test_weak_field_registry_empty() {
        let registry = WeakFieldRegistryBox::new();

        assert!(!registry.is_weak_field("TreeNode", "child"));
    }

    #[test]
    fn test_weak_field_registry_multiple_boxes() {
        let mut registry = WeakFieldRegistryBox::new();

        let mut tree_weak = HashSet::new();
        tree_weak.insert("left".to_string());
        tree_weak.insert("right".to_string());
        registry.register("BinaryTree".to_string(), tree_weak);

        let mut list_weak = HashSet::new();
        list_weak.insert("next".to_string());
        registry.register("LinkedList".to_string(), list_weak);

        assert!(registry.is_weak_field("BinaryTree", "left"));
        assert!(registry.is_weak_field("BinaryTree", "right"));
        assert!(!registry.is_weak_field("BinaryTree", "next"));

        assert!(registry.is_weak_field("LinkedList", "next"));
        assert!(!registry.is_weak_field("LinkedList", "left"));
    }
}
