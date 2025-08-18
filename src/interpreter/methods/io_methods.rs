/*!
 * I/O Operations Box Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains method implementations for I/O and error handling operations:
 * - FileBox (execute_file_method) - File I/O operations
 * - ResultBox (execute_result_method) - Error handling and result operations
 */

use super::super::*;
use crate::box_trait::{ResultBox, StringBox, NyashBox};
use crate::boxes::FileBox;
use crate::bid::plugin_box::PluginFileBox;

impl NyashInterpreter {
    /// FileBoxのメソッド呼び出しを実行
    /// Handles file I/O operations including read, write, exists, delete, and copy
    pub(in crate::interpreter) fn execute_file_method(&mut self, file_box: &FileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "read" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.read())
            }
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let content = self.execute_expression(&arguments[0])?;
                Ok(file_box.write(content))
            }
            "exists" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("exists() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.exists())
            }
            "delete" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.delete())
            }
            "copy" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("copy() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let dest_value = self.execute_expression(&arguments[0])?;
                if let Some(dest_str) = dest_value.as_any().downcast_ref::<StringBox>() {
                    Ok(file_box.copy(&dest_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "copy() requires string destination path".to_string(),
                    })
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for FileBox", method),
            })
        }
    }

    /// ResultBoxのメソッド呼び出しを実行
    /// Handles result/error checking operations for error handling patterns
    pub(in crate::interpreter) fn execute_result_method(&mut self, result_box: &ResultBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "isOk" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isOk() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.is_ok())
            }
            "getValue" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getValue() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_value())
            }
            "getError" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getError() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_error())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for ResultBox", method),
            })
        }
    }

    /// 汎用プラグインメソッド呼び出し実行 (BID-FFI system)
    /// Handles generic plugin method calls via dynamic method discovery
    pub(in crate::interpreter) fn execute_plugin_method_generic(&mut self, plugin_box: &PluginFileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        eprintln!("🔍 execute_plugin_method_generic: method='{}', args_count={}", method, arguments.len());
        
        // まず利用可能なメソッドを確認
        match plugin_box.get_available_methods() {
            Ok(methods) => {
                eprintln!("🔍 Available plugin methods:");
                for (id, name, sig) in &methods {
                    eprintln!("🔍   - {} [ID: {}, Sig: 0x{:08X}]", name, id, sig);
                }
            }
            Err(e) => eprintln!("⚠️ Failed to get plugin methods: {:?}", e),
        }
        
        // 引数をTLVエンコード（メソッド名も渡す）
        let encoded_args = self.encode_arguments_to_tlv(arguments, method)?;
        eprintln!("🔍 Encoded args length: {} bytes", encoded_args.len());
        
        // プラグインのメソッドを動的呼び出し
        match plugin_box.call_method(method, &encoded_args) {
            Ok(response_bytes) => {
                eprintln!("🔍 Plugin method '{}' succeeded, response length: {} bytes", method, response_bytes.len());
                // レスポンスをデコードしてNyashBoxに変換
                self.decode_tlv_to_nyash_box(&response_bytes, method)
            }
            Err(e) => {
                eprintln!("🔍 Plugin method '{}' failed with error: {:?}", method, e);
                Err(RuntimeError::InvalidOperation {
                    message: format!("Plugin method '{}' failed: {:?}", method, e),
                })
            }
        }
    }

    /// 引数をTLVエンコード（メソッドに応じて特殊処理）
    fn encode_arguments_to_tlv(&mut self, arguments: &[ASTNode], method_name: &str) -> Result<Vec<u8>, RuntimeError> {
        use crate::bid::tlv::TlvEncoder;
        
        let mut encoder = TlvEncoder::new();
        
        // 特殊ケース: readメソッドは引数がなくても、サイズ引数が必要
        if method_name == "read" && arguments.is_empty() {
            // デフォルトで8192バイト読み取り
            encoder.encode_i32(8192)
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("TLV i32 encoding failed: {:?}", e),
                })?;
        } else {
            // 通常の引数エンコード
            for arg in arguments {
                let value = self.execute_expression(arg)?;
                
                // 型に応じてエンコード
                if let Some(str_box) = value.as_any().downcast_ref::<StringBox>() {
                    // 🔍 writeメソッドなど、文字列データはBytesとしてエンコード
                    // プラグインは通常、文字列データをBytesタグ（7）で期待する
                    encoder.encode_bytes(str_box.value.as_bytes())
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV bytes encoding failed: {:?}", e),
                        })?;
                } else if let Some(int_box) = value.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    encoder.encode_i32(int_box.value as i32)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV integer encoding failed: {:?}", e),
                        })?;
                } else if let Some(bool_box) = value.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                    encoder.encode_bool(bool_box.value)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV bool encoding failed: {:?}", e),
                        })?;
                } else {
                    // デフォルト: バイトデータとして扱う
                    let str_val = value.to_string_box().value;
                    encoder.encode_bytes(str_val.as_bytes())
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV default bytes encoding failed: {:?}", e),
                        })?;
                }
            }
        }
        
        Ok(encoder.finish())
    }
    
    /// TLVレスポンスをNyashBoxに変換
    fn decode_tlv_to_nyash_box(&self, response_bytes: &[u8], method_name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        use crate::bid::tlv::TlvDecoder;
        use crate::bid::types::BidTag;
        
        if response_bytes.is_empty() {
            return Ok(Box::new(StringBox::new("".to_string())));
        }
        
        let mut decoder = TlvDecoder::new(response_bytes)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("TLV decoder creation failed: {:?}", e),
            })?;
        
        if let Some((tag, payload)) = decoder.decode_next()
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("TLV decoding failed: {:?}", e),
            })? {
            
            match tag {
                BidTag::String => {
                    let text = String::from_utf8_lossy(payload).to_string();
                    Ok(Box::new(StringBox::new(text)))
                }
                BidTag::Bytes => {
                    // ファイル読み取り等のバイトデータは文字列として返す
                    let text = String::from_utf8_lossy(payload).to_string();
                    Ok(Box::new(StringBox::new(text)))
                }
                BidTag::I32 => {
                    let value = TlvDecoder::decode_i32(payload)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV i32 decoding failed: {:?}", e),
                        })?;
                    Ok(Box::new(crate::box_trait::IntegerBox::new(value as i64)))
                }
                BidTag::Bool => {
                    let value = TlvDecoder::decode_bool(payload)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV bool decoding failed: {:?}", e),
                        })?;
                    Ok(Box::new(crate::box_trait::BoolBox::new(value)))
                }
                BidTag::Void => {
                    Ok(Box::new(StringBox::new("OK".to_string())))
                }
                _ => {
                    Ok(Box::new(StringBox::new(format!("Unknown TLV tag: {:?}", tag))))
                }
            }
        } else {
            Ok(Box::new(StringBox::new("".to_string())))
        }
    }

    /// PluginFileBoxのメソッド呼び出しを実行 (BID-FFI system) - LEGACY HARDCODED VERSION
    /// Handles plugin-backed file I/O operations via FFI interface
    /// 🚨 DEPRECATED: This method has hardcoded method names and violates BID-FFI principles
    /// Use execute_plugin_method_generic instead for true dynamic method calling
    pub(in crate::interpreter) fn execute_plugin_file_method(&mut self, plugin_file_box: &PluginFileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 🎯 新しい汎用システムにリダイレクト
        self.execute_plugin_method_generic(plugin_file_box, method, arguments)
    }
}