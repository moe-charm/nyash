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
use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase, IntegerBox};
use crate::boxes::FloatBox;

lazy_static::lazy_static! {
    /// グローバルプラグインキャッシュ
    pub(crate) static ref PLUGIN_CACHE: RwLock<HashMap<String, LoadedPlugin>> = 
        RwLock::new(HashMap::new());
}

/// ロード済みプラグイン情報
#[cfg(feature = "dynamic-file")]
pub(crate) struct LoadedPlugin {
    pub(crate) library: Library,
    pub(crate) info: PluginInfo,
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

/// MathBoxハンドル
#[derive(Debug)]
struct MathBoxHandle {
    ptr: *mut c_void,
}

impl Drop for MathBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_math_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for MathBoxHandle {}
unsafe impl Sync for MathBoxHandle {}

/// RandomBoxハンドル
#[derive(Debug)]
struct RandomBoxHandle {
    ptr: *mut c_void,
}

impl Drop for RandomBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_random_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for RandomBoxHandle {}
unsafe impl Sync for RandomBoxHandle {}

/// TimeBoxハンドル
#[derive(Debug)]
struct TimeBoxHandle {
    ptr: *mut c_void,
}

impl Drop for TimeBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_time_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for TimeBoxHandle {}
unsafe impl Sync for TimeBoxHandle {}

/// DateTimeBoxハンドル
#[derive(Debug)]
pub(crate) struct DateTimeBoxHandle {
    pub(crate) ptr: *mut c_void,
}

impl Drop for DateTimeBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_datetime_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for DateTimeBoxHandle {}
unsafe impl Sync for DateTimeBoxHandle {}

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

// ================== MathBoxProxy ==================

/// MathBoxプロキシ - 動的ライブラリのMathBoxをラップ
#[derive(Debug)]
pub struct MathBoxProxy {
    handle: Arc<MathBoxHandle>,
    base: BoxBase,
}

unsafe impl Send for MathBoxProxy {}
unsafe impl Sync for MathBoxProxy {}

impl MathBoxProxy {
    pub fn new(handle: *mut c_void) -> Self {
        MathBoxProxy {
            handle: Arc::new(MathBoxHandle { ptr: handle }),
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for MathBoxProxy {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None // プロキシ型は継承しない
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MathBox")
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for MathBoxProxy {
    fn type_name(&self) -> &'static str {
        "MathBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        match PluginLoader::create_math_box() {
            Ok(new_box) => new_box,
            Err(_) => Box::new(MathBoxProxy {
                handle: Arc::clone(&self.handle),
                base: BoxBase::new(),
            })
        }
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("MathBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(_) = other.as_any().downcast_ref::<MathBoxProxy>() {
            BoolBox::new(true)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for MathBoxProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// ================== RandomBoxProxy ==================

/// RandomBoxプロキシ - 動的ライブラリのRandomBoxをラップ
#[derive(Debug)]
pub struct RandomBoxProxy {
    handle: Arc<RandomBoxHandle>,
    base: BoxBase,
}

unsafe impl Send for RandomBoxProxy {}
unsafe impl Sync for RandomBoxProxy {}

impl RandomBoxProxy {
    pub fn new(handle: *mut c_void) -> Self {
        RandomBoxProxy {
            handle: Arc::new(RandomBoxHandle { ptr: handle }),
            base: BoxBase::new(),
        }
    }
    
    pub fn next(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let next_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> f64> = 
                        plugin.library.get(b"nyash_random_next\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_random_next: {}", e)
                            }
                        })?;
                    
                    let value = next_fn(self.handle.ptr);
                    Ok(Box::new(FloatBox::new(value)))
                }
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "Math plugin not loaded".to_string()
                })
            }
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(RuntimeError::InvalidOperation {
                message: "Dynamic loading not enabled".to_string()
            })
        }
    }
    
    pub fn range(&self, min: f64, max: f64) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let range_fn: Symbol<unsafe extern "C" fn(*mut c_void, f64, f64) -> f64> = 
                        plugin.library.get(b"nyash_random_range\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_random_range: {}", e)
                            }
                        })?;
                    
                    let value = range_fn(self.handle.ptr, min, max);
                    Ok(Box::new(FloatBox::new(value)))
                }
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "Math plugin not loaded".to_string()
                })
            }
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(RuntimeError::InvalidOperation {
                message: "Dynamic loading not enabled".to_string()
            })
        }
    }
    
    pub fn int(&self, min: i64, max: i64) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let int_fn: Symbol<unsafe extern "C" fn(*mut c_void, i64, i64) -> i64> = 
                        plugin.library.get(b"nyash_random_int\0").map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Failed to get nyash_random_int: {}", e)
                            }
                        })?;
                    
                    let value = int_fn(self.handle.ptr, min, max);
                    Ok(Box::new(IntegerBox::new(value)))
                }
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "Math plugin not loaded".to_string()
                })
            }
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(RuntimeError::InvalidOperation {
                message: "Dynamic loading not enabled".to_string()
            })
        }
    }
}

