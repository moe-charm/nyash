//! Arithmetic operations for Nyash boxes
//!
//! This module provides arithmetic operations between different Box types:
//! - AddBox: Addition and string concatenation
//! - SubtractBox: Subtraction operations
//! - MultiplyBox: Multiplication operations  
//! - DivideBox: Division operations with zero-division error handling
//! - ModuloBox: Modulo operations with zero-modulo error handling
//! - CompareBox: Comparison operations (equals, less, greater, etc.)
//!
//! Each operation is implemented as a separate Box type with execute() method.
//! This provides a clean separation of concerns and makes the code more maintainable.

// Individual arithmetic operation implementations
mod add_box;
mod subtract_box;
mod multiply_box;
mod divide_box;
mod modulo_box;
mod compare_box;

// Re-export all arithmetic box types
pub use add_box::AddBox;
pub use subtract_box::SubtractBox;
pub use multiply_box::MultiplyBox;
pub use divide_box::DivideBox;
pub use modulo_box::ModuloBox;
pub use compare_box::CompareBox;

// Re-export for convenience - common pattern in arithmetic operations
pub use crate::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::IntegerBox;

    #[test]
    fn test_add_box_integers() {
        let left = Box::new(IntegerBox::new(10)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(32)) as Box<dyn NyashBox>;
        let add_box = AddBox::new(left, right);
        let result = add_box.execute();

        assert_eq!(result.to_string_box().value, "42");
    }

    #[test]
    fn test_add_box_strings() {
        let left = Box::new(StringBox::new("Hello, ".to_string())) as Box<dyn NyashBox>;
        let right = Box::new(StringBox::new("World!".to_string())) as Box<dyn NyashBox>;
        let add_box = AddBox::new(left, right);
        let result = add_box.execute();

        assert_eq!(result.to_string_box().value, "Hello, World!");
    }

    #[test]
    fn test_subtract_box() {
        let left = Box::new(IntegerBox::new(50)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(8)) as Box<dyn NyashBox>;
        let sub_box = SubtractBox::new(left, right);
        let result = sub_box.execute();

        assert_eq!(result.to_string_box().value, "42");
    }

    #[test]
    fn test_multiply_box() {
        let left = Box::new(IntegerBox::new(6)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(7)) as Box<dyn NyashBox>;
        let mul_box = MultiplyBox::new(left, right);
        let result = mul_box.execute();

        assert_eq!(result.to_string_box().value, "42");
    }

    #[test]
    fn test_divide_box() {
        let left = Box::new(IntegerBox::new(84)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(2)) as Box<dyn NyashBox>;
        let div_box = DivideBox::new(left, right);
        let result = div_box.execute();

        // Division returns float
        assert_eq!(result.to_string_box().value, "42");
    }

    #[test]
    fn test_divide_by_zero() {
        let left = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(0)) as Box<dyn NyashBox>;
        let div_box = DivideBox::new(left, right);
        let result = div_box.execute();

        assert!(result.to_string_box().value.contains("Division by zero"));
    }

    #[test]
    fn test_modulo_box() {
        let left = Box::new(IntegerBox::new(10)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(3)) as Box<dyn NyashBox>;
        let mod_box = ModuloBox::new(left, right);
        let result = mod_box.execute();

        assert_eq!(result.to_string_box().value, "1");
    }

    #[test]
    fn test_modulo_by_zero() {
        let left = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(0)) as Box<dyn NyashBox>;
        let mod_box = ModuloBox::new(left, right);
        let result = mod_box.execute();

        assert!(result.to_string_box().value.contains("Modulo by zero"));
    }

    #[test]
    fn test_compare_box() {
        let left = IntegerBox::new(10);
        let right = IntegerBox::new(20);

        assert_eq!(CompareBox::less(&left, &right).value, true);
        assert_eq!(CompareBox::greater(&left, &right).value, false);
        assert_eq!(CompareBox::equals(&left, &right).value, false);
    }
}