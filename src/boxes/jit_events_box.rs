use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitEventsBox {
    base: BoxBase,
}

impl JitEventsBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for JitEventsBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitEventsBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitEventsBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("JitEventsBox")
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitEventsBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitEventsBox"
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

impl JitEventsBox {
    pub fn set_path(&self, path: &str) -> Box<dyn NyashBox> {
        std::env::set_var("NYASH_JIT_EVENTS_PATH", path);
        Box::new(VoidBox::new())
    }
    pub fn enable(&self, on: bool) -> Box<dyn NyashBox> {
        if on {
            std::env::set_var("NYASH_JIT_EVENTS", "1");
        } else {
            std::env::remove_var("NYASH_JIT_EVENTS");
        }
        Box::new(VoidBox::new())
    }
    pub fn log(&self, kind: &str, function: &str, note_json: &str) -> Box<dyn NyashBox> {
        let extra = serde_json::from_str::<serde_json::Value>(note_json)
            .unwrap_or_else(|_| serde_json::json!({"note": note_json}));
        crate::jit::events::emit(kind, function, None, None, extra);
        Box::new(VoidBox::new())
    }
}
