//! Central builtin method dispatcher (thin wrapper)

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, BoxCore};
use crate::boxes::{ArrayBox, FloatBox, BufferBox, ResultBox, FutureBox, JSONBox, HttpClientBox, StreamBox, RegexBox, MathBox};
use crate::boxes::{null_box, time_box, map_box, random_box, sound_box, debug_box, console_box};
use crate::boxes::{gc_config_box::GcConfigBox, debug_config_box::DebugConfigBox};
use crate::boxes::file;
use crate::channel_box::ChannelBox;
use super::{NyashInterpreter, RuntimeError};

impl NyashInterpreter {
    /// Try dispatching a builtin method based on dynamic type.
    /// Returns Some(Result) if handled, or None to let caller continue other paths.
    pub(crate) fn dispatch_builtin_method(
        &mut self,
        obj: &Box<dyn NyashBox>,
        method: &str,
        arguments: &[ASTNode],
    ) -> Option<Result<Box<dyn NyashBox>, RuntimeError>> {
        // StringBox
        if let Some(b) = obj.as_any().downcast_ref::<StringBox>() {
            return Some(self.execute_string_method(b, method, arguments));
        }
        // IntegerBox
        if let Some(b) = obj.as_any().downcast_ref::<IntegerBox>() {
            return Some(self.execute_integer_method(b, method, arguments));
        }
        // FloatBox
        if let Some(b) = obj.as_any().downcast_ref::<FloatBox>() {
            return Some(self.execute_float_method(b, method, arguments));
        }
        // BoolBox
        if let Some(b) = obj.as_any().downcast_ref::<BoolBox>() {
            return Some(self.execute_bool_method(b, method, arguments));
        }
        // ArrayBox
        if let Some(b) = obj.as_any().downcast_ref::<ArrayBox>() {
            return Some(self.execute_array_method(b, method, arguments));
        }
        // BufferBox
        if let Some(b) = obj.as_any().downcast_ref::<BufferBox>() {
            return Some(self.execute_buffer_method(b, method, arguments));
        }
        // FileBox
        if let Some(b) = obj.as_any().downcast_ref::<file::FileBox>() {
            return Some(self.execute_file_method(b, method, arguments));
        }
        // ResultBox
        if let Some(b) = obj.as_any().downcast_ref::<ResultBox>() {
            return Some(self.execute_result_method(b, method, arguments));
        }
        // FutureBox
        if let Some(b) = obj.as_any().downcast_ref::<FutureBox>() {
            return Some(self.execute_future_method(b, method, arguments));
        }
        // ChannelBox
        if let Some(b) = obj.as_any().downcast_ref::<ChannelBox>() {
            return Some(self.execute_channel_method(b, method, arguments));
        }
        // JSONBox
        if let Some(b) = obj.as_any().downcast_ref::<JSONBox>() {
            return Some(self.execute_json_method(b, method, arguments));
        }
        // HttpClientBox
        if let Some(b) = obj.as_any().downcast_ref::<HttpClientBox>() {
            return Some(self.execute_http_method(b, method, arguments));
        }
        // StreamBox
        if let Some(b) = obj.as_any().downcast_ref::<StreamBox>() {
            return Some(self.execute_stream_method(b, method, arguments));
        }
        // RegexBox
        if let Some(b) = obj.as_any().downcast_ref::<RegexBox>() {
            return Some(self.execute_regex_method(b, method, arguments));
        }
        // MathBox
        if let Some(b) = obj.as_any().downcast_ref::<MathBox>() {
            return Some(self.execute_math_method(b, method, arguments));
        }
        // NullBox
        if let Some(b) = obj.as_any().downcast_ref::<null_box::NullBox>() {
            return Some(self.execute_null_method(b, method, arguments));
        }
        // TimeBox
        if let Some(b) = obj.as_any().downcast_ref::<time_box::TimeBox>() {
            return Some(self.execute_time_method(b, method, arguments));
        }
        // TimerBox
        if let Some(b) = obj.as_any().downcast_ref::<time_box::TimerBox>() {
            return Some(self.execute_timer_method(b, method, arguments));
        }
        // MapBox
        if let Some(b) = obj.as_any().downcast_ref::<map_box::MapBox>() {
            return Some(self.execute_map_method(b, method, arguments));
        }
        // RandomBox
        if let Some(b) = obj.as_any().downcast_ref::<random_box::RandomBox>() {
            return Some(self.execute_random_method(b, method, arguments));
        }
        // SoundBox
        if let Some(b) = obj.as_any().downcast_ref::<sound_box::SoundBox>() {
            return Some(self.execute_sound_method(b, method, arguments));
        }
        // DebugBox
        if let Some(b) = obj.as_any().downcast_ref::<debug_box::DebugBox>() {
            return Some(self.execute_debug_method(b, method, arguments));
        }
        // ConsoleBox
        if let Some(b) = obj.as_any().downcast_ref::<console_box::ConsoleBox>() {
            return Some(self.execute_console_method(b, method, arguments));
        }
        // GcConfigBox
        if let Some(b) = obj.as_any().downcast_ref::<GcConfigBox>() {
            return Some(self.execute_gc_config_method(b, method, arguments));
        }
        // DebugConfigBox
        if let Some(b) = obj.as_any().downcast_ref::<DebugConfigBox>() {
            return Some(self.execute_debug_config_method(b, method, arguments));
        }

        None
    }

