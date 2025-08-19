/*!
 * Nyash Instance System v2 - Simplified Box Instance Implementation
 * 
 * 🎯 Phase 9.78d: 簡素化InstanceBox統一実装
 * Everything is Box哲学に基づく統一オブジェクト指向システム
 * 
 * 🔄 設計方針: trait objectによる完全統一
 * - すべてのBox型を同じように扱う
 * - Option<T>による柔軟性
 * - レガシー負債の完全削除
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use crate::ast::ASTNode;
use crate::value::NyashValue;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// 🎯 簡素化InstanceBox - すべてのBox型を統一管理
#[derive(Debug)]
pub struct InstanceBox {
    /// クラス名（StringBox, MyUserBox等統一）
    pub class_name: String,
    
    /// 統一フィールド管理（レガシーfields削除）
    pub fields_ng: Arc<Mutex<HashMap<String, NyashValue>>>,
    
    /// メソッド定義（ユーザー定義時のみ使用、ビルトインは空）
    pub methods: Arc<HashMap<String, ASTNode>>,
    
    /// 🏭 統一内容 - すべてのBox型を同じように扱う
    pub inner_content: Option<Box<dyn NyashBox>>,
    
    /// Box基底 + ライフサイクル管理
    base: BoxBase,
    finalized: Arc<Mutex<bool>>,
}

impl InstanceBox {
    /// 🎯 統一コンストラクタ - すべてのBox型対応
    pub fn from_any_box(class_name: String, inner: Box<dyn NyashBox>) -> Self {
        Self {
            class_name,
            fields_ng: Arc::new(Mutex::new(HashMap::new())),
            methods: Arc::new(HashMap::new()), // ビルトインは空、ユーザー定義時は設定
            inner_content: Some(inner), // 統一内包
            base: BoxBase::new(),
            finalized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// ユーザー定義Box専用コンストラクタ
    pub fn from_declaration(class_name: String, fields: Vec<String>, methods: HashMap<String, ASTNode>) -> Self {
        let mut field_map = HashMap::new();
        for field in fields {
            field_map.insert(field, NyashValue::Null);
        }
        
        Self {
            class_name,
            fields_ng: Arc::new(Mutex::new(field_map)),
            methods: Arc::new(methods),
            inner_content: None, // ユーザー定義は内包Boxなし
            base: BoxBase::new(),
            finalized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 🔄 レガシー互換性メソッド - 段階移行用
    pub fn new(class_name: String, fields: Vec<String>, methods: HashMap<String, ASTNode>) -> Self {
        Self::from_declaration(class_name, fields, methods)
    }
    
    /// 🔄 レガシー互換性 - 高度なfiniシステムを簡素化して対応
    pub fn new_with_box_info(
        class_name: String, 
        fields: Vec<String>, 
        methods: HashMap<String, ASTNode>,
        _init_field_order: Vec<String>,  // 簡素化により無視
        _weak_fields: Vec<String>        // 簡素化により無視
    ) -> Self {
        eprintln!("⚠️  new_with_box_info: Advanced fini system simplified - init_order and weak_fields ignored");
        Self::from_declaration(class_name, fields, methods)
    }
    
    /// 🎯 統一フィールドアクセス
    pub fn get_field(&self, field_name: &str) -> Option<NyashValue> {
        self.fields_ng.lock().unwrap().get(field_name).cloned()
    }
    
    /// 🎯 統一フィールド設定
    pub fn set_field(&self, field_name: String, value: NyashValue) -> Result<(), String> {
        self.fields_ng.lock().unwrap().insert(field_name, value);
        Ok(())
    }
    
    /// 動的フィールド追加（GlobalBox用）
    pub fn set_field_dynamic(&self, field_name: String, value: NyashValue) {
        self.fields_ng.lock().unwrap().insert(field_name, value);
    }
    
    /// メソッド定義を取得
    pub fn get_method(&self, method_name: &str) -> Option<&ASTNode> {
        self.methods.get(method_name)
    }
    
    /// メソッドが存在するかチェック
    pub fn has_method(&self, method_name: &str) -> bool {
        self.methods.contains_key(method_name)
    }
    
    /// メソッド動的追加（GlobalBox用）
    pub fn add_method(&mut self, method_name: String, method_ast: ASTNode) -> Result<(), String> {
        let mut new_methods = (*self.methods).clone();
        new_methods.insert(method_name, method_ast);
        self.methods = Arc::new(new_methods);
        Ok(())
    }
    
    /// 🎯 統一初期化処理
    pub fn init(&mut self, args: &[Box<dyn NyashBox>]) -> Result<(), String> {
        match &self.inner_content {
            Some(_) => Ok(()), // ビルトイン・プラグインは初期化済み
            None => {
                // ユーザー定義のinit実行（インタープリター側で実装）
                // TODO: インタープリター統合時に実装
                Ok(())
            }
        }
    }
    
    /// 🎯 統一解放処理
    pub fn fini(&self) -> Result<(), String> {
        let mut finalized = self.finalized.lock().unwrap();
        if *finalized {
            return Ok(()); // 既に解放済み
        }
        
        // フィールドクリア
        self.fields_ng.lock().unwrap().clear();
        
        *finalized = true;
        eprintln!("🎯 fini(): Instance {} (ID: {}) finalized", self.class_name, self.base.id);
        Ok(())
    }
    
    /// 解放済みかチェック
    pub fn is_finalized(&self) -> bool {
        *self.finalized.lock().unwrap()
    }
}

/// 🎯 統一NyashBoxトレイト実装
impl NyashBox for InstanceBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<{} instance #{}>", self.class_name, self.base.id))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_instance) = other.as_any().downcast_ref::<InstanceBox>() {
            BoolBox::new(self.base.id == other_instance.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        // 内包Boxがあれば、その型名を返す（ビルトインBox用）
        if let Some(inner) = &self.inner_content {
            inner.type_name()
        } else {
            // ユーザー定義Boxの場合はclass_nameを使用したいが、
            // &'static strを要求されているので一時的に"InstanceBox"を返す
            // TODO: type_nameの戻り値型をStringに変更することを検討
            "InstanceBox"
        }
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(InstanceBox {
            class_name: self.class_name.clone(),
            fields_ng: Arc::clone(&self.fields_ng),
            methods: Arc::clone(&self.methods),
            inner_content: self.inner_content.as_ref().map(|inner| inner.clone_box()),
            base: self.base.clone(),
            finalized: Arc::clone(&self.finalized),
        })
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        // TODO: 正しいshare_boxセマンティクス実装
        self.clone_box()
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
    fn test_from_any_box_creation() {
        let string_box = Box::new(crate::box_trait::StringBox::new("hello"));
        let instance = InstanceBox::from_any_box("StringBox".to_string(), string_box);
        
        assert_eq!(instance.class_name, "StringBox");
        assert!(instance.inner_content.is_some());
        assert!(instance.methods.is_empty()); // ビルトインは空
    }
    
    #[test]
    fn test_from_declaration_creation() {
        let fields = vec!["x".to_string(), "y".to_string()];
        let methods = HashMap::new();
        let instance = InstanceBox::from_declaration("Point".to_string(), fields, methods);
        
        assert_eq!(instance.class_name, "Point");
        assert!(instance.inner_content.is_none()); // ユーザー定義は内包なし
        assert_eq!(instance.get_field("x"), Some(NyashValue::Null));
        assert_eq!(instance.get_field("y"), Some(NyashValue::Null));
    }
    
    #[test]
    fn test_field_operations() {
        let instance = InstanceBox::from_declaration("TestBox".to_string(), vec!["value".to_string()], HashMap::new());
        
        // フィールド設定
        instance.set_field("value".to_string(), NyashValue::Integer(42)).unwrap();
        
        // フィールド取得
        assert_eq!(instance.get_field("value"), Some(NyashValue::Integer(42)));
    }
    
    #[test]
    fn test_unified_approach() {
        // ビルトインBox
        let string_instance = InstanceBox::from_any_box(
            "StringBox".to_string(), 
            Box::new(crate::box_trait::StringBox::new("test"))
        );
        
        // ユーザー定義Box
        let user_instance = InstanceBox::from_declaration(
            "MyBox".to_string(), 
            vec!["field1".to_string()], 
            HashMap::new()
        );
        
        // どちらも同じ型として扱える！
        let instances: Vec<InstanceBox> = vec![string_instance, user_instance];
        
        for instance in instances {
            println!("Instance: {}", instance.class_name);
            // すべて Box<dyn NyashBox> として統一処理可能
            let _box_ref: &dyn NyashBox = &instance;
        }
    }
}