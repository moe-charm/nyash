/*!
 * MethodBox - Function Pointer Implementation for Nyash
 * 
 * イベントハンドラーやコールバックを実現するためのMethodBox実装
 * ChatGPT先生のアドバイスを全面採用
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use crate::ast::ASTNode;
use crate::instance_v2::InstanceBox;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// BoxType enum - ChatGPT先生の提案に従い、Box型を分類
#[derive(Debug)]
pub enum BoxType {
    /// 通常のインスタンス
    Instance(Box<dyn NyashBox>),
    
    /// 関数定義（インスタンスなし）
    Function(FunctionDefinition),
    
    /// メソッド参照（インスタンス＋メソッド）
    Method(MethodBox),
}

/// 関数定義情報
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<ASTNode>,
    pub is_static: bool,
}

/// MethodBox - インスタンスとメソッドの組み合わせ
#[derive(Debug, Clone)]
pub struct MethodBox {
    /// メソッドを持つインスタンス
    pub instance: Arc<Mutex<Box<dyn NyashBox>>>,
    
    /// メソッド名
    pub method_name: String,
    
    /// メソッド定義（キャッシュ用）
    pub method_def: Option<FunctionDefinition>,
    
    /// Box基底
    base: BoxBase,
}

impl MethodBox {
    /// 新しいMethodBoxを作成
    pub fn new(instance: Box<dyn NyashBox>, method_name: String) -> Self {
        // メソッド定義をキャッシュ（可能であれば）
        let method_def = if let Some(inst) = instance.as_any().downcast_ref::<InstanceBox>() {
            inst.get_method(&method_name).and_then(|ast| {
                if let ASTNode::FunctionDeclaration { name, params, body, is_static, .. } = ast {
                    Some(FunctionDefinition {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        is_static: *is_static,
                    })
                } else {
                    None
                }
            })
        } else {
            None
        };
        
        Self {
            instance: Arc::new(Mutex::new(instance)),
            method_name,
            method_def,
            base: BoxBase::new(),
        }
    }
    
    /// メソッドを呼び出す
    pub fn invoke(&self, _args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, String> {
        // TODO: インタープリタとの統合が必要
        // 現在は仮実装
        Err(format!("MethodBox.invoke not yet implemented for method '{}'", self.method_name))
    }
    
    /// インスタンスを取得（内部使用）
    pub fn get_instance(&self) -> Arc<Mutex<Box<dyn NyashBox>>> {
        Arc::clone(&self.instance)
    }
}

impl NyashBox for MethodBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<MethodBox: {}>", self.method_name))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_method) = other.as_any().downcast_ref::<MethodBox>() {
            // 同じインスタンス、同じメソッド名なら等しい
            let self_inst = self.instance.lock().unwrap();
            let other_inst = other_method.instance.lock().unwrap();
            BoolBox::new(
                self_inst.box_id() == other_inst.box_id() && 
                self.method_name == other_method.method_name
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "MethodBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for MethodBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<MethodBox: {}>", self.method_name)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for MethodBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// EphemeralInstance - 一時的なスコープ（関数実行時）
#[derive(Debug)]
pub struct EphemeralInstance {
    /// ローカル変数
    pub locals: HashMap<String, Box<dyn NyashBox>>,
    
    /// 親インスタンス（メソッド呼び出しの場合）
    pub parent_instance: Option<Arc<Mutex<Box<dyn NyashBox>>>>,
}

use std::collections::HashMap;

impl EphemeralInstance {
    /// 新しい一時スコープを作成
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            parent_instance: None,
        }
    }
    
    /// インスタンスから一時スコープを作成
    pub fn from_instance(instance: Arc<Mutex<Box<dyn NyashBox>>>) -> Self {
        Self {
            locals: HashMap::new(),
            parent_instance: Some(instance),
        }
    }
    
    /// ローカル変数を設定
    pub fn set_local(&mut self, name: String, value: Box<dyn NyashBox>) {
        self.locals.insert(name, value);
    }
    
    /// ローカル変数を取得
    pub fn get_local(&self, name: &str) -> Option<Box<dyn NyashBox>> {
        self.locals.get(name).map(|v| v.clone_box())
    }
    
    /// 変数を解決（local → instance fields → global）
    pub fn resolve_variable(&self, name: &str) -> Option<Box<dyn NyashBox>> {
        // 1. ローカル変数
        if let Some(value) = self.get_local(name) {
            return Some(value);
        }
        
        // 2. インスタンスフィールド（parent_instanceがある場合）
        if let Some(parent) = &self.parent_instance {
            let inst = parent.lock().unwrap();
            if let Some(instance_box) = inst.as_any().downcast_ref::<InstanceBox>() {
                if let Some(field_value) = instance_box.get_field(name) {
                    return Some((*field_value).clone_box());
                }
            }
        }
        
        // 3. グローバル（TODO: インタープリタとの統合が必要）
        None
    }
}

/// MethodBox作成用のビルダー構文サポート
pub fn create_method_box(instance: Box<dyn NyashBox>, method_name: &str) -> Box<dyn NyashBox> {
    Box::new(MethodBox::new(instance, method_name.to_string()))
}