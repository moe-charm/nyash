use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitHostcallRegistryBox {
    base: BoxBase,
}

impl JitHostcallRegistryBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for JitHostcallRegistryBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitHostcallRegistryBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitHostcallRegistryBox {
    fn to_string_box(&self) -> StringBox {
        let (ro, mu) = crate::jit::hostcall_registry::snapshot();
        let payload = serde_json::json!({ "readonly": ro, "mutating": mu });
        StringBox::new(payload.to_string())
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitHostcallRegistryBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitHostcallRegistryBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(Self {
            base: self.base.clone(),
        })
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl JitHostcallRegistryBox {
    pub fn add_readonly(&self, sym: &str) -> Box<dyn NyashBox> {
        crate::jit::hostcall_registry::add_readonly(sym);
        Box::new(VoidBox::new())
    }
    pub fn add_mutating(&self, sym: &str) -> Box<dyn NyashBox> {
        crate::jit::hostcall_registry::add_mutating(sym);
        Box::new(VoidBox::new())
    }
    pub fn set_from_csv(&self, ro_csv: &str, mu_csv: &str) -> Box<dyn NyashBox> {
        crate::jit::hostcall_registry::set_from_csv(ro_csv, mu_csv);
        Box::new(VoidBox::new())
    }
    pub fn add_signature(&self, sym: &str, args_csv: &str, ret_str: &str) -> Box<dyn NyashBox> {
        let ok = crate::jit::hostcall_registry::set_signature_csv(sym, args_csv, ret_str);
        if ok {
            Box::new(VoidBox::new())
        } else {
            Box::new(StringBox::new("Invalid signature"))
        }
    }
}
