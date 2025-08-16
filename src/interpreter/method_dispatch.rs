/*!
 * Method Dispatch Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles method call dispatch for all Box types
 * Core philosophy: "Everything is Box" with clean method resolution
 */

use super::*;
use crate::boxes::{buffer::BufferBox, JSONBox, HttpClientBox, StreamBox, RegexBox, IntentBox, SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox};
use crate::boxes::{FloatBox, MathBox, ConsoleBox, TimeBox, DateTimeBox, RandomBox, SoundBox, DebugBox, file::FileBox, MapBox};
use crate::box_trait::{BoolBox, SharedNyashBox};

impl NyashInterpreter {
    /// メソッド呼び出しを実行 - Method call processing
    pub(super) fn execute_method_call(&mut self, object: &ASTNode, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🔥 static関数のチェック
        if let ASTNode::Variable { name, .. } = object {
            // static関数が存在するかチェック
            let static_func = {
                let static_funcs = self.shared.static_functions.read().unwrap();
                if let Some(box_statics) = static_funcs.get(name) {
                    if let Some(func) = box_statics.get(method) {
                        Some(func.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            if let Some(static_func) = static_func {
                // static関数を実行
                if let ASTNode::FunctionDeclaration { params, body, .. } = static_func {
                    // 引数を評価
                    let mut arg_values = Vec::new();
                    for arg in arguments {
                        arg_values.push(self.execute_expression(arg)?);
                    }
                    
                    // パラメータ数チェック
                    if arg_values.len() != params.len() {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("Static method {}.{} expects {} arguments, got {}", 
                                           name, method, params.len(), arg_values.len()),
                        });
                    }
                    
                    // 🌍 local変数スタックを保存・クリア（static関数呼び出し開始）
                    let saved_locals = self.save_local_vars();
                    self.local_vars.clear();
                    
                    // 📤 outbox変数スタックも保存・クリア（static関数専用）
                    let saved_outbox = self.save_outbox_vars();
                    self.outbox_vars.clear();
                    
                    // 引数をlocal変数として設定
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        self.declare_local_variable(param, value.clone_box());
                    }
                    
                    // 関数本体を実行
                    let mut result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
                    for stmt in body {
                        match self.execute_statement(stmt)? {
                            ControlFlow::Return(value) => {
                                result = value;
                                break;
                            }
                            ControlFlow::Break => break,
                            ControlFlow::Throw(error) => {
                                // 🌍 static関数例外: local変数とoutbox変数を復元してから再throw
                                self.restore_local_vars(saved_locals);
                                self.restore_outbox_vars(saved_outbox);
                                return Err(RuntimeError::CustomException { value: error });
                            }
                            ControlFlow::None => {}
                        }
                    }
                    
                    // 🌍 local変数スタックを復元（static関数呼び出し終了）
                    self.restore_local_vars(saved_locals);
                    
                    // 📤 outbox変数スタックを復元（static関数終了）
                    self.restore_outbox_vars(saved_outbox);
                    
                    return Ok(result);
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Invalid static function declaration for {}.{}", name, method),
                    });
                }
            }
        }
        
        let obj_value = self.execute_expression(object)?;
        
        // ConsoleBox method calls
        if let Some(console_box) = obj_value.as_any().downcast_ref::<crate::boxes::console_box::ConsoleBox>() {
            return self.execute_console_method(console_box, method, arguments);
        }
        
        // StringBox method calls
        if let Some(string_box) = obj_value.as_any().downcast_ref::<StringBox>() {
            return self.execute_string_method(string_box, method, arguments);
        }
        
        // IntegerBox method calls
        if let Some(integer_box) = obj_value.as_any().downcast_ref::<IntegerBox>() {
            return self.execute_integer_method(integer_box, method, arguments);
        }
        
        // FloatBox method calls
        if let Some(float_box) = obj_value.as_any().downcast_ref::<FloatBox>() {
            return self.execute_float_method(float_box, method, arguments);
        }
        
        // BoolBox method calls
        if let Some(bool_box) = obj_value.as_any().downcast_ref::<BoolBox>() {
            return self.execute_bool_method(bool_box, method, arguments);
        }
        
        // ArrayBox method calls  
        if let Some(array_box) = obj_value.as_any().downcast_ref::<ArrayBox>() {
            return self.execute_array_method(array_box, method, arguments);
        }
        
        // BufferBox method calls
        if let Some(buffer_box) = obj_value.as_any().downcast_ref::<BufferBox>() {
            return self.execute_buffer_method(buffer_box, method, arguments);
        }
        
        // FileBox method calls
        if let Some(file_box) = obj_value.as_any().downcast_ref::<crate::boxes::file::FileBox>() {
            return self.execute_file_method(file_box, method, arguments);
        }
        
        // ResultBox method calls
        if let Some(result_box) = obj_value.as_any().downcast_ref::<ResultBox>() {
            return self.execute_result_method(result_box, method, arguments);
        }
        
        // FutureBox method calls
        if let Some(future_box) = obj_value.as_any().downcast_ref::<FutureBox>() {
            return self.execute_future_method(future_box, method, arguments);
        }
        
