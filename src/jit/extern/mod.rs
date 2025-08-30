//! Host-callable externs for JIT-compiled code
//!
//! Phase 10_d: Provide thin bridges for Array/Map hot operations that
//! JIT can call via symbol names. Lowering will resolve MIR ops into
//! these externs once call emission is added.

pub mod collections;
pub mod handles;
pub mod birth;
pub mod runtime;
