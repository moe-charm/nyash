//! emission: MIR命令の薄い発行箱（仕様不変）。
//! - constant.rs: Const発行を一箇所に集約
//! - compare.rs: Compare命令の薄い発行
//! - branch.rs: Branch/Jump 発行の薄い関数

pub mod constant;
pub mod compare;
pub mod branch;
