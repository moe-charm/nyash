/*!
 * Method Dispatch Module
 * 
 * Extracted from expressions.rs lines 383-900 (~517 lines)
 * Handles method call dispatch for all Box types and static function calls
 * Core philosophy: "Everything is Box" with unified method dispatch
 */

use super::*;
use crate::boxes::{buffer::BufferBox, JSONBox, HttpClientBox, StreamBox, RegexBox, IntentBox, SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox};
use crate::boxes::{FloatBox, MathBox, ConsoleBox, TimeBox, DateTimeBox, RandomBox, SoundBox, DebugBox, file::FileBox, MapBox};
use std::sync::Arc;

impl NyashInterpreter {
    /// メソッド呼び出しを実行 - 全Box型の統一ディスパッチ
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
                return self.execute_static_function(static_func, name, method, arguments);
            }
            
            // 📚 nyashstd標準ライブラリのメソッドチェック
            if let Some(stdlib_result) = self.try_execute_stdlib_method(name, method, arguments)? {
                return Ok(stdlib_result);
            }
        }
        
        // オブジェクトを評価（通常のメソッド呼び出し）
        let obj_value = self.execute_expression(object)?;
        
        // 各Box型に対するメソッドディスパッチ
        self.dispatch_builtin_method(&obj_value, method, arguments, object)
    }

    /// static関数を実行
    fn execute_static_function(
        &mut self, 
        static_func: ASTNode, 
        box_name: &str, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
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
                                   box_name, method, params.len(), arg_values.len()),
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
            
            // static関数の本体を実行
            let mut result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
            for statement in &body {
                result = self.execute_statement(statement)?;
                
                // return文チェック
                if let super::ControlFlow::Return(return_val) = &self.control_flow {
                    result = return_val.clone_box();
                    self.control_flow = super::ControlFlow::None;
                    break;
                }
            }
            
            // local変数スタックを復元
            self.restore_local_vars(saved_locals);
            
            // outbox変数スタックを復元
            self.restore_outbox_vars(saved_outbox);
            
            Ok(result)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Invalid static function: {}.{}", box_name, method),
            })
        }
    }

    /// nyashstd標準ライブラリメソッド実行を試行
    fn try_execute_stdlib_method(
        &mut self, 
        box_name: &str, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Option<Box<dyn NyashBox>>, RuntimeError> {
        let stdlib_method = if let Some(ref stdlib) = self.stdlib {
            if let Some(nyashstd_namespace) = stdlib.namespaces.get("nyashstd") {
                if let Some(static_box) = nyashstd_namespace.static_boxes.get(box_name) {
                    if let Some(builtin_method) = static_box.methods.get(method) {
                        Some(*builtin_method) // Copyトレイトで関数ポインターをコピー
                    } else {
                        eprintln!("🔍 Method '{}' not found in nyashstd.{}", method, box_name);
                        None
                    }
                } else {
                    eprintln!("🔍 Static box '{}' not found in nyashstd", box_name);
                    None
                }
            } else {
                eprintln!("🔍 nyashstd namespace not found in stdlib");
                None
            }
        } else {
            eprintln!("🔍 stdlib not initialized for method call");
            None
        };
        
        if let Some(builtin_method) = stdlib_method {
            eprintln!("🌟 Calling nyashstd method: {}.{}", box_name, method);
            
            // 引数を評価
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            
            // 標準ライブラリのメソッドを実行
            let result = builtin_method(&arg_values)?;
            eprintln!("✅ nyashstd method completed: {}.{}", box_name, method);
            return Ok(Some(result));
        }
        
        Ok(None)
    }

    /// ビルトインBox型メソッドディスパッチ
    fn dispatch_builtin_method(
        &mut self, 
        obj_value: &Box<dyn NyashBox>, 
        method: &str, 
        arguments: &[ASTNode],
        object: &ASTNode
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
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
        
        // DebugBox method calls
        if let Some(debug_box) = obj_value.as_any().downcast_ref::<DebugBox>() {
            return self.execute_debug_method(debug_box, method, arguments);
        }
        
        // ConsoleBox method calls
        if let Some(console_box) = obj_value.as_any().downcast_ref::<crate::boxes::console_box::ConsoleBox>() {
            return self.execute_console_method(console_box, method, arguments);
        }
        
        // IntentBox method calls
        if let Some(intent_box) = obj_value.as_any().downcast_ref::<IntentBox>() {
            return self.execute_intent_box_method(intent_box, method, arguments);
        }
        
        // SocketBox method calls
        if let Some(socket_box) = obj_value.as_any().downcast_ref::<SocketBox>() {
            let result = self.execute_socket_method(socket_box, method, arguments)?;
            
            // 🔧 FIX: Update stored variable for stateful SocketBox methods
            if matches!(method, "bind" | "connect" | "close") {
                self.update_stateful_socket_box(object, socket_box)?;
            }
            
            return Ok(result);
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
        
        // EguiBox method calls (非WASM環境のみ)
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        if let Some(egui_box) = obj_value.as_any().downcast_ref::<crate::boxes::EguiBox>() {
            return self.execute_egui_method(egui_box, method, arguments);
        }
        
        // WebDisplayBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_display_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebDisplayBox>() {
            return self.execute_web_display_method(web_display_box, method, arguments);
        }

        // WebConsoleBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_console_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebConsoleBox>() {
            return self.execute_web_console_method(web_console_box, method, arguments);
        }

        // WebCanvasBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_canvas_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebCanvasBox>() {
            return self.execute_web_canvas_method(web_canvas_box, method, arguments);
        }

        // ユーザー定義Boxのメソッド呼び出し
        self.execute_user_defined_method(obj_value, method, arguments)
    }

    /// SocketBoxの状態変更を反映
    fn update_stateful_socket_box(
        &mut self, 
        object: &ASTNode, 
        socket_box: &SocketBox
    ) -> Result<(), RuntimeError> {
        eprintln!("🔧 DEBUG: Stateful method called, updating stored instance");
        let updated_instance = socket_box.clone();
        eprintln!("🔧 DEBUG: Updated instance created with ID={}", updated_instance.box_id());
        
        match object {
            ASTNode::Variable { name, .. } => {
                eprintln!("🔧 DEBUG: Updating local variable '{}'", name);
                if let Some(stored_var) = self.local_vars.get_mut(name) {
                    eprintln!("🔧 DEBUG: Found local variable '{}', updating from id={} to id={}", 
                             name, stored_var.box_id(), updated_instance.box_id());
                    *stored_var = Arc::new(updated_instance);
                } else {
                    eprintln!("🔧 DEBUG: Local variable '{}' not found", name);
                }
            },
            ASTNode::FieldAccess { object: field_obj, field, .. } => {
                eprintln!("🔧 DEBUG: Updating field access '{}'", field);
                self.update_field_with_socket_box(field_obj, field, updated_instance)?;
            },
            _ => {
                eprintln!("🔧 DEBUG: Object type not handled: {:?}", object);
            }
        }
        
        Ok(())
    }

    /// フィールドアクセスでのSocketBox更新
    fn update_field_with_socket_box(
        &mut self, 
        field_obj: &ASTNode, 
        field: &str, 
        updated_instance: SocketBox
    ) -> Result<(), RuntimeError> {
        match field_obj {
            ASTNode::Variable { name, .. } => {
                eprintln!("🔧 DEBUG: Field object is variable '{}'", name);
                if name == "me" {
                    eprintln!("🔧 DEBUG: Updating me.{} (via variable)", field);
                    if let Ok(me_instance) = self.resolve_variable("me") {
                        eprintln!("🔧 DEBUG: Resolved 'me' instance id={}", me_instance.box_id());
                        if let Some(instance) = (*me_instance).as_any().downcast_ref::<InstanceBox>() {
                            eprintln!("🔧 DEBUG: me is InstanceBox, setting field '{}' to updated instance id={}", field, updated_instance.box_id());
                            let result = instance.set_field(field, Arc::new(updated_instance));
                            eprintln!("🔧 DEBUG: set_field result: {:?}", result);
                        } else {
                            eprintln!("🔧 DEBUG: me is not an InstanceBox, type: {}", me_instance.type_name());
                        }
                    } else {
                        eprintln!("🔧 DEBUG: Failed to resolve 'me'");
                    }
                } else {
                    eprintln!("🔧 DEBUG: Field object is not 'me', it's '{}'", name);
                }
            },
            ASTNode::Me { .. } => {
                eprintln!("🔧 DEBUG: Field object is Me node, updating me.{}", field);
                if let Ok(me_instance) = self.resolve_variable("me") {
                    eprintln!("🔧 DEBUG: Resolved 'me' instance id={}", me_instance.box_id());
                    if let Some(instance) = (*me_instance).as_any().downcast_ref::<InstanceBox>() {
                        eprintln!("🔧 DEBUG: me is InstanceBox, setting field '{}' to updated instance id={}", field, updated_instance.box_id());
                        let result = instance.set_field(field, Arc::new(updated_instance));
                        eprintln!("🔧 DEBUG: set_field result: {:?}", result);
                    } else {
                        eprintln!("🔧 DEBUG: me is not an InstanceBox, type: {}", me_instance.type_name());
                    }
                } else {
                    eprintln!("🔧 DEBUG: Failed to resolve 'me'");
                }
            },
            _ => {
                eprintln!("🔧 DEBUG: Field object is not a variable or me, type: {:?}", field_obj);
            }
        }
        
        Ok(())
    }

    /// ユーザー定義Boxメソッド実行
    fn execute_user_defined_method(
        &mut self, 
        obj_value: &Box<dyn NyashBox>, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // InstanceBox method calls (user-defined methods)
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            return self.execute_instance_method(instance, method, arguments);
        }
        
        // Static box method calls would be handled here if implemented
        // (Currently handled via different mechanism in static function dispatch)
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Method '{}' not found on type '{}'", method, obj_value.type_name()),
        })
    }
}