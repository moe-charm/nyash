use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitStatsBox {
    base: BoxBase,
}

impl JitStatsBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
    pub fn to_json(&self) -> Box<dyn NyashBox> {
        let cfg = crate::jit::config::current();
        let caps = crate::jit::config::probe_capabilities();
        let mode = if cfg.native_bool_abi && caps.supports_b1_sig {
            "b1_bool"
        } else {
            "i64_bool"
        };
        let payload = serde_json::json!({
            "version": 1,
            "abi_mode": mode,
            "abi_b1_enabled": cfg.native_bool_abi,
            "abi_b1_supported": caps.supports_b1_sig,
            "b1_norm_count": crate::jit::rt::b1_norm_get(),
            "ret_bool_hint_count": crate::jit::rt::ret_bool_hint_get(),
            "phi_total_slots": crate::jit::rt::phi_total_get(),
            "phi_b1_slots": crate::jit::rt::phi_b1_get(),
        });
        Box::new(StringBox::new(payload.to_string()))
    }
}

impl BoxCore for JitStatsBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitStatsBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitStatsBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.to_json().to_string_box().value)
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitStatsBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitStatsBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
