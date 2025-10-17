#![doc = "このファイルは meta 層の責務を定義します"]
/// 層名
pub const LAYER_NAME: &str = "runtime::meta";
/// 許可される依存（概念）
pub const ALLOWED_DEPENDENCIES: &[&str] = &[
    "vm/scheduler",
    "gc",
    "router",
];
/// 禁止される依存（概念）
pub const FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "plugins",
    "external-io",
];

