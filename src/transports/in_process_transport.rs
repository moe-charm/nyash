/**
 * InProcessTransport - Local process communication transport
 * 
 * Based on NyaMesh InProcessTransport design:
 * - Synchronous-first implementation (parallelSafe flag support)
 * - Direct function pointer callbacks (no async complexity)
 * - Simple message routing through global MessageBus
 * 
 * Key features from NyaMesh:
 * - parallelSafe = false by default (GUI thread safe)
 * - Direct callback execution
 * - Statistics tracking
 */

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::transport_trait::{Transport, TransportStats};
use crate::message_bus::{get_global_message_bus, BusMessage};
use crate::NyashBox;

/// InProcessTransport - for local communication within same process
pub struct InProcessTransport {
    /// Node ID for this transport
    node_id: String,
    
    /// Whether transport is initialized
    initialized: AtomicBool,
    
    /// Statistics
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    errors: AtomicU64,
}

impl InProcessTransport {
    /// Create new InProcessTransport with given node ID
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            initialized: AtomicBool::new(false),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    
    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

impl Transport for InProcessTransport {
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        // Register with global message bus
        let bus = get_global_message_bus();
        bus.register_node(&self.node_id)?;
        
        self.initialized.store(true, Ordering::Relaxed);
        Ok(())
    }
    
    fn send(&self, to: &str, intent: &str, data: Box<dyn NyashBox>) -> Result<(), String> {
        if !self.initialized.load(Ordering::Relaxed) {
            self.errors.fetch_add(1, Ordering::Relaxed);
            return Err("Transport not initialized".to_string());
        }
        
        // Create bus message
        let message = BusMessage {
            from: self.node_id.clone(),
            to: to.to_string(),
            intent: intent.to_string(),
            data,
            timestamp: std::time::SystemTime::now(),
        };
        
        // Route through global message bus
        let bus = get_global_message_bus();
        
        // Check if target is local
        if bus.has_node(to) {
            // Local routing - direct through bus
            match bus.route(message) {
                Ok(_) => {
                    self.messages_sent.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(e) => {
                    self.errors.fetch_add(1, Ordering::Relaxed);
                    Err(e)
                }
            }
        } else {
            // Target not found locally
            self.errors.fetch_add(1, Ordering::Relaxed);
            Err(format!("Target node '{}' not found in process", to))
        }
    }
    
    fn transport_type(&self) -> &'static str {
        "inprocess"
    }
    
    fn is_ready(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }
    
    fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        // Unregister from global message bus
        let bus = get_global_message_bus();
        bus.unregister_node(&self.node_id);
        
        self.initialized.store(false, Ordering::Relaxed);
        Ok(())
    }
    
    fn get_stats(&self) -> TransportStats {
        TransportStats {
            transport_type: self.transport_type().to_string(),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            is_ready: self.is_ready(),
        }
    }
}