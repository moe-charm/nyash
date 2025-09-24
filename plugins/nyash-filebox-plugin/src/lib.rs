//! Nyash FileBox Plugin — TypeBox v2
//!
//! Provides file I/O operations as a Nyash plugin

use std::os::raw::c_char;

// Module declarations
mod constants;
mod ffi;
mod filebox_impl;
mod state;
mod tlv_helpers;

// Re-exports
use ffi::{NyashTypeBoxFfi, ABI_TAG_TYBX, ABI_VERSION};
use filebox_impl::{filebox_invoke_id, filebox_resolve};

// ============ TypeBox v2 Export ============

/// FileBox TypeBox export
#[no_mangle]
pub static nyash_typebox_FileBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: ABI_TAG_TYBX,
    version: ABI_VERSION,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"FileBox\0".as_ptr() as *const c_char,
    resolve: Some(filebox_resolve),
    invoke_id: Some(filebox_invoke_id),
    capabilities: 0,
};

// ============ Plugin Metadata (optional) ============

#[no_mangle]
pub static nyash_plugin_name: &[u8] = b"nyash-filebox\0";

#[no_mangle]
pub static nyash_plugin_version: &[u8] = b"0.1.0\0";

/// Optional shutdown hook for host runtimes that expect a cleanup entrypoint
#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() {
    state::clear_instances();
}

// ============ Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_plugin_metadata() {
        // Verify plugin name is null-terminated
        assert_eq!(nyash_plugin_name[nyash_plugin_name.len() - 1], 0);

        // Verify plugin version is null-terminated
        assert_eq!(nyash_plugin_version[nyash_plugin_version.len() - 1], 0);
    }

    #[test]
    fn test_typebox_structure() {
        // Verify ABI tag
        assert_eq!(nyash_typebox_FileBox.abi_tag, ABI_TAG_TYBX);

        // Verify version
        assert_eq!(nyash_typebox_FileBox.version, ABI_VERSION);

        // Verify struct size matches
        let expected_size = std::mem::size_of::<NyashTypeBoxFfi>() as u16;
        assert_eq!(nyash_typebox_FileBox.struct_size, expected_size);

        // Verify name is not null
        assert!(!nyash_typebox_FileBox.name.is_null());

        // Verify callbacks are set
        assert!(nyash_typebox_FileBox.resolve.is_some());
        assert!(nyash_typebox_FileBox.invoke_id.is_some());
    }

    #[test]
    fn test_method_resolution() {
        // Test resolve function
        let resolve = nyash_typebox_FileBox.resolve.unwrap();

        // Test known methods
        unsafe {
            let open = b"open\0";
            assert_eq!(resolve(open.as_ptr() as *const c_char), METHOD_OPEN);

            let read = b"read\0";
            assert_eq!(resolve(read.as_ptr() as *const c_char), METHOD_READ);

            let write = b"write\0";
            assert_eq!(resolve(write.as_ptr() as *const c_char), METHOD_WRITE);

            let unknown = b"unknown\0";
            assert_eq!(resolve(unknown.as_ptr() as *const c_char), 0);
        }
    }
}
