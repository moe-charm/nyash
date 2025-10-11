#![doc = "This file defines the layer boundary for method_router_box."]
#![allow(dead_code)]
pub const LAYER_NAME: &str = "method_router_box";
pub const ALLOWED_IMPORTS: &[&str] = &[
    "backend", "box_trait", "runtime::plugin_host_box", "runtime::type_registry",
    "runtime::adapters"
];
pub const FORBIDDEN_IMPORTS: &[&str] = &[
    "boxes::array", "boxes::map_box", // no direct builtin implementations here
];
