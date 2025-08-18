use crate::bid::{BidError, BidResult, NyashHostVtable, NyashPluginInfo, PluginHandle, PLUGIN_ABI_SYMBOL, PLUGIN_INIT_SYMBOL, PLUGIN_INVOKE_SYMBOL, PLUGIN_SHUTDOWN_SYMBOL};
use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::path::{Path, PathBuf};

/// Loaded plugin with FFI entry points and metadata
pub struct LoadedPlugin {
    pub library: Library,
    pub handle: PluginHandle,
    pub type_id: u32,
    pub plugin_info: NyashPluginInfo, // プラグイン情報を保存
}

impl LoadedPlugin {
    /// Load a plugin dynamic library from file path and initialize it
    pub fn load_from_file(path: &Path) -> BidResult<Self> {
        // Load library
        let library = unsafe { Library::new(path) }
            .map_err(|_| BidError::PluginError)?;

        // Resolve symbols
        unsafe {
            let abi: Symbol<unsafe extern "C" fn() -> u32> = library
                .get(PLUGIN_ABI_SYMBOL.as_bytes())
                .map_err(|_| BidError::PluginError)?;
            let init: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = library
                .get(PLUGIN_INIT_SYMBOL.as_bytes())
                .map_err(|_| BidError::PluginError)?;
            let invoke: Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32> = library
                .get(PLUGIN_INVOKE_SYMBOL.as_bytes())
                .map_err(|_| BidError::PluginError)?;
            let shutdown: Symbol<unsafe extern "C" fn()> = library
                .get(PLUGIN_SHUTDOWN_SYMBOL.as_bytes())
                .map_err(|_| BidError::PluginError)?;

            let handle = PluginHandle {
                abi: *abi,
                init: *init,
                invoke: *invoke,
                shutdown: *shutdown,
            };

            // ABI check
            handle.check_abi()?;

            // Initialize plugin
            let host = default_host_vtable();
            let mut info = NyashPluginInfo::empty();
            handle.initialize(&host, &mut info)?;
            let type_id = info.type_id;

            Ok(Self { library, handle, type_id, plugin_info: info })
        }
    }

    /// Get the plugin's Box type name
    pub fn get_type_name(&self) -> BidResult<&str> {
        unsafe { self.plugin_info.name() }
    }

    /// Get all available methods for this plugin
    pub fn get_methods(&self) -> BidResult<Vec<(u32, String, u32)>> {
        let mut methods = Vec::new();
        
        unsafe {
            let methods_slice = self.plugin_info.methods_slice()?;
            for method_info in methods_slice {
                let method_name = method_info.name()?.to_string();
                methods.push((
                    method_info.method_id,
                    method_name,
                    method_info.signature_hash,
                ));
            }
        }
        
        Ok(methods)
    }

    /// Find a method by name and return its info
    pub fn find_method(&self, method_name: &str) -> BidResult<Option<(u32, u32)>> {
        unsafe {
            let methods_slice = self.plugin_info.methods_slice()?;
            for method_info in methods_slice {
                if method_info.name()? == method_name {
                    return Ok(Some((method_info.method_id, method_info.signature_hash)));
                }
            }
        }
        Ok(None)
    }
}

// Static host vtable to ensure lifetime
static HOST_VTABLE_STORAGE: std::sync::LazyLock<NyashHostVtable> = std::sync::LazyLock::new(|| {
    unsafe extern "C" fn host_alloc(size: usize) -> *mut u8 {
        let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
        std::alloc::alloc(layout)
    }
    unsafe extern "C" fn host_free(_ptr: *mut u8) {
        // In this prototype we cannot deallocate without size. No-op.
    }
    unsafe extern "C" fn host_wake(_id: u64) {}
    unsafe extern "C" fn host_log(_level: i32, _msg: *const i8) {
        // Debug output for plugin logs
        if !_msg.is_null() {
            let msg = unsafe { std::ffi::CStr::from_ptr(_msg) };
            if let Ok(s) = msg.to_str() {
                eprintln!("🔌 Plugin log [{}]: {}", _level, s);
            }
        }
    }

    NyashHostVtable { alloc: host_alloc, free: host_free, wake: host_wake, log: host_log }
});

/// Build a minimal host vtable for plugins
fn default_host_vtable() -> &'static NyashHostVtable {
    &*HOST_VTABLE_STORAGE
}

/// Helper: find plugin file path by name and candidate directories
pub fn resolve_plugin_path(plugin_name: &str, candidates: &[PathBuf]) -> Option<PathBuf> {
    // Expected filenames by platform (Linux only for now)
    let file = format!("lib{}{}.so", plugin_name.replace('-', "_"), "");
    for dir in candidates {
        let candidate = dir.join(&file);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
