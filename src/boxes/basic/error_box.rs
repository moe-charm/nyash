//! ErrorBox - Error handling in Nyash
//!
//! Implements the ErrorBox type for representing error information.

use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};
use std::any::Any;
use std::fmt::{Debug, Display};

/// Error values in Nyash - represents error information
#[derive(Debug, Clone)]
pub struct ErrorBox {
    pub error_type: String,
    pub message: String,
    base: BoxBase,
}

impl ErrorBox {
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for ErrorBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for ErrorBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("{}: {}", self.error_type, self.message))
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_error) = other.as_any().downcast_ref::<ErrorBox>() {
            BoolBox::new(
                self.error_type == other_error.error_type && self.message == other_error.message,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "ErrorBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for ErrorBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}