    /// Dispatch user-defined instance methods (InstanceBox path).
    /// Returns Some(Result) if handled, or None if obj is not an InstanceBox.
    pub(crate) fn dispatch_instance_method(
        &mut self,
        object_ast: &ASTNode,
        obj_value: &Box<dyn NyashBox>,
        method: &str,
        arguments: &[ASTNode],
    ) -> Option<Result<Box<dyn NyashBox>, RuntimeError>> {
        use crate::box_trait::{StringBox, IntegerBox};
        use crate::boxes::MathBox;
        use crate::instance_v2::InstanceBox;
        use crate::finalization;

        let instance = match obj_value.as_any().downcast_ref::<InstanceBox>() {
            Some(i) => i,
            None => return None,
        };

        // fini() special handling (idempotent, weak prohibition)
        if method == "fini" {
            // weak-fini prohibition check: me.<weak_field>.fini()
            if let ASTNode::FieldAccess { object: field_object, field, .. } = object_ast {
                if let ASTNode::Variable { name, .. } = field_object.as_ref() {
                    if name == "me" {
                        if let Ok(current_me) = self.resolve_variable("me") {
                            if let Some(current_instance) = (*current_me).as_any().downcast_ref::<InstanceBox>() {
                                if current_instance.is_weak_field(field) {
                                    return Some(Err(RuntimeError::InvalidOperation {
                                        message: format!(
                                            "Cannot finalize weak field '{}' (non-owning reference)",
                                            field
                                        ),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            if instance.is_finalized() {
                return Some(Ok(Box::new(crate::box_trait::VoidBox::new())));
            }
            if let Some(fini_method) = instance.get_method("fini") {
                if let ASTNode::FunctionDeclaration { body, .. } = fini_method.clone() {
                    let saved = self.save_local_vars();
                    self.local_vars.clear();
                    self.declare_local_variable("me", obj_value.clone_or_share());
                    let mut _result = Box::new(crate::box_trait::VoidBox::new()) as Box<dyn NyashBox>;
                    for statement in &body {
                        match self.execute_statement(statement) {
                            Ok(v) => { _result = v; },
                            Err(e) => { self.restore_local_vars(saved); return Some(Err(e)); }
                        }
                        if let super::ControlFlow::Return(_) = &self.control_flow {
                            self.control_flow = super::ControlFlow::None;
                            break;
                        }
                    }
                    self.restore_local_vars(saved);
                }
            }
            let target_info = obj_value.to_string_box().value;
            self.trigger_weak_reference_invalidation(&target_info);
            if let Err(e) = instance.fini() { return Some(Err(RuntimeError::InvalidOperation { message: e })); }
            finalization::mark_as_finalized(instance.box_id());
            return Some(Ok(Box::new(crate::box_trait::VoidBox::new())));
        }

        // Local method on instance
        if let Some(method_ast) = instance.get_method(method) {
            if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast.clone() {
                // Evaluate args in current context
                let mut arg_values = Vec::new();
                for a in arguments {
                    match self.execute_expression(a) {
                        Ok(v) => arg_values.push(v),
                        Err(e) => return Some(Err(e)),
                    }
                }
                if arg_values.len() != params.len() {
                    return Some(Err(RuntimeError::InvalidOperation {
                        message: format!("Method {} expects {} arguments, got {}", method, params.len(), arg_values.len()),
                    }));
                }
                let saved = self.save_local_vars();
                self.local_vars.clear();
                self.declare_local_variable("me", obj_value.clone_or_share());
                for (p, v) in params.iter().zip(arg_values.iter()) {
                    self.declare_local_variable(p, v.clone_or_share());
                }
                let mut result: Box<dyn NyashBox> = Box::new(crate::box_trait::VoidBox::new());
                for stmt in &body {
                    match self.execute_statement(stmt) {
                        Ok(v) => { result = v; },
                        Err(e) => return Some(Err(e)),
                    }
                    if let super::ControlFlow::Return(ret) = &self.control_flow {
                        result = ret.clone_box();
                        self.control_flow = super::ControlFlow::None;
                        break;
                    }
                }
                self.restore_local_vars(saved);
                return Some(Ok(result));
            } else {
                return Some(Err(RuntimeError::InvalidOperation { message: format!("Method '{}' is not a valid function declaration", method) }));
            }
        }

        // Builtin parent method promotion (StringBox/IntegerBox/MathBox)
        let parent_names = {
            let decls = self.shared.box_declarations.read().unwrap();
            decls.get(&instance.class_name).map(|d| d.extends.clone()).unwrap_or_default()
        };
        for parent_name in &parent_names {
            if crate::box_trait::is_builtin_box(parent_name) {
                if parent_name == "StringBox" {
                    if let Some(builtin_value) = instance.get_field_ng("__builtin_content") {
                        if let crate::value::NyashValue::Box(boxed) = builtin_value {
                            let g = boxed.lock().unwrap();
                            if let Some(sb) = g.as_any().downcast_ref::<StringBox>() {
                                return Some(self.execute_string_method(sb, method, arguments));
                            }
                        }
                    }
                    let sb = StringBox::new("");
                    return Some(self.execute_string_method(&sb, method, arguments));
                } else if parent_name == "IntegerBox" {
                    if let Some(builtin_value) = instance.get_field_ng("__builtin_content") {
                        if let crate::value::NyashValue::Box(boxed) = builtin_value {
                            let g = boxed.lock().unwrap();
                            if let Some(ib) = g.as_any().downcast_ref::<IntegerBox>() {
                                return Some(self.execute_integer_method(ib, method, arguments));
                            }
                        }
                    }
                    let ib = IntegerBox::new(0);
                    return Some(self.execute_integer_method(&ib, method, arguments));
                } else if parent_name == "MathBox" {
                    let math_box = MathBox::new();
                    return Some(self.execute_math_method(&math_box, method, arguments));
                }
            }
        }

        // Plugin parent via __plugin_content
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            if let Some(plugin_shared) = instance.get_field_legacy("__plugin_content") {
                let plugin_ref = &*plugin_shared;
                if let Some(plugin) = plugin_ref.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    return Some(self.call_plugin_method(plugin, method, arguments));
                }
            }
        }

        // Not handled here
        Some(Err(RuntimeError::InvalidOperation { message: format!("Method '{}' not found in {}", method, instance.class_name) }))
    }
}
