/*!
 * Host Handle Registry (global)
 *
 * 目的:
 * - C ABI(TLV)でユーザー/内蔵Boxを渡すためのホスト管理ハンドルを提供。
 * - u64ハンドルID → Arc<dyn NyashBox> をグローバルに保持し、VM/PluginHost/JITから参照可能にする。
 */

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};

use crate::box_trait::NyashBox;

struct Registry {
    next: AtomicU64,
    map: RwLock<HashMap<u64, Arc<dyn NyashBox>>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
            map: RwLock::new(HashMap::new()),
        }
    }
    fn alloc(&self, obj: Arc<dyn NyashBox>) -> u64 {
        let h = self.next.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut m) = self.map.write() {
            m.insert(h, obj);
        }
        h
    }
    fn get(&self, h: u64) -> Option<Arc<dyn NyashBox>> {
        self.map.read().ok().and_then(|m| m.get(&h).cloned())
    }
    fn snapshot(&self) -> Vec<Arc<dyn NyashBox>> {
        if let Ok(m) = self.map.read() {
            return m.values().cloned().collect();
        }
        Vec::new()
    }
    #[allow(dead_code)]
    fn drop_handle(&self, h: u64) {
        if let Ok(mut m) = self.map.write() {
            m.remove(&h);
        }
    }
}

static REG: OnceCell<Registry> = OnceCell::new();
fn reg() -> &'static Registry {
    REG.get_or_init(Registry::new)
}

// --- Lightweight diagnostics (opt-in) ---
fn trace_enabled() -> bool {
    static ON: OnceCell<bool> = OnceCell::new();
    *ON.get_or_init(|| match std::env::var("HAKO_TRACE_HOST_HANDLE").ok().as_deref() {
        Some("1") | Some("true") | Some("on") | Some("yes") => true,
        _ => false,
    })
}

fn trace(msg: &str) {
    if trace_enabled() {
        eprintln!("[host-handle] {}", msg);
    }
}

/// Box<dyn NyashBox> → HostHandle (u64)
pub fn to_handle_box(bx: Box<dyn NyashBox>) -> u64 {
    let ty = bx.type_name();
    let ptr = format!("{:p}", &*bx);
    let arc: Arc<dyn NyashBox> = Arc::from(bx);
    let arc_ptr = format!("{:p}", Arc::as_ptr(&arc));
    let h = reg().alloc(arc);
    trace(&format!(
        "alloc from Box: id={} type={} box_ptr={} arc_ptr={}",
        h, ty, ptr, arc_ptr
    ));
    h
}
/// Arc<dyn NyashBox> → HostHandle (u64)
pub fn to_handle_arc(arc: Arc<dyn NyashBox>) -> u64 {
    let ty = arc.type_name();
    let arc_ptr = format!("{:p}", Arc::as_ptr(&arc));

    // ✅ PluginBoxV2の場合、キャッシュに登録（identity保持のため）
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    {
        use crate::runtime::plugin_loader_v2::{PluginBoxV2, cache};
        if let Some(pb) = arc.as_any().downcast_ref::<PluginBoxV2>() {
            if let Ok(mut map) = cache().write() {
                let key = (pb.inner.type_id, pb.inner.instance_id);
                map.insert(key, std::sync::Arc::downgrade(&pb.inner));
                if crate::runtime::env_gate_box::debug_plugin() {
                    eprintln!("[host-handle] PluginBoxV2 cache insert: type_id={} instance_id={} cache_size={}",
                             pb.inner.type_id, pb.inner.instance_id, map.len());
                }
            }
        }
    }

    let h = reg().alloc(arc);
    trace(&format!("alloc from Arc: id={} type={} arc_ptr={}", h, ty, arc_ptr));
    h
}
/// HostHandle(u64) → Arc<dyn NyashBox>
pub fn get(h: u64) -> Option<Arc<dyn NyashBox>> {
    let out = reg().get(h);
    if let Some(ref arc) = out {
        let ty = arc.type_name();
        let arc_ptr = format!("{:p}", Arc::as_ptr(arc));
        trace(&format!("get: id={} type={} arc_ptr={}", h, ty, arc_ptr));
    } else {
        trace(&format!("get: id={} miss", h));
    }
    out
}

/// Snapshot all current handles as Arc<dyn NyashBox> roots for diagnostics/GC traversal.
pub fn snapshot() -> Vec<Arc<dyn NyashBox>> {
    reg().snapshot()
}
