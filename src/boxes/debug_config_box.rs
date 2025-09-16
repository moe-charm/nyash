use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct DebugConfigBox {
    pub base: BoxBase,
    // toggles/paths (staged until apply())
    pub jit_events: bool,
    pub jit_events_compile: bool,
    pub jit_events_runtime: bool,
    pub jit_stats: bool,
    pub jit_stats_json: bool,
    pub jit_dump: bool,
    pub jit_dot_path: Option<String>,
    pub jit_events_path: Option<String>,
}

impl DebugConfigBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            jit_events: std::env::var("NYASH_JIT_EVENTS").ok().as_deref() == Some("1"),
            jit_events_compile: std::env::var("NYASH_JIT_EVENTS_COMPILE").ok().as_deref()
                == Some("1"),
            jit_events_runtime: std::env::var("NYASH_JIT_EVENTS_RUNTIME").ok().as_deref()
                == Some("1"),
            jit_stats: std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1"),
            jit_stats_json: std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1"),
            jit_dump: std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1"),
            jit_dot_path: std::env::var("NYASH_JIT_DOT")
                .ok()
                .filter(|s| !s.is_empty()),
            jit_events_path: std::env::var("NYASH_JIT_EVENTS_PATH")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }

    pub fn set_flag(&mut self, name: &str, on: bool) -> Box<dyn NyashBox> {
        match name {
            "jit_events" => self.jit_events = on,
            "jit_events_compile" => self.jit_events_compile = on,
            "jit_events_runtime" => self.jit_events_runtime = on,
            "jit_stats" => self.jit_stats = on,
            "jit_stats_json" => self.jit_stats_json = on,
            "jit_dump" => self.jit_dump = on,
            _ => return Box::new(StringBox::new(format!("Unknown flag: {}", name))),
        }
        Box::new(VoidBox::new())
    }

    pub fn set_path(&mut self, name: &str, value: &str) -> Box<dyn NyashBox> {
        match name {
            "jit_dot" | "jit_dot_path" => self.jit_dot_path = Some(value.to_string()),
            "jit_events_path" => self.jit_events_path = Some(value.to_string()),
            _ => return Box::new(StringBox::new(format!("Unknown path: {}", name))),
        }
        Box::new(VoidBox::new())
    }

    pub fn get_flag(&self, name: &str) -> Box<dyn NyashBox> {
        let v = match name {
            "jit_events" => self.jit_events,
            "jit_events_compile" => self.jit_events_compile,
            "jit_events_runtime" => self.jit_events_runtime,
            "jit_stats" => self.jit_stats,
            "jit_stats_json" => self.jit_stats_json,
            "jit_dump" => self.jit_dump,
            _ => false,
        };
        Box::new(BoolBox::new(v))
    }

    pub fn get_path(&self, name: &str) -> Box<dyn NyashBox> {
        let v = match name {
            "jit_dot" | "jit_dot_path" => self.jit_dot_path.clone().unwrap_or_default(),
            "jit_events_path" => self.jit_events_path.clone().unwrap_or_default(),
            _ => String::new(),
        };
        Box::new(StringBox::new(v))
    }

    pub fn apply(&self) -> Box<dyn NyashBox> {
        let setb = |k: &str, v: bool| {
            if v {
                std::env::set_var(k, "1");
            } else {
                std::env::remove_var(k);
            }
        };
        setb("NYASH_JIT_EVENTS", self.jit_events);
        setb("NYASH_JIT_EVENTS_COMPILE", self.jit_events_compile);
        setb("NYASH_JIT_EVENTS_RUNTIME", self.jit_events_runtime);
        setb("NYASH_JIT_STATS", self.jit_stats);
        setb("NYASH_JIT_STATS_JSON", self.jit_stats_json);
        setb("NYASH_JIT_DUMP", self.jit_dump);
        if let Some(p) = &self.jit_dot_path {
            std::env::set_var("NYASH_JIT_DOT", p);
        } else {
            std::env::remove_var("NYASH_JIT_DOT");
        }
        if let Some(p) = &self.jit_events_path {
            std::env::set_var("NYASH_JIT_EVENTS_PATH", p);
        } else {
            std::env::remove_var("NYASH_JIT_EVENTS_PATH");
        }
        // If any events are enabled and threshold is not set, default to 1 so lowering runs early
        if (self.jit_events || self.jit_events_compile || self.jit_events_runtime)
            && std::env::var("NYASH_JIT_THRESHOLD").is_err()
        {
            std::env::set_var("NYASH_JIT_THRESHOLD", "1");
        }
        Box::new(VoidBox::new())
    }

    pub fn summary(&self) -> Box<dyn NyashBox> {
        let s = format!(
            "jit_events={} jit_events_compile={} jit_events_runtime={} jit_stats={} jit_stats_json={} jit_dump={} jit_dot={} jit_events_path={}",
            self.jit_events, self.jit_events_compile, self.jit_events_runtime,
            self.jit_stats, self.jit_stats_json, self.jit_dump,
            self.jit_dot_path.clone().unwrap_or_else(|| "<none>".to_string()),
            self.jit_events_path.clone().unwrap_or_else(|| "<none>".to_string())
        );
        Box::new(StringBox::new(s))
    }
}

impl BoxCore for DebugConfigBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugConfigBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for DebugConfigBox {
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<DebugConfigBox>())
    }
    fn type_name(&self) -> &'static str {
        "DebugConfigBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(Self {
            base: self.base.clone(),
            ..self.clone()
        })
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    fn to_string_box(&self) -> StringBox {
        StringBox::new("DebugConfigBox".to_string())
    }
}
