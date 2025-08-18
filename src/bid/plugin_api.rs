use super::{BidError, BidResult, NyashHostVtable, NyashPluginInfo};
use std::os::raw::c_char;

/// Plugin API function signatures for C FFI
///
/// These are the function signatures that plugins must implement.
/// They are defined as Rust types for type safety when loading plugins.

/// Get plugin ABI version
/// Returns: BID version number (1 for BID-1)
pub type PluginAbiFn = unsafe extern "C" fn() -> u32;

/// Initialize plugin
/// Parameters:
/// - host: Host function table for plugin to use
/// - info: Plugin information to be filled by plugin
/// Returns: 0 on success, negative error code on failure
pub type PluginInitFn = unsafe extern "C" fn(
    host: *const NyashHostVtable,
    info: *mut NyashPluginInfo,
) -> i32;

/// Invoke a plugin method
/// Parameters:
/// - type_id: Box type ID
/// - method_id: Method ID
/// - instance_id: Instance ID (for instance methods)
/// - args: BID-1 TLV encoded arguments
/// - args_len: Length of arguments
/// - result: Buffer for BID-1 TLV encoded result
/// - result_len: Input: buffer size, Output: actual result size
/// Returns: 0 on success, negative error code on failure
pub type PluginInvokeFn = unsafe extern "C" fn(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32;

/// Shutdown plugin and cleanup resources
pub type PluginShutdownFn = unsafe extern "C" fn();

/// Plugin API entry points
pub const PLUGIN_ABI_SYMBOL: &str = "nyash_plugin_abi";
pub const PLUGIN_INIT_SYMBOL: &str = "nyash_plugin_init";
pub const PLUGIN_INVOKE_SYMBOL: &str = "nyash_plugin_invoke";
pub const PLUGIN_SHUTDOWN_SYMBOL: &str = "nyash_plugin_shutdown";

/// Plugin handle containing loaded functions
pub struct PluginHandle {
    pub abi: PluginAbiFn,
    pub init: PluginInitFn,
    pub invoke: PluginInvokeFn,
    pub shutdown: PluginShutdownFn,
}

impl PluginHandle {
    /// Validate ABI version
    pub fn check_abi(&self) -> BidResult<()> {
        let version = unsafe { (self.abi)() };
        if version != super::BID_VERSION as u32 {
            return Err(BidError::version_mismatch());
        }
        Ok(())
    }
    
    /// Initialize plugin with host vtable
    pub fn initialize(
        &self,
        host: &NyashHostVtable,
        info: &mut NyashPluginInfo,
    ) -> BidResult<()> {
        let result = unsafe {
            (self.init)(
                host as *const NyashHostVtable,
                info as *mut NyashPluginInfo,
            )
        };
        
        if result != 0 {
            Err(BidError::from_raw(result))
        } else {
            Ok(())
        }
    }
    
    /// Invoke a plugin method
    pub fn invoke(
        &self,
        type_id: u32,
        method_id: u32,
        instance_id: u32,
        args: &[u8],
        result_buffer: &mut Vec<u8>,
    ) -> BidResult<()> {
        // First call: get required size
        let mut required_size = 0;
        let result = unsafe {
            (self.invoke)(
                type_id,
                method_id,
                instance_id,
                args.as_ptr(),
                args.len(),
                std::ptr::null_mut(),
                &mut required_size,
            )
        };
        
        // Check for error (except buffer too small)
        if result != 0 && result != -1 {
            return Err(BidError::from_raw(result));
        }
        
        // Allocate buffer if needed
        if required_size > 0 {
            result_buffer.resize(required_size, 0);
            
            // Second call: get actual data
            let mut actual_size = required_size;
            let result = unsafe {
                (self.invoke)(
                    type_id,
                    method_id,
                    instance_id,
                    args.as_ptr(),
                    args.len(),
                    result_buffer.as_mut_ptr(),
                    &mut actual_size,
                )
            };
            
            if result != 0 {
                return Err(BidError::from_raw(result));
            }
            
            // Trim to actual size
            result_buffer.truncate(actual_size);
        }
        
        Ok(())
    }
    
    /// Shutdown plugin
    pub fn shutdown(&self) {
        unsafe {
            (self.shutdown)();
        }
    }
}

/// Helper for creating host vtable with Rust closures
pub struct HostVtableBuilder {
    vtable: NyashHostVtable,
}

impl HostVtableBuilder {
    pub fn new() -> Self {
        Self {
            vtable: NyashHostVtable::empty(),
        }
    }
    
    pub fn with_alloc<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> *mut std::os::raw::c_void + 'static,
    {
        // Note: In real implementation, would need to store the closure
        // and create a proper extern "C" function. This is simplified.
        self
    }
    
    pub fn with_free<F>(mut self, f: F) -> Self
    where
        F: Fn(*mut std::os::raw::c_void) + 'static,
    {
        self
    }
    
    pub fn with_log<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self
    }
    
    pub fn build(self) -> NyashHostVtable {
        self.vtable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock plugin functions for testing
    unsafe extern "C" fn mock_abi() -> u32 {
        1 // BID-1
    }
    
    unsafe extern "C" fn mock_init(
        _host: *const NyashHostVtable,
        info: *mut NyashPluginInfo,
    ) -> i32 {
        if !info.is_null() {
            (*info).type_id = 99;
            (*info).method_count = 0;
        }
        0
    }
    
    unsafe extern "C" fn mock_invoke(
        _type_id: u32,
        _method_id: u32,
        _instance_id: u32,
        _args: *const u8,
        _args_len: usize,
        _result: *mut u8,
        result_len: *mut usize,
    ) -> i32 {
        if !result_len.is_null() {
            *result_len = 0;
        }
        0
    }
    
    unsafe extern "C" fn mock_shutdown() {}
    
    #[test]
    fn test_plugin_handle() {
        let handle = PluginHandle {
            abi: mock_abi,
            init: mock_init,
            invoke: mock_invoke,
            shutdown: mock_shutdown,
        };
        
        // Check ABI
        assert!(handle.check_abi().is_ok());
        
        // Initialize
        let host = NyashHostVtable::empty();
        let mut info = NyashPluginInfo::empty();
        assert!(handle.initialize(&host, &mut info).is_ok());
        assert_eq!(info.type_id, 99);
        
        // Invoke
        let mut result = Vec::new();
        assert!(handle.invoke(99, 1, 0, &[], &mut result).is_ok());
    }
}