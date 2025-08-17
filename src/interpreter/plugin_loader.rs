//! Dynamic Plugin Loader for Nyash
//! 
//! Phase 9.75f: 動的ライブラリによるビルトインBox実装

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::os::raw::c_int;
use std::sync::{Arc, RwLock};

#[cfg(feature = "dynamic-file")]
use libloading::{Library, Symbol};

use crate::interpreter::RuntimeError;
use crate::box_trait::{NyashBox, StringBox, BoolBox, VoidBox, BoxCore, BoxBase};

lazy_static::lazy_static! {
    /// グローバルプラグインキャッシュ
    static ref PLUGIN_CACHE: RwLock<HashMap<String, LoadedPlugin>> = 
        RwLock::new(HashMap::new());
}

/// ロード済みプラグイン情報
#[cfg(feature = "dynamic-file")]
struct LoadedPlugin {
    library: Library,
    info: PluginInfo,
}

/// プラグイン情報
#[derive(Clone)]
struct PluginInfo {
    name: String,
    version: u32,
    api_version: u32,
}

/// FileBoxハンドルの参照カウント管理用構造体
#[derive(Debug)]
struct FileBoxHandle {
    ptr: *mut c_void,
}

impl Drop for FileBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("file") {
                    unsafe {
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_file_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for FileBoxHandle {}
unsafe impl Sync for FileBoxHandle {}

/// FileBoxプロキシ - 動的ライブラリのFileBoxをラップ
#[derive(Debug)]
pub struct FileBoxProxy {
    handle: Arc<FileBoxHandle>,
    path: String,
    base: BoxBase,
}

// FileBoxProxyは手動でSendとSyncを実装
unsafe impl Send for FileBoxProxy {}
unsafe impl Sync for FileBoxProxy {}

impl FileBoxProxy {
    /// 新しいFileBoxProxyを作成
    pub fn new(handle: *mut c_void, path: String) -> Self {
        FileBoxProxy {
            handle: Arc::new(FileBoxHandle { ptr: handle }),
            path,
            base: BoxBase::new(),
        }
    }
    
    /// ファイル読み取り
    pub fn read(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("file") {
                unsafe {
                    let read_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_char> = 
                        plugin.library.get(b"nyash_file_read\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_file_read: {}", e)
                            }
                        })?;
                    
                    let result_ptr = read_fn(self.handle.ptr);
                    if result_ptr.is_null() {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Failed to read file".to_string()
                        });
                    }
                    
                    let content = CStr::from_ptr(result_ptr).to_string_lossy().into_owned();
                    
                    // 文字列を解放
                    let free_fn: Symbol<unsafe extern "C" fn(*mut c_char)> = 
                        plugin.library.get(b"nyash_string_free\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_string_free: {}", e)
                            }
                        })?;
                    free_fn(result_ptr);
                    
                    Ok(Box::new(StringBox::new(content)))
                }
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "File plugin not loaded".to_string()
                })
            }
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(RuntimeError::InvalidOperation {
                message: "Dynamic file support not enabled".to_string()
            })
        }
    }
    
    /// ファイル書き込み
    pub fn write(&self, content: Box<dyn NyashBox>) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("file") {
                let content_str = content.to_string_box().value;
                let c_content = CString::new(content_str).map_err(|_| {
                    RuntimeError::InvalidOperation {
                        message: "Invalid content string".to_string()
                    }
                })?;
                
                unsafe {
                    let write_fn: Symbol<unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int> = 
                        plugin.library.get(b"nyash_file_write\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_file_write: {}", e)
                            }
                        })?;
                    
                    let result = write_fn(self.handle.ptr, c_content.as_ptr());
                    if result == 0 {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Failed to write file".to_string()
                        });
                    }
                    
                    Ok(Box::new(StringBox::new("ok")))
                }
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "File plugin not loaded".to_string()
                })
            }
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(RuntimeError::InvalidOperation {
                message: "Dynamic file support not enabled".to_string()
            })
        }
    }
    
    /// ファイル存在確認
    pub fn exists(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Ok(Box::new(BoolBox::new(std::path::Path::new(&self.path).exists())))
    }
}

// FileBoxProxyのDropは不要 - FileBoxHandleが自動的に管理

