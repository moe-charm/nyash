//! Basic Box shim for plugin-only builds (legacy-boxes OFF)
//! Provides minimal implementations of BoolBox, IntegerBox, StringBox, VoidBox
//! to satisfy trait signatures and common call sites during migration.

use crate::box_core::{BoxBase, BoxCore, NyashBox};

#[derive(Debug, Clone)]
pub struct StringBox {
    pub base: BoxBase,
    pub value: String,
}
impl StringBox {
    pub fn new<S: Into<String>>(s: S) -> Self { Self { base: BoxBase::new(), value: s.into() } }
}
impl BoxCore for StringBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.value) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl NyashBox for StringBox {
    fn to_string_box(&self) -> StringBox { self.clone() }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(s) = other.as_any().downcast_ref::<StringBox>() {
            BoolBox::new(self.value == s.value)
        } else {
            BoolBox::new(false)
        }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

#[derive(Debug, Clone)]
pub struct IntegerBox {
    pub base: BoxBase,
    pub value: i64,
}
impl IntegerBox {
    pub fn new(v: i64) -> Self { Self { base: BoxBase::new(), value: v } }
}
impl BoxCore for IntegerBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.value) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl NyashBox for IntegerBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(self.value.to_string()) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(x) = other.as_any().downcast_ref::<IntegerBox>() { BoolBox::new(self.value == x.value) }
        else { BoolBox::new(false) }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

#[derive(Debug, Clone)]
pub struct BoolBox {
    pub base: BoxBase,
    pub value: bool,
}
impl BoolBox { pub fn new(v: bool) -> Self { Self { base: BoxBase::new(), value: v } } }
impl BoxCore for BoolBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.value) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl NyashBox for BoolBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(self.value.to_string()) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(x) = other.as_any().downcast_ref::<BoolBox>() { BoolBox::new(self.value == x.value) }
        else { BoolBox::new(false) }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

#[derive(Debug, Clone)]
pub struct VoidBox { pub base: BoxBase }
impl VoidBox { pub fn new() -> Self { Self { base: BoxBase::new() } } }
impl BoxCore for VoidBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "void") }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl NyashBox for VoidBox {
    fn to_string_box(&self) -> StringBox { StringBox::new("void") }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        // Void equals only Void by default
        let is_void = other.as_any().downcast_ref::<VoidBox>().is_some();
        BoolBox::new(is_void)
    }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

