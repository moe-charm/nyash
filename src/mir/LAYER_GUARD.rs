#![doc = "このファイルは層の責務を定義します"]
pub const LAYER_NAME: &str = "mir";
pub const ALLOWED_IMPORTS: &[&str] = &["types", "instruction", "builder", "optimizer", "verification"];
pub const FORBIDDEN_IMPORTS: &[&str] = &["runtime", "backend"];
