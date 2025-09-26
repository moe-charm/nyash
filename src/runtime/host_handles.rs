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

/// Box<dyn NyashBox> → HostHandle (u64)
pub fn to_handle_box(bx: Box<dyn NyashBox>) -> u64 {
    reg().alloc(Arc::from(bx))
}
/// Arc<dyn NyashBox> → HostHandle (u64)
pub fn to_handle_arc(arc: Arc<dyn NyashBox>) -> u64 {
    reg().alloc(arc)
}
/// HostHandle(u64) → Arc<dyn NyashBox>
pub fn get(h: u64) -> Option<Arc<dyn NyashBox>> {
    reg().get(h)
}

/// Snapshot all current handles as Arc<dyn NyashBox> roots for diagnostics/GC traversal.
pub fn snapshot() -> Vec<Arc<dyn NyashBox>> {
    reg().snapshot()
}
