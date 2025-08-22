/*!
 * Nyash Environment System - Rust所有権ベースのスコープ管理
 * 
 * Python版の曖昧なメモリ管理をRustの借用チェッカーで完全解決！
 * Everything is Box哲学 + 所有権システム = 完璧なスコープ管理
 */

use crate::box_trait::{NyashBox, VoidBox};
use crate::finalization::BoxFinalizer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Nyash変数環境 - スコープチェーンを安全に管理
#[derive(Debug)]
pub struct Environment {
    /// 現在のスコープの変数バインディング
    bindings: Mutex<HashMap<String, Box<dyn NyashBox>>>,
    
    /// 親環境への参照 (Arc<Mutex<>>でスレッド安全)
    parent: Option<Arc<Mutex<Environment>>>,
    
    /// スコープレベル (デバッグ用)
    level: usize,
    
    /// スコープ名 (関数名、クラス名など)
    scope_name: String,
    
    /// Box解放管理
    finalizer: Mutex<BoxFinalizer>,
}

/// Environment操作エラー
#[derive(Error, Debug)]
pub enum EnvironmentError {
    #[error("Variable '{name}' is not defined")]
    UndefinedVariable { name: String },
    
    #[error("Cannot access parent environment: {reason}")]
    ParentAccessError { reason: String },
    
    #[error("Borrowing conflict in environment: {details}")]
    BorrowError { details: String },
}

impl Environment {
    /// 新しいグローバル環境を作成
    pub fn new_global() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            bindings: Mutex::new(HashMap::new()),
            parent: None,
            level: 0,
            scope_name: "global".to_string(),
            finalizer: Mutex::new(BoxFinalizer::new()),
        }))
    }
    
    /// 親環境を持つ新しい環境を作成 (関数、クラス用)
    pub fn new_child(
        parent: Arc<Mutex<Environment>>, 
        scope_name: impl Into<String>
    ) -> Arc<Mutex<Self>> {
        let level = parent.lock().unwrap().level + 1;
        
        Arc::new(Mutex::new(Self {
            bindings: Mutex::new(HashMap::new()),
            parent: Some(parent),
            level,
            scope_name: scope_name.into(),
            finalizer: Mutex::new(BoxFinalizer::new()),
        }))
    }
    
    /// 変数を現在のスコープに定義
    pub fn define(&self, name: impl Into<String>, value: Box<dyn NyashBox>) {
        let name = name.into();
        self.bindings.lock().unwrap().insert(name, value);
    }
    
    /// 変数を取得 (スコープチェーンを辿る)
    pub fn get(&self, name: &str) -> Result<Box<dyn NyashBox>, EnvironmentError> {
        // 現在のスコープから検索
        if let Some(value) = self.bindings.lock().unwrap().get(name) {
            return Ok(value.clone_or_share());
        }
        
        // 親スコープから検索
        if let Some(parent) = &self.parent {
            return parent.lock().unwrap().get(name);
        }
        
        // 見つからない
        Err(EnvironmentError::UndefinedVariable { 
            name: name.to_string() 
        })
    }
    
    /// 変数を設定 (既存変数の更新 or 新規定義)
    pub fn set(&self, name: impl Into<String>, value: Box<dyn NyashBox>) -> Result<(), EnvironmentError> {
        let name = name.into();
        
        // 現在のスコープにある場合は更新
        if self.bindings.lock().unwrap().contains_key(&name) {
            self.bindings.lock().unwrap().insert(name, value);
            return Ok(());
        }
        
        // 親スコープで再帰的に検索・設定
        if let Some(parent) = &self.parent {
            match parent.lock().unwrap().set(&name, value.clone_or_share()) {
                Ok(()) => return Ok(()),
                Err(EnvironmentError::UndefinedVariable { .. }) => {
                    // 親にもない場合は現在のスコープに新規定義
                }
                Err(e) => return Err(e),
            }
        }
        
        // 新規定義として現在のスコープに追加
        self.bindings.lock().unwrap().insert(name, value);
        Ok(())
    }
    
    /// 変数が存在するかチェック
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_ok()
    }
    
    /// 変数を削除 (現在のスコープからのみ)
    pub fn undefine(&self, name: &str) -> bool {
        self.bindings.lock().unwrap().remove(name).is_some()
    }
    
    /// 現在のスコープの変数一覧を取得
    pub fn list_variables(&self) -> Vec<String> {
        self.bindings.lock().unwrap().keys().cloned().collect()
    }
    
    /// スコープ情報を取得 (デバッグ用)
    pub fn scope_info(&self) -> String {
        format!("{}[{}] (level {})", self.scope_name, self.bindings.lock().unwrap().len(), self.level)
    }
    
    /// スコープチェーン全体の情報を取得
    pub fn scope_chain_info(&self) -> Vec<String> {
        let mut chain = vec![self.scope_info()];
        
        if let Some(parent) = &self.parent {
            chain.extend(parent.lock().unwrap().scope_chain_info());
        }
        
        chain
    }
    
    /// 全変数をダンプ (デバッグ用)
    pub fn dump_all_variables(&self) -> HashMap<String, String> {
        let mut all_vars = HashMap::new();
        
        // 現在のスコープの変数
        for (name, value) in self.bindings.lock().unwrap().iter() {
            all_vars.insert(
                format!("{}::{}", self.scope_name, name),
                value.to_string_box().value
            );
        }
        
        // 親スコープの変数 (再帰的)
        if let Some(parent) = &self.parent {
            all_vars.extend(parent.lock().unwrap().dump_all_variables());
        }
        
        all_vars
    }
    
    /// 新しいBoxを追跡対象に追加
    pub fn track_box(&self, nyash_box: Box<dyn NyashBox>) {
        self.finalizer.lock().unwrap().track(nyash_box);
    }
    
    /// スコープ終了時にすべてのBoxを解放
    pub fn finalize_all_boxes(&self) {
        self.finalizer.lock().unwrap().finalize_all();
    }
    
    /// 指定したBoxを解放対象から除外（関数の返り値など）
    pub fn exclude_from_finalization(&self, nyash_box: &Box<dyn NyashBox>) {
        self.finalizer.lock().unwrap().exclude_from_finalization(nyash_box);
    }
}

