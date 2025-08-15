/*!
 * Field Access Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles field access for instances and static boxes
 * Core philosophy: "Everything is Box" with weak reference support
 */

use super::*;
use crate::box_trait::{BoolBox, SharedNyashBox};
use std::sync::Arc;

impl NyashInterpreter {
    /// フィールドアクセスを実行 - Field access processing with weak reference support
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
        let obj_value = self.execute_expression(object);
        
        let obj_value = obj_value?;
        
        // InstanceBoxにキャスト
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // 🔥 Usage prohibition guard - check if instance is finalized
            if instance.is_finalized() {
                return Err(RuntimeError::InvalidOperation {
                    message: "Instance was finalized; further use is prohibited".to_string(),
                });
            }
            
            // フィールドの値を取得
            let field_value = instance.get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in {}", field, instance.class_name),
                })?;
            
            eprintln!("✅ FIELD ACCESS: Returning shared reference id={}", field_value.box_id());
            
            // 🔗 Weak Reference Check: Use unified accessor for weak fields
            let box_decls = self.shared.box_declarations.read().unwrap();
            if let Some(box_decl) = box_decls.get(&instance.class_name) {
                if box_decl.weak_fields.contains(&field.to_string()) {
                    eprintln!("🔗 DEBUG: Accessing weak field '{}' in class '{}'", field, instance.class_name);
                    
                    // 🎯 PHASE 2: Use unified accessor for auto-nil weak reference handling
                    if let Some(weak_value) = instance.get_weak_field(field, self) { // Pass self
                        match &weak_value {
                            crate::value::NyashValue::Null => {
                                eprintln!("🔗 DEBUG: Weak field '{}' is null (reference dropped)", field);
                                return Ok(Arc::new(Box::new(crate::boxes::null_box::NullBox::new())));
                            }
                            _ => {
                                eprintln!("🔗 DEBUG: Weak field '{}' resolved to: {:?}", field, weak_value.type_name());
                                return Ok(Arc::new(weak_value.to_nyash_box()));
                            }
                        }
                    } else {
                        eprintln!("🔗 DEBUG: Weak field '{}' accessor returned None", field);
                        return Ok(Arc::new(Box::new(crate::boxes::null_box::NullBox::new())));
                    }
                }
            }
            
            Ok(field_value)
        } else {
            Err(RuntimeError::TypeError {
                message: format!("Cannot access field '{}' on non-instance type. Type: {}", field, obj_value.type_name()),
            })
        }
    }

    /// 🔥 Static Box名前空間のフィールドアクセス
    fn execute_static_field_access(&mut self, static_box_name: &str, field: &str) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🔥 Static Box初期化済みかチェック
        let is_initialized = {
            let static_boxes = self.shared.static_boxes.read().unwrap();
            if let Some(static_def) = static_boxes.get(static_box_name) {
                static_def.initialization_state == StaticBoxState::Initialized
            } else {
                false
            }
        };
        
        if !is_initialized {
            // まだ初期化されていない場合は初期化を実行
            eprintln!("🔥 DEBUG: Static box '{}' not initialized, initializing now", static_box_name);
            self.initialize_static_box(static_box_name)?;
        }
        
        // Static Box フィールドから値を取得
        let shared_field = {
            let static_instances = self.shared.static_instances.read().unwrap();
            static_instances.get(static_box_name)
                .and_then(|fields| fields.get(field))
                .cloned()
        };
        
        let shared_field = shared_field.ok_or(RuntimeError::InvalidOperation {
                message: format!("Field '{}' not found in static box '{}'", field, static_box_name),
            })?;
        
        // Convert Arc to Box for compatibility
        Ok((*shared_field).clone_box())
    }
}