        // ChannelBox method calls
        if let Some(channel_box) = obj_value.as_any().downcast_ref::<ChannelBox>() {
            return self.execute_channel_method(channel_box, method, arguments);
        }
        
        // JSONBox method calls
        if let Some(json_box) = obj_value.as_any().downcast_ref::<JSONBox>() {
            return self.execute_json_method(json_box, method, arguments);
        }
        
        // HttpClientBox method calls
        if let Some(http_box) = obj_value.as_any().downcast_ref::<HttpClientBox>() {
            return self.execute_http_method(http_box, method, arguments);
        }
        
        // StreamBox method calls
        if let Some(stream_box) = obj_value.as_any().downcast_ref::<StreamBox>() {
            return self.execute_stream_method(stream_box, method, arguments);
        }
        
        // RegexBox method calls
        if let Some(regex_box) = obj_value.as_any().downcast_ref::<RegexBox>() {
            return self.execute_regex_method(regex_box, method, arguments);
        }
        
        // MathBox method calls
        if let Some(math_box) = obj_value.as_any().downcast_ref::<MathBox>() {
            return self.execute_math_method(math_box, method, arguments);
        }
        
        // NullBox method calls
        if let Some(null_box) = obj_value.as_any().downcast_ref::<crate::boxes::null_box::NullBox>() {
            return self.execute_null_method(null_box, method, arguments);
        }
        
        // TimeBox method calls
        if let Some(time_box) = obj_value.as_any().downcast_ref::<TimeBox>() {
            return self.execute_time_method(time_box, method, arguments);
        }
        
        // DateTimeBox method calls
        if let Some(datetime_box) = obj_value.as_any().downcast_ref::<DateTimeBox>() {
            return self.execute_datetime_method(datetime_box, method, arguments);
        }
        
        // TimerBox method calls
        if let Some(timer_box) = obj_value.as_any().downcast_ref::<TimerBox>() {
            return self.execute_timer_method(timer_box, method, arguments);
        }
        
        // MapBox method calls
        if let Some(map_box) = obj_value.as_any().downcast_ref::<MapBox>() {
            return self.execute_map_method(map_box, method, arguments);
        }
        
        // RandomBox method calls
        if let Some(random_box) = obj_value.as_any().downcast_ref::<RandomBox>() {
            return self.execute_random_method(random_box, method, arguments);
        }
        
        // SoundBox method calls
        if let Some(sound_box) = obj_value.as_any().downcast_ref::<SoundBox>() {
            return self.execute_sound_method(sound_box, method, arguments);
        }
        
        // EguiBox method calls (non-WASM only)
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        if let Some(egui_box) = obj_value.as_any().downcast_ref::<crate::boxes::EguiBox>() {
            return self.execute_egui_method(egui_box, method, arguments);
        }
        
        // DebugBox method calls
        if let Some(debug_box) = obj_value.as_any().downcast_ref::<DebugBox>() {
            return self.execute_debug_method(debug_box, method, arguments);
        }
        
        // MethodBox method calls
        if let Some(method_box) = obj_value.as_any().downcast_ref::<MethodBox>() {
            return self.execute_method_method(method_box, method, arguments);
        }
        
        // IntentBox method calls
        if let Some(intent_box) = obj_value.as_any().downcast_ref::<IntentBox>() {
            return self.execute_intent_method(intent_box, method, arguments);
        }
        
        // SocketBox method calls
        if let Some(socket_box) = obj_value.as_any().downcast_ref::<SocketBox>() {
            return self.execute_socket_method(socket_box, method, arguments);
        }
        
        // HTTPServerBox method calls
        if let Some(http_server_box) = obj_value.as_any().downcast_ref::<HTTPServerBox>() {
            return self.execute_http_server_method(http_server_box, method, arguments);
        }
        
        // HTTPRequestBox method calls
        if let Some(http_request_box) = obj_value.as_any().downcast_ref::<HTTPRequestBox>() {
            return self.execute_http_request_method(http_request_box, method, arguments);
        }
        
        // HTTPResponseBox method calls
        if let Some(http_response_box) = obj_value.as_any().downcast_ref::<HTTPResponseBox>() {
            return self.execute_http_response_method(http_response_box, method, arguments);
        }
        
        // P2PBox method calls - Temporarily disabled
        // if let Some(p2p_box) = obj_value.as_any().downcast_ref::<P2PBox>() {
        //     return self.execute_p2p_box_method(p2p_box, method, arguments);
        // }
        
        // Instance method calls
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // Check if method exists in Box declaration
            let method_impl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                if let Some(box_decl) = box_decls.get(&instance.class_name) {
                    box_decl.methods.get(method).cloned()
                } else {
                    None
                }
            };
            
            if let Some(method_node) = method_impl {
                // Execute instance method with 'me' context
                return self.execute_instance_method(instance, &method_node, arguments);
            }
            
            return Err(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in class '{}'", method, instance.class_name),
            });
        }
        
        // If not an instance or built-in Box, error
        Err(RuntimeError::TypeError {
            message: format!("Cannot call method '{}' on non-instance type", method),
        })
    }
}