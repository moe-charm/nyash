/*! 📡 Messaging Module - P2P Communication Infrastructure
 * 
 * This module provides the core messaging infrastructure for P2P communication
 * in Nyash, implementing the MessageBus singleton pattern for local message routing.
 */

pub mod message_bus;

pub use message_bus::{MessageBus, MessageBusData, BusEndpoint, IntentHandler, SendError};