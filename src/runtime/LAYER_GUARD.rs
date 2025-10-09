#![doc = "このファイルは層の責務を定義します"]
pub const LAYER_NAME: &str = "runtime";
pub const ALLOWED_IMPORTS: &[&str] = &["provider_verify", "plugin_loader_v2", "type_meta"];
pub const FORBIDDEN_IMPORTS: &[&str] = &["parser", "resolver", "mir::builder"];
