/*!
 * Builtin box methods and birth methods
 */

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, VoidBox};
use crate::boxes::{ArrayBox, MapBox, MathBox, ConsoleBox, TimeBox, RandomBox, DebugBox, SoundBox, SocketBox};
use crate::boxes::{HTTPServerBox, HTTPRequestBox, HTTPResponseBox};
use crate::interpreter::core::{NyashInterpreter, RuntimeError};
use std::sync::{Arc, Mutex};

impl NyashInterpreter {
    /// 🔥 ビルトインBoxのメソッド呼び出し
    pub(super) fn execute_builtin_box_method(&mut self, parent: &str, method: &str, _current_instance: Box<dyn NyashBox>, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🌟 Phase 8.9: birth method support for builtin boxes
        if method == "birth" {
            return self.execute_builtin_birth_method(parent, _current_instance, arguments);
        }
        
        // ビルトインBoxのインスタンスを作成または取得
        // 現在のインスタンスからビルトインBoxのデータを取得し、ビルトインBoxとしてメソッド実行
        
        match parent {
            "StringBox" => {
                // StringBoxのインスタンスを作成（デフォルト値）
                let string_box = StringBox::new("");
                self.execute_string_method(&string_box, method, arguments)
            }
            "IntegerBox" => {
                // IntegerBoxのインスタンスを作成（デフォルト値）
                let integer_box = IntegerBox::new(0);
                self.execute_integer_method(&integer_box, method, arguments)
            }
            "ArrayBox" => {
                let array_box = ArrayBox::new();
                self.execute_array_method(&array_box, method, arguments)
            }
            "MapBox" => {
                let map_box = MapBox::new();
                self.execute_map_method(&map_box, method, arguments)
            }
            "MathBox" => {
                let math_box = MathBox::new();
                self.execute_math_method(&math_box, method, arguments)
            }
            "P2PBox" => {
                // P2PBoxの場合、現在のインスタンスからP2PBoxインスタンスを取得する必要がある
                // TODO: 現在のインスタンスのフィールドからP2PBoxを取得
                return Err(RuntimeError::InvalidOperation {
                    message: format!("P2PBox delegation not yet fully implemented: {}.{}", parent, method),
                });
            }
            "FileBox" => {
                let file_box = crate::boxes::file::FileBox::new();
                self.execute_file_method(&file_box, method, arguments)
            }
            "ConsoleBox" => {
                let console_box = ConsoleBox::new();
                self.execute_console_method(&console_box, method, arguments)
            }
            "TimeBox" => {
                let time_box = TimeBox::new();
                self.execute_time_method(&time_box, method, arguments)
            }
            "RandomBox" => {
                let random_box = RandomBox::new();
                self.execute_random_method(&random_box, method, arguments)
            }
            "DebugBox" => {
                let debug_box = DebugBox::new();
                self.execute_debug_method(&debug_box, method, arguments)
            }
            "SoundBox" => {
                let sound_box = SoundBox::new();
                self.execute_sound_method(&sound_box, method, arguments)
            }
            "SocketBox" => {
                let socket_box = SocketBox::new();
                self.execute_socket_method(&socket_box, method, arguments)
            }
            "HTTPServerBox" => {
                let http_server_box = HTTPServerBox::new();
                self.execute_http_server_method(&http_server_box, method, arguments)
            }
            "HTTPRequestBox" => {
                let http_request_box = HTTPRequestBox::new();
                self.execute_http_request_method(&http_request_box, method, arguments)
            }
            "HTTPResponseBox" => {
                let http_response_box = HTTPResponseBox::new();
                self.execute_http_response_method(&http_response_box, method, arguments)
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown built-in Box type for delegation: {}", parent),
                })
            }
        }
    }
    
    /// 🌟 Phase 8.9: Execute birth method for builtin boxes
    /// Provides constructor functionality for builtin boxes through explicit birth() calls
    pub(super) fn execute_builtin_birth_method(&mut self, builtin_name: &str, current_instance: Box<dyn NyashBox>, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // ビルトインBoxの種類に応じて適切なインスタンスを作成して返す
        match builtin_name {
            "StringBox" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StringBox.birth() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                
                let content = arg_values[0].to_string_box().value;
                let string_box = StringBox::new(content.clone());
                
                // 現在のインスタンスがInstanceBoxの場合、StringBoxを特別なフィールドに保存
                if let Some(instance) = current_instance.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    // 特別な内部フィールド "__builtin_content" にStringBoxを保存
                    let string_box_arc: Arc<Mutex<dyn NyashBox>> = Arc::new(Mutex::new(string_box));
                    instance.set_field_dynamic("__builtin_content".to_string(), 
                        crate::value::NyashValue::Box(string_box_arc));
                }
                
                Ok(Box::new(VoidBox::new())) // Return void to indicate successful initialization
            }
            "IntegerBox" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntegerBox.birth() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                
                let value = if let Ok(int_val) = arg_values[0].to_string_box().value.parse::<i64>() {
                    int_val
                } else {
                    return Err(RuntimeError::TypeError {
                        message: format!("Cannot convert '{}' to integer", arg_values[0].to_string_box().value),
                    });
                };
                
                let _integer_box = IntegerBox::new(value);
                Ok(Box::new(VoidBox::new()))
            }
            "MathBox" => {
                // MathBoxは引数なしのコンストラクタ
                if arg_values.len() != 0 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MathBox.birth() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                let _math_box = MathBox::new();
                Ok(Box::new(VoidBox::new()))
            }
            "ArrayBox" => {
                // ArrayBoxも引数なしのコンストラクタ
                if arg_values.len() != 0 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ArrayBox.birth() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                let _array_box = ArrayBox::new();
                eprintln!("🌟 DEBUG: ArrayBox.birth() created");
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                // 他のビルトインBoxは今後追加
                Err(RuntimeError::InvalidOperation {
                    message: format!("birth() method not yet implemented for builtin box '{}'", builtin_name),
                })
            }
        }
    }
}