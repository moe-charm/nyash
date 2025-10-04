use std::collections::HashMap;
use std::sync::OnceLock;

use crate::mir::{EffectMask, MirType};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ExternCallSpec {
    pub interface: String,   // e.g., "nyrt.time"
    pub method: String,      // e.g., "now_ms"
    pub args: Vec<MirType>,  // argument types (best-effort)
    pub returns: MirType,    // return type (best-effort)
    pub effects: EffectMask, // resolver用（READ/IO/CONTROLなど）
}

#[derive(Default)]
pub struct ExternCallRegistryBox {
    entries: HashMap<(String, String), ExternCallSpec>,
}

impl ExternCallRegistryBox {
    pub fn new() -> Self {
        let mut reg = Self { entries: HashMap::new() };
        reg.register_builtins();
        reg
    }

    fn register_builtins(&mut self) {
        // Timer
        self.register(ExternCallSpec {
            interface: "nyrt.time".into(),
            method: "now_ms".into(),
            args: vec![],
            returns: MirType::Integer,
            effects: EffectMask::READ,
        });
        // Array.size
        self.register(ExternCallSpec {
            interface: "nyrt.array".into(),
            method: "size".into(),
            args: vec![MirType::Box("ArrayBox".into())],
            returns: MirType::Integer,
            effects: EffectMask::READ,
        });
        // Map.size
        self.register(ExternCallSpec {
            interface: "nyrt.map".into(),
            method: "size".into(),
            args: vec![MirType::Box("MapBox".into())],
            returns: MirType::Integer,
            effects: EffectMask::READ,
        });
        // 今後: env.console.log などを必要に応じて追記
    }

    pub fn register(&mut self, info: ExternCallSpec) {
        let key = (info.interface.clone(), info.method.clone());
        self.entries.insert(key, info);
    }

    pub fn get(&self, iface: &str, method: &str) -> Option<&ExternCallSpec> {
        self.entries.get(&(iface.to_string(), method.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&(String, String), &ExternCallSpec)> {
        self.entries.iter()
    }
}

static REGISTRY: OnceLock<ExternCallRegistryBox> = OnceLock::new();

pub fn registry() -> &'static ExternCallRegistryBox {
    REGISTRY.get_or_init(|| ExternCallRegistryBox::new())
}

// Helper accessors (backend向け)
pub fn effects_for(iface: &str, method: &str) -> Option<EffectMask> {
    registry().get(iface, method).map(|i| i.effects)
}

// ---- JSON export (abstract spec only) ----

#[derive(Debug, Serialize)]
struct ExternSpecJson<'a> {
    interface: &'a str,
    method: &'a str,
    params: Vec<String>,
    returns: String,
    effects: String,
}

fn mir_type_to_string(t: &MirType) -> String {
    match t {
        MirType::Integer => "Integer".into(),
        MirType::Float => "Float".into(),
        MirType::Bool => "Bool".into(),
        MirType::String => "String".into(),
        MirType::Void => "Void".into(),
        MirType::Unknown => "Unknown".into(),
        MirType::Box(b) => format!("Box:{}", b),
        _ => format!("{:?}", t),
    }
}

fn effects_to_string(e: EffectMask) -> String {
    let mut kinds: Vec<&str> = Vec::new();
    if e.is_pure() { kinds.push("pure"); }
    if e.is_read_only() && !e.is_pure() { kinds.push("read"); }
    if e.is_mut() { kinds.push("mut"); }
    if e.is_io() { kinds.push("io"); }
    if e.is_control() { kinds.push("control"); }
    if kinds.is_empty() { kinds.push("unknown"); }
    kinds.join("|")
}

/// Export current registry to a JSON file (abstract spec only)
pub fn export_json<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {
    let mut out: Vec<ExternSpecJson> = Vec::new();
    for (k, v) in registry().iter() {
        let (iface, method) = (&k.0, &k.1);
        out.push(ExternSpecJson {
            interface: iface,
            method,
            params: v.args.iter().map(mir_type_to_string).collect(),
            returns: mir_type_to_string(&v.returns),
            effects: effects_to_string(v.effects),
        });
    }
    let text = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.as_ref().parent().unwrap_or_else(|| std::path::Path::new("."))).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}
