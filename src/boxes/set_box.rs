#![cfg(feature = "legacy-boxes")]
use std::fmt::{Debug, Display};
use std::sync::Arc;

use crate::box_trait::{BoolBox, BoxCore, NyashBox, StringBox};

use super::map_box::MapBox;

/// SetBox — Map<Key, Unit> ベースの集合
/// 内部は MapBox を共有参照（Arc）で保持し、API は Null/Bool/Integer/Array に正規化する。
pub struct SetBox {
    inner: Arc<MapBox>,
    base: crate::box_trait::BoxBase,
}

impl SetBox {
    pub fn new() -> Self {
        let map = MapBox::new();
        Self { inner: Arc::new(map), base: crate::box_trait::BoxBase::new() }
    }

    pub fn add(&self, v: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // 値は Unit（観測不可）として Null を格納
        let _ = self.inner.set(v, Box::new(crate::boxes::null_box::NullBox::new()));
        Box::new(crate::boxes::null_box::NullBox::new())
    }

    pub fn remove(&self, v: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _ = self.inner.delete(v);
        Box::new(crate::boxes::null_box::NullBox::new())
    }

    pub fn has(&self, v: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        self.inner.has(v)
    }

    pub fn size(&self) -> Box<dyn NyashBox> {
        self.inner.size()
    }

    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.inner.clear()
    }

    pub fn toArray(&self) -> Box<dyn NyashBox> {
        self.inner.keys()
    }
}

impl BoxCore for SetBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let n = self.inner.get_data().read().unwrap().len();
        write!(f, "SetBox(size={})", n)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for SetBox {
    fn is_identity(&self) -> bool { true }
    fn type_name(&self) -> &'static str { "SetBox" }
    fn to_string_box(&self) -> StringBox {
        let n = self.inner.get_data().read().unwrap().len();
        StringBox::new(&format!("SetBox(size={})", n))
    }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> {
        let s = SetBox { inner: Arc::clone(&self.inner), base: crate::box_trait::BoxBase::new() };
        Box::new(s)
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(o) = other.as_any().downcast_ref::<SetBox>() {
            BoolBox::new(std::sync::Arc::ptr_eq(&self.inner, &o.inner))
        } else { BoolBox::new(false) }
    }
}

impl Clone for SetBox {
    fn clone(&self) -> Self {
        SetBox { inner: Arc::clone(&self.inner), base: crate::box_trait::BoxBase::new() }
    }
}

impl Display for SetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) }
}

impl Debug for SetBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.inner.get_data().read().unwrap().len();
        f.debug_struct("SetBox").field("id", &self.base.id).field("size", &n).finish()
    }
}
