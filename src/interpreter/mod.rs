#![cfg(feature = "interpreter-legacy")]
// Shim module to re-export legacy interpreter from archive path
#[path = "../archive/interpreter_legacy/mod.rs"]
mod legacy_interpreter_mod;
pub use legacy_interpreter_mod::*;

