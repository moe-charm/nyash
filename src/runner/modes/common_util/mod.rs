/*!
 * Shared helpers for runner/modes/common.rs
 *
 * Minimal extraction to reduce duplication and prepare for full split.
 */

#[cfg(feature = "pyvm-bridge")]
pub mod pyvm;
pub mod selfhost_exe;
pub mod io;
pub mod selfhost;
pub mod resolve;
pub mod exec;
pub mod prelex;
