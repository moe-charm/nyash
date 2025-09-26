//! SubtractBox - Subtraction operations
//!
//! Implements subtraction between numeric types with integer fallback.

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

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
            self.right.as_any().downcast_ref::<IntegerBox>(),
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
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>()
            {
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
                self.left.equals(other_sub.left.as_ref()).value
                    && self.right.equals(other_sub.right.as_ref()).value,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "SubtractBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(SubtractBox::new(
            self.left.clone_box(),
            self.right.clone_box(),
        ))
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for SubtractBox {
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

impl Display for SubtractBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}