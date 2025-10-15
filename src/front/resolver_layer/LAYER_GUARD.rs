#![allow(dead_code)]
#![doc = "このファイルは層の責務を定義します (resolver layer)"]
pub const LAYER_NAME: &str = "resolver";
pub const ALLOWED_IMPORTS: &[&str] = &["ast", "layers", "common"];
pub const FORBIDDEN_IMPORTS: &[&str] = &["mir", "runtime", "parser"];

