//! VoidBox - Void/null values in Nyash
//!
//! Implements the core VoidBox type representing empty or null results.

use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};
use std::any::Any;
use std::fmt::{Debug, Display};

/// Void/null values in Nyash - represents empty or null results
#[derive(Debug, Clone, PartialEq)]
pub struct VoidBox {
    base: BoxBase,
}

impl VoidBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl Default for VoidBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxCore for VoidBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "void")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for VoidBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("void")
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<VoidBox>())
    }

    fn type_name(&self) -> &'static str {
        "VoidBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for VoidBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}