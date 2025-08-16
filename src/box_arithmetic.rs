/*!
 * Box Operations - Binary and unary operations between boxes
 * 
 * This module contains the implementation of operation boxes that perform
 * arithmetic, logical, and comparison operations between different Box types.
 */

use crate::box_trait::{NyashBox, BoxCore, StringBox, IntegerBox, BoolBox, BoxBase};
use std::fmt::{Debug, Display};
use std::any::Any;

// ===== Binary Operation Boxes =====

/// Binary operations between boxes (addition, concatenation, etc.)
pub struct AddBox {
    pub left: Box<dyn NyashBox>,
    pub right: Box<dyn NyashBox>,
    base: BoxBase,
}

impl AddBox {
    pub fn new(left: Box<dyn NyashBox>, right: Box<dyn NyashBox>) -> Self {
        Self { 
            left, 
            right, 
            base: BoxBase::new(),
        }
    }
    
    /// Execute the addition operation and return the result
    pub fn execute(&self) -> Box<dyn NyashBox> {
        use crate::boxes::math_box::FloatBox;
        
        // 1. Integer + Integer
        if let (Some(left_int), Some(right_int)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            let result = left_int.value + right_int.value;
            return Box::new(IntegerBox::new(result));
        }
        
        // 2. Float + Float (or mixed with Integer)
        if let (Some(left_float), Some(right_float)) = (
            self.left.as_any().downcast_ref::<FloatBox>(),
            self.right.as_any().downcast_ref::<FloatBox>()
        ) {
            let result = left_float.value + right_float.value;
            return Box::new(FloatBox::new(result));
        }
        
        // 3. Integer + Float
        if let (Some(left_int), Some(right_float)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<FloatBox>()
        ) {
            let result = left_int.value as f64 + right_float.value;
            return Box::new(FloatBox::new(result));
        }
        
        // 4. Float + Integer
        if let (Some(left_float), Some(right_int)) = (
            self.left.as_any().downcast_ref::<FloatBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            let result = left_float.value + right_int.value as f64;
            return Box::new(FloatBox::new(result));
        }
        
        // 5. String concatenation (fallback for any types)
        let left_str = self.left.to_string_box();
        let right_str = self.right.to_string_box();
        let result = format!("{}{}", left_str.value, right_str.value);
        Box::new(StringBox::new(result))
    }
}

impl Debug for AddBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddBox")
            .field("left", &self.left.to_string_box().value)
            .field("right", &self.right.to_string_box().value)
            .field("id", &self.base.id)
            .finish()
    }
}

impl NyashBox for AddBox {
    fn to_string_box(&self) -> StringBox {
        let result = self.execute();
        result.to_string_box()
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_add) = other.as_any().downcast_ref::<AddBox>() {
            BoolBox::new(
                self.left.equals(other_add.left.as_ref()).value && 
                self.right.equals(other_add.right.as_ref()).value
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "AddBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(AddBox::new(
            self.left.clone_box(),
            self.right.clone_box()
        ))
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for AddBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for AddBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Subtraction operations between boxes
pub struct SubtractBox {
    pub left: Box<dyn NyashBox>,
    pub right: Box<dyn NyashBox>,
    base: BoxBase,
}

impl SubtractBox {
    pub fn new(left: Box<dyn NyashBox>, right: Box<dyn NyashBox>) -> Self {
        Self { 
            left, 
            right, 
            base: BoxBase::new(),
        }
    }
    
    /// Execute the subtraction operation and return the result
    pub fn execute(&self) -> Box<dyn NyashBox> {
        // For now, only handle integer subtraction
        if let (Some(left_int), Some(right_int)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            let result = left_int.value - right_int.value;
            Box::new(IntegerBox::new(result))
        } else {
            // Convert to integers and subtract
            // For simplicity, default to 0 for non-integer types
            let left_val = if let Some(int_box) = self.left.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let result = left_val - right_val;
            Box::new(IntegerBox::new(result))
        }
    }
}

impl Debug for SubtractBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubtractBox")
            .field("left", &self.left.to_string_box().value)
            .field("right", &self.right.to_string_box().value)
            .field("id", &self.base.id)
            .finish()
    }
}

