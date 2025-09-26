/*!
 * MIR Definitions Module
 *
 * Central location for all MIR type and instruction definitions
 */

pub mod call_unified;

// Re-export commonly used types
pub use call_unified::{Callee, CallFlags, MirCall};