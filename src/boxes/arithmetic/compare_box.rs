//! CompareBox - Comparison operations for all Box types
//!
//! Implements comparison operations (equals, less, greater, etc.) with integer and string fallback.

use crate::box_trait::{BoolBox, IntegerBox, NyashBox};

/// Comparison operations between boxes
pub struct CompareBox;

impl CompareBox {
    /// Compare two boxes for equality
    pub fn equals(left: &dyn NyashBox, right: &dyn NyashBox) -> BoolBox {
        left.equals(right)
    }

    /// Compare two boxes for less than
    pub fn less(left: &dyn NyashBox, right: &dyn NyashBox) -> BoolBox {
        // Try integer comparison first
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>(),
        ) {
            return BoolBox::new(left_int.value < right_int.value);
        }

        // Fall back to string comparison
        let left_str = left.to_string_box();
        let right_str = right.to_string_box();
        BoolBox::new(left_str.value < right_str.value)
    }

    /// Compare two boxes for greater than
    pub fn greater(left: &dyn NyashBox, right: &dyn NyashBox) -> BoolBox {
        // Try integer comparison first
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>(),
        ) {
            return BoolBox::new(left_int.value > right_int.value);
        }

        // Fall back to string comparison
        let left_str = left.to_string_box();
        let right_str = right.to_string_box();
        BoolBox::new(left_str.value > right_str.value)
    }

    /// Compare two boxes for less than or equal
    pub fn less_equal(left: &dyn NyashBox, right: &dyn NyashBox) -> BoolBox {
        // Try integer comparison first
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>(),
        ) {
            return BoolBox::new(left_int.value <= right_int.value);
        }

        // Fall back to string comparison
        let left_str = left.to_string_box();
        let right_str = right.to_string_box();
        BoolBox::new(left_str.value <= right_str.value)
    }

    /// Compare two boxes for greater than or equal
    pub fn greater_equal(left: &dyn NyashBox, right: &dyn NyashBox) -> BoolBox {
        // Try integer comparison first
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>(),
        ) {
            return BoolBox::new(left_int.value >= right_int.value);
        }

        // Fall back to string comparison
        let left_str = left.to_string_box();
        let right_str = right.to_string_box();
        BoolBox::new(left_str.value >= right_str.value)
    }
}