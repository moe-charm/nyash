//! IntegerBox - Integer values in Nyash
//!
//! Implements the core IntegerBox type for 64-bit signed integers.

use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};
use std::any::Any;
use std::fmt::{Debug, Display};

/// Integer values in Nyash - 64-bit signed integers
#[derive(Debug, Clone, PartialEq)]
pub struct IntegerBox {
    pub value: i64,
    base: BoxBase,
}

impl IntegerBox {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            base: BoxBase::new(),
        }
    }

    pub fn zero() -> Self {
        Self::new(0)
    }
}

impl BoxCore for IntegerBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for IntegerBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.value.to_string())
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            BoolBox::new(self.value == other_int.value)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "IntegerBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for IntegerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}