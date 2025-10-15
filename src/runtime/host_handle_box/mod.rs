use crate::box_trait::{BoxCore, NyashBox, StringBox};
use std::any::Any;

#[derive(Clone, Debug)]
pub struct HostHandleBox {
    pub id: u64,
}

impl HostHandleBox {
    pub fn new(id: u64) -> Self { Self { id } }
}

impl NyashBox for HostHandleBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(format!("<HostHandle:{}>", self.id)) }
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        if let Some(h) = other.as_any().downcast_ref::<HostHandleBox>() {
            crate::box_trait::BoolBox::new(self.id == h.id)
        } else { crate::box_trait::BoolBox::new(false) }
    }
    fn type_name(&self) -> &'static str { "HostHandle" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

impl BoxCore for HostHandleBox {
    fn box_id(&self) -> u64 { 0 }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "<HostHandle:{}>", self.id) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
