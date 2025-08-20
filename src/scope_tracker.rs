/*!
 * ScopeTracker - Track Box instances for proper lifecycle management
 * 
 * Phase 9.78a: Unified Box lifecycle management for VM
 */

use std::sync::Arc;
use crate::box_trait::NyashBox;

/// Tracks Box instances created in different scopes for proper fini calls
pub struct ScopeTracker {
    /// Stack of scopes, each containing Boxes created in that scope
    scopes: Vec<Vec<Arc<dyn NyashBox>>>,
}

impl ScopeTracker {
    /// Create a new scope tracker
    pub fn new() -> Self {
        Self {
            scopes: vec![Vec::new()], // Start with one root scope
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
                // For now, fini handling is simplified
                // In a full implementation, we would check if the Box has a fini method
                // and call it appropriately
                // TODO: Implement proper fini dispatch
                let _ = arc_box; // Suppress unused warning
            }
        }
        
        // Ensure we always have at least one scope
        if self.scopes.is_empty() {
            self.scopes.push(Vec::new());
        }
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
    }
}

impl Default for ScopeTracker {
    fn default() -> Self {
        Self::new()
    }
}