/*!
 * Nyash Box Trait System - Everything is Box in Rust
 *
 * This module is now a compatibility layer that re-exports from split modules:
 * - box_core.rs: Core trait definitions (rarely change)
 * - box_registry.rs: Built-in box registry (changes when adding new boxes)
 *
 * This split reduces compilation cascades - changes to the registry won't
 * trigger recompilation of files that only need the core traits.
 */

// ===== Core Traits & Types (from box_core.rs) =====
pub use crate::box_core::{
    BoxBase, BoxCore, NyashBox, SharedNyashBox, next_box_id,
};

// ===== Built-in Box Registry (from box_registry.rs) =====
pub use crate::box_registry::{BUILTIN_BOXES, is_builtin_box};

// ===== Basic Box Types (Re-exported from basic module) =====

// Re-export all basic box types from the dedicated basic module
pub use crate::boxes::basic::{
    BoolBox, ErrorBox, FileBox, IntegerBox, StringBox, VoidBox,
};







// Old Box implementations have been moved to separate files
// ArrayBox is now defined in boxes::array module
pub use crate::boxes::array::ArrayBox;

// FutureBox is now implemented in src/boxes/future/mod.rs using RwLock pattern
// and re-exported from src/boxes/mod.rs as both NyashFutureBox and FutureBox

// Re-export operation boxes from the dedicated operations module
pub use crate::box_arithmetic::{
    AddBox, CompareBox, DivideBox, ModuloBox, MultiplyBox, SubtractBox,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_box_creation() {
        let s = StringBox::new("Hello, Rust!");
        assert_eq!(s.value, "Hello, Rust!");
        assert_eq!(s.type_name(), "StringBox");
        assert_eq!(s.to_string_box().value, "Hello, Rust!");
    }

    #[test]
    fn test_integer_box_creation() {
        let i = IntegerBox::new(42);
        assert_eq!(i.value, 42);
        assert_eq!(i.type_name(), "IntegerBox");
        assert_eq!(i.to_string_box().value, "42");
    }

    #[test]
    fn test_bool_box_creation() {
        let b = BoolBox::new(true);
        assert_eq!(b.value, true);
        assert_eq!(b.type_name(), "BoolBox");
        assert_eq!(b.to_string_box().value, "true");
    }

    #[test]
    fn test_box_equality() {
        let s1 = StringBox::new("test");
        let s2 = StringBox::new("test");
        let s3 = StringBox::new("different");

        assert!(s1.equals(&s2).value);
        assert!(!s1.equals(&s3).value);
    }

    #[test]
    fn test_add_box_integers() {
        let left = Box::new(IntegerBox::new(5)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(3)) as Box<dyn NyashBox>;
        let add = AddBox::new(left, right);

        let result = add.execute();
        let result_int = result.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(result_int.value, 8);
    }

    #[test]
    fn test_add_box_strings() {
        let left = Box::new(StringBox::new("Hello, ")) as Box<dyn NyashBox>;
        let right = Box::new(StringBox::new("Rust!")) as Box<dyn NyashBox>;
        let add = AddBox::new(left, right);

        let result = add.execute();
        let result_str = result.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(result_str.value, "Hello, Rust!");
    }

    #[test]
    fn test_box_ids_unique() {
        let s1 = StringBox::new("test");
        let s2 = StringBox::new("test");

        // Same content but different IDs
        assert_ne!(s1.box_id(), s2.box_id());
    }

    #[test]
    fn test_void_box() {
        let v = VoidBox::new();
        assert_eq!(v.type_name(), "VoidBox");
        assert_eq!(v.to_string_box().value, "void");
    }
}
