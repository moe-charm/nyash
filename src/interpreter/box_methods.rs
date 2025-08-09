/*!
 * Box Method Handlers Module
 * 
 * Extracted from interpreter.rs lines 1389-2515 (1,126 lines)  
 * Contains Box type-specific method implementations:
 * 
 * MOVED TO methods/basic_methods.rs:
 * - execute_string_method (StringBox)
 * - execute_integer_method (IntegerBox)
 * - execute_bool_method (BoolBox) - NEW
 * - execute_float_method (FloatBox) - NEW
 * 
 * MOVED TO methods/collection_methods.rs:
 * - execute_array_method (ArrayBox)  
 * - execute_map_method (MapBox)
 * 
 * MOVED TO methods/io_methods.rs:
 * - execute_file_method (FileBox)
 * - execute_result_method (ResultBox)
 * 
 * MOVED TO methods/math_methods.rs:
 * - execute_math_method (MathBox)
 * - execute_random_method (RandomBox)
 * 
 * MOVED TO system_methods.rs:
 * - execute_time_method (TimeBox)
 * - execute_datetime_method (DateTimeBox)
 * - execute_timer_method (TimerBox)
 * - execute_debug_method (DebugBox)
 * 
 * MOVED TO async_methods.rs:
 * - execute_future_method (FutureBox)
 * - execute_channel_method (ChannelBox)
 * 
 * MOVED TO web_methods.rs:
 * - execute_web_display_method (WebDisplayBox)
 * - execute_web_console_method (WebConsoleBox)  
 * - execute_web_canvas_method (WebCanvasBox)
 * 
 * MOVED TO special_methods.rs:
 * - execute_sound_method (SoundBox)
 * - execute_method_box_method (MethodBox)
 * 
 * REMAINING IN THIS MODULE:
 * - execute_console_method
 * - execute_null_method
 */

use super::*;
use crate::boxes::null_box::NullBox;

impl NyashInterpreter {
    // StringBox methods moved to methods/basic_methods.rs

    // IntegerBox methods moved to methods/basic_methods.rs

    // ArrayBox methods moved to methods/collection_methods.rs
    
    // FileBox methods moved to methods/io_methods.rs
    
    // ResultBox methods moved to methods/io_methods.rs

    // FutureBox methods moved to async_methods.rs

    // ChannelBox methods moved to async_methods.rs

    // MathBox methods moved to methods/math_methods.rs

    /// NullBoxのメソッド呼び出しを実行
    pub(super) fn execute_null_method(&mut self, _null_box: &NullBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "is_null" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("is_null() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(true)))
            }
            "is_not_null" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("is_not_null() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(false)))
            }
            "equals" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let other = &arg_values[0];
                // NullBoxは他のNullBoxとのみ等しい
                let is_equal = other.as_any().downcast_ref::<NullBox>().is_some();
                Ok(Box::new(BoolBox::new(is_equal)))
            }
            "get_or_default" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get_or_default() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                // nullの場合はデフォルト値を返す
                Ok(arg_values[0].clone_box())
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown NullBox method: {}", method),
                })
            }
        }
    }

    // TimeBox methods moved to system_methods.rs

    // DateTimeBox methods moved to system_methods.rs

    // TimerBox methods moved to system_methods.rs

    // MapBox methods moved to methods/collection_methods.rs

    // RandomBox methods moved to methods/math_methods.rs

    // SoundBox methods moved to special_methods.rs

    // DebugBox methods moved to system_methods.rs

    /// ConsoleBoxのメソッド呼び出しを実行
    pub(super) fn execute_console_method(&mut self, console_box: &crate::boxes::console_box::ConsoleBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "log" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.log() requires at least 1 argument".to_string(),
                    });
                }
                
                // 引数をすべて文字列に変換
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.log(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "warn" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.warn() requires at least 1 argument".to_string(),
                    });
                }
                
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.warn(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "error" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.error() requires at least 1 argument".to_string(),
                    });
                }
                
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.error(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("console.clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                console_box.clear();
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown ConsoleBox method: {}", method),
                })
            }
        }
    }

    // MethodBox methods moved to special_methods.rs
    
    // Web methods moved to web_methods.rs
}