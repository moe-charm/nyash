//! Runtime-facing CallableBox abstraction (host-owned).
//!
//! When legacy builtins are enabled we reuse the historical implementation.
//! For plugin-only builds we provide a thin shim with the same surface so
//! router code can stay feature-agnostic.

#[cfg(feature = "legacy-boxes")]
pub use crate::boxes::callable::CallableBox;

#[cfg(not(feature = "legacy-boxes"))]
mod shim {
    use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
    use std::any::Any;

    #[derive(Debug)]
    pub struct CallableBox {
        base: BoxBase,
        pub(crate) receiver: Option<Box<dyn NyashBox>>, // optional bound receiver
        pub(crate) method: String,
        pub(crate) arity: usize,
    }

    impl CallableBox {
        pub fn new(receiver: Option<Box<dyn NyashBox>>, method: String, arity: usize) -> Self {
            Self { base: BoxBase::new(), receiver, method, arity }
        }
        #[inline]
        pub fn arity(&self) -> usize { self.arity }
        #[inline]
        pub fn share_clone(&self) -> Box<dyn NyashBox> {
            Box::new(Self {
                base: BoxBase::new(),
                receiver: self.receiver.as_ref().map(|r| r.share_box()),
                method: self.method.clone(),
                arity: self.arity,
            })
        }
    }

    impl Clone for CallableBox {
        fn clone(&self) -> Self {
            Self {
                base: BoxBase::new(),
                receiver: self.receiver.as_ref().map(|r| r.share_box()),
                method: self.method.clone(),
                arity: self.arity,
            }
        }
    }

    impl BoxCore for CallableBox {
        fn box_id(&self) -> u64 { self.base.id }
        fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
        fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Callable(method={}, arity={})", self.method, self.arity)
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    impl NyashBox for CallableBox {
        fn is_identity(&self) -> bool { true }
        fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
        fn share_box(&self) -> Box<dyn NyashBox> { self.share_clone() }
        fn to_string_box(&self) -> StringBox {
            StringBox::new(format!("Callable(method={}, arity={})", self.method, self.arity))
        }
        fn type_name(&self) -> &'static str { "CallableBox" }
        fn equals(&self, other: &dyn NyashBox) -> BoolBox {
            if let Some(o) = other.as_any().downcast_ref::<CallableBox>() {
                BoolBox::new(self.box_id() == o.box_id() && self.method == o.method && self.arity == o.arity)
            } else { BoolBox::new(false) }
        }
    }
}

#[cfg(not(feature = "legacy-boxes"))]
pub use shim::CallableBox;

