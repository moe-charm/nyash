use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitStrictBox {
    base: BoxBase,
}

impl JitStrictBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for JitStrictBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitStrictBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitStrictBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("JitStrictBox".to_string())
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitStrictBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitStrictBox"
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

impl JitStrictBox {
    /// Enable/disable strict mode. When enabling, also set JIT-only and args-handle-only by default.
    pub fn enable(&self, on: bool) -> Box<dyn NyashBox> {
        if on {
            std::env::set_var("NYASH_JIT_STRICT", "1");
            if std::env::var("NYASH_JIT_ONLY").ok().is_none() {
                std::env::set_var("NYASH_JIT_ONLY", "1");
            }
            if std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().is_none() {
                std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
            }
        } else {
            std::env::remove_var("NYASH_JIT_STRICT");
        }
        Box::new(VoidBox::new())
    }

    pub fn set_only(&self, on: bool) -> Box<dyn NyashBox> {
        if on {
            std::env::set_var("NYASH_JIT_ONLY", "1");
        } else {
            std::env::remove_var("NYASH_JIT_ONLY");
        }
        Box::new(VoidBox::new())
    }
    pub fn set_handle_only(&self, on: bool) -> Box<dyn NyashBox> {
        if on {
            std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
        } else {
            std::env::remove_var("NYASH_JIT_ARGS_HANDLE_ONLY");
        }
        Box::new(VoidBox::new())
    }

    pub fn status(&self) -> Box<dyn NyashBox> {
        let s = serde_json::json!({
            "strict": std::env::var("NYASH_JIT_STRICT").ok().as_deref() == Some("1"),
            "jit_only": std::env::var("NYASH_JIT_ONLY").ok().as_deref() == Some("1"),
            "args_handle_only": std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() == Some("1"),
            "lower_fallbacks": crate::jit::events::lower_fallbacks_get(),
        });
        Box::new(StringBox::new(s.to_string()))
    }

    /// Reset compile-time counters (e.g., lower fallback count) before next compile.
    pub fn reset_counters(&self) -> Box<dyn NyashBox> {
        crate::jit::events::lower_counters_reset();
        Box::new(VoidBox::new())
    }
}
