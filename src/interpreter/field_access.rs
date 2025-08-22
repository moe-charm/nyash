/*!
 * Field Access Processing Module
 * 
 * Extracted from expressions.rs lines 901-1019 (~118 lines)
 * Handles field access for static boxes and instance boxes
 * Core philosophy: "Everything is Box" with unified field access
 */

use super::*;
use crate::box_trait::SharedNyashBox;
use std::sync::Arc;

impl NyashInterpreter {
    /// フィールドアクセスを実行 - static box と instance box の統一処理
    pub(super) fn execute_field_access(&mut self, object: &ASTNode, field: &str) 
        -> Result<SharedNyashBox, RuntimeError> {
        
        // 🔥 Static Boxアクセスチェック
        if let ASTNode::Variable { name, .. } = object {
            // Static boxの可能性をチェック
            if self.is_static_box(name) {
                let static_result = self.execute_static_field_access(name, field)?;
                return Ok(Arc::from(static_result));
            }
        }
        
        // オブジェクトを評価（通常のフィールドアクセス）  
        let obj_value = self.execute_expression(object)?;
        
        // InstanceBoxにキャスト
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            return self.execute_instance_field_access(instance, field);
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Cannot access field '{}' on type '{}'", field, obj_value.type_name()),
        })
    }

    /// Static Boxフィールドアクセス実行
    pub(super) fn execute_static_field_access(&mut self, box_name: &str, field: &str) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        let static_boxes = self.shared.static_boxes.read().unwrap();
        if let Some(static_box) = static_boxes.get(box_name) {
            let field_value = static_box.get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in static box '{}'", field, box_name),
                })?;
            
            Ok((*field_value).clone_or_share())
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Static box '{}' not found", box_name),
            })
        }
    }

    /// Instance Boxフィールドアクセス実行
    fn execute_instance_field_access(&mut self, instance: &InstanceBox, field: &str) 
        -> Result<SharedNyashBox, RuntimeError> {
        
        // 🔥 finiは何回呼ばれてもエラーにしない（ユーザー要求）
        // is_finalized()チェックを削除
        
        // フィールドの値を取得
        let field_value = instance.get_field(field)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Field '{}' not found in {}", field, instance.class_name),
            })?;
        
        eprintln!("✅ FIELD ACCESS: Returning shared reference id={}", field_value.box_id());
        
        // 🔗 Weak Reference Check: Use unified accessor for weak fields
        let is_weak_field = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            if let Some(box_decl) = box_decls.get(&instance.class_name) {
                box_decl.weak_fields.contains(&field.to_string())
            } else {
                false
            }
        };
        
        if is_weak_field {
            return self.handle_weak_field_access(instance, field);
        }
        
        // 通常のフィールドアクセス
        Ok(field_value)
    }

    /// Weak参照フィールドアクセス処理
    fn handle_weak_field_access(&mut self, instance: &InstanceBox, field: &str) 
        -> Result<SharedNyashBox, RuntimeError> {
        
        eprintln!("🔗 DEBUG: Accessing weak field '{}' in class '{}'", field, instance.class_name);
        
        // 🎯 PHASE 2: Use unified accessor for auto-nil weak reference handling
        if let Some(weak_value) = instance.get_weak_field(field, self) { // Pass self
            match &weak_value {
                crate::value::NyashValue::Null => {
                    eprintln!("🔗 DEBUG: Weak field '{}' is null (reference dropped)", field);
                    // Return null box for compatibility
                    Ok(Arc::new(crate::boxes::null_box::NullBox::new()))
                }
                _ => {
                    eprintln!("🔗 DEBUG: Weak field '{}' has live reference", field);
                    let converted_box = weak_value.to_nyash_box();
                    Ok(Arc::new(converted_box))
                }
            }
        } else {
            eprintln!("🔗 DEBUG: Weak field '{}' not found, falling back to normal access", field);
            // Fallback to normal field access if weak accessor fails
            let field_value = instance.get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in {}", field, instance.class_name),
                })?;
            Ok(field_value)
        }
    }

    /// Static Boxかどうかを判定
    pub(super) fn is_static_box(&self, name: &str) -> bool {
        let static_boxes = self.shared.static_boxes.read().unwrap();
        static_boxes.contains_key(name)
    }
}
