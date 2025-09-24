//! DivideBox - Division operations with zero-division error handling
//!
//! Implements division between numeric types, returning float results and error strings for zero division.

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

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
            self.right.as_any().downcast_ref::<IntegerBox>(),
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
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>()
            {
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
                self.left.equals(other_div.left.as_ref()).value
                    && self.right.equals(other_div.right.as_ref()).value,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "DivideBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(DivideBox::new(
            self.left.clone_box(),
            self.right.clone_box(),
        ))
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for DivideBox {
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

impl Display for DivideBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}