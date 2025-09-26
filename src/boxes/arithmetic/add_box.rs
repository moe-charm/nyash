//! AddBox - Addition and string concatenation operations
//!
//! Implements addition between numeric types and string concatenation for all types.

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

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
            self.right.as_any().downcast_ref::<IntegerBox>(),
        ) {
            let result = left_int.value + right_int.value;
            return Box::new(IntegerBox::new(result));
        }

        // 2. Float + Float (or mixed with Integer)
        if let (Some(left_float), Some(right_float)) = (
            self.left.as_any().downcast_ref::<FloatBox>(),
            self.right.as_any().downcast_ref::<FloatBox>(),
        ) {
            let result = left_float.value + right_float.value;
            return Box::new(FloatBox::new(result));
        }

        // 3. Integer + Float
        if let (Some(left_int), Some(right_float)) = (
            self.left.as_any().downcast_ref::<IntegerBox>(),
            self.right.as_any().downcast_ref::<FloatBox>(),
        ) {
            let result = left_int.value as f64 + right_float.value;
            return Box::new(FloatBox::new(result));
        }

        // 4. Float + Integer
        if let (Some(left_float), Some(right_int)) = (
            self.left.as_any().downcast_ref::<FloatBox>(),
            self.right.as_any().downcast_ref::<IntegerBox>(),
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
                self.left.equals(other_add.left.as_ref()).value
                    && self.right.equals(other_add.right.as_ref()).value,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "AddBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(AddBox::new(self.left.clone_box(), self.right.clone_box()))
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