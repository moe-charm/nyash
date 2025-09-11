/*!
 * Method calls and from delegation calls
 */

use super::*;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, VoidBox};
use crate::boxes::MapBox;
use crate::boxes::{IntentBox, SocketBox};
use crate::boxes::{HTTPServerBox, HTTPRequestBox, HTTPResponseBox, DateTimeBox};
use crate::boxes::{RandomBox, SoundBox, DebugBox};
use crate::instance_v2::InstanceBox;
use crate::interpreter::{NyashInterpreter, RuntimeError};

// Debug macro gated by NYASH_DEBUG=1
macro_rules! idebug {
    ($($arg:tt)*) => {
        if crate::interpreter::utils::debug_on() { eprintln!($($arg)*); }
    };
}
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
                            self.declare_local_variable(param, value.clone_or_share());
                        }
                        
                        // static関数の本体を実行（TaskGroupスコープ）
                        crate::runtime::global_hooks::push_task_scope();
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
                        crate::runtime::global_hooks::pop_task_scope();
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
                            idebug!("🔍 Method '{}' not found in nyashstd.{}", method, name);
                            None
                        }
                    } else {
                        idebug!("🔍 Static box '{}' not found in nyashstd", name);
                        None
                    }
                } else {
                    idebug!("🔍 nyashstd namespace not found in stdlib");
                    None
                }
            } else {
                idebug!("🔍 stdlib not initialized for method call");
                None
            };
            
            if let Some(builtin_method) = stdlib_method {
                idebug!("🌟 Calling nyashstd method: {}.{}", name, method);
                
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_expression(arg)?);
                }
                
                // 標準ライブラリのメソッドを実行
                let result = builtin_method(&arg_values)?;
                idebug!("✅ nyashstd method completed: {}.{}", name, method);
                return Ok(result);
            }
            
            // 🔥 ユーザー定義のStatic Boxメソッドチェック
            if self.is_static_box(name) {
                idebug!("🔍 Checking user-defined static box: {}", name);
                
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
                
                idebug!("🌟 Calling static box method: {}.{}", name, method);
                
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
                        self.declare_local_variable(param, value.clone_or_share());
                    }
                    
                // メソッドの本体を実行（TaskGroupスコープ）
                crate::runtime::global_hooks::push_task_scope();
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
                crate::runtime::global_hooks::pop_task_scope();
                self.restore_local_vars(saved_locals);
                
                idebug!("✅ Static box method completed: {}.{}", name, method);
                return Ok(result);
            }
            }
        }
        
        // オブジェクトを評価（通常のメソッド呼び出し）
        let obj_value = self.execute_expression(object)?;
        idebug!("🔍 DEBUG: execute_method_call - object type: {}, method: {}", obj_value.type_name(), method);

        // 🌟 ユニバーサルメソッド前段ディスパッチ（非侵襲）
        // toString()/type()/equals(x)/clone() をトレイトに直結
        match method {
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("toString() expects 0 arguments, got {}", arguments.len()) });
                }
                return Ok(Box::new(obj_value.to_string_box()));
            }
            "type" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("type() expects 0 arguments, got {}", arguments.len()) });
                }
                return Ok(Box::new(StringBox::new(obj_value.type_name())));
            }
            "equals" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation { message: format!("equals() expects 1 argument, got {}", arguments.len()) });
                }
                let rhs = self.execute_expression(&arguments[0])?;
                let eq = obj_value.equals(&*rhs);
                return Ok(Box::new(eq));
            }
            "clone" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("clone() expects 0 arguments, got {}", arguments.len()) });
                }
                return Ok(obj_value.clone_box());
            }
            _ => {}
        }
        
        // Builtin dispatch (centralized)
        if let Some(res) = self.dispatch_builtin_method(&obj_value, method, arguments) {
            return res;
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
                idebug!("🔧 DEBUG: Stateful method '{}' called, updating stored instance", method);
                let updated_instance = socket_box.clone();
                idebug!("🔧 DEBUG: Updated instance created with ID={}", updated_instance.box_id());
                
                match object {
                    ASTNode::Variable { name, .. } => {
                        idebug!("🔧 DEBUG: Updating local variable '{}'", name);
                        // Handle local variables
                        if let Some(stored_var) = self.local_vars.get_mut(name) {
                            idebug!("🔧 DEBUG: Found local variable '{}', updating from id={} to id={}", 
                                     name, stored_var.box_id(), updated_instance.box_id());
                            *stored_var = Arc::new(updated_instance);
                        } else {
                            idebug!("🔧 DEBUG: Local variable '{}' not found", name);
                        }
                    },
                    ASTNode::FieldAccess { object: field_obj, field, .. } => {
                        idebug!("🔧 DEBUG: Updating field access '{}'", field);
                        // Handle StaticBox fields like me.server
                        match field_obj.as_ref() {
                            ASTNode::Variable { name, .. } => {
                                idebug!("🔧 DEBUG: Field object is variable '{}'", name);
                                if name == "me" {
                                    idebug!("🔧 DEBUG: Updating me.{} (via variable)", field);
                                    if let Ok(me_instance) = self.resolve_variable("me") {
                                        idebug!("🔧 DEBUG: Resolved 'me' instance id={}", me_instance.box_id());
                                        if let Some(instance) = (*me_instance).as_any().downcast_ref::<InstanceBox>() {
                                            idebug!("🔧 DEBUG: me is InstanceBox, setting field '{}' to updated instance id={}", field, updated_instance.box_id());
                                            let result = instance.set_field(field, Arc::new(updated_instance));
                                            idebug!("🔧 DEBUG: set_field result: {:?}", result);
                                        } else {
                                            idebug!("🔧 DEBUG: me is not an InstanceBox, type: {}", me_instance.type_name());
                                        }
                                    } else {
                                        idebug!("🔧 DEBUG: Failed to resolve 'me'");
                                    }
                                } else {
                                    idebug!("🔧 DEBUG: Field object is not 'me', it's '{}'", name);
                                }
                            },
                            ASTNode::Me { .. } => {
                                idebug!("🔧 DEBUG: Field object is Me node, updating me.{}", field);
                                if let Ok(me_instance) = self.resolve_variable("me") {
                                    idebug!("🔧 DEBUG: Resolved 'me' instance id={}", me_instance.box_id());
                                    if let Some(instance) = (*me_instance).as_any().downcast_ref::<InstanceBox>() {
                                        idebug!("🔧 DEBUG: me is InstanceBox, setting field '{}' to updated instance id={}", field, updated_instance.box_id());
                                        let result = instance.set_field(field, Arc::new(updated_instance));
                                        idebug!("🔧 DEBUG: set_field result: {:?}", result);
                                    } else {
                                        idebug!("🔧 DEBUG: me is not an InstanceBox, type: {}", me_instance.type_name());
                                    }
                                } else {
                                    idebug!("🔧 DEBUG: Failed to resolve 'me'");
                                }
                            },
                            _ => {
                                idebug!("🔧 DEBUG: Field object is not a variable or me, type: {:?}", field_obj);
                            }
                        }
                    },
                    _ => {
                        idebug!("🔧 DEBUG: Object type not handled: {:?}", object);
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
        
        // P2PBox method calls
        if let Some(p2p_box) = obj_value.as_any().downcast_ref::<crate::boxes::P2PBox>() {
            return self.execute_p2p_box_method(p2p_box, method, arguments);
        }
        
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
        
        // InstanceBox dispatch
        if let Some(res) = self.dispatch_instance_method(object, &obj_value, method, arguments) { return res; }
        idebug!("🔍 DEBUG: Reached non-instance type error for type: {}, method: {}", obj_value.type_name(), method);
        Err(RuntimeError::TypeError { message: format!("Cannot call method '{}' on non-instance type", method) })
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
        // ここでは短期ロックで必要な情報だけ抜き出してすぐ解放する
        let (has_parent_in_ext, has_parent_in_impl) = {
            let box_declarations = self.shared.box_declarations.read().unwrap();
            let current_box_decl = box_declarations.get(current_class)
                .ok_or(RuntimeError::UndefinedClass { name: current_class.clone() })?;
            (current_box_decl.extends.contains(&parent.to_string()),
             current_box_decl.implements.contains(&parent.to_string()))
        };
        // extendsまたはimplementsでparentが指定されているか確認 (Multi-delegation) 🚀
        let is_valid_delegation = has_parent_in_ext || has_parent_in_impl;
        
        if !is_valid_delegation {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Class '{}' does not delegate to '{}'. Use 'box {} from {}' to establish delegation.", 
                               current_class, parent, current_class, parent),
            });
        }
        
        // 先にプラグイン親のコンストラクタ/メソッドを優先的に処理（v2プラグイン対応）
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            // 親がプラグインで提供されているかを確認
            if self.is_plugin_box_type(parent) {
                // コンストラクタ相当（birth もしくは 親名と同名）の場合は、
                // プラグインBoxを生成して __plugin_content に格納
                if method == "birth" || method == parent {
                    match self.create_plugin_box(parent, arguments) {
                        Ok(pbox) => {
                            use std::sync::Arc;
                            let _ = current_instance.set_field_legacy("__plugin_content", Arc::from(pbox));
                            return Ok(Box::new(crate::box_trait::VoidBox::new()));
                        }
                        Err(e) => {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!("Failed to construct plugin parent '{}': {:?}", parent, e),
                            });
                        }
                    }
                } else {
                    // 非コンストラクタ: 既存の __plugin_content を通じてメソッド呼び出し
                    if let Some(plugin_shared) = current_instance.get_field_legacy("__plugin_content") {
                        let plugin_ref = &*plugin_shared;
                        if let Some(plugin) = plugin_ref.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                            return self.execute_plugin_box_v2_method(plugin, method, arguments);
                        }
                    }
                }
            }
        }

        // 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定
        use crate::box_trait::is_builtin_box;
        // GUI機能が有効な場合はEguiBoxも追加判定（mut不要の形に）
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        let is_builtin = is_builtin_box(parent) || parent == "EguiBox";
        #[cfg(not(all(feature = "gui", not(target_arch = "wasm32"))))]
        let is_builtin = is_builtin_box(parent);
        
        // 🔥 Phase 8.9: Transparency system removed - all delegation must be explicit
        // Removed: if is_builtin && method == parent { ... execute_builtin_constructor_call ... }
        
        if is_builtin {
            // ビルトインBoxの場合、直接ビルトインメソッドを実行
            return self.execute_builtin_box_method(parent, method, current_instance_val.clone_box(), arguments);
        }
        
        // プラグイン親（__plugin_content）
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            if let Some(plugin_shared) = current_instance.get_field_legacy("__plugin_content") {
                let plugin_ref = &*plugin_shared;
                if let Some(plugin) = plugin_ref.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    return self.execute_plugin_box_v2_method(plugin, method, arguments);
                }
            }
        }
        
        // 3. 親クラスのBox宣言を取得（ユーザー定義Boxの場合）
        let parent_box_decl = {
            let box_declarations = self.shared.box_declarations.read().unwrap();
            box_declarations.get(parent)
            .ok_or(RuntimeError::UndefinedClass { 
                name: parent.to_string() 
            })?
            .clone()
        };
        
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
            self.declare_local_variable("me", current_instance_val.clone_or_share());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_or_share());
            }
            
            // 親メソッドの本体を実行（TaskGroupスコープ）
            crate::runtime::global_hooks::push_task_scope();
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
            idebug!("🔍 DEBUG: FromCall {}.{} result: {}", parent, method, result.to_string_box().value);
            
            // local変数スタックを復元
            crate::runtime::global_hooks::pop_task_scope();
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
            self.declare_local_variable("me", current_instance.clone_or_share());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_or_share());
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
        self.call_plugin_method(plugin_box, method, arguments)
    }
}
