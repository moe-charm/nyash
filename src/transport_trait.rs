/**
 * Transport trait abstraction - NyaMesh style implementation
 * 
 * Design principles from NyaMesh:
 * - Transport = NIC (Network Interface Card) - handles communication method only
 * - Bus = Local OS - handles routing, subscription, monitoring
 * - Clean separation between transport mechanism and message routing
 * 
 * Based on ChatGPT discussion P2PBox architecture:
 * - P2PBox always has MessageBus (even for network transport)
 * - Transport abstraction allows switching InProcess/WebSocket/WebRTC
 * - Synchronous-first implementation strategy
 */

use crate::NyashBox;
use crate::transports::InProcessTransport;

/// Transport trait - represents different communication mechanisms
/// Like NyaMesh's TransportInterface, this abstracts the "how to send" part
pub trait Transport: Send + Sync {
    /// Initialize the transport (async-compatible but synchronous first)
    fn initialize(&mut self) -> Result<(), String>;
    
    /// Send message through this transport mechanism
    /// to: target node ID
    /// intent: message intent type
    /// data: message payload
    fn send(&self, to: &str, intent: &str, data: Box<dyn NyashBox>) -> Result<(), String>;
    
    /// Get transport type identifier (e.g., "inprocess", "websocket", "webrtc")
    fn transport_type(&self) -> &'static str;
    
    /// Check if transport is ready
    fn is_ready(&self) -> bool;
    
    /// Shutdown transport cleanly
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// Get transport statistics
    fn get_stats(&self) -> TransportStats;
}

/// Transport statistics - standardized across all transport types
#[derive(Debug, Clone)]
pub struct TransportStats {
    pub transport_type: String,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors: u64,
    pub is_ready: bool,
}

impl TransportStats {
    pub fn new(transport_type: &str) -> Self {
        Self {
            transport_type: transport_type.to_string(),
            messages_sent: 0,
            messages_received: 0,
            errors: 0,
            is_ready: false,
        }
    }
}

/// TransportKind - 通信方式の選択（Nyash同期・シンプル版）
#[derive(Debug, Clone)]
pub enum TransportKind {
    InProcess,      // プロセス内通信（最初に実装）
    WebSocket,      // WebSocket通信（将来実装）
    WebRTC,         // P2P直接通信（将来実装）
}

/// シンプルファクトリ関数
pub fn create_transport(kind: TransportKind, node_id: &str) -> Box<dyn Transport> {
    match kind {
        TransportKind::InProcess => Box::new(InProcessTransport::new(node_id.to_string())),
        TransportKind::WebSocket => todo!("WebSocket transport - 将来実装"),
        TransportKind::WebRTC => todo!("WebRTC transport - 将来実装"),
    }
}