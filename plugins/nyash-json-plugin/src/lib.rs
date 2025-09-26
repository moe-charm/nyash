//! Nyash JSON Plugin
//!
//! High-performance JSON processing plugin with dual provider support:
//! - serde_json: Pure Rust implementation (default)
//! - yyjson: Ultra-fast C library with SIMD optimizations (experimental)

use std::os::raw::c_char;

// Module declarations
mod constants;
mod doc_box;
mod ffi;
mod node_box;
mod provider;
mod tlv_helpers;

// Re-export key components for FFI
use doc_box::{jsondoc_invoke_id, jsondoc_resolve};
use node_box::{jsonnode_invoke_id, jsonnode_resolve};

// NyashTypeBoxFfi structure
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u8,
    pub struct_size: u16,
    pub name: *const c_char,
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u32,
}

unsafe impl Sync for NyashTypeBoxFfi {}
unsafe impl Send for NyashTypeBoxFfi {}

// Export JsonDocBox
#[no_mangle]
pub static nyash_typebox_JsonDocBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"JsonDocBox\0".as_ptr() as *const c_char,
    resolve: Some(jsondoc_resolve),
    invoke_id: Some(jsondoc_invoke_id),
    capabilities: 0,
};

// Export JsonNodeBox
#[no_mangle]
pub static nyash_typebox_JsonNodeBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"JsonNodeBox\0".as_ptr() as *const c_char,
    resolve: Some(jsonnode_resolve),
    invoke_id: Some(jsonnode_invoke_id),
    capabilities: 0,
};

// Plugin metadata export
#[no_mangle]
pub static nyash_plugin_name: &[u8] = b"nyash-json\0";

#[no_mangle]
pub static nyash_plugin_version: &[u8] = b"0.1.0\0";

// Plugin initialization (if needed in future)
#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 {
    // Currently no initialization needed
    0 // OK
}

// Plugin cleanup (if needed in future)
#[no_mangle]
pub extern "C" fn nyash_plugin_fini() -> i32 {
    // Currently no cleanup needed
    0 // OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        // Verify plugin name is null-terminated
        assert_eq!(nyash_plugin_name[nyash_plugin_name.len() - 1], 0);

        // Verify plugin version is null-terminated
        assert_eq!(nyash_plugin_version[nyash_plugin_version.len() - 1], 0);
    }

    #[test]
    fn test_typebox_ffi_structure() {
        // Verify ABI tag
        assert_eq!(nyash_typebox_JsonDocBox.abi_tag, 0x54594258);
        assert_eq!(nyash_typebox_JsonNodeBox.abi_tag, 0x54594258);

        // Verify version
        assert_eq!(nyash_typebox_JsonDocBox.version, 1);
        assert_eq!(nyash_typebox_JsonNodeBox.version, 1);

        // Verify struct size
        let expected_size = std::mem::size_of::<NyashTypeBoxFfi>() as u16;
        assert_eq!(nyash_typebox_JsonDocBox.struct_size, expected_size);
        assert_eq!(nyash_typebox_JsonNodeBox.struct_size, expected_size);
    }
}
