#![allow(dead_code)]
#![doc = "このファイルは層の責務を定義します (parser layer)"]
pub const LAYER_NAME: &str = "parser";
pub const ALLOWED_IMPORTS: &[&str] = &["ast", "tokenizer", "common"];
pub const FORBIDDEN_IMPORTS: &[&str] = &["mir", "runtime", "resolver"];

