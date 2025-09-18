/*!
 * Field access operations
 */

// Removed super::* import - specific imports below
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, SharedNyashBox};
use crate::boxes::FutureBox;
use crate::instance_v2::InstanceBox;
use crate::interpreter::{NyashInterpreter, RuntimeError};
use std::sync::Arc;

// Conditional debug macro - only outputs if NYASH_DEBUG=1 environment variable is set
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        if std::env::var("NYASH_DEBUG").unwrap_or_default() == "1" {
            eprintln!($($arg)*);
        }
    };
}

impl NyashInterpreter {
    /// フィールドアクセスを実行 - Field access processing with weak reference support
    pub(super) fn execute_field_access(
        &mut self,
        object: &ASTNode,
        field: &str,
    ) -> Result<SharedNyashBox, RuntimeError> {
        // 🔥 Static Boxアクセスチェック
        if let ASTNode::Variable { name, .. } = object {
            // Static boxの可能性をチェック
            if self.is_static_box(name) {
                let static_result = self.execute_static_field_access(name, field)?;
                return Ok(Arc::from(static_result));
            }
        }

        // 外からのフィールドアクセスか（me/this以外）を判定
        let is_internal_access = match object {
            ASTNode::This { .. } | ASTNode::Me { .. } => true,
            ASTNode::Variable { name, .. } if name == "me" => true,
            _ => false,
        };

        // オブジェクトを評価（通常のフィールドアクセス）
        let obj_value = self.execute_expression(object);

        let obj_value = obj_value?;

        // InstanceBoxにキャスト
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // 可視性チェック（互換性: public/privateのどちらかが定義されていれば強制）
            if !is_internal_access {
                let box_decls = self.shared.box_declarations.read().unwrap();
                if let Some(box_decl) = box_decls.get(&instance.class_name) {
                    let has_visibility =
                        !box_decl.public_fields.is_empty() || !box_decl.private_fields.is_empty();
                    if has_visibility {
                        if !box_decl.public_fields.contains(&field.to_string()) {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!(
                                    "Field '{}' is private in {}",
                                    field, instance.class_name
                                ),
                            });
                        }
                    }
                }
            }
            // 🔥 finiは何回呼ばれてもエラーにしない（ユーザー要求）
            // is_finalized()チェックを削除

            // フィールドの値を取得
            let field_value = instance
                .get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in {}", field, instance.class_name),
                })?;

            // 🔗 Weak Reference Check: Use unified accessor for weak fields
            let box_decls = self.shared.box_declarations.read().unwrap();
            if let Some(box_decl) = box_decls.get(&instance.class_name) {
                if box_decl.weak_fields.contains(&field.to_string()) {
                    // 🎯 PHASE 2: Use unified accessor for auto-nil weak reference handling
                    if let Some(weak_value) = instance.get_weak_field(field, self) {
                        // Pass self
                        match &weak_value {
                            crate::value::NyashValue::Null => {
                                debug_trace!(
                                    "🔗 DEBUG: Weak field '{}' is null (reference dropped)",
                                    field
                                );
                                // Return null box for compatibility
                                return Ok(Arc::new(crate::boxes::null_box::NullBox::new()));
                            }
                            _ => {
                                debug_trace!(
                                    "🔗 DEBUG: Weak field '{}' still has valid reference",
                                    field
                                );
                                // Convert back to Box<dyn NyashBox> for now
                                if let Ok(box_value) = weak_value.to_box() {
                                    if let Ok(inner_box) = box_value.try_lock() {
                                        return Ok(Arc::from(inner_box.clone_or_share()));
                                    }
                                }
                            }
                        }
                    }
                    // If weak field access failed, fall through to normal access
                }
            }

            // Return the shared Arc reference directly
            Ok(field_value)
        } else {
            Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot access field '{}' on non-instance type. Type: {}",
                    field,
                    obj_value.type_name()
                ),
            })
        }
    }

    /// 🔥 Static Box名前空間のフィールドアクセス
    fn execute_static_field_access(
        &mut self,
        static_box_name: &str,
        field: &str,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 1. Static Boxの初期化を確実に実行
        self.ensure_static_box_initialized(static_box_name)?;

        // 2. GlobalBox.statics.{static_box_name} からインスタンスを取得
        let global_box =
            self.shared
                .global_box
                .lock()
                .map_err(|_| RuntimeError::RuntimeFailure {
                    message: "Failed to acquire global box lock".to_string(),
                })?;

        let statics_box = global_box
            .get_field("statics")
            .ok_or(RuntimeError::RuntimeFailure {
                message: "statics namespace not found in GlobalBox".to_string(),
            })?;

        let statics_instance =
            statics_box
                .as_any()
                .downcast_ref::<InstanceBox>()
                .ok_or(RuntimeError::TypeError {
                    message: "statics field is not an InstanceBox".to_string(),
                })?;

        let static_box_instance =
            statics_instance
                .get_field(static_box_name)
                .ok_or(RuntimeError::RuntimeFailure {
                    message: format!(
                        "Static box '{}' instance not found in statics namespace",
                        static_box_name
                    ),
                })?;

        let instance = static_box_instance
            .as_any()
            .downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: format!("Static box '{}' is not an InstanceBox", static_box_name),
            })?;

        // 3. フィールドアクセス
        let shared_field = instance
            .get_field(field)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!(
                    "Field '{}' not found in static box '{}'",
                    field, static_box_name
                ),
            })?;

        // Convert Arc to Box for compatibility
        Ok((*shared_field).clone_or_share())
    }

    /// await式を実行 - Execute await expression (Result.Ok/Err統一)
    pub(super) fn execute_await(
        &mut self,
        expression: &ASTNode,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value = self.execute_expression(expression)?;
        if let Some(future) = value.as_any().downcast_ref::<FutureBox>() {
            let max_ms: u64 = crate::config::env::await_max_ms();
            let start = std::time::Instant::now();
            let mut spins = 0usize;
            while !future.ready() {
                crate::runtime::global_hooks::safepoint_and_poll();
                std::thread::yield_now();
                spins += 1;
                if spins % 1024 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                    let err = Box::new(crate::box_trait::StringBox::new("Timeout"));
                    return Ok(Box::new(crate::boxes::result::NyashResultBox::new_err(err)));
                }
            }
            let v = future.get();
            Ok(Box::new(crate::boxes::result::NyashResultBox::new_ok(v)))
        } else {
            Ok(Box::new(crate::boxes::result::NyashResultBox::new_ok(
                value,
            )))
        }
    }
}
