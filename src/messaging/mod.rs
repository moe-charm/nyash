/*! 📡 Messaging Module - P2P Communication Infrastructure
 *
 * This module provides the core messaging infrastructure for P2P communication
 * in Nyash, implementing the MessageBus singleton pattern for local message routing.
 */

#[cfg(feature = "legacy-boxes")]
pub mod message_bus;

#[cfg(feature = "legacy-boxes")]
pub use message_bus::{BusEndpoint, IntentHandler, MessageBus, MessageBusData, SendError};
