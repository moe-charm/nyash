/*!
 * ScopeTracker - Track Box instances for proper lifecycle management
 * 
 * Phase 9.78a: Unified Box lifecycle management for VM
 */

use std::sync::Arc;
use crate::box_trait::NyashBox;
use crate::instance_v2::InstanceBox;
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
use crate::runtime::plugin_loader_v2::PluginBoxV2;

/// Tracks Box instances created in different scopes for proper fini calls
pub struct ScopeTracker {
    /// Stack of scopes, each containing Boxes created in that scope
    scopes: Vec<Vec<Arc<dyn NyashBox>>>,
    /// Root regions for GC (values pinned as roots during a dynamic region)
    roots: Vec<Vec<crate::backend::vm::VMValue>>,
}

impl ScopeTracker {
    /// Create a new scope tracker
    pub fn new() -> Self {
        Self {
            scopes: vec![Vec::new()], // Start with one root scope
            roots: vec![Vec::new()],  // Start with one root region
        }
    }
    
    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }
    
    /// Exit current scope and call fini on all Boxes created in it
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Call fini in reverse order of creation
            for arc_box in scope.into_iter().rev() {
                // InstanceBox: call fini()
                if let Some(instance) = arc_box.as_any().downcast_ref::<InstanceBox>() {
                    let _ = instance.fini();
                    continue;
                }
                // PluginBoxV2: 明示ライフサイクルに合わせ、スコープ終了時にfini（自己責任運用）
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if let Some(p) = arc_box.as_any().downcast_ref::<PluginBoxV2>() {
                    p.finalize_now();
                    continue;
                }
                // Builtin and others: no-op for now
            }
        }
        
        // Ensure we always have at least one scope
        if self.scopes.is_empty() { self.scopes.push(Vec::new()); }
    }
    
    /// Register a Box in the current scope
    pub fn register_box(&mut self, nyash_box: Arc<dyn NyashBox>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.push(nyash_box);
        }
    }
    
    /// Clear all scopes (used when resetting VM state)
    pub fn clear(&mut self) {
        // Pop all scopes and call fini
        while self.scopes.len() > 1 {
            self.pop_scope();
        }
        
        // Clear the root scope
        if let Some(root_scope) = self.scopes.first_mut() {
            root_scope.clear();
        }

        // Reset roots to a single empty region
        self.roots.clear();
        self.roots.push(Vec::new());
    }

    // ===== GC root region API (Phase 10.4 prep) =====
    /// Enter a new GC root region
    pub fn enter_root_region(&mut self) {
        if crate::config::env::gc_trace() {
            eprintln!("[GC] roots: enter");
        }
        self.roots.push(Vec::new());
    }

    /// Leave current GC root region (dropping all pinned values)
    pub fn leave_root_region(&mut self) {
        if let Some(_) = self.roots.pop() {
            if crate::config::env::gc_trace() {
                eprintln!("[GC] roots: leave");
            }
        }
        if self.roots.is_empty() { self.roots.push(Vec::new()); }
    }

    /// Pin a VMValue into the current root region (cheap clone)
    pub fn pin_root(&mut self, v: &crate::backend::vm::VMValue) {
        if let Some(cur) = self.roots.last_mut() {
            cur.push(v.clone());
            if crate::config::env::gc_trace() {
                eprintln!("[GC] roots: pin {:?}", v);
            }
        }
    }

    /// Total number of pinned roots across all regions (for GC PoC diagnostics)
    pub fn root_count_total(&self) -> usize { self.roots.iter().map(|r| r.len()).sum() }

    /// Number of active root regions
    pub fn root_regions(&self) -> usize { self.roots.len() }

    /// Snapshot a flat vector of current roots (cloned) for diagnostics
    pub fn roots_snapshot(&self) -> Vec<crate::backend::vm::VMValue> {
        let mut out = Vec::new();
        for region in &self.roots { out.extend(region.iter().cloned()); }
        out
    }
}

impl Default for ScopeTracker {
    fn default() -> Self {
        Self::new()
    }
}