impl NyashBox for SubtractBox {
    fn to_string_box(&self) -> StringBox {
        let result = self.execute();
        result.to_string_box()
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_sub) = other.as_any().downcast_ref::<SubtractBox>() {
            BoolBox::new(
                self.left.equals(other_sub.left.as_ref()).value && 
                self.right.equals(other_sub.right.as_ref()).value
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str { "SubtractBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(SubtractBox::new(self.left.clone_box(), self.right.clone_box()))
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for SubtractBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Display for SubtractBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Multiplication operations between boxes
pub struct MultiplyBox {
    pub left: Box<dyn NyashBox>,
    pub right: Box<dyn NyashBox>,
    base: BoxBase,
}

impl MultiplyBox {
    pub fn new(left: Box<dyn NyashBox>, right: Box<dyn NyashBox>) -> Self {
        Self { 
            left, 
            right, 
            base: BoxBase::new(),
        }
    }
    
    /// Execute the multiplication operation and return the result
    pub fn execute(&self) -> Box<dyn NyashBox> {
        // For now, only handle integer multiplication
        if let (Some(left_int), Some(right_int)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            let result = left_int.value * right_int.value;
            Box::new(IntegerBox::new(result))
        } else {
            // Convert to integers and multiply
            let left_val = if let Some(int_box) = self.left.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let result = left_val * right_val;
            Box::new(IntegerBox::new(result))
        }
    }
}

impl Debug for MultiplyBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiplyBox")
            .field("left", &self.left.to_string_box().value)
            .field("right", &self.right.to_string_box().value)
            .field("id", &self.base.id)
            .finish()
    }
}

impl NyashBox for MultiplyBox {
    fn to_string_box(&self) -> StringBox {
        let result = self.execute();
        result.to_string_box()
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_mul) = other.as_any().downcast_ref::<MultiplyBox>() {
            BoolBox::new(
                self.left.equals(other_mul.left.as_ref()).value && 
                self.right.equals(other_mul.right.as_ref()).value
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str { "MultiplyBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(MultiplyBox::new(self.left.clone_box(), self.right.clone_box()))
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for MultiplyBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Display for MultiplyBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Division operations between boxes
pub struct DivideBox {
    pub left: Box<dyn NyashBox>,
    pub right: Box<dyn NyashBox>,
    base: BoxBase,
}

impl DivideBox {
    pub fn new(left: Box<dyn NyashBox>, right: Box<dyn NyashBox>) -> Self {
        Self { 
            left, 
            right, 
            base: BoxBase::new(),
        }
    }
    
    /// Execute the division operation and return the result
    pub fn execute(&self) -> Box<dyn NyashBox> {
        use crate::boxes::math_box::FloatBox;
        
        // Handle integer division, but return float result
        if let (Some(left_int), Some(right_int)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            if right_int.value == 0 {
                // Return error for division by zero
                return Box::new(StringBox::new("Error: Division by zero".to_string()));
            }
            let result = left_int.value as f64 / right_int.value as f64;
            Box::new(FloatBox::new(result))
        } else {
            // Convert to integers and divide
            let left_val = if let Some(int_box) = self.left.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                1 // Avoid division by zero
            };
            if right_val == 0 {
                return Box::new(StringBox::new("Error: Division by zero".to_string()));
            }
            let result = left_val as f64 / right_val as f64;
            Box::new(FloatBox::new(result))
        }
    }
}

impl Debug for DivideBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DivideBox")
            .field("left", &self.left.to_string_box().value)
            .field("right", &self.right.to_string_box().value)
            .field("id", &self.base.id)
            .finish()
    }
}

impl NyashBox for DivideBox {
    fn to_string_box(&self) -> StringBox {
        let result = self.execute();
        result.to_string_box()
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_div) = other.as_any().downcast_ref::<DivideBox>() {
            BoolBox::new(
                self.left.equals(other_div.left.as_ref()).value && 
                self.right.equals(other_div.right.as_ref()).value
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str { "DivideBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(DivideBox::new(self.left.clone_box(), self.right.clone_box()))
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for DivideBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Display for DivideBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Modulo operations between boxes
pub struct ModuloBox {
    pub left: Box<dyn NyashBox>,
    pub right: Box<dyn NyashBox>,
    base: BoxBase,
}

impl ModuloBox {
    pub fn new(left: Box<dyn NyashBox>, right: Box<dyn NyashBox>) -> Self {
        Self { 
            left, 
            right, 
            base: BoxBase::new(),
        }
    }
    
    /// Execute the modulo operation and return the result
    pub fn execute(&self) -> Box<dyn NyashBox> {
        // Handle integer modulo operation
        if let (Some(left_int), Some(right_int)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>()
        ) {
            if right_int.value == 0 {
                // Return error for modulo by zero
                return Box::new(StringBox::new("Error: Modulo by zero".to_string()));
            }
            let result = left_int.value % right_int.value;
            Box::new(IntegerBox::new(result))
        } else {
            // Convert to integers and compute modulo
            let left_val = if let Some(int_box) = self.left.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                0
            };
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>() {
                int_box.value
            } else {
                1 // Avoid modulo by zero
            };
            if right_val == 0 {
                return Box::new(StringBox::new("Error: Modulo by zero".to_string()));
            }
            let result = left_val % right_val;
            Box::new(IntegerBox::new(result))
        }
    }
}

impl Debug for ModuloBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuloBox")
            .field("left", &self.left.type_name())
            .field("right", &self.right.type_name())
            .finish()
    }
}

impl BoxCore for ModuloBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ModuloBox[{}]", self.box_id())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for ModuloBox {
    fn to_string_box(&self) -> StringBox {
        let result = self.execute();
        result.to_string_box()
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_modulo) = other.as_any().downcast_ref::<ModuloBox>() {
            BoolBox::new(
                self.left.equals(other_modulo.left.as_ref()).value && 
                self.right.equals(other_modulo.right.as_ref()).value
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "ModuloBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(ModuloBox::new(self.left.clone_box(), self.right.clone_box()))
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for ModuloBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

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
            right.as_any().downcast_ref::<IntegerBox>()
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
            right.as_any().downcast_ref::<IntegerBox>()
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
            right.as_any().downcast_ref::<IntegerBox>()
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
            right.as_any().downcast_ref::<IntegerBox>()
        ) {
            return BoolBox::new(left_int.value >= right_int.value);
        }
        
        // Fall back to string comparison
        let left_str = left.to_string_box();
        let right_str = right.to_string_box();
        BoolBox::new(left_str.value >= right_str.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_modulo_chip8_pattern() {
        // Test Chip-8 style bit operations using modulo
        let left = Box::new(IntegerBox::new(4096)) as Box<dyn NyashBox>;  // 0x1000
        let right = Box::new(IntegerBox::new(4096)) as Box<dyn NyashBox>; // 0x1000
        let mod_box = ModuloBox::new(left, right);
        let result = mod_box.execute();
        
        assert_eq!(result.to_string_box().value, "0");  // 4096 % 4096 = 0
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