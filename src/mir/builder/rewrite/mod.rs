//! Rewrite helpers (P1)
//!
//! Responsibility
//! - Known 経路の instance→function 正規化（obj.m → Class.m(me,…)）。
//! - 特殊規則（toString→str（互換:stringify）, equals など）の集約。
//! - 既定挙動は不変。dev 観測（resolve.try/choose）は observe 経由で発火。

pub mod known;
pub mod special;
pub mod gate;
