//! Handle type definitions for Hakorune ABI

/// Opaque handle to Hakorune objects (u64 internally)
pub type HakoHandle = u64;

/// Invalid/null handle constant
pub const HAKO_INVALID_HANDLE: HakoHandle = 0;
