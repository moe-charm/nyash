use crate::box_trait::{NyashBox, StringBox, BoolBox, VoidBox, BoxCore, BoxBase};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitPolicyBox { base: BoxBase }

impl JitPolicyBox { pub fn new() -> Self { Self { base: BoxBase::new() } } }

impl BoxCore for JitPolicyBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "JitPolicyBox") }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl NyashBox for JitPolicyBox {
    fn to_string_box(&self) -> StringBox {
        let p = crate::jit::policy::current();
        let s = format!("read_only={} whitelist={}", p.read_only, p.hostcall_whitelist.join(","));
        StringBox::new(s)
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { BoolBox::new(other.as_any().is::<JitPolicyBox>()) }
    fn type_name(&self) -> &'static str { "JitPolicyBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(Self { base: self.base.clone() }) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

// Methods (exposed via VM dispatch):
impl JitPolicyBox {
    pub fn set_flag(&self, name: &str, on: bool) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        match name {
            "read_only" | "readonly" => cur.read_only = on,
            _ => return Box::new(StringBox::new(format!("Unknown flag: {}", name)))
        }
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }
    pub fn get_flag(&self, name: &str) -> Box<dyn NyashBox> {
        let cur = crate::jit::policy::current();
        let v = match name { "read_only" | "readonly" => cur.read_only, _ => false };
        Box::new(BoolBox::new(v))
    }
    pub fn set_whitelist_csv(&self, csv: &str) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        cur.hostcall_whitelist = csv.split(',').map(|t| t.trim().to_string()).filter(|s| !s.is_empty()).collect();
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }
}

