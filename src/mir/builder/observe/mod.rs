//! Builder observability helpers (dev‑only; default OFF)
//!
//! - ssa: PHI/SSA related debug emissions
//! - resolve: method resolution try/choose（既存呼び出しの置換は段階的に）

pub mod ssa;
pub mod resolve;

