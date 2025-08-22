/*!
 * I/O Operations Box Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains method implementations for I/O and error handling operations:
 * - FileBox (execute_file_method) - File I/O operations
 * - ResultBox (execute_result_method) - Error handling and result operations
 */

use super::super::*;
use crate::boxes::ResultBox;
use crate::box_trait::{StringBox, NyashBox};
use crate::boxes::FileBox;
// use crate::bid::plugin_box::PluginFileBox;  // legacy - FileBox専用

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
            "isOk" | "is_ok" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isOk() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.is_ok())
            }
            "getValue" | "get_value" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getValue() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_value())
            }
            "getError" | "get_error" => {
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

    /* legacy - PluginFileBox専用
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

    /// 引数をTLVエンコード（型情報に基づく美しい実装！）
    fn encode_arguments_to_tlv(&mut self, arguments: &[ASTNode], method_name: &str) -> Result<Vec<u8>, RuntimeError> {
        use crate::bid::tlv::TlvEncoder;
        use crate::bid::registry;
        
        let mut encoder = TlvEncoder::new();
        
        // 型情報を取得（FileBoxのみ対応、後で拡張）
        let type_info = registry::global()
            .and_then(|reg| reg.get_method_type_info("FileBox", method_name));
        
        // 型情報がある場合は、それに従って変換
        if let Some(type_info) = type_info {
            eprintln!("✨ Using type info for method '{}'", method_name);
            
            // 引数の数をチェック
            if arguments.len() != type_info.args.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("{} expects {} arguments, got {}", 
                                   method_name, type_info.args.len(), arguments.len()),
                });
            }
            
            // 各引数を型情報に従ってエンコード
            for (i, (arg, mapping)) in arguments.iter().zip(&type_info.args).enumerate() {
                eprintln!("  🔄 Arg[{}]: {} -> {} conversion", i, mapping.from, mapping.to);
                let value = self.execute_expression(arg)?;
                self.encode_value_with_mapping(&mut encoder, value, mapping)?;
            }
        } else {
            // 型情報がない場合は、従来のデフォルト動作
            eprintln!("⚠️ No type info for method '{}', using default encoding", method_name);
            for arg in arguments {
                let value = self.execute_expression(arg)?;
                self.encode_value_default(&mut encoder, value)?;
            }
        }
        
        Ok(encoder.finish())
    }
    
    /// 型マッピングに基づいて値をエンコード（美しい！）
    fn encode_value_with_mapping(
        &self, 
        encoder: &mut crate::bid::tlv::TlvEncoder, 
        value: Box<dyn NyashBox>, 
        mapping: &crate::bid::ArgTypeMapping
    ) -> Result<(), RuntimeError> {
        // determine_bid_tag()を使って適切なタグを決定
        let tag = mapping.determine_bid_tag()
            .ok_or_else(|| RuntimeError::InvalidOperation {
                message: format!("Unsupported type mapping: {} -> {}", mapping.from, mapping.to),
            })?;
        
        // タグに応じてエンコード
        match tag {
            crate::bid::BidTag::String => {
                let str_val = value.to_string_box().value;
                encoder.encode_string(&str_val)
                    .map_err(|e| RuntimeError::InvalidOperation {
                        message: format!("TLV string encoding failed: {:?}", e),
                    })
            }
            crate::bid::BidTag::Bytes => {
                let str_val = value.to_string_box().value;
                encoder.encode_bytes(str_val.as_bytes())
                    .map_err(|e| RuntimeError::InvalidOperation {
                        message: format!("TLV bytes encoding failed: {:?}", e),
                    })
            }
            crate::bid::BidTag::I32 => {
                if let Some(int_box) = value.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    encoder.encode_i32(int_box.value as i32)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV i32 encoding failed: {:?}", e),
                        })
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Expected integer for {} -> i32 conversion", mapping.from),
                    })
                }
            }
            crate::bid::BidTag::Bool => {
                if let Some(bool_box) = value.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                    encoder.encode_bool(bool_box.value)
                        .map_err(|e| RuntimeError::InvalidOperation {
                            message: format!("TLV bool encoding failed: {:?}", e),
                        })
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Expected bool for {} -> bool conversion", mapping.from),
                    })
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unsupported BID tag: {:?}", tag),
            })
        }
    }
    
    /// デフォルトエンコード（型情報がない場合のフォールバック）
    fn encode_value_default(
        &self,
        encoder: &mut crate::bid::tlv::TlvEncoder,
        value: Box<dyn NyashBox>
    ) -> Result<(), RuntimeError> {
        if let Some(str_box) = value.as_any().downcast_ref::<StringBox>() {
            encoder.encode_bytes(str_box.value.as_bytes())
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("TLV bytes encoding failed: {:?}", e),
                })
        } else if let Some(int_box) = value.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            encoder.encode_i32(int_box.value as i32)
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("TLV integer encoding failed: {:?}", e),
                })
        } else if let Some(bool_box) = value.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            encoder.encode_bool(bool_box.value)
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("TLV bool encoding failed: {:?}", e),
                })
        } else {
            let str_val = value.to_string_box().value;
            encoder.encode_bytes(str_val.as_bytes())
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("TLV default bytes encoding failed: {:?}", e),
                })
        }
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
    */

    /* legacy - PluginFileBox専用
    /// PluginFileBoxのメソッド呼び出しを実行 (BID-FFI system) - LEGACY HARDCODED VERSION
    /// Handles plugin-backed file I/O operations via FFI interface
    /// 🚨 DEPRECATED: This method has hardcoded method names and violates BID-FFI principles
    /// Use execute_plugin_method_generic instead for true dynamic method calling
    pub(in crate::interpreter) fn execute_plugin_file_method(&mut self, plugin_file_box: &PluginFileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 🎯 新しい汎用システムにリダイレクト
        self.execute_plugin_method_generic(plugin_file_box, method, arguments)
    }
    */
}
