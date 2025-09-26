use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

/// Cancellation token as a Box for structured concurrency
#[derive(Debug, Clone)]
pub struct TokenBox {
    base: BoxBase,
    token: crate::runtime::scheduler::CancellationToken,
}

impl TokenBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            token: crate::runtime::scheduler::CancellationToken::new(),
        }
    }
    pub fn from_token(token: crate::runtime::scheduler::CancellationToken) -> Self {
        Self {
            base: BoxBase::new(),
            token,
        }
    }
    pub fn cancel(&self) {
        self.token.cancel();
    }
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
    pub fn inner(&self) -> crate::runtime::scheduler::CancellationToken {
        self.token.clone()
    }
}

impl BoxCore for TokenBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CancellationToken(cancelled={})",
            self.token.is_cancelled()
        )
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for TokenBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "CancellationToken(cancelled={})",
            self.token.is_cancelled()
        ))
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(o) = other.as_any().downcast_ref::<TokenBox>() {
            BoolBox::new(self.is_cancelled() == o.is_cancelled())
        } else {
            BoolBox::new(false)
        }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
}
