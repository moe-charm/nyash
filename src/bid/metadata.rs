use super::{BidError, BidResult};
use std::os::raw::{c_char, c_void};
use std::ffi::{CStr, CString};

/// Host function table provided to plugins
#[repr(C)]
pub struct NyashHostVtable {
    /// Allocate memory
    pub alloc: Option<extern "C" fn(size: usize) -> *mut c_void>,
    
    /// Free memory
    pub free: Option<extern "C" fn(ptr: *mut c_void)>,
    
    /// Wake a future (for FutureBox support)
    pub wake: Option<extern "C" fn(future_id: u32)>,
    
    /// Log a message
    pub log: Option<extern "C" fn(msg: *const c_char)>,
}

impl NyashHostVtable {
    /// Create an empty vtable
    pub fn empty() -> Self {
        Self {
            alloc: None,
            free: None,
            wake: None,
            log: None,
        }
    }
    
    /// Check if all required functions are present
    pub fn is_complete(&self) -> bool {
        self.alloc.is_some() && 
        self.free.is_some() && 
        self.log.is_some()
        // wake is optional for async support
    }
}

/// Method information
#[repr(C)]
pub struct NyashMethodInfo {
    /// Method ID (unique within the Box type)
    pub method_id: u32,
    
    /// Method name (null-terminated C string)
    pub method_name: *const c_char,
    
    /// Type signature hash for validation
    pub signature_hash: u32,
}

impl NyashMethodInfo {
    /// Create method info with safe string handling
    pub fn new(method_id: u32, method_name: &str, signature_hash: u32) -> BidResult<(Self, CString)> {
        let c_name = CString::new(method_name)
            .map_err(|_| BidError::InvalidUtf8)?;
        
        let info = Self {
            method_id,
            method_name: c_name.as_ptr(),
            signature_hash,
        };
        
        Ok((info, c_name))
    }
    
    /// Get method name as string (unsafe: requires valid pointer)
    pub unsafe fn name(&self) -> BidResult<&str> {
        if self.method_name.is_null() {
            return Err(BidError::InvalidArgs);
        }
        
        CStr::from_ptr(self.method_name)
            .to_str()
            .map_err(|_| BidError::InvalidUtf8)
    }
}

/// Plugin information
#[repr(C)]
pub struct NyashPluginInfo {
    /// Box type ID (e.g., 6 for FileBox)
    pub type_id: u32,
    
    /// Type name (null-terminated C string)
    pub type_name: *const c_char,
    
    /// Number of methods
    pub method_count: u32,
    
    /// Method information array
    pub methods: *const NyashMethodInfo,
}

impl NyashPluginInfo {
    /// Create an empty plugin info
    pub fn empty() -> Self {
        Self {
            type_id: 0,
            type_name: std::ptr::null(),
            method_count: 0,
            methods: std::ptr::null(),
        }
    }
    
    /// Get type name as string (unsafe: requires valid pointer)
    pub unsafe fn name(&self) -> BidResult<&str> {
        if self.type_name.is_null() {
            return Err(BidError::InvalidArgs);
        }
        
        CStr::from_ptr(self.type_name)
            .to_str()
            .map_err(|_| BidError::InvalidUtf8)
    }
    
    /// Get methods as slice (unsafe: requires valid pointer and count)
    pub unsafe fn methods_slice(&self) -> BidResult<&[NyashMethodInfo]> {
        if self.methods.is_null() || self.method_count == 0 {
            return Ok(&[]);
        }
        
        Ok(std::slice::from_raw_parts(
            self.methods,
            self.method_count as usize,
        ))
    }
}

/// Plugin lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin loaded but not initialized
    Loaded,
    
    /// Plugin initialized and ready
    Ready,
    
    /// Plugin shutting down
    ShuttingDown,
    
    /// Plugin unloaded
    Unloaded,
}

/// Plugin metadata holder for Rust side
pub struct PluginMetadata {
    pub info: NyashPluginInfo,
    pub state: PluginState,
    
    // Keep CStrings alive for C interop
    type_name_holder: Option<CString>,
    method_holders: Vec<(NyashMethodInfo, CString)>,
}

impl PluginMetadata {
    /// Create metadata from plugin info
    pub fn new(
        type_id: u32,
        type_name: &str,
        methods: Vec<(u32, &str, u32)>, // (id, name, hash)
    ) -> BidResult<Self> {
        // Create type name
        let type_name_holder = CString::new(type_name)
            .map_err(|_| BidError::InvalidUtf8)?;
        
        // Create method infos
        let mut method_holders = Vec::new();
        for (id, name, hash) in methods {
            let (info, holder) = NyashMethodInfo::new(id, name, hash)?;
            method_holders.push((info, holder));
        }
        
        // Build plugin info
        let info = NyashPluginInfo {
            type_id,
            type_name: type_name_holder.as_ptr(),
            method_count: method_holders.len() as u32,
            methods: if method_holders.is_empty() {
                std::ptr::null()
            } else {
                method_holders.as_ptr() as *const NyashMethodInfo
            },
        };
        
        Ok(Self {
            info,
            state: PluginState::Loaded,
            type_name_holder: Some(type_name_holder),
            method_holders,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_metadata_creation() {
        let methods = vec![
            (1, "open", 0x12345678),
            (2, "read", 0x87654321),
            (3, "write", 0x11223344),
            (4, "close", 0xABCDEF00),
        ];
        
        let metadata = PluginMetadata::new(6, "FileBox", methods).unwrap();
        
        assert_eq!(metadata.info.type_id, 6);
        assert_eq!(metadata.info.method_count, 4);
        assert_eq!(metadata.state, PluginState::Loaded);
        
        unsafe {
            assert_eq!(metadata.info.name().unwrap(), "FileBox");
            
            let methods = metadata.info.methods_slice().unwrap();
            assert_eq!(methods.len(), 4);
            assert_eq!(methods[0].method_id, 1);
            assert_eq!(methods[0].name().unwrap(), "open");
        }
    }
    
    #[test]
    fn test_host_vtable() {
        let vtable = NyashHostVtable::empty();
        assert!(!vtable.is_complete());
        
        // In real usage, would set actual function pointers
        let vtable = NyashHostVtable {
            alloc: Some(dummy_alloc),
            free: Some(dummy_free),
            wake: None,
            log: Some(dummy_log),
        };
        assert!(vtable.is_complete());
    }
    
    extern "C" fn dummy_alloc(_size: usize) -> *mut c_void {
        std::ptr::null_mut()
    }
    
    extern "C" fn dummy_free(_ptr: *mut c_void) {}
    
    extern "C" fn dummy_log(_msg: *const c_char) {}
}