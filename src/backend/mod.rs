/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;

pub use vm::{VM, VMError};