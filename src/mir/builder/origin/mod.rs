//! Origin inference utilities (P0)
//!
//! Responsibility
//! - Attach and maintain simple "origin" metadata (receiver/me/class) for Known化。
//! - Keep logic minimal and spec‑neutral（挙動不変）。
//!
//! Modules
//! - infer: entry points for annotating origins（me/receiver/newbox）
//! - phi: lightweight propagation at PHI when全入力が一致

pub mod infer;
pub mod phi;

