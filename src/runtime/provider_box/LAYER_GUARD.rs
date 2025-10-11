#![doc = "Layer guard for provider_box"]
#![allow(dead_code)]
pub const LAYER_NAME: &str = "provider_box";
pub const ALLOWED_IMPORTS: &[&str] = &[
    "runtime::plugin_boot_box", "runtime::plugin_loader_unified", "runtime::type_registry",
    "box_trait", "box_factory"
];
pub const FORBIDDEN_IMPORTS: &[&str] = &[
    "boxes::array", "boxes::map_box", "backend::mir_interpreter" // no direct VM/builtin boxes
];