/// PythonのEnvironmentクラスとの互換性レイヤー
#[derive(Debug)]
pub struct PythonCompatEnvironment {
    inner: Arc<Mutex<Environment>>,
    pub _bindings: HashMap<String, Box<dyn NyashBox>>,
}

impl PythonCompatEnvironment {
    pub fn new() -> Self {
        Self {
            inner: Environment::new_global(),
            _bindings: HashMap::new(),
        }
    }
    
    pub fn new_with_parent(parent: Arc<Mutex<Environment>>) -> Self {
        Self {
            inner: Environment::new_child(parent, "python_compat"),
            _bindings: HashMap::new(),
        }
    }
    
    /// Python版のdefineメソッド互換
    pub fn define(&mut self, name: impl Into<String>, value: Box<dyn NyashBox>) {
        let name = name.into();
        self.inner.lock().unwrap().define(&name, value.clone_box());
        self._bindings.insert(name, value);
    }
    
    /// Python版のgetメソッド互換
    pub fn get(&self, name: &str) -> Box<dyn NyashBox> {
        self.inner.lock().unwrap().get(name).unwrap_or_else(|_| {
            Box::new(VoidBox::new())
        })
    }
    
    /// Rustネイティブ環境への参照を取得
    pub fn as_native(&self) -> Arc<Mutex<Environment>> {
        self.inner.clone()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::{StringBox, IntegerBox, BoolBox};
    
    #[test]
    fn test_global_environment() {
        let env = Environment::new_global();
        
        // 変数定義
        env.lock().unwrap().define("test_var", Box::new(StringBox::new("hello")));
        
        // 変数取得
        let value = env.lock().unwrap().get("test_var").unwrap();
        let string_val = value.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(string_val.value, "hello");
    }
    
    #[test]
    fn test_nested_scopes() {
        let global = Environment::new_global();
        let function_scope = Environment::new_child(global.clone(), "test_function");
        
        // グローバルスコープに変数定義
        global.lock().unwrap().define("global_var", Box::new(StringBox::new("global")));
        
        // 関数スコープに変数定義
        function_scope.lock().unwrap().define("local_var", Box::new(StringBox::new("local")));
        
        // 関数スコープからグローバル変数にアクセス可能
        let global_from_function = function_scope.lock().unwrap().get("global_var").unwrap();
        let global_str = global_from_function.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(global_str.value, "global");
        
        // グローバルスコープからローカル変数にはアクセス不可
        assert!(global.lock().unwrap().get("local_var").is_err());
    }
    
    #[test]
    fn test_variable_shadowing() {
        let global = Environment::new_global();
        let local = Environment::new_child(global.clone(), "local_scope");
        
        // 同名変数を両スコープに定義
        global.lock().unwrap().define("same_name", Box::new(StringBox::new("global_value")));
        local.lock().unwrap().define("same_name", Box::new(StringBox::new("local_value")));
        
        // ローカルスコープからはローカル値が取得される (シャドウイング)
        let value = local.lock().unwrap().get("same_name").unwrap();
        let string_val = value.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(string_val.value, "local_value");
        
        // グローバルスコープからはグローバル値が取得される
        let global_value = global.lock().unwrap().get("same_name").unwrap();
        let global_str = global_value.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(global_str.value, "global_value");
    }
    
    #[test]
    fn test_variable_setting() {
        let global = Environment::new_global();
        let local = Environment::new_child(global.clone(), "local_scope");
        
        // グローバルに変数定義
        global.lock().unwrap().define("shared_var", Box::new(IntegerBox::new(100)));
        
        // ローカルスコープから変数を更新
        local.lock().unwrap().set("shared_var", Box::new(IntegerBox::new(200))).unwrap();
        
        // グローバルスコープの値が更新されている
        let updated_value = global.lock().unwrap().get("shared_var").unwrap();
        let int_val = updated_value.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(int_val.value, 200);
    }
    
    #[test]
    fn test_scope_info() {
        let global = Environment::new_global();
        let func1 = Environment::new_child(global.clone(), "function1");
        let func2 = Environment::new_child(func1.clone(), "function2");
        
        // 各スコープに変数を追加
        global.lock().unwrap().define("g1", Box::new(StringBox::new("global1")));
        func1.lock().unwrap().define("f1", Box::new(StringBox::new("func1")));
        func2.lock().unwrap().define("f2", Box::new(StringBox::new("func2")));
        
        // スコープチェーン情報を確認
        let chain = func2.lock().unwrap().scope_chain_info();
        assert_eq!(chain.len(), 3);
        assert!(chain[0].contains("function2"));
        assert!(chain[1].contains("function1"));
        assert!(chain[2].contains("global"));
    }
    
    #[test]
    fn test_python_compat() {
        let mut env = PythonCompatEnvironment::new();
        
        // Python互換インターフェースで変数操作
        env.define("test", Box::new(BoolBox::new(true)));
        
        let value = env.get("test");
        let bool_val = value.as_any().downcast_ref::<BoolBox>().unwrap();
        assert_eq!(bool_val.value, true);
        
        // _bindingsでも確認可能
        assert!(env._bindings.contains_key("test"));
    }
    
    #[test]
    fn test_error_handling() {
        let env = Environment::new_global();
        
        // 存在しない変数へのアクセス
        let result = env.lock().unwrap().get("nonexistent");
        assert!(result.is_err());
        
        match result {
            Err(EnvironmentError::UndefinedVariable { name }) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }
}
