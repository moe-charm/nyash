//! BoolBox - Boolean values in Nyash
//!
//! Implements the core BoolBox type for true/false values.

use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

/// Boolean values in Nyash - true/false
#[derive(Debug, Clone, PartialEq)]
pub struct BoolBox {
    pub value: bool,
    base: BoxBase,
}

impl BoolBox {
    pub fn new(value: bool) -> Self {
        Self {
            value,
            base: BoxBase::new(),
        }
    }

    pub fn true_box() -> Self {
        Self::new(true)
    }

    pub fn false_box() -> Self {
        Self::new(false)
    }
}

impl BoxCore for BoolBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", if self.value { "true" } else { "false" })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for BoolBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(if self.value { "true" } else { "false" })
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            BoolBox::new(self.value == other_bool.value)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "BoolBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for BoolBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}