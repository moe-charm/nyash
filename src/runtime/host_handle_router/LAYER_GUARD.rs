#![doc = "Layer guard for host_handle_router"]
#![allow(dead_code)]
pub const LAYER_NAME: &str = "host_handle_router";
pub const ALLOWED_IMPORTS: &[&str] = &[
    // intent: only route via HostHandle indirection; do not downcast into builtin boxes here
];
pub const FORBIDDEN_IMPORTS: &[&str] = &[
    "boxes::array", "boxes::map_box", "backend::mir_interpreter",
];
