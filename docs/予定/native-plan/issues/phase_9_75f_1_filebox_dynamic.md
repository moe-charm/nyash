# Phase 9.75f-1: FileBox動的ライブラリ化（第一段階）

## 🎯 目的
- FileBoxを最初の動的ライブラリ化対象として実装
- ビルド時間短縮効果の実証（目標: 15秒短縮）
- 動的ライブラリアーキテクチャの検証

## 📋 実装計画

### Step 1: プロジェクト構造準備
```toml
# Cargo.toml (workspace)
[workspace]
members = [
    "nyash-core",
    "plugins/nyash-file",
]

# nyash-core/Cargo.toml
[dependencies]
libloading = "0.8"

[features]
default = ["static-boxes"]
static-boxes = []
dynamic-file = []
```

### Step 2: FileBox切り離し
```rust
// plugins/nyash-file/src/lib.rs
#[repr(C)]
pub struct FileBoxPlugin {
    magic: u32,  // 0x4E594153 ('NYAS')
    version: u32,
    api_version: u32,
}

#[no_mangle]
extern "C" fn nyash_plugin_init() -> *const FileBoxPlugin {
    Box::into_raw(Box::new(FileBoxPlugin {
        magic: 0x4E594153,
        version: 1,
        api_version: 1,
    }))
}

// FileBox methods as C ABI functions
#[no_mangle]
extern "C" fn nyash_file_open(path: *const c_char) -> *mut c_void {
    // FileBox::open implementation
}

#[no_mangle]
extern "C" fn nyash_file_read(handle: *mut c_void) -> *mut c_char {
    // FileBox::read implementation
}

#[no_mangle]
extern "C" fn nyash_file_write(handle: *mut c_void, content: *const c_char) -> i32 {
    // FileBox::write implementation
}

#[no_mangle]
extern "C" fn nyash_file_free(handle: *mut c_void) {
    // Cleanup
}
```

### Step 3: インタープリター統合
```rust
// src/interpreter/plugin_loader.rs
use libloading::{Library, Symbol};
use std::sync::RwLock;
use std::collections::HashMap;

lazy_static! {
    static ref PLUGIN_CACHE: RwLock<HashMap<String, Library>> = 
        RwLock::new(HashMap::new());
}

impl NyashInterpreter {
    fn load_file_plugin(&mut self) -> Result<(), RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let mut cache = PLUGIN_CACHE.write().unwrap();
            if !cache.contains_key("file") {
                let lib_path = if cfg!(windows) {
                    "./plugins/nyash_file.dll"
                } else if cfg!(target_os = "macos") {
                    "./plugins/libnyash_file.dylib"
                } else {
                    "./plugins/libnyash_file.so"
                };
                
                unsafe {
                    let lib = Library::new(lib_path)?;
                    cache.insert("file".to_string(), lib);
                }
            }
        }
        Ok(())
    }
}
```

### Step 4: execute_new修正
```rust
// src/interpreter/objects.rs
"FileBox" => {
    #[cfg(feature = "static-boxes")]
    {
        // 既存の静的実装
        let file_box = FileBox::open(&path)?;
        Ok(Box::new(file_box))
    }
    
    #[cfg(feature = "dynamic-file")]
    {
        // 動的ライブラリ経由
        self.load_file_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        let lib = cache.get("file").unwrap();
        
        unsafe {
            let open_fn: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = 
                lib.get(b"nyash_file_open")?;
            let handle = open_fn(CString::new(path)?.as_ptr());
            
            // FileBoxProxyでラップ
            Ok(Box::new(FileBoxProxy { handle }))
        }
    }
}
```

### Step 5: FileBoxProxy実装
```rust
// src/interpreter/proxy_boxes.rs
struct FileBoxProxy {
    handle: *mut c_void,
}

impl NyashBox for FileBoxProxy {
    fn type_name(&self) -> &'static str {
        "FileBox"
    }
    
    // メソッド呼び出しは動的ライブラリへ委譲
}

impl Drop for FileBoxProxy {
    fn drop(&mut self) {
        // nyash_file_free呼び出し
    }
}
```

## 🎯 成功条件
1. ✅ `new FileBox(path)` が動的ライブラリ経由で動作
2. ✅ FileBoxのメソッド（read, write, exists）が正常動作
3. ✅ ビルド時間が15秒以上短縮
4. ✅ 静的/動的をfeature flagで切り替え可能
5. ✅ メモリリークなし（valgrindで確認）

## ⚠️ 注意事項
- Windows/Mac/Linuxでのパス解決
- エラーハンドリング（プラグイン読み込み失敗時）
- ABI互換性（C ABIで安定化）

## 📊 測定項目
- ビルド時間（クリーンビルド）
- 起動時間（プラグインロード込み）
- FileBox操作のベンチマーク
- バイナリサイズ削減量