//! TypeMeta and Thunk table for unified Box dispatch
//!
//! Phase 9.79b.3 scaffolding:
//! - Provides per-type metadata with a vector of method thunks (by slot index)
//! - Each thunk currently holds a target function name (MIR function),
//!   which VM can call via `call_function_by_name`.
//! - Versioning is sourced from `cache_versions` using label `BoxRef:{class}`.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// Target of a method thunk
#[derive(Clone, Debug)]
pub enum ThunkTarget {
    /// Call a lowered MIR function by name
    MirFunction(String),
    /// Call plugin invoke function using receiver's plugin handle
    PluginInvoke { method_id: u16 },
    /// Call builtin NyashBox method by name (name-based dispatch)
    BuiltinCall { method: String },
}

/// A single method thunk entry (scaffolding)
#[derive(Default)]
pub struct MethodThunk {
    /// Target info
    target: RwLock<Option<ThunkTarget>>,
    /// Flags placeholder (e.g., universal, builtin, plugin etc.)
    _flags: RwLock<u32>,
}

impl MethodThunk {
    pub fn new() -> Self { Self { target: RwLock::new(None), _flags: RwLock::new(0) } }
    pub fn get_target(&self) -> Option<ThunkTarget> { self.target.read().ok().and_then(|g| g.clone()) }
    pub fn set_mir_target(&self, name: String) { if let Ok(mut g) = self.target.write() { *g = Some(ThunkTarget::MirFunction(name)); } }
    pub fn set_plugin_invoke(&self, method_id: u16) { if let Ok(mut g) = self.target.write() { *g = Some(ThunkTarget::PluginInvoke { method_id }); } }
}

/// Per-type metadata including thunk table
pub struct TypeMeta {
    class_name: String,
    /// Thunk table indexed by method slot (method_id)
    thunks: RwLock<Vec<Arc<MethodThunk>>>,
}

impl TypeMeta {
    fn new(class_name: String) -> Self {
        Self { class_name, thunks: RwLock::new(Vec::new()) }
    }

    /// Ensure that the thunk table length is at least `len`.
    pub fn ensure_len(&self, len: usize) {
        if let Ok(mut tbl) = self.thunks.write() {
            if tbl.len() < len {
                let to_add = len - tbl.len();
                for _ in 0..to_add { tbl.push(Arc::new(MethodThunk::new())); }
            }
        }
    }

    /// Get thunk for slot, if present
    pub fn get_thunk(&self, slot: usize) -> Option<Arc<MethodThunk>> {
        let tbl = self.thunks.read().ok()?;
        tbl.get(slot).cloned()
    }

    /// Set thunk target name for slot
    pub fn set_thunk_mir_target(&self, slot: usize, target_name: String) {
        self.ensure_len(slot + 1);
        if let Some(th) = self.get_thunk(slot) { th.set_mir_target(target_name); }
    }

    pub fn set_thunk_plugin_invoke(&self, slot: usize, method_id: u16) {
        self.ensure_len(slot + 1);
        if let Some(th) = self.get_thunk(slot) { th.set_plugin_invoke(method_id); }
    }

    pub fn set_thunk_builtin(&self, slot: usize, method: String) {
        self.ensure_len(slot + 1);
        if let Some(th) = self.get_thunk(slot) {
            if let Ok(mut g) = th.target.write() {
                *g = Some(ThunkTarget::BuiltinCall { method });
            }
        }
    }

    /// Current version from global cache (for diagnostics / PIC keys)
    pub fn current_version(&self) -> u32 {
        crate::runtime::cache_versions::get_version(&format!("BoxRef:{}", self.class_name))
    }
}

static TYPE_META_REGISTRY: Lazy<Mutex<HashMap<String, Arc<TypeMeta>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create TypeMeta for a given class name
pub fn get_or_create_type_meta(class_name: &str) -> Arc<TypeMeta> {
    let mut map = TYPE_META_REGISTRY.lock().unwrap();
    if let Some(m) = map.get(class_name) { return m.clone(); }
    let m = Arc::new(TypeMeta::new(class_name.to_string()));
    map.insert(class_name.to_string(), m.clone());
    m
}

/// Dump registry contents for diagnostics
pub fn dump_registry() {
    let map = TYPE_META_REGISTRY.lock().unwrap();
    eprintln!("[REG] TypeMeta registry dump ({} types)", map.len());
    for (name, meta) in map.iter() {
        let tbl = meta.thunks.read().ok();
        let len = tbl.as_ref().map(|t| t.len()).unwrap_or(0);
        eprintln!("  - {}: thunks={} v{}", name, len, meta.current_version());
        if let Some(t) = tbl {
            for (i, th) in t.iter().enumerate() {
                if let Some(target) = th.get_target() {
                    eprintln!("      slot {} -> {:?}", i, target);
                }
            }
        }
    }
}
