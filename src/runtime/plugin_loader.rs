//! プラグイン動的ローダー - libloadingによるFFI実行
//! 
//! PluginBoxプロキシからFFI経由でプラグインメソッドを呼び出す

use crate::bid::{BidHandle, BidError, TlvEncoder, TlvDecoder};
use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::runtime::plugin_box::PluginBox;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "dynamic-file")]
use libloading::{Library, Symbol};

/// プラグインライブラリハンドル
pub struct PluginLibrary {
    #[cfg(feature = "dynamic-file")]
    library: Library,
    
    #[cfg(not(feature = "dynamic-file"))]
    _placeholder: (),
}

/// プラグインローダー - 動的ライブラリ管理
pub struct PluginLoader {
    /// プラグイン名 → ライブラリのマッピング
    libraries: RwLock<HashMap<String, Arc<PluginLibrary>>>,
}

impl PluginLoader {
    /// 新しいプラグインローダーを作成
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
        }
    }
    
    /// プラグインライブラリをロード
    pub fn load_plugin(&self, plugin_name: &str, library_path: &str) -> Result<(), String> {
        #[cfg(feature = "dynamic-file")]
        {
            let library = unsafe { 
                Library::new(library_path)
                    .map_err(|e| format!("Failed to load plugin {}: {}", plugin_name, e))?
            };
            
            let plugin_lib = Arc::new(PluginLibrary { library });
            let mut libraries = self.libraries.write().unwrap();
            libraries.insert(plugin_name.to_string(), plugin_lib);
            
            Ok(())
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(format!("Dynamic library loading disabled. Cannot load plugin: {}", plugin_name))
        }
    }
    
    /// プラグインメソッドを呼び出し
    pub fn invoke_plugin_method(
        &self,
        plugin_name: &str,
        handle: BidHandle,
        method_name: &str,
        args: &[Box<dyn NyashBox>]
    ) -> Result<Box<dyn NyashBox>, String> {
        #[cfg(feature = "dynamic-file")]
        {
            let libraries = self.libraries.read().unwrap();
            let library = libraries.get(plugin_name)
                .ok_or_else(|| format!("Plugin not loaded: {}", plugin_name))?;
            
            // プラグインメソッド呼び出し
            self.call_plugin_method(&library.library, handle, method_name, args)
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(format!("Dynamic library loading disabled. Cannot invoke: {}.{}", plugin_name, method_name))
        }
    }
    
    #[cfg(feature = "dynamic-file")]
    fn call_plugin_method(
        &self,
        library: &Library,
        handle: BidHandle,
        method_name: &str,
        args: &[Box<dyn NyashBox>]
    ) -> Result<Box<dyn NyashBox>, String> {
        // BID-1 TLV引数エンコード
        let mut encoder = TlvEncoder::new();
        for arg in args {
            // TODO: NyashBox to TLV encoding
            encoder.encode_string(&arg.to_string_box().value)
                .map_err(|e| format!("Failed to encode argument: {:?}", e))?;
        }
        let args_data = encoder.finish();
        
        // プラグイン関数呼び出し
        let function_name = format!("nyash_plugin_invoke");
        let invoke_fn: Symbol<unsafe extern "C" fn(
            u32, u32, u32,          // type_id, method_id, instance_id
            *const u8, usize,       // args, args_len
            *mut u8, *mut usize     // result, result_len
        ) -> i32> = unsafe {
            library.get(function_name.as_bytes())
                .map_err(|e| format!("Function {} not found: {}", function_name, e))?
        };
        
        // メソッドIDを決定（簡易版）
        let method_id = match method_name {
            "open" => 1,
            "read" => 2,
            "write" => 3,
            "close" => 4,
            _ => return Err(format!("Unknown method: {}", method_name)),
        };
        
        // 結果バッファ準備
        let mut result_size = 0usize;
        
        // 1回目: サイズ取得
        let status = unsafe {
            invoke_fn(
                handle.type_id,
                method_id,
                handle.instance_id,
                args_data.as_ptr(),
                args_data.len(),
                std::ptr::null_mut(),
                &mut result_size as *mut usize
            )
        };
        
        if status != 0 {
            return Err(format!("Plugin method failed: status {}", status));
        }
        
        // 2回目: 結果取得
        let mut result_buffer = vec![0u8; result_size];
        let status = unsafe {
            invoke_fn(
                handle.type_id,
                method_id,
                handle.instance_id,
                args_data.as_ptr(),
                args_data.len(),
                result_buffer.as_mut_ptr(),
                &mut result_size as *mut usize
            )
        };
        
        if status != 0 {
            return Err(format!("Plugin method failed: status {}", status));
        }
        
        // BID-1 TLV結果デコード
        let decoder = TlvDecoder::new(&result_buffer)
            .map_err(|e| format!("Failed to decode result: {:?}", e))?;
        // TODO: TLV to NyashBox decoding
        Ok(Box::new(crate::box_trait::StringBox::new("Plugin result")))
    }
}

/// グローバルプラグインローダー
use once_cell::sync::Lazy;

static GLOBAL_LOADER: Lazy<Arc<PluginLoader>> = 
    Lazy::new(|| Arc::new(PluginLoader::new()));

/// グローバルプラグインローダーを取得
pub fn get_global_loader() -> Arc<PluginLoader> {
    GLOBAL_LOADER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        // 基本的な作成テスト
        assert!(loader.libraries.read().unwrap().is_empty());
    }
    
    #[cfg(feature = "dynamic-file")]
    #[test]
    fn test_plugin_loading_error() {
        let loader = PluginLoader::new();
        let result = loader.load_plugin("test", "/nonexistent/path.so");
        assert!(result.is_err());
    }
}