/*! 🚀 Transport Module - Communication Layer Abstraction
 * 
 * This module defines the Transport trait and implementations for different
 * communication methods (InProcess, WebSocket, WebRTC, etc.)
 */

pub mod inprocess;

use crate::boxes::IntentBox;

/// Envelope containing message with metadata
#[derive(Debug, Clone)]
pub struct IntentEnvelope {
    pub from: String,
    pub to: String,
    pub intent: IntentBox,
    pub timestamp: std::time::Instant,
}

/// Options for sending messages
#[derive(Debug, Clone, Default)]
pub struct SendOpts {
    pub timeout_ms: Option<u64>,
    pub priority: Option<u8>,
}

/// Transport errors
#[derive(Debug, Clone)]
pub enum TransportError {
    NodeNotFound(String),
    NetworkError(String),
    Timeout(String),
    SerializationError(String),
}

/// Abstract transport trait for different communication methods
pub trait Transport: Send + Sync {
    /// Get the node ID of this transport
    fn node_id(&self) -> &str;
    
    /// Send a message to a specific node
    fn send(&self, to: &str, intent: IntentBox, opts: SendOpts) -> Result<(), TransportError>;
    
    /// Register a callback for receiving messages
    fn on_receive(&mut self, callback: Box<dyn Fn(IntentEnvelope) + Send + Sync>);
    
    /// Check if a node is reachable
    fn is_reachable(&self, node_id: &str) -> bool;
    
    /// Get transport type identifier
    fn transport_type(&self) -> &'static str;
}

pub use inprocess::InProcessTransport;