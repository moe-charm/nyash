//! Builder Guards — structural checks that run during MIR emission
//!
//! Parameter overwrite guard is placed here to keep `builder.rs` lean and
//! make the policy reversible via ENV without touching emission code.

pub mod parameter_guard;

