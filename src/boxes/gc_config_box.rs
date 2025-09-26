use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct GcConfigBox {
    base: BoxBase,
    counting: bool,
    trace: bool,
    barrier_strict: bool,
}

impl GcConfigBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            counting: std::env::var("NYASH_GC_COUNTING").ok().as_deref() == Some("1"),
            trace: std::env::var("NYASH_GC_TRACE").ok().as_deref() == Some("1"),
            barrier_strict: std::env::var("NYASH_GC_BARRIER_STRICT").ok().as_deref() == Some("1"),
        }
    }
    pub fn set_flag(&mut self, name: &str, on: bool) -> Box<dyn NyashBox> {
        match name {
            "counting" => self.counting = on,
            "trace" => self.trace = on,
            "barrier_strict" | "strict" => self.barrier_strict = on,
            _ => return Box::new(StringBox::new(format!("Unknown flag: {}", name))),
        }
        Box::new(VoidBox::new())
    }
    pub fn get_flag(&self, name: &str) -> Box<dyn NyashBox> {
        let v = match name {
            "counting" => self.counting,
            "trace" => self.trace,
            "barrier_strict" | "strict" => self.barrier_strict,
            _ => false,
        };
        Box::new(BoolBox::new(v))
    }
    pub fn apply(&self) -> Box<dyn NyashBox> {
        let setb = |k: &str, v: bool| {
            if v {
                std::env::set_var(k, "1");
            } else {
                std::env::remove_var(k);
            }
        };
        setb("NYASH_GC_COUNTING", self.counting);
        setb("NYASH_GC_TRACE", self.trace);
        setb("NYASH_GC_BARRIER_STRICT", self.barrier_strict);
        Box::new(VoidBox::new())
    }
    pub fn summary(&self) -> Box<dyn NyashBox> {
        let s = format!(
            "counting={} trace={} barrier_strict={}",
            self.counting, self.trace, self.barrier_strict
        );
        Box::new(StringBox::new(s))
    }
}

impl BoxCore for GcConfigBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GcConfigBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for GcConfigBox {
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<GcConfigBox>())
    }
    fn type_name(&self) -> &'static str {
        "GcConfigBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.summary().to_string_box().value)
    }
}
