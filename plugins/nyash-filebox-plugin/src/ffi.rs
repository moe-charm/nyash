//! FFI (Foreign Function Interface) type definitions for FileBox plugin

use std::os::raw::c_char;

// ============ FFI Types ============

#[allow(dead_code)]
#[repr(C)]
pub struct NyashMethodInfo {
    pub method_id: u32,
    pub name: *const c_char,
    pub signature: u32,
}

#[allow(dead_code)]
#[repr(C)]
pub struct NyashPluginInfo {
    pub type_id: u32,
    pub type_name: *const c_char,
    pub method_count: usize,
    pub methods: *const NyashMethodInfo,
}

/// TypeBox FFI structure for plugin export
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,        // 'TYBX' (0x54594258)
    pub version: u16,        // 1
    pub struct_size: u16,    // sizeof(NyashTypeBoxFfi)
    pub name: *const c_char, // C string, e.g., "FileBox\0"
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}

unsafe impl Sync for NyashTypeBoxFfi {}
unsafe impl Send for NyashTypeBoxFfi {}

// ABI Constants
pub const ABI_TAG_TYBX: u32 = 0x54594258; // 'TYBX'
pub const ABI_VERSION: u16 = 1;
