//! WASM backend configuration

/// Enable WASM lowering pass for BoxCall → MirCall(Method/Extern)
pub fn wasm_lower_boxcall() -> bool {
    std::env::var("NYASH_WASM_LOWER_BOXCALL").ok().as_deref() == Some("1")
}

