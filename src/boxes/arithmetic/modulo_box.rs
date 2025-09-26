//! ModuloBox - Modulo operations with zero-modulo error handling
//!
//! Implements modulo operations between integer types with error handling.

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

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
            self.right.as_any().downcast_ref::<IntegerBox>(),
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
            let right_val = if let Some(int_box) = self.right.as_any().downcast_ref::<IntegerBox>()
            {
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
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
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
                self.left.equals(other_modulo.left.as_ref()).value
                    && self.right.equals(other_modulo.right.as_ref()).value,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "ModuloBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(ModuloBox::new(
            self.left.clone_box(),
            self.right.clone_box(),
        ))
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