impl BoxCore for FileBoxProxy {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FileBox({})", self.path)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for FileBoxProxy {
    fn type_name(&self) -> &'static str {
        "FileBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // FileBoxProxyの複製：新しいファイルハンドルを作成
        match PluginLoader::create_file_box(&self.path) {
            Ok(new_box) => new_box,
            Err(_) => {
                // エラー時は同じハンドルを共有（フォールバック）
                Box::new(FileBoxProxy {
                    handle: Arc::clone(&self.handle),
                    path: self.path.clone(),
                    base: BoxBase::new(),
                })
            }
        }
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        // 状態共有：自分自身の複製を返す
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("FileBox({})", self.path))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBoxProxy>() {
            BoolBox::new(self.path == other_file.path)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for FileBoxProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// プラグインローダー公開API
pub struct PluginLoader;

impl PluginLoader {
    /// FileBoxプラグインをロード
    #[cfg(feature = "dynamic-file")]
    pub fn load_file_plugin() -> Result<(), RuntimeError> {
        let mut cache = PLUGIN_CACHE.write().unwrap();
        
        if cache.contains_key("file") {
            return Ok(()); // 既にロード済み
        }
        
        // プラグインパスを決定（複数の場所を試す）
        let lib_name = if cfg!(target_os = "windows") {
            "nyash_file.dll"
        } else if cfg!(target_os = "macos") {
            "libnyash_file.dylib"
        } else {
            "libnyash_file.so"
        };
        
        // 複数のパスを試す
        let possible_paths = vec![
            format!("./target/release/{}", lib_name),
            format!("./target/debug/{}", lib_name),
            format!("./plugins/{}", lib_name),
            format!("./{}", lib_name),
        ];
        
        let mut lib_path = None;
        for path in &possible_paths {
            if std::path::Path::new(path).exists() {
                lib_path = Some(path.clone());
                break;
            }
        }
        
        let lib_path = lib_path.ok_or_else(|| {
            RuntimeError::InvalidOperation {
                message: format!("Failed to find file plugin library. Searched paths: {:?}", possible_paths)
            }
        })?;
        
        // ライブラリをロード
        unsafe {
            let library = Library::new(&lib_path).map_err(|e| {
                RuntimeError::InvalidOperation {
                    message: format!("Failed to load file plugin: {}", e)
                }
            })?;
            
            // プラグイン初期化
            let init_fn: Symbol<unsafe extern "C" fn() -> *const c_void> = 
                library.get(b"nyash_plugin_init\0").map_err(|e| {
                    RuntimeError::InvalidOperation {
                        message: format!("Failed to get plugin init: {}", e)
                    }
                })?;
            
            let plugin_info_ptr = init_fn();
            if plugin_info_ptr.is_null() {
                return Err(RuntimeError::InvalidOperation {
                    message: "Plugin initialization failed".to_string()
                });
            }
            
            // マジックナンバーとバージョンチェック（簡略化）
            let info = PluginInfo {
                name: "file".to_string(),
                version: 1,
                api_version: 1,
            };
            
            cache.insert("file".to_string(), LoadedPlugin {
                library,
                info,
            });
        }
        
        Ok(())
    }
    
    /// FileBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_file_box(path: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // プラグインがロードされているか確認
        Self::load_file_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("file") {
            let c_path = CString::new(path).map_err(|_| {
                RuntimeError::InvalidOperation {
                    message: "Invalid path string".to_string()
                }
            })?;
            
            unsafe {
                let open_fn: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = 
                    plugin.library.get(b"nyash_file_open\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_file_open: {}", e)
                        }
                    })?;
                
                let handle = open_fn(c_path.as_ptr());
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Failed to open file: {}", path)
                    });
                }
                
                Ok(Box::new(FileBoxProxy::new(handle, path.to_string())))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "File plugin not loaded".to_string()
            })
        }
    }
    
    /// FileBoxが存在するかチェック（静的メソッド）
    #[cfg(feature = "dynamic-file")]
    pub fn file_exists(path: &str) -> Result<bool, RuntimeError> {
        // プラグインがロードされているか確認
        Self::load_file_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("file") {
            let c_path = CString::new(path).map_err(|_| {
                RuntimeError::InvalidOperation {
                    message: "Invalid path string".to_string()
                }
            })?;
            
            unsafe {
                let exists_fn: Symbol<unsafe extern "C" fn(*const c_char) -> c_int> = 
                    plugin.library.get(b"nyash_file_exists\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_file_exists: {}", e)
                        }
                    })?;
                
                Ok(exists_fn(c_path.as_ptr()) != 0)
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "File plugin not loaded".to_string()
            })
        }
    }
}