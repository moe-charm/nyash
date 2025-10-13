//! Diagnostics (stable messages) for MIR interpreter
//! Keep user-facing error strings centralized to avoid drift between
//! implementation, tests, and documentation.

/// Plugin-only build rejects Extern callee execution (legacy-only route).
pub const DIAG_EXTERN_DISABLED: &str = "extern calls disabled (legacy-only)";

