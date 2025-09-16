use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct RefCellBox {
    inner: Arc<Mutex<Box<dyn NyashBox>>>,
    base: BoxBase,
}

impl RefCellBox {
    pub fn new(initial: Box<dyn NyashBox>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            base: BoxBase::new(),
        }
    }
    pub fn with_inner(inner: Arc<Mutex<Box<dyn NyashBox>>>) -> Self {
        Self {
            inner,
            base: BoxBase::new(),
        }
    }
    pub fn borrow(&self) -> Box<dyn NyashBox> {
        self.inner.lock().unwrap().clone_box()
    }
    pub fn replace(&self, value: Box<dyn NyashBox>) {
        let mut guard = self.inner.lock().unwrap();
        *guard = value;
    }
    pub fn inner_arc(&self) -> Arc<Mutex<Box<dyn NyashBox>>> {
        Arc::clone(&self.inner)
    }
}

impl BoxCore for RefCellBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RefCellBox(..)")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for RefCellBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(Self::with_inner(self.inner_arc()))
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    fn to_string_box(&self) -> StringBox {
        let inner = self.inner.lock().unwrap();
        StringBox::new(format!("RefCell({})", inner.to_string_box().value))
    }
    fn type_name(&self) -> &'static str {
        "RefCellBox"
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(o) = other.as_any().downcast_ref::<RefCellBox>() {
            BoolBox::new(Arc::ptr_eq(&self.inner, &o.inner))
        } else {
            BoolBox::new(false)
        }
    }
}
