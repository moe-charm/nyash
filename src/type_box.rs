/*! 
 * TypeBox - Everything is Box極限実現
 * 
 * 型情報もBoxとして表現し、実行時型チェック、メタプログラミング、
 * ジェネリクス基盤を提供する革命的システム
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::{Debug, Display};
use std::any::Any;

/// メソッドシグニチャ情報
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<String>,
    pub parameter_types: Vec<Arc<TypeBox>>,
    pub return_type: Arc<TypeBox>,
    pub is_static: bool,
}

impl MethodSignature {
    pub fn new(name: String, parameters: Vec<String>) -> Self {
        Self {
            name,
            parameters,
            parameter_types: Vec::new(),
            return_type: Arc::new(TypeBox::void_type()),
            is_static: false,
        }
    }
    
    pub fn with_types(
        name: String, 
        parameters: Vec<String>, 
        parameter_types: Vec<Arc<TypeBox>>, 
        return_type: Arc<TypeBox>
    ) -> Self {
        Self {
            name,
            parameters,
            parameter_types,
            return_type,
            is_static: false,
        }
    }
}

/// 🔥 TypeBox - 型情報をBoxとして表現
#[derive(Debug, Clone)]
pub struct TypeBox {
    /// 型名
    pub name: String,
    
    /// フィールド情報 (field_name -> field_type)
    pub fields: HashMap<String, Arc<TypeBox>>,
    
    /// メソッドシグニチャ情報
    pub methods: HashMap<String, MethodSignature>,
    
    /// 親型（継承）
    pub parent_type: Option<Arc<TypeBox>>,
    
    /// ジェネリクス型パラメータ
    pub type_parameters: Vec<String>,
    
    /// インスタンス化された具体型（ジェネリクス用）
    pub concrete_types: HashMap<String, Arc<TypeBox>>,
    
    /// ビルトイン型かどうか
    pub is_builtin: bool,
    
    /// Box基底
    base: BoxBase,
}

impl TypeBox {
    /// 新しいTypeBoxを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            parent_type: None,
            type_parameters: Vec::new(),
            concrete_types: HashMap::new(),
            is_builtin: false,
            base: BoxBase::new(),
        }
    }
    
    /// ビルトイン型を作成
    pub fn builtin(name: &str) -> Self {
        let mut type_box = Self::new(name);
        type_box.is_builtin = true;
        type_box
    }
    
    /// フィールドを追加
    pub fn add_field(&mut self, name: &str, field_type: Arc<TypeBox>) {
        self.fields.insert(name.to_string(), field_type);
    }
    
    /// メソッドを追加
    pub fn add_method(&mut self, method: MethodSignature) {
        self.methods.insert(method.name.clone(), method);
    }
    
    /// 親型を設定
    pub fn set_parent(&mut self, parent: Arc<TypeBox>) {
        self.parent_type = Some(parent);
    }
    
    /// 型パラメータを追加
    pub fn add_type_parameter(&mut self, param: String) {
        self.type_parameters.push(param);
    }
    
    /// 具体型を設定（ジェネリクス用）
    pub fn set_concrete_type(&mut self, param: &str, concrete_type: Arc<TypeBox>) {
        self.concrete_types.insert(param.to_string(), concrete_type);
    }
    
    /// フィールドの型を取得
    pub fn get_field_type(&self, field_name: &str) -> Option<Arc<TypeBox>> {
        // 自分のフィールドをチェック
        if let Some(field_type) = self.fields.get(field_name) {
            return Some(Arc::clone(field_type));
        }
        
        // 親型のフィールドをチェック（継承）
        if let Some(parent) = &self.parent_type {
            return parent.get_field_type(field_name);
        }
        
        None
    }
    
    /// メソッドシグニチャを取得
    pub fn get_method_signature(&self, method_name: &str) -> Option<&MethodSignature> {
        // 自分のメソッドをチェック
        if let Some(method) = self.methods.get(method_name) {
            return Some(method);
        }
        
        // 親型のメソッドをチェック（継承）
        if let Some(parent) = &self.parent_type {
            return parent.get_method_signature(method_name);
        }
        
        None
    }
    
    /// 型互換性チェック
    pub fn is_compatible_with(&self, other: &TypeBox) -> bool {
        // 同じ型
        if self.name == other.name {
            return true;
        }
        
        // 継承チェック
        if let Some(parent) = &self.parent_type {
            if parent.is_compatible_with(other) {
                return true;
            }
        }
        
        false
    }
    
    /// ジェネリクス型かどうか
    pub fn is_generic(&self) -> bool {
        !self.type_parameters.is_empty()
    }
    
    /// 具体化されたジェネリクス型かどうか
    pub fn is_concrete_generic(&self) -> bool {
        !self.concrete_types.is_empty()
    }
    
    /// 型名を完全表示（ジェネリクス対応）
    pub fn full_name(&self) -> String {
        if self.concrete_types.is_empty() {
            self.name.clone()
        } else {
            let mut result = self.name.clone();
            result.push('<');
            
            let concrete_names: Vec<String> = self.type_parameters.iter()
                .map(|param| {
                    self.concrete_types.get(param)
                        .map(|t| t.name.clone())
                        .unwrap_or_else(|| param.clone())
                })
                .collect();
                
            result.push_str(&concrete_names.join(", "));
            result.push('>');
            result
        }
    }
    
    /// 基本型の定数
    pub fn void_type() -> TypeBox {
        TypeBox::builtin("Void")
    }
    
    pub fn string_type() -> TypeBox {
        TypeBox::builtin("StringBox")
    }
    
    pub fn integer_type() -> TypeBox {
        TypeBox::builtin("IntegerBox")
    }
    
    pub fn bool_type() -> TypeBox {
        TypeBox::builtin("BoolBox")
    }
    
    pub fn array_type() -> TypeBox {
        let mut type_box = TypeBox::builtin("ArrayBox");
        type_box.add_type_parameter("T".to_string());
        type_box
    }
    
    pub fn method_box_type() -> TypeBox {
        let mut type_box = TypeBox::builtin("MethodBox");
        type_box.add_type_parameter("T".to_string());
        type_box
    }
}

/// TypeBoxをNyashBoxとして実装
impl NyashBox for TypeBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<TypeBox: {}>", self.full_name()))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_type) = other.as_any().downcast_ref::<TypeBox>() {
            BoolBox::new(self.name == other_type.name)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "TypeBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    
}

impl BoxCore for TypeBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<TypeBox: {}>", self.full_name())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for TypeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// TypeBoxレジストリ - グローバル型管理
#[derive(Debug)]
pub struct TypeRegistry {
    /// 登録済み型
    types: HashMap<String, Arc<TypeBox>>,
    
    /// 継承チェーン情報（高速化用）
    inheritance_chains: HashMap<String, Vec<String>>,
    
    /// メソッドキャッシュ（将来の最適化用）
    #[allow(dead_code)]
    method_cache: HashMap<(String, String), MethodSignature>,
}

impl TypeRegistry {
    /// 新しいTypeRegistryを作成
    pub fn new() -> Self {
        let mut registry = Self {
            types: HashMap::new(),
            inheritance_chains: HashMap::new(),
            method_cache: HashMap::new(),
        };
        
        // ビルトイン型を登録
        registry.register_builtin_types();
        registry
    }
    
    /// ビルトイン型を登録
    fn register_builtin_types(&mut self) {
        self.register_type(Arc::new(TypeBox::void_type()));
        self.register_type(Arc::new(TypeBox::string_type()));
        self.register_type(Arc::new(TypeBox::integer_type()));
        self.register_type(Arc::new(TypeBox::bool_type()));
        self.register_type(Arc::new(TypeBox::array_type()));
        self.register_type(Arc::new(TypeBox::method_box_type()));
    }
    
    /// 型を登録
    pub fn register_type(&mut self, type_box: Arc<TypeBox>) {
        let name = type_box.name.clone();
        
        // 継承チェーンを構築
        let mut chain = vec![name.clone()];
        let mut current = &type_box.parent_type;
        while let Some(parent) = current {
            chain.push(parent.name.clone());
            current = &parent.parent_type;
        }
        
        self.inheritance_chains.insert(name.clone(), chain);
        self.types.insert(name, type_box);
    }
    
    /// 型を取得
    pub fn get_type(&self, name: &str) -> Option<Arc<TypeBox>> {
        self.types.get(name).map(Arc::clone)
    }
    
    /// 型互換性チェック
    pub fn is_compatible(&self, from_type: &str, to_type: &str) -> bool {
        if from_type == to_type {
            return true;
        }
        
        if let Some(chain) = self.inheritance_chains.get(from_type) {
            chain.contains(&to_type.to_string())
        } else {
            false
        }
    }
    
    /// すべての型名を取得
    pub fn get_all_type_names(&self) -> Vec<String> {
        self.types.keys().cloned().collect()
    }
    
    /// ジェネリクス型をインスタンス化
    pub fn instantiate_generic(&mut self, base_type: &str, concrete_types: &[&str]) -> Result<Arc<TypeBox>, String> {
        let base = self.get_type(base_type)
            .ok_or_else(|| format!("Base type '{}' not found", base_type))?;
            
        if !base.is_generic() {
            return Err(format!("Type '{}' is not generic", base_type));
        }
        
        if base.type_parameters.len() != concrete_types.len() {
            return Err(format!(
                "Generic type '{}' expects {} type parameters, got {}",
                base_type, base.type_parameters.len(), concrete_types.len()
            ));
        }
        
        // 新しい具体化型を作成
        let mut concrete_type = (*base).clone();
        concrete_type.name = format!("{}_{}", base_type, concrete_types.join("_"));
        concrete_type.concrete_types.clear();
        
        // 具体型を設定
        for (i, param) in base.type_parameters.iter().enumerate() {
            let concrete = self.get_type(concrete_types[i])
                .ok_or_else(|| format!("Concrete type '{}' not found", concrete_types[i]))?;
            concrete_type.set_concrete_type(param, concrete);
        }
        
        let result = Arc::new(concrete_type);
        
        // レジストリに登録
        self.register_type(Arc::clone(&result));
        
        Ok(result)
    }
}

/// TypeBoxビルダー - 便利な構築関数
pub struct TypeBoxBuilder {
    type_box: TypeBox,
}

impl TypeBoxBuilder {
    /// 新しいビルダーを作成
    pub fn new(name: &str) -> Self {
        Self {
            type_box: TypeBox::new(name),
        }
    }
    
    /// フィールドを追加
    pub fn field(mut self, name: &str, field_type: Arc<TypeBox>) -> Self {
        self.type_box.add_field(name, field_type);
        self
    }
    
    /// メソッドを追加
    pub fn method(mut self, method: MethodSignature) -> Self {
        self.type_box.add_method(method);
        self
    }
    
    /// 親型を設定
    pub fn parent(mut self, parent: Arc<TypeBox>) -> Self {
        self.type_box.set_parent(parent);
        self
    }
    
    /// 型パラメータを追加
    pub fn type_param(mut self, param: &str) -> Self {
        self.type_box.add_type_parameter(param.to_string());
        self
    }
    
    /// TypeBoxを完成
    pub fn build(self) -> TypeBox {
        self.type_box
    }
}