impl BoxCore for RandomBoxProxy {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None // プロキシ型は継承しない
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomBox")
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for RandomBoxProxy {
    fn type_name(&self) -> &'static str {
        "RandomBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        match PluginLoader::create_random_box() {
            Ok(new_box) => new_box,
            Err(_) => Box::new(RandomBoxProxy {
                handle: Arc::clone(&self.handle),
                base: BoxBase::new(),
            })
        }
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("RandomBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(_) = other.as_any().downcast_ref::<RandomBoxProxy>() {
            BoolBox::new(true)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for RandomBoxProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// ================== TimeBoxProxy ==================

/// TimeBoxプロキシ - 動的ライブラリのTimeBoxをラップ
#[derive(Debug)]
pub struct TimeBoxProxy {
    handle: Arc<TimeBoxHandle>,
    base: BoxBase,
}

unsafe impl Send for TimeBoxProxy {}
unsafe impl Sync for TimeBoxProxy {}

impl TimeBoxProxy {
    pub fn new(handle: *mut c_void) -> Self {
        TimeBoxProxy {
            handle: Arc::new(TimeBoxHandle { ptr: handle }),
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for TimeBoxProxy {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None // プロキシ型は継承しない
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeBox")
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for TimeBoxProxy {
    fn type_name(&self) -> &'static str {
        "TimeBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        match PluginLoader::create_time_box() {
            Ok(new_box) => new_box,
            Err(_) => Box::new(TimeBoxProxy {
                handle: Arc::clone(&self.handle),
                base: BoxBase::new(),
            })
        }
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("TimeBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(_) = other.as_any().downcast_ref::<TimeBoxProxy>() {
            BoolBox::new(true)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for TimeBoxProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// ================== DateTimeBoxProxy ==================

/// DateTimeBoxプロキシ - 動的ライブラリのDateTimeBoxをラップ
#[derive(Debug)]
pub struct DateTimeBoxProxy {
    pub(crate) handle: Arc<DateTimeBoxHandle>,
    base: BoxBase,
}

unsafe impl Send for DateTimeBoxProxy {}
unsafe impl Sync for DateTimeBoxProxy {}

impl DateTimeBoxProxy {
    pub fn new(handle: *mut c_void) -> Self {
        DateTimeBoxProxy {
            handle: Arc::new(DateTimeBoxHandle { ptr: handle }),
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for DateTimeBoxProxy {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None // プロキシ型は継承しない
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    if let Ok(to_string_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_char>>(b"nyash_datetime_to_string\0") {
                        let str_ptr = to_string_fn(self.handle.ptr);
                        if !str_ptr.is_null() {
                            let c_str = CStr::from_ptr(str_ptr);
                            if let Ok(rust_str) = c_str.to_str() {
                                let result = write!(f, "{}", rust_str);
                                
                                // 文字列を解放
                                if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_char)>>(b"nyash_string_free\0") {
                                    free_fn(str_ptr);
                                }
                                
                                return result;
                            }
                        }
                    }
                }
            }
        }
        
        write!(f, "DateTimeBox")
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for DateTimeBoxProxy {
    fn type_name(&self) -> &'static str {
        "DateTimeBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // DateTimeBoxは不変なので、ハンドルを共有
        Box::new(DateTimeBoxProxy {
            handle: Arc::clone(&self.handle),
            base: BoxBase::new(),
        })
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    if let Ok(to_string_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_char>>(b"nyash_datetime_to_string\0") {
                        let str_ptr = to_string_fn(self.handle.ptr);
                        if !str_ptr.is_null() {
                            let c_str = CStr::from_ptr(str_ptr);
                            if let Ok(rust_str) = c_str.to_str() {
                                let result = StringBox::new(rust_str);
                                
                                // 文字列を解放
                                if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_char)>>(b"nyash_string_free\0") {
                                    free_fn(str_ptr);
                                }
                                
                                return result;
                            }
                        }
                    }
                }
            }
        }
        
        StringBox::new("DateTimeBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_datetime) = other.as_any().downcast_ref::<DateTimeBoxProxy>() {
            // タイムスタンプで比較
            #[cfg(feature = "dynamic-file")]
            {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(timestamp_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void) -> i64>>(b"nyash_datetime_timestamp\0") {
                            let this_ts = timestamp_fn(self.handle.ptr);
                            let other_ts = timestamp_fn(other_datetime.handle.ptr);
                            return BoolBox::new(this_ts == other_ts);
                        }
                    }
                }
            }
            BoolBox::new(false)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for DateTimeBoxProxy {
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
    
    /// Mathプラグインをロード
    #[cfg(feature = "dynamic-file")]
    pub fn load_math_plugin() -> Result<(), RuntimeError> {
        let mut cache = PLUGIN_CACHE.write().unwrap();
        
        if cache.contains_key("math") {
            return Ok(()); // 既にロード済み
        }
        
        // プラグインパスを決定（複数の場所を試す）
        let lib_name = if cfg!(target_os = "windows") {
            "nyash_math.dll"
        } else if cfg!(target_os = "macos") {
            "libnyash_math.dylib"
        } else {
            "libnyash_math.so"
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
                message: format!("Failed to find math plugin library. Searched paths: {:?}", possible_paths)
            }
        })?;
        
        // ライブラリをロード
        unsafe {
            let library = Library::new(&lib_path).map_err(|e| {
                RuntimeError::InvalidOperation {
                    message: format!("Failed to load math plugin: {}", e)
                }
            })?;
            
            // プラグイン情報（簡略化）
            let info = PluginInfo {
                name: "math".to_string(),
                version: 1,
                api_version: 1,
            };
            
            cache.insert("math".to_string(), LoadedPlugin {
                library,
                info,
            });
        }
        
        Ok(())
    }
    
    /// MathBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_math_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = 
                    plugin.library.get(b"nyash_math_create\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_math_create: {}", e)
                        }
                    })?;
                
                let handle = create_fn();
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "Failed to create MathBox".to_string()
                    });
                }
                
                Ok(Box::new(MathBoxProxy::new(handle)))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Math plugin not loaded".to_string()
            })
        }
    }
    
    /// RandomBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_random_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = 
                    plugin.library.get(b"nyash_random_create\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_random_create: {}", e)
                        }
                    })?;
                
                let handle = create_fn();
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "Failed to create RandomBox".to_string()
                    });
                }
                
                Ok(Box::new(RandomBoxProxy::new(handle)))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Math plugin not loaded".to_string()
            })
        }
    }
    
    /// TimeBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_time_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = 
                    plugin.library.get(b"nyash_time_create\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_time_create: {}", e)
                        }
                    })?;
                
                let handle = create_fn();
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "Failed to create TimeBox".to_string()
                    });
                }
                
                Ok(Box::new(TimeBoxProxy::new(handle)))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Math plugin not loaded".to_string()
            })
        }
    }
    
    /// 現在時刻のDateTimeBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_datetime_now() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let now_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = 
                    plugin.library.get(b"nyash_time_now\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_time_now: {}", e)
                        }
                    })?;
                
                let handle = now_fn();
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "Failed to create DateTimeBox".to_string()
                    });
                }
                
                Ok(Box::new(DateTimeBoxProxy::new(handle)))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Math plugin not loaded".to_string()
            })
        }
    }
    
    /// 文字列からDateTimeBoxを作成
    #[cfg(feature = "dynamic-file")]
    pub fn create_datetime_from_string(time_str: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            let c_str = CString::new(time_str).map_err(|_| {
                RuntimeError::InvalidOperation {
                    message: "Invalid time string".to_string()
                }
            })?;
            
            unsafe {
                let parse_fn: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = 
                    plugin.library.get(b"nyash_time_parse\0").map_err(|e| {
                        RuntimeError::InvalidOperation {
                            message: format!("Failed to get nyash_time_parse: {}", e)
                        }
                    })?;
                
                let handle = parse_fn(c_str.as_ptr());
                if handle.is_null() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Failed to parse time string: {}", time_str)
                    });
                }
                
                Ok(Box::new(DateTimeBoxProxy::new(handle)))
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Math plugin not loaded".to_string()
            })
        }
    }
}