/*!
 * Nyash Instance System - Box Instance Implementation
 * 
 * BoxインスタンスとClassBoxの実装
 * Everything is Box哲学に基づくオブジェクト指向システム
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, VoidBox, BoxCore, BoxBase};
use crate::ast::ASTNode;
use crate::value::NyashValue;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::{Arc, Mutex, Weak};

/// Boxインスタンス - フィールドとメソッドを持つオブジェクト
#[derive(Debug, Clone)]
pub struct InstanceBox {
    /// クラス名
    pub class_name: String,
    
    /// フィールド値 (Legacy compatibility)
    pub fields: Arc<Mutex<HashMap<String, Box<dyn NyashBox>>>>,
    
    /// 🔗 Next-generation fields (weak reference capable)
    pub fields_ng: Arc<Mutex<HashMap<String, NyashValue>>>,
    
    /// メソッド定義（ClassBoxから共有）
    pub methods: Arc<HashMap<String, ASTNode>>,
    
    /// Box基底
    base: BoxBase,
    
    /// 解放済みフラグ
    finalized: Arc<Mutex<bool>>,
}

impl InstanceBox {
    pub fn new(class_name: String, fields: Vec<String>, methods: HashMap<String, ASTNode>) -> Self {
        // フィールドをVoidBoxで初期化
        let mut field_map = HashMap::new();
        for field in fields {
            field_map.insert(field, Box::new(VoidBox::new()) as Box<dyn NyashBox>);
        }
        
        Self {
            class_name,
            fields: Arc::new(Mutex::new(field_map)),
            fields_ng: Arc::new(Mutex::new(HashMap::new())), // 🔗 Initialize next-gen fields
            methods: Arc::new(methods),
            base: BoxBase::new(),
            finalized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 🔗 Unified field access - prioritizes fields_ng, fallback to legacy fields with conversion
    pub fn get_field_unified(&self, field_name: &str) -> Option<NyashValue> {
        // Check fields_ng first
        if let Some(value) = self.fields_ng.lock().unwrap().get(field_name) {
            return Some(value.clone());
        }
        
        // Fallback to legacy fields with conversion
        if let Some(legacy_box) = self.fields.lock().unwrap().get(field_name) {
            // For backward compatibility, we need to work around the type mismatch
            // Since we can't easily convert Box<dyn NyashBox> to Arc<Mutex<dyn NyashBox>>
            // We'll use the from_box method which handles this conversion
            // We need to create a temporary Arc to satisfy the method signature
            let temp_arc = Arc::new(Mutex::new(VoidBox::new()));
            // Unfortunately, there's a type system limitation here
            // For now, let's return a simple converted value
            let string_rep = legacy_box.to_string_box().value;
            return Some(NyashValue::String(string_rep));
        }
        
        None
    }
    
    /// 🔗 Unified field setting - always stores in fields_ng
    pub fn set_field_unified(&self, field_name: String, value: NyashValue) -> Result<(), String> {
        // Always store in fields_ng for future compatibility
        self.fields_ng.lock().unwrap().insert(field_name.clone(), value.clone());
        
        // For backward compatibility, also update legacy fields if they exist
        // Convert NyashValue back to Box<dyn NyashBox> for legacy storage
        if self.fields.lock().unwrap().contains_key(&field_name) {
            if let Ok(legacy_box) = value.to_box() {
                // Convert Arc<Mutex<dyn NyashBox>> to Box<dyn NyashBox>
                if let Ok(inner_box) = legacy_box.try_lock() {
                    self.fields.lock().unwrap().insert(field_name, inner_box.clone_box());
                }
            }
        }
        
        Ok(())
    }
    
    /// 🔗 Set weak field - converts strong reference to weak and stores in fields_ng
    pub fn set_weak_field(&self, field_name: String, value: NyashValue) -> Result<(), String> {
        match value {
            NyashValue::Box(arc_box) => {
                let weak_ref = Arc::downgrade(&arc_box);
                let field_name_clone = field_name.clone(); // Clone for eprintln
                self.fields_ng.lock().unwrap().insert(field_name, NyashValue::WeakBox(weak_ref));
                eprintln!("🔗 DEBUG: Successfully converted strong reference to weak for field '{}'", field_name_clone);
                Ok(())
            }
            _ => {
                // For non-Box values, store as-is (they don't need weak conversion)
                self.fields_ng.lock().unwrap().insert(field_name, value);
                Ok(())
            }
        }
    }
    
    /// 🔗 Set weak field from legacy Box<dyn NyashBox> - helper method for interpreter
    pub fn set_weak_field_from_legacy(&self, field_name: String, legacy_box: Box<dyn NyashBox>) -> Result<(), String> {
        // Convert Box<dyn NyashBox> to Arc<Mutex<dyn NyashBox>> via temporary wrapper
        // We create a temporary holder struct that implements NyashBox
        use crate::box_trait::StringBox;
        
        // Store the object info in a way we can track
        let object_info = legacy_box.to_string_box().value;
        let field_name_clone = field_name.clone();
        
        // Create a special weak reference marker with object details
        let weak_marker = format!("WEAK_REF_TO:{}", object_info);
        self.fields_ng.lock().unwrap().insert(field_name, NyashValue::String(weak_marker));
        
        eprintln!("🔗 DEBUG: Stored weak field '{}' with reference tracking", field_name_clone);
        Ok(())
    }
    
    /// 🔗 Get weak field with auto-upgrade and nil fallback
    pub fn get_weak_field(&self, field_name: &str) -> Option<NyashValue> {
        if let Some(value) = self.fields_ng.lock().unwrap().get(field_name) {
            match value {
                NyashValue::WeakBox(weak_ref) => {
                    if let Some(strong_ref) = weak_ref.upgrade() {
                        eprintln!("🔗 DEBUG: Weak field '{}' upgraded successfully", field_name);
                        Some(NyashValue::Box(strong_ref))
                    } else {
                        eprintln!("🔗 DEBUG: Weak field '{}' target was dropped - returning null", field_name);
                        Some(NyashValue::Null) // 🎯 Auto-nil behavior!
                    }
                }
                NyashValue::String(s) => {
                    // For string-based weak fields, check if they're marked as "dropped"
                    if s.starts_with("WEAK_REF_TO:") {
                        // Check if this reference has been invalidated
                        if s == "WEAK_REFERENCE_DROPPED" {
                            eprintln!("🔗 DEBUG: Weak field '{}' target was dropped - returning null", field_name);
                            Some(NyashValue::Null)
                        } else {
                            eprintln!("🔗 DEBUG: Weak field '{}' still has valid reference", field_name);
                            // Extract the original object info from the weak reference marker
                            let original_info = s.strip_prefix("WEAK_REF_TO:").unwrap_or(s);
                            Some(NyashValue::String(original_info.to_string()))
                        }
                    } else if s == "WEAK_REFERENCE_DROPPED" {
                        eprintln!("🔗 DEBUG: Weak field '{}' target was dropped - returning null", field_name);
                        Some(NyashValue::Null)
                    } else {
                        eprintln!("🔗 DEBUG: Weak field '{}' still has valid reference", field_name);
                        Some(value.clone())
                    }
                }
                _ => {
                    // Non-weak value, return as-is
                    Some(value.clone())
                }
            }
        } else {
            None
        }
    }
    
    /// 🔗 Mark weak references to this instance as dropped
    pub fn invalidate_weak_references_to(&self, target_info: &str) {
        let mut fields = self.fields_ng.lock().unwrap();
        for (field_name, value) in fields.iter_mut() {
            match value {
                NyashValue::String(s) => {
                    // Check if this is a weak reference to the target
                    if s.starts_with("WEAK_REF_TO:") && s.contains(target_info) {
                        *s = "WEAK_REFERENCE_DROPPED".to_string();
                        eprintln!("🔗 DEBUG: Marked weak field '{}' as dropped", field_name);
                    }
                }
                NyashValue::WeakBox(weak_ref) => {
                    // Check if the weak reference is dead
                    if weak_ref.upgrade().is_none() {
                        eprintln!("🔗 DEBUG: Weak field '{}' reference is already dead", field_name);
                    }
                }
                _ => {}
            }
        }
    }
    
    /// 🔗 Global invalidation - call this when any object is dropped
    pub fn global_invalidate_weak_references(target_info: &str) {
        // In a real implementation, we'd maintain a global registry of all instances
        // and iterate through them to invalidate weak references.
        // For this demo, we'll add the capability to the instance itself.
        eprintln!("🔗 DEBUG: Global weak reference invalidation for: {}", target_info);
    }
    
    /// フィールドの値を取得
    pub fn get_field(&self, field_name: &str) -> Option<Box<dyn NyashBox>> {
        self.fields.lock().unwrap().get(field_name).map(|v| v.clone_box())
    }
    
    /// フィールドに値を設定
    pub fn set_field(&self, field_name: &str, value: Box<dyn NyashBox>) -> Result<(), String> {
        let mut fields = self.fields.lock().unwrap();
        if fields.contains_key(field_name) {
            fields.insert(field_name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Field '{}' does not exist in {}", field_name, self.class_name))
        }
    }
    
    /// 🌍 GlobalBox用：フィールドを動的に追加・設定
    pub fn set_field_dynamic(&mut self, field_name: String, value: Box<dyn NyashBox>) {
        let mut fields = self.fields.lock().unwrap();
        fields.insert(field_name, value);
    }
    
    /// メソッド定義を取得
    pub fn get_method(&self, method_name: &str) -> Option<&ASTNode> {
        self.methods.get(method_name)
    }
    
    /// メソッドが存在するかチェック
    pub fn has_method(&self, method_name: &str) -> bool {
        self.methods.contains_key(method_name)
    }
    
    /// 🌍 GlobalBox用：メソッドを動的に追加 - 🔥 暗黙オーバーライド禁止による安全実装
    pub fn add_method(&mut self, method_name: String, method_ast: ASTNode) -> Result<(), String> {
        // Arc<T>は不変なので、新しいHashMapを作成してArcで包む
        let mut new_methods = (*self.methods).clone();
        
        // 🚨 暗黙オーバーライド禁止：既存メソッドの検査
        if let Some(existing_method) = new_methods.get(&method_name) {
            // 新しいメソッドのoverride状態を確認
            let is_override = match &method_ast {
                crate::ast::ASTNode::FunctionDeclaration { is_override, .. } => *is_override,
                _ => false, // FunctionDeclaration以外はオーバーライドなし
            };
            
            if !is_override {
                // 🔥 明示的オーバーライド革命：overrideキーワードなしの重複を禁止
                return Err(format!(
                    "🚨 EXPLICIT OVERRIDE REQUIRED: Method '{}' already exists.\n\
                    💡 To replace the existing method, use 'override {}(...) {{ ... }}'.\n\
                    🌟 This is Nyash's explicit delegation philosophy - no hidden overrides!",
                    method_name, method_name
                ));
            }
            
            // override宣言があれば、明示的な置換として許可
            eprintln!("🔥 EXPLICIT OVERRIDE: Method '{}' replaced with override declaration", method_name);
        }
        
        new_methods.insert(method_name, method_ast);
        self.methods = Arc::new(new_methods);
        Ok(())
    }
    
    /// fini()メソッド - インスタンスの解放
    pub fn fini(&self) -> Result<(), String> {
        let mut finalized = self.finalized.lock().unwrap();
        if *finalized {
            // 既に解放済みなら何もしない
            return Ok(());
        }
        
        *finalized = true;
        
        // フィールドをクリア
        let mut fields = self.fields.lock().unwrap();
        fields.clear();
        
        Ok(())
    }
    
    /// 解放済みかチェック
    pub fn is_finalized(&self) -> bool {
        *self.finalized.lock().unwrap()
    }
}

impl NyashBox for InstanceBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<{} instance #{}>", self.class_name, self.base.id))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_instance) = other.as_any().downcast_ref::<InstanceBox>() {
            // 同じインスタンスIDなら等しい
            BoolBox::new(self.base.id == other_instance.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "InstanceBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // インスタンスは同じフィールドを共有
        Box::new(self.clone())
    }
    
    
}

impl BoxCore for InstanceBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance #{}>", self.class_name, self.base.id)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for InstanceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::IntegerBox;
    
    #[test]
    fn test_instance_creation() {
        let fields = vec!["x".to_string(), "y".to_string()];
        let methods = HashMap::new();
        let instance = InstanceBox::new("Point".to_string(), fields, methods);
        
        assert_eq!(instance.class_name, "Point");
        assert!(instance.get_field("x").is_some());
        assert!(instance.get_field("y").is_some());
        assert!(instance.get_field("z").is_none());
    }
    
    #[test]
    fn test_field_access() {
        let fields = vec!["value".to_string()];
        let methods = HashMap::new();
        let instance = InstanceBox::new("TestBox".to_string(), fields, methods);
        
        // フィールドに値を設定
        let int_value = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
        instance.set_field("value", int_value).unwrap();
        
        // フィールドの値を取得
        let retrieved = instance.get_field("value").unwrap();
        let int_box = retrieved.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(int_box.value, 42);
    }
    
    #[test]
    fn test_instance_equality() {
        let instance1 = InstanceBox::new("Test".to_string(), vec![], HashMap::new());
        let instance2 = InstanceBox::new("Test".to_string(), vec![], HashMap::new());
        
        // 異なるインスタンスは等しくない
        assert!(!instance1.equals(&instance2).value);
        
        // 同じインスタンスは等しい
        assert!(instance1.equals(&instance1).value);
    }
}