//! VerifiedPath — newtype for paths that were successfully read/parsed.
//!
//! Purpose
//! - Prevent accidental storage of invalid config paths by requiring construction
//!   only after successful I/O/parse.
//! - Keep API minimal and zero-cost. Intended as a guardrail for loader fields.

#[derive(Clone, Debug)]
pub struct VerifiedPath(String);

impl VerifiedPath {
    /// Construct only after a successful read/parse.
    pub fn new_ok(p: String) -> Self { Self(p) }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn into_inner(self) -> String { self.0 }
}

