/**
 * MessageBus - Central communication hub (Bus = Local OS metaphor)
 * 
 * Design principles from ChatGPT discussion:
 * - Always present in P2PBox (even for network transport)
 * - Handles local routing, subscription, monitoring
 * - Singleton pattern for process-wide message coordination
 * - Synchronous-first implementation
 * 
 * NyaMesh inspiration:
 * - InProcessMessageBus singleton pattern
 * - Node registration/unregistration
 * - Statistics tracking
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::NyashBox;

/// Message structure for internal routing
#[derive(Debug)]
pub struct BusMessage {
    pub from: String,
    pub to: String,
    pub intent: String,
    pub data: Box<dyn NyashBox>,
    pub timestamp: std::time::SystemTime,
}

impl Clone for BusMessage {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            intent: self.intent.clone(),
            data: self.data.clone_box(),  // NyashBoxのclone_box()メソッドを使用
            timestamp: self.timestamp,
        }
    }
}

/// Node registration information
struct NodeInfo {
    node_id: String,
    callbacks: HashMap<String, Vec<Box<dyn Fn(&BusMessage) + Send + Sync>>>,
}

/// Central MessageBus - handles all local message routing
pub struct MessageBus {
    /// Registered nodes in this process
    nodes: RwLock<HashMap<String, Arc<Mutex<NodeInfo>>>>,
    
    /// Bus-level statistics
    stats: Mutex<BusStats>,
}

#[derive(Debug, Clone, Default)]
pub struct BusStats {
    pub messages_routed: u64,
    pub routing_errors: u64,
    pub nodes_registered: u64,
    pub total_callbacks: u64,
}

impl MessageBus {
    /// Create new MessageBus instance
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            stats: Mutex::new(BusStats::default()),
        }
    }
    
    /// Register a node in the message bus
    pub fn register_node(&self, node_id: &str) -> Result<(), String> {
        let mut nodes = self.nodes.write().unwrap();
        
        if nodes.contains_key(node_id) {
            return Err(format!("Node '{}' already registered", node_id));
        }
        
        let node_info = NodeInfo {
            node_id: node_id.to_string(),
            callbacks: HashMap::new(),
        };
        
        nodes.insert(node_id.to_string(), Arc::new(Mutex::new(node_info)));
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.nodes_registered += 1;
        
        Ok(())
    }
    
    /// Unregister a node from the message bus
    pub fn unregister_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.remove(node_id);
    }
    
    /// Check if a node is registered locally
    pub fn has_node(&self, node_id: &str) -> bool {
        let nodes = self.nodes.read().unwrap();
        nodes.contains_key(node_id)
    }
    
    /// Route message to local node
    pub fn route(&self, message: BusMessage) -> Result<(), String> {
        let nodes = self.nodes.read().unwrap();
        
        if let Some(node) = nodes.get(&message.to) {
            let node = node.lock().unwrap();
            
            // Find callbacks for this intent
            if let Some(callbacks) = node.callbacks.get(&message.intent) {
                for callback in callbacks {
                    callback(&message);
                }
            }
            
            // Update stats
            let mut stats = self.stats.lock().unwrap();
            stats.messages_routed += 1;
            
            Ok(())
        } else {
            let mut stats = self.stats.lock().unwrap();
            stats.routing_errors += 1;
            Err(format!("Node '{}' not found for routing", message.to))
        }
    }
    
    /// Register callback for specific intent on a node
    pub fn on(&self, node_id: &str, intent: &str, callback: Box<dyn Fn(&BusMessage) + Send + Sync>) -> Result<(), String> {
        let nodes = self.nodes.read().unwrap();
        
        if let Some(node) = nodes.get(node_id) {
            let mut node = node.lock().unwrap();
            node.callbacks.entry(intent.to_string()).or_insert_with(Vec::new).push(callback);
            
            // Update stats
            let mut stats = self.stats.lock().unwrap();
            stats.total_callbacks += 1;
            
            Ok(())
        } else {
            Err(format!("Node '{}' not found for callback registration", node_id))
        }
    }
    
    /// Get list of registered nodes
    pub fn get_registered_nodes(&self) -> Vec<String> {
        let nodes = self.nodes.read().unwrap();
        nodes.keys().cloned().collect()
    }
    
    /// Get bus statistics
    pub fn get_stats(&self) -> BusStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }
}

use std::sync::OnceLock;

/// Global MessageBus singleton
static GLOBAL_MESSAGE_BUS: OnceLock<Arc<MessageBus>> = OnceLock::new();

/// Get global message bus instance
pub fn get_global_message_bus() -> Arc<MessageBus> {
    GLOBAL_MESSAGE_BUS.get_or_init(|| Arc::new(MessageBus::new())).clone()
}