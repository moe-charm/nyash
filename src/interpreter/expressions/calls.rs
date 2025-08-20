/*!
 * Method calls and from delegation calls
 */

use super::*;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox};
use crate::boxes::{ArrayBox, FloatBox, MapBox, FutureBox};
use crate::boxes::{BufferBox, JSONBox, HttpClientBox, StreamBox, RegexBox, IntentBox, SocketBox};
use crate::boxes::{HTTPServerBox, HTTPRequestBox, HTTPResponseBox, MathBox, TimeBox, DateTimeBox};
use crate::boxes::{RandomBox, SoundBox, DebugBox};
use crate::instance_v2::InstanceBox;
use crate::channel_box::ChannelBox;
use crate::interpreter::core::{NyashInterpreter, RuntimeError};
use crate::interpreter::finalization;
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
use crate::runtime::plugin_loader_v2::PluginBoxV2;
use std::sync::Arc;

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
                        
                        return Ok(result);
                }
            }
            
            // 📚 nyashstd標準ライブラリのメソッドチェック
            let stdlib_method = if let Some(ref stdlib) = self.stdlib {
                if let Some(nyashstd_namespace) = stdlib.namespaces.get("nyashstd") {
                    if let Some(static_box) = nyashstd_namespace.static_boxes.get(name) {
                        if let Some(builtin_method) = static_box.methods.get(method) {
                            Some(*builtin_method) // Copyトレイトで関数ポインターをコピー
                        } else {
                            eprintln!("🔍 Method '{}' not found in nyashstd.{}", method, name);
                            None
                        }
                    } else {
                        eprintln!("🔍 Static box '{}' not found in nyashstd", name);
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
                eprintln!("🌟 Calling nyashstd method: {}.{}", name, method);
                
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_expression(arg)?);
                }
                
                // 標準ライブラリのメソッドを実行
                let result = builtin_method(&arg_values)?;
                eprintln!("✅ nyashstd method completed: {}.{}", name, method);
                return Ok(result);
            }
            
            // 🔥 ユーザー定義のStatic Boxメソッドチェック
            if self.is_static_box(name) {
                eprintln!("🔍 Checking user-defined static box: {}", name);
                
                // Static Boxの初期化を確実に実行
                self.ensure_static_box_initialized(name)?;
                
                // GlobalBox.statics.{name} からメソッドを取得してクローン
                let (method_clone, static_instance_clone) = {
                    let global_box = self.shared.global_box.lock()
                        .map_err(|_| RuntimeError::RuntimeFailure {
                            message: "Failed to acquire global box lock".to_string()
                        })?;
                        
                    let statics_box = global_box.get_field("statics")
                        .ok_or(RuntimeError::RuntimeFailure {
                            message: "statics namespace not found in GlobalBox".to_string()
                        })?;
                        
                    let statics_instance = statics_box.as_any()
                        .downcast_ref::<InstanceBox>()
                        .ok_or(RuntimeError::TypeError {
                            message: "statics field is not an InstanceBox".to_string()
                        })?;
                        
                    let static_instance = statics_instance.get_field(name)
                        .ok_or(RuntimeError::InvalidOperation {
                            message: format!("Static box '{}' not found in statics namespace", name),
                        })?;
                        
                    let instance = static_instance.as_any()
                        .downcast_ref::<InstanceBox>()
                        .ok_or(RuntimeError::TypeError {
                            message: format!("Static box '{}' is not an InstanceBox", name),
                        })?;
                    
                    // メソッドを探す
                    if let Some(method_node) = instance.get_method(method) {
                        (method_node.clone(), static_instance.clone_box())
                    } else {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("Method '{}' not found in static box '{}'", method, name),
                        });
                    }
                }; // lockはここで解放される
                
                eprintln!("🌟 Calling static box method: {}.{}", name, method);
                
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_expression(arg)?);
                }
                
                // メソッドのパラメータと本体を取得
                if let ASTNode::FunctionDeclaration { params, body, .. } = &method_clone {
                    // local変数スタックを保存
                    let saved_locals = self.save_local_vars();
                    self.local_vars.clear();
                    
                    // meをstatic boxインスタンスに設定
                    self.declare_local_variable("me", static_instance_clone);
                    
                    // 引数をlocal変数として設定
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        self.declare_local_variable(param, value.clone_box());
                    }
                    
                    // メソッドの本体を実行
                    let mut result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
                    for statement in body {
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
                    
                    eprintln!("✅ Static box method completed: {}.{}", name, method);
                    return Ok(result);
                }
            }
        }
        
        // オブジェクトを評価（通常のメソッド呼び出し）
        let obj_value = self.execute_expression(object)?;
        eprintln!("🔍 DEBUG: execute_method_call - object type: {}, method: {}", obj_value.type_name(), method);
        
        // StringBox method calls
        eprintln!("🔍 DEBUG: Checking StringBox downcast for type: {}", obj_value.type_name());
        if let Some(string_box) = obj_value.as_any().downcast_ref::<StringBox>() {
            eprintln!("🔍 DEBUG: StringBox detected, calling execute_string_method");
            return self.execute_string_method(string_box, method, arguments);
        } else {
            eprintln!("🔍 DEBUG: StringBox downcast failed");
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
        
        /* legacy - PluginFileBox専用
        // PluginFileBox method calls (BID-FFI system)
        if let Some(plugin_file_box) = obj_value.as_any().downcast_ref::<crate::bid::plugin_box::PluginFileBox>() {
            return self.execute_plugin_file_method(plugin_file_box, method, arguments);
        }
        */
        
        // ResultBox method calls
        if let Some(result_box) = obj_value.as_any().downcast_ref::<crate::box_trait::ResultBox>() {
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
        if let Some(timer_box) = obj_value.as_any().downcast_ref::<crate::boxes::time_box::TimerBox>() {
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
            // These methods modify the SocketBox internal state, so we need to update
            // the stored variable/field to ensure subsequent accesses get the updated state
            if matches!(method, "bind" | "connect" | "close") {
                eprintln!("🔧 DEBUG: Stateful method '{}' called, updating stored instance", method);
                let updated_instance = socket_box.clone();
                eprintln!("🔧 DEBUG: Updated instance created with ID={}", updated_instance.box_id());
                
                match object {
                    ASTNode::Variable { name, .. } => {
                        eprintln!("🔧 DEBUG: Updating local variable '{}'", name);
                        // Handle local variables
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
                        // Handle StaticBox fields like me.server
                        match field_obj.as_ref() {
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
                    },
                    _ => {
                        eprintln!("🔧 DEBUG: Object type not handled: {:?}", object);
                    }
                }
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
        
        // MethodBox method calls
        if let Some(method_box) = obj_value.as_any().downcast_ref::<crate::method_box::MethodBox>() {
            return self.execute_method_box_method(method_box, method, arguments);
        }
        
        // IntegerBox method calls  
        if let Some(integer_box) = obj_value.as_any().downcast_ref::<IntegerBox>() {
            return self.execute_integer_method(integer_box, method, arguments);
        }
        
        // FloatBox method calls (将来的に追加予定)
        
        // RangeBox method calls (将来的に追加予定)
        
        // PluginBoxV2 method calls
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        if let Some(plugin_box) = obj_value.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            return self.execute_plugin_box_v2_method(plugin_box, method, arguments);
        }
        
        // ⚠️ InstanceBox method calls (最後にチェック、ビルトインBoxの後)
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // 🔥 finiは何回呼ばれてもエラーにしない（ユーザー要求）
            // is_finalized()チェックを削除
            
            // fini()は特別処理
            if method == "fini" {
                // 🔥 weak-fini prohibition check - prevent fini() on weak fields
                if let ASTNode::FieldAccess { object: field_object, field, .. } = object {
                    // Check if this is me.<field>.fini() pattern
                    if let ASTNode::Variable { name, .. } = field_object.as_ref() {
                        if name == "me" {
                            // Get current instance to check if field is weak
                            if let Ok(current_me) = self.resolve_variable("me") {
                                if let Some(current_instance) = (*current_me).as_any().downcast_ref::<InstanceBox>() {
                                    if current_instance.is_weak_field(field) {
                                        return Err(RuntimeError::InvalidOperation {
                                            message: format!(
                                                "Cannot finalize weak field '{}' (non-owning reference)",
                                                field
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 既に解放済みの場合は何もしない（二重fini()対策）
                if instance.is_finalized() {
                    return Ok(Box::new(VoidBox::new()));
                }
                
                // まず、Box内で定義されたfini()メソッドがあれば実行
                if let Some(fini_method) = instance.get_method("fini") {
                    if let ASTNode::FunctionDeclaration { body, .. } = fini_method.clone() {
                        // 🌍 革命的メソッド実行：local変数スタックを使用
                        let saved_locals = self.save_local_vars();
                        self.local_vars.clear();
                        
                        // thisをlocal変数として設定
                        self.declare_local_variable("me", obj_value.clone_box());
                        
                        // fini()メソッドの本体を実行
                        let mut _result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
                        for statement in &body {
                            _result = self.execute_statement(statement)?;
                            
                            // return文チェック
                            if let super::ControlFlow::Return(_) = &self.control_flow {
                                self.control_flow = super::ControlFlow::None;
                                break;
                            }
                        }
                        
                        // local変数スタックを復元
                        self.restore_local_vars(saved_locals);
                    }
                }
                
                // 🔗 Phase 8.9: Weak reference invalidation after user fini
                let target_info = obj_value.to_string_box().value;
                eprintln!("🔗 DEBUG: Triggering weak reference invalidation for fini: {}", target_info);
                self.trigger_weak_reference_invalidation(&target_info);
                
                // インスタンスの内部的な解放処理
                instance.fini().map_err(|e| RuntimeError::InvalidOperation {
                    message: e,
                })?;
                finalization::mark_as_finalized(instance.box_id());
                return Ok(Box::new(VoidBox::new()));
            }
            
            // メソッドを取得（まずローカルメソッドを確認）
            if let Some(method_ast) = instance.get_method(method) {
                let method_ast = method_ast.clone();
                
                // メソッドが関数宣言の形式であることを確認
                if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast {
                // 🚨 FIX: 引数評価を完全に現在のコンテキストで完了させる
                let mut arg_values = Vec::new();
                for (_i, arg) in arguments.iter().enumerate() {
                    let arg_value = self.execute_expression(arg)?;
                    arg_values.push(arg_value);
                }
                
                // パラメータ数チェック
                if arg_values.len() != params.len() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Method {} expects {} arguments, got {}", 
                                       method, params.len(), arg_values.len()),
                    });
                }
                
                // 🌍 NOW SAFE: すべての引数評価完了後にコンテキスト切り替え
                let saved_locals = self.save_local_vars();
                self.local_vars.clear();
                
                // thisをlocal変数として設定
                self.declare_local_variable("me", obj_value.clone_box());
                
                // パラメータをlocal変数として設定
                for (param, value) in params.iter().zip(arg_values.iter()) {
                    self.declare_local_variable(param, value.clone_box());
                }
                
                // メソッド本体を実行
                let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
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
                
                Ok(result)
                } else {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("Method '{}' is not a valid function declaration", method),
                    })
                }
            } else {
                // ローカルメソッドが見つからない場合、親のビルトインBoxメソッドを確認
                let box_declarations = self.shared.box_declarations.read().unwrap();
                let parent_names = if let Some(box_decl) = box_declarations.get(&instance.class_name) {
                    box_decl.extends.clone()
                } else {
                    vec![]
                };
                drop(box_declarations);
                
                // 親がビルトインBoxか確認
                for parent_name in &parent_names {
                    if crate::box_trait::is_builtin_box(parent_name) {
                        // ビルトインBoxメソッドを実行
                        if parent_name == "StringBox" {
                                // ユーザー定義BoxがStringBoxを継承している場合
                                // __builtin_contentフィールドからStringBoxを取得
                                if let Some(builtin_value) = instance.get_field_ng("__builtin_content") {
                                    if let crate::value::NyashValue::Box(boxed) = builtin_value {
                                        let boxed_guard = boxed.lock().unwrap();
                                        if let Some(string_box) = boxed_guard.as_any().downcast_ref::<StringBox>() {
                                            return self.execute_string_method(string_box, method, arguments);
                                        }
                                    }
                                } else {
                                }
                                // フィールドが見つからない場合は空のStringBoxを使用（互換性のため）
                                let string_box = StringBox::new("");
                                return self.execute_string_method(&string_box, method, arguments);
                        } else if parent_name == "IntegerBox" {
                                // __builtin_contentフィールドからIntegerBoxを取得
                                if let Some(builtin_value) = instance.get_field_ng("__builtin_content") {
                                    if let crate::value::NyashValue::Box(boxed) = builtin_value {
                                        let boxed_guard = boxed.lock().unwrap();
                                        if let Some(integer_box) = boxed_guard.as_any().downcast_ref::<IntegerBox>() {
                                            return self.execute_integer_method(integer_box, method, arguments);
                                        }
                                    }
                                }
                                // フィールドが見つからない場合は0のIntegerBoxを使用
                                let integer_box = IntegerBox::new(0);
                                return self.execute_integer_method(&integer_box, method, arguments);
                        } else if parent_name == "MathBox" {
                                // MathBoxはステートレスなので、新しいインスタンスを作成
                                let math_box = MathBox::new();
                                return self.execute_math_method(&math_box, method, arguments);
                        }
                        // 他のビルトインBoxも必要に応じて追加
                    }
                }
                
                // メソッドが見つからない
                Err(RuntimeError::InvalidOperation {
                    message: format!("Method '{}' not found in {}", method, instance.class_name),
                })
            }
        } else {
            eprintln!("🔍 DEBUG: Reached non-instance type error for type: {}, method: {}", obj_value.type_name(), method);
            Err(RuntimeError::TypeError {
                message: format!("Cannot call method '{}' on non-instance type", method),
            })
        }
    }
    
    /// 🔥 FromCall実行処理 - from Parent.method(arguments) or from Parent.constructor(arguments)
    pub(super) fn execute_from_call(&mut self, parent: &str, method: &str, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 1. 現在のコンテキストで'me'変数を取得（現在のインスタンス）
        let current_instance_val = self.resolve_variable("me")
            .map_err(|_| RuntimeError::InvalidOperation {
                message: "'from' can only be used inside methods".to_string(),
            })?;
        
        let current_instance = (*current_instance_val).as_any().downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: "'from' requires current instance to be InstanceBox".to_string(),
            })?;
        
        // 2. 現在のクラスのデリゲーション関係を検証
        let current_class = &current_instance.class_name;
        let box_declarations = self.shared.box_declarations.read().unwrap();
        
        let current_box_decl = box_declarations.get(current_class)
            .ok_or(RuntimeError::UndefinedClass { 
                name: current_class.clone() 
            })?;
        
        // extendsまたはimplementsでparentが指定されているか確認 (Multi-delegation) 🚀
        let is_valid_delegation = current_box_decl.extends.contains(&parent.to_string()) || 
                                 current_box_decl.implements.contains(&parent.to_string());
        
        if !is_valid_delegation {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Class '{}' does not delegate to '{}'. Use 'box {} from {}' to establish delegation.", 
                               current_class, parent, current_class, parent),
            });
        }
        
        // 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定
        use crate::box_trait::is_builtin_box;
        
        let mut is_builtin = is_builtin_box(parent);
        
        // GUI機能が有効な場合はEguiBoxも追加判定
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        {
            if parent == "EguiBox" {
                is_builtin = true;
            }
        }
        
        // 🔥 Phase 8.9: Transparency system removed - all delegation must be explicit
        // Removed: if is_builtin && method == parent { ... execute_builtin_constructor_call ... }
        
        if is_builtin {
            // ビルトインBoxの場合、ロックを解放してからメソッド呼び出し
            drop(box_declarations);
            return self.execute_builtin_box_method(parent, method, current_instance_val.clone_box(), arguments);
        }
        
        // 3. 親クラスのBox宣言を取得（ユーザー定義Boxの場合）
        let parent_box_decl = box_declarations.get(parent)
            .ok_or(RuntimeError::UndefinedClass { 
                name: parent.to_string() 
            })?
            .clone();
        
        drop(box_declarations); // ロック早期解放
        
        // 4. constructorまたはinitまたはpackまたはbirthの場合の特別処理
        if method == "constructor" || method == "init" || method == "pack" || method == "birth" || method == parent {
            return self.execute_from_parent_constructor(parent, &parent_box_decl, current_instance_val.clone_box(), arguments);
        }
        
        // 5. 親クラスのメソッドを取得
        let parent_method = parent_box_decl.methods.get(method)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in parent class '{}'", method, parent),
            })?
            .clone();
        
        // 6. 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // 7. 親メソッドを実行
        if let ASTNode::FunctionDeclaration { params, body, .. } = parent_method {
            // パラメータ数チェック
            if arg_values.len() != params.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parent method {}.{} expects {} arguments, got {}", 
                                   parent, method, params.len(), arg_values.len()),
                });
            }
            
            // 🌍 local変数スタックを保存・クリア（親メソッド実行開始）
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // 'me'を現在のインスタンスに設定（重要：現在のインスタンスを維持）
            self.declare_local_variable("me", current_instance_val.clone_box());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // 親メソッドの本体を実行
            let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
            for statement in &body {
                result = self.execute_statement(statement)?;
                
                // return文チェック
                if let super::ControlFlow::Return(return_val) = &self.control_flow {
                    result = return_val.clone_box();
                    self.control_flow = super::ControlFlow::None;
                    break;
                }
            }
            
            // 🔍 DEBUG: FromCall実行結果をログ出力
            eprintln!("🔍 DEBUG: FromCall {}.{} result: {}", parent, method, result.to_string_box().value);
            
            // local変数スタックを復元
            self.restore_local_vars(saved_locals);
            
            Ok(result)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Parent method '{}' is not a valid function declaration", method),
            })
        }
    }
    
    /// 🔥 fromCall専用親コンストラクタ実行処理 - from Parent.constructor(arguments)
    fn execute_from_parent_constructor(&mut self, parent: &str, parent_box_decl: &super::BoxDeclaration, 
                                       current_instance: Box<dyn NyashBox>, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 1. 親クラスのコンストラクタを取得（引数の数でキーを作成）
        // "birth/引数数"、"pack/引数数"、"init/引数数"、"Box名/引数数" の順で試す
        let birth_key = format!("birth/{}", arguments.len());
        let pack_key = format!("pack/{}", arguments.len());
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", parent, arguments.len());
        
        let parent_constructor = parent_box_decl.constructors.get(&birth_key)
            .or_else(|| parent_box_decl.constructors.get(&pack_key))
            .or_else(|| parent_box_decl.constructors.get(&init_key))
            .or_else(|| parent_box_decl.constructors.get(&box_name_key))
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("No constructor found for parent class '{}' with {} arguments", parent, arguments.len()),
            })?
            .clone();
        
        // 2. 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // 3. 親コンストラクタを実行
        if let ASTNode::FunctionDeclaration { params, body, .. } = parent_constructor {
            // パラメータ数チェック
            if arg_values.len() != params.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parent constructor {} expects {} arguments, got {}", 
                                   parent, params.len(), arg_values.len()),
                });
            }
            
            // 🌍 local変数スタックを保存・クリア（親コンストラクタ実行開始）
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // 'me'を現在のインスタンスに設定
            self.declare_local_variable("me", current_instance.clone_box());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // 親コンストラクタの本体を実行
            let mut _result: Box<dyn NyashBox> = Box::new(VoidBox::new());
            for statement in &body {
                _result = self.execute_statement(statement)?;
                
                // return文チェック
                if let super::ControlFlow::Return(return_val) = &self.control_flow {
                    _result = return_val.clone_box();
                    self.control_flow = super::ControlFlow::None;
                    break;
                }
            }
            
            // local変数スタックを復元
            self.restore_local_vars(saved_locals);
            
            // 親コンストラクタは通常現在のインスタンスを返す
            Ok(current_instance)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Parent constructor is not a valid function declaration"),
            })
        }
    }
    
    /// Execute method call on PluginBoxV2
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    fn execute_plugin_box_v2_method(
        &mut self,
        plugin_box: &PluginBoxV2,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        eprintln!("🔍 execute_plugin_box_v2_method called: {}.{}", plugin_box.box_type, method);
        
        // Get global loader to access configuration
        let loader = crate::runtime::plugin_loader_v2::get_global_loader_v2();
        let loader = loader.read().unwrap();
        
        // Get method_id from configuration
        let method_id = if let Some(config) = &loader.config {
            // Find library that provides this box type
            let (lib_name, _) = config.find_library_for_box(&plugin_box.box_type)
                .ok_or_else(|| RuntimeError::InvalidOperation { 
                    message: format!("No plugin provides box type: {}", plugin_box.box_type) 
                })?;
            
            // Get method_id from toml
            if let Ok(toml_content) = std::fs::read_to_string("nyash.toml") {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(box_config) = config.get_box_config(lib_name, &plugin_box.box_type, &toml_value) {
                        if let Some(method_config) = box_config.methods.get(method) {
                            eprintln!("🔍 Found method {} with id: {}", method, method_config.method_id);
                            method_config.method_id
                        } else {
                            return Err(RuntimeError::InvalidOperation { 
                                message: format!("Unknown method '{}' for {}", method, plugin_box.box_type) 
                            });
                        }
                    } else {
                        return Err(RuntimeError::InvalidOperation { 
                            message: format!("No configuration for box type: {}", plugin_box.box_type) 
                        });
                    }
                } else {
                    return Err(RuntimeError::InvalidOperation { 
                        message: "Failed to parse nyash.toml".into() 
                    });
                }
            } else {
                return Err(RuntimeError::InvalidOperation { 
                    message: "Failed to read nyash.toml".into() 
                });
            }
        } else {
            return Err(RuntimeError::InvalidOperation { 
                message: "No configuration loaded".into() 
            });
        };
        
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // Encode arguments using TLV (plugin's expected format)
        let mut tlv_data = Vec::new();
        
        // Header: version(2 bytes) + argc(2 bytes)
        tlv_data.extend_from_slice(&1u16.to_le_bytes()); // version = 1
        tlv_data.extend_from_slice(&(arg_values.len() as u16).to_le_bytes()); // argc
        
        // Encode each argument
        for arg in arg_values.iter() {
            // For now, convert all arguments to strings
            let arg_str = arg.to_string_box().value;
            let arg_bytes = arg_str.as_bytes();
            
            // TLV entry: tag(1) + reserved(1) + size(2) + data
            tlv_data.push(6);  // tag = 6 (String)
            tlv_data.push(0);  // reserved
            tlv_data.extend_from_slice(&(arg_bytes.len() as u16).to_le_bytes()); // size
            tlv_data.extend_from_slice(arg_bytes); // data
        }
        
        // Prepare output buffer
        let mut output_buffer = vec![0u8; 4096]; // 4KB buffer
        let mut output_len = output_buffer.len();
        
        eprintln!("🔍 Calling plugin invoke_fn: type_id={}, method_id={}, instance_id={}", 
                 plugin_box.type_id, method_id, plugin_box.instance_id);
        
        // Call plugin method
        let result = unsafe {
            (plugin_box.invoke_fn)(
                plugin_box.type_id,        // type_id from PluginBoxV2
                method_id,                  // method_id
                plugin_box.instance_id,     // instance_id
                tlv_data.as_ptr(),         // arguments
                tlv_data.len(),            // arguments length
                output_buffer.as_mut_ptr(), // output buffer
                &mut output_len,           // output length
            )
        };
        
        eprintln!("🔍 Plugin method returned: {}", result);
        
        if result != 0 {
            return Err(RuntimeError::RuntimeFailure { 
                message: format!("Plugin method {} failed with code: {}", method, result) 
            });
        }
        
        // Parse TLV output dynamically
        if output_len >= 4 {
            // Parse TLV header
            let version = u16::from_le_bytes([output_buffer[0], output_buffer[1]]);
            let argc = u16::from_le_bytes([output_buffer[2], output_buffer[3]]);
            
            eprintln!("🔍 TLV response: version={}, argc={}", version, argc);
            
            if version == 1 && argc > 0 && output_len >= 8 {
                // Parse first TLV entry
                let tag = output_buffer[4];
                let _reserved = output_buffer[5];
                let size = u16::from_le_bytes([output_buffer[6], output_buffer[7]]) as usize;
                
                eprintln!("🔍 TLV entry: tag={}, size={}", tag, size);
                
                if output_len >= 8 + size {
                    match tag {
                        2 => {
                            // I32 type
                            if size == 4 {
                                let value = i32::from_le_bytes([
                                    output_buffer[8], output_buffer[9], 
                                    output_buffer[10], output_buffer[11]
                                ]);
                                Ok(Box::new(IntegerBox::new(value as i64)))
                            } else {
                                Ok(Box::new(StringBox::new("ok")))
                            }
                        }
                        6 | 7 => {
                            // String or Bytes type
                            let data = &output_buffer[8..8+size];
                            let string = String::from_utf8_lossy(data).to_string();
                            Ok(Box::new(StringBox::new(string)))
                        }
                        9 => {
                            // Void type
                            Ok(Box::new(StringBox::new("ok")))
                        }
                        _ => {
                            // Unknown type, treat as string
                            eprintln!("🔍 Unknown TLV tag: {}", tag);
                            Ok(Box::new(StringBox::new("ok")))
                        }
                    }
                } else {
                    Ok(Box::new(StringBox::new("ok")))
                }
            } else {
                // No output, return void
                Ok(Box::new(VoidBox::new()))
            }
        } else {
            // No output, return void
            Ok(Box::new(VoidBox::new()))
        }
    }
}
