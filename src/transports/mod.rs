/**
 * Transport implementations module
 * 
 * Contains various transport implementations:
 * - InProcessTransport: For local communication within same process
 * - Future: WebSocketTransport, WebRTCTransport, etc.
 */

pub mod in_process_transport;

pub use in_process_transport::InProcessTransport;