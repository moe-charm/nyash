/*!
 * Nyash Interpreter - Rust Implementation
 * 
 * Python版nyashc_v4.pyのインタープリターをRustで完全再実装
 * Everything is Box哲学に基づくAST実行エンジン
 */

use crate::ast::{ASTNode, Span};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, SharedNyashBox};
use crate::instance::InstanceBox;
use crate::parser::ParseError;
use super::BuiltinStdlib;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use super::{ControlFlow, BoxDeclaration, ConstructorContext, StaticBoxDefinition, StaticBoxState};
use std::fs::OpenOptions;
use std::io::Write;

// ファイルロガー（expressions.rsと同じ）
fn debug_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/mnt/c/git/nyash/development/debug_hang_issue/debug_trace.log") 
    {
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

/// 実行時エラー
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Undefined variable '{name}'")]
    UndefinedVariable { name: String },
    
    #[error("Undefined function '{name}'")]
    UndefinedFunction { name: String },
    
    #[error("Undefined class '{name}'")]
    UndefinedClass { name: String },
    
    #[error("Type error: {message}")]
    TypeError { message: String },
    
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },
    
    #[error("Break outside of loop")]
    BreakOutsideLoop,
    
    #[error("Return outside of function")]
    ReturnOutsideFunction,
    
    #[error("Uncaught exception")]
    UncaughtException,
    
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    
    #[error("Environment error: {0}")]
    EnvironmentError(String),
    
    // === 🔥 Enhanced Errors with Span Information ===
    
    #[error("Undefined variable '{name}' at {span}")]
    UndefinedVariableAt { name: String, span: Span },
    
    #[error("Type error: {message} at {span}")]
    TypeErrorAt { message: String, span: Span },
    
    #[error("Invalid operation: {message} at {span}")]
    InvalidOperationAt { message: String, span: Span },
    
    #[error("Break outside of loop at {span}")]
    BreakOutsideLoopAt { span: Span },
    
    #[error("Return outside of function at {span}")]
    ReturnOutsideFunctionAt { span: Span },
    
    #[error("Runtime failure: {message}")]
    RuntimeFailure { message: String },
}

impl RuntimeError {
    /// エラーの詳細な文脈付きメッセージを生成
    pub fn detailed_message(&self, source: Option<&str>) -> String {
        match self {
            // Enhanced errors with span information
            RuntimeError::UndefinedVariableAt { name, span } => {
                let mut msg = format!("⚠️  Undefined variable '{}'", name);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }
            
            RuntimeError::TypeErrorAt { message, span } => {
                let mut msg = format!("⚠️  Type error: {}", message);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }
            
            RuntimeError::InvalidOperationAt { message, span } => {
                let mut msg = format!("⚠️  Invalid operation: {}", message);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }
            
            RuntimeError::BreakOutsideLoopAt { span } => {
                let mut msg = "⚠️  Break statement outside of loop".to_string();
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }
            
            RuntimeError::ReturnOutsideFunctionAt { span } => {
                let mut msg = "⚠️  Return statement outside of function".to_string();
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }
            
            // Fallback for old error variants without span
            _ => format!("⚠️  {}", self),
        }
    }
}

/// スレッド間で共有される状態
#[derive(Clone)]
pub struct SharedState {
    /// 🌍 GlobalBox - すべてのトップレベル関数とグローバル変数を管理
    pub global_box: Arc<Mutex<InstanceBox>>,
    
    /// Box宣言のレジストリ（読み込みが多いのでRwLock）
    pub box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
    
    /// 🔥 静的関数のレジストリ（読み込みが多いのでRwLock）
    pub static_functions: Arc<RwLock<HashMap<String, HashMap<String, ASTNode>>>>,
    
    /// 🔥 Static Box定義レジストリ（遅延初期化用）
    pub static_box_definitions: Arc<RwLock<HashMap<String, StaticBoxDefinition>>>,
    
    /// 読み込み済みファイル（重複防止）
    pub included_files: Arc<Mutex<HashSet<String>>>,
}

impl SharedState {
    /// 新しい共有状態を作成
    pub fn new() -> Self {
        let global_box = InstanceBox::new(
            "Global".to_string(),
            vec![],          // フィールド名（空から始める）
            HashMap::new(),  // メソッド（グローバル関数）
        );
        
        Self {
            global_box: Arc::new(Mutex::new(global_box)),
            box_declarations: Arc::new(RwLock::new(HashMap::new())),
            static_functions: Arc::new(RwLock::new(HashMap::new())),
            static_box_definitions: Arc::new(RwLock::new(HashMap::new())),
            included_files: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

/// Nyashインタープリター - AST実行エンジン
pub struct NyashInterpreter {
    /// 共有状態（スレッド間で共有）
    pub(super) shared: SharedState,
    
    /// 📦 local変数スタック（関数呼び出し時の一時変数）
    pub(super) local_vars: HashMap<String, SharedNyashBox>,
    
    /// 📤 outbox変数スタック（static関数内の所有権移転変数）
    pub(super) outbox_vars: HashMap<String, SharedNyashBox>,
    
    /// 制御フロー状態
    pub(super) control_flow: ControlFlow,
    
    /// 現在実行中のコンストラクタ情報
    pub(super) current_constructor_context: Option<ConstructorContext>,
    
    /// 🔄 評価スタック - 循環参照検出用
    pub(super) evaluation_stack: Vec<usize>,
    
    /// 🔗 Invalidated object IDs for weak reference system
    pub invalidated_ids: Arc<Mutex<HashSet<u64>>>,
    
    /// 📚 組み込み標準ライブラリ
    pub(super) stdlib: Option<BuiltinStdlib>,
}

impl NyashInterpreter {
    /// 新しいインタープリターを作成
    pub fn new() -> Self {
        let shared = SharedState::new();
        
        Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None, // 遅延初期化
        }
    }
    
    /// 共有状態から新しいインタープリターを作成（非同期実行用）
    pub fn with_shared(shared: SharedState) -> Self {
        Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None, // 遅延初期化
        }
    }
    
    /// ASTを実行
    pub fn execute(&mut self, ast: ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        debug_log("=== NYASH EXECUTION START ===");
        eprintln!("🔍 DEBUG: Starting interpreter execution...");
        let result = self.execute_node(&ast);
        debug_log("=== NYASH EXECUTION END ===");
        eprintln!("🔍 DEBUG: Interpreter execution completed");
        result
    }
    
    /// ノードを実行
    fn execute_node(&mut self, node: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        eprintln!("🔍 DEBUG: execute_node called with node type: {}", node.node_type());
        match node {
            ASTNode::Program { statements, .. } => {
                eprintln!("🔍 DEBUG: Executing program with {} statements", statements.len());
                let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
                
                for (i, statement) in statements.iter().enumerate() {
                    eprintln!("🔍 DEBUG: Executing statement {} of {}: {}", i + 1, statements.len(), statement.node_type());
                    result = self.execute_statement(statement)?;
                    eprintln!("🔍 DEBUG: Statement {} completed", i + 1);
                    
                    // 制御フローチェック
                    match &self.control_flow {
                        ControlFlow::Break => {
                            return Err(RuntimeError::BreakOutsideLoop);
                        }
                        ControlFlow::Return(_) => {
                            return Err(RuntimeError::ReturnOutsideFunction);
                        }
                        ControlFlow::Throw(_) => {
                            return Err(RuntimeError::UncaughtException);
                        }
                        ControlFlow::None => {}
                    }
                }
                
                // 🎯 Static Box Main パターン - main()メソッドの自動実行
                let has_main_method = {
                    if let Ok(definitions) = self.shared.static_box_definitions.read() {
                        if let Some(main_definition) = definitions.get("Main") {
                            main_definition.methods.contains_key("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
                
                if has_main_method {
                    // Main static boxを初期化
                    self.ensure_static_box_initialized("Main")?;
                    
                    // Main.main() を呼び出し
                    let main_call_ast = ASTNode::MethodCall {
                        object: Box::new(ASTNode::FieldAccess {
                            object: Box::new(ASTNode::Variable {
                                name: "statics".to_string(),
                                span: crate::ast::Span::unknown(),
                            }),
                            field: "Main".to_string(),
                            span: crate::ast::Span::unknown(),
                        }),
                        method: "main".to_string(),
                        arguments: vec![],
                        span: crate::ast::Span::unknown(),
                    };
                    
                    // main()の戻り値を最終結果として使用
                    result = self.execute_statement(&main_call_ast)?;
                }
                
                Ok(result)
            }
            _ => self.execute_statement(node),
        }
    }
    
    // ========== 🌍 GlobalBox変数解決システム ==========
    
    /// 革命的変数解決: local変数 → GlobalBoxフィールド → エラー
    pub(super) fn resolve_variable(&self, name: &str) -> Result<SharedNyashBox, RuntimeError> {
        let log_msg = format!("resolve_variable: name='{}', local_vars={:?}", 
                             name, self.local_vars.keys().collect::<Vec<_>>());
        debug_log(&log_msg);
        eprintln!("🔍 DEBUG: {}", log_msg);
        
        // 1. outbox変数を最初にチェック（static関数内で優先）
        if let Some(outbox_value) = self.outbox_vars.get(name) {
            eprintln!("🔍 DEBUG: Found '{}' in outbox_vars", name);
            
            // 🔧 修正：clone_box() → Arc::clone() で参照共有
            let shared_value = Arc::clone(outbox_value);
            
            eprintln!("✅ RESOLVE_VARIABLE shared reference: {} id={}", 
                     name, shared_value.box_id());
            
            return Ok(shared_value);
        }
        
        // 2. local変数をチェック
        if let Some(local_value) = self.local_vars.get(name) {
            eprintln!("🔍 DEBUG: Found '{}' in local_vars", name);
            
            // 🔧 修正：clone_box() → Arc::clone() で参照共有
            let shared_value = Arc::clone(local_value);
            
            eprintln!("✅ RESOLVE_VARIABLE shared reference: {} id={}", 
                     name, shared_value.box_id());
            
            return Ok(shared_value);
        }
        
        // 3. GlobalBoxのフィールドをチェック
        eprintln!("🔍 DEBUG: Checking GlobalBox for '{}'...", name);
        let global_box = self.shared.global_box.lock().unwrap();
        if let Some(field_value) = global_box.get_field(name) {
            eprintln!("🔍 DEBUG: Found '{}' in GlobalBox", name);
            return Ok(field_value);
        }
        
        // 4. statics名前空間内のstatic boxをチェック
        eprintln!("🔍 DEBUG: Checking statics namespace for '{}'...", name);
        if let Some(statics_namespace) = global_box.get_field("statics") {
            eprintln!("🔍 DEBUG: statics namespace type: {}", statics_namespace.type_name());
            
            // MapBoxとして試す
            if let Some(map_box) = statics_namespace.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                eprintln!("🔍 DEBUG: statics is a MapBox, looking for '{}'", name);
                let key_box: Box<dyn NyashBox> = Box::new(StringBox::new(name));
                let static_box_result = map_box.get(key_box);
                
                // NullBoxでないかチェック（MapBoxは見つからない場合NullBoxを返す）
                if static_box_result.type_name() != "NullBox" {
                    eprintln!("🔍 DEBUG: Found '{}' in statics namespace", name);
                    return Ok(Arc::from(static_box_result));
                } else {
                    eprintln!("🔍 DEBUG: '{}' not found in statics MapBox", name);
                }
            } else if let Some(instance) = statics_namespace.as_any().downcast_ref::<crate::instance::InstanceBox>() {
                eprintln!("🔍 DEBUG: statics is an InstanceBox, looking for '{}'", name);
                if let Some(static_box) = instance.get_field(name) {
                    eprintln!("🔍 DEBUG: Found '{}' in statics namespace", name);
                    return Ok(static_box);
                } else {
                    eprintln!("🔍 DEBUG: '{}' not found in statics InstanceBox", name);
                }
            } else {
                eprintln!("🔍 DEBUG: statics namespace is neither MapBox nor InstanceBox");
            }
        }
        
        drop(global_box); // lockを解放してからstdlibチェック
        
        // 5. nyashstd標準ライブラリ名前空間をチェック  
        eprintln!("🔍 DEBUG: Checking nyashstd stdlib for '{}'...", name);
        if let Some(ref stdlib) = self.stdlib {
            eprintln!("🔍 DEBUG: stdlib is initialized, checking namespaces...");
            eprintln!("🔍 DEBUG: Available namespaces: {:?}", stdlib.namespaces.keys().collect::<Vec<_>>());
            
            if let Some(nyashstd_namespace) = stdlib.namespaces.get("nyashstd") {
                eprintln!("🔍 DEBUG: nyashstd namespace found, checking static boxes...");
                eprintln!("🔍 DEBUG: Available static boxes: {:?}", nyashstd_namespace.static_boxes.keys().collect::<Vec<_>>());
                
                if let Some(static_box) = nyashstd_namespace.static_boxes.get(name) {
                    eprintln!("🔍 DEBUG: Found '{}' in nyashstd namespace", name);
                    
                    // BuiltinStaticBoxをInstanceBoxとしてラップ
                    let static_instance = InstanceBox::new(
                        format!("{}_builtin", name),
                        vec![], // フィールドなし
                        HashMap::new(), // メソッドは動的に解決される
                    );
                    
                    return Ok(Arc::new(static_instance));
                } else {
                    eprintln!("🔍 DEBUG: '{}' not found in nyashstd namespace", name);
                }
            } else {
                eprintln!("🔍 DEBUG: nyashstd namespace not found in stdlib");
            }
        } else {
            eprintln!("🔍 DEBUG: stdlib not initialized");
        }
        
        // 6. エラー：見つからない
        eprintln!("🔍 DEBUG: '{}' not found anywhere!", name);
        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
        })
    }
    
    /// 🔥 厳密変数設定: 明示的宣言のみ許可 - Everything is Box哲学
    pub(super) fn set_variable(&mut self, name: &str, value: Box<dyn NyashBox>) -> Result<(), RuntimeError> {
        let shared_value = Arc::from(value); // Convert Box to Arc
        
        // 1. outbox変数が存在する場合は更新
        if self.outbox_vars.contains_key(name) {
            self.outbox_vars.insert(name.to_string(), shared_value);
            return Ok(());
        }
        
        // 2. local変数が存在する場合は更新
        if self.local_vars.contains_key(name) {
            self.local_vars.insert(name.to_string(), shared_value);
            return Ok(());
        }
        
        // 3. GlobalBoxのフィールドが既に存在する場合は更新
        {
            let global_box = self.shared.global_box.lock().unwrap();
            if global_box.get_field(name).is_some() {
                drop(global_box); // lockを解放
                let mut global_box = self.shared.global_box.lock().unwrap();
                global_box.set_field_dynamic(name.to_string(), shared_value);
                return Ok(());
            }
        }
        
        // 4. 🚨 未宣言変数への代入は厳密にエラー
        Err(RuntimeError::UndefinedVariable {
            name: format!(
                "{}\n💡 Suggestion: Declare the variable first:\n  • For fields: Add '{}' to 'init {{ }}' block\n  • For local variables: Use 'local {}'\n  • For field access: Use 'me.{}'", 
                name, name, name, name
            ),
        })
    }
    
    /// local変数を宣言（関数内でのみ有効）
    pub(super) fn declare_local_variable(&mut self, name: &str, value: Box<dyn NyashBox>) {
        self.local_vars.insert(name.to_string(), Arc::from(value));
    }
    
    /// outbox変数を宣言（static関数内で所有権移転）
    pub(super) fn declare_outbox_variable(&mut self, name: &str, value: Box<dyn NyashBox>) {
        self.outbox_vars.insert(name.to_string(), Arc::from(value));
    }
    
    /// local変数スタックを保存・復元（関数呼び出し時）
    pub(super) fn save_local_vars(&self) -> HashMap<String, Box<dyn NyashBox>> {
        self.local_vars.iter()
            .map(|(k, v)| (k.clone(), (**v).clone_box()))  // Deref Arc to get the Box
            .collect()
    }
    
    pub(super) fn restore_local_vars(&mut self, saved: HashMap<String, Box<dyn NyashBox>>) {
        self.local_vars = saved.into_iter()
            .map(|(k, v)| (k, Arc::from(v)))  // Convert Box to Arc
            .collect();
    }
    
    /// outbox変数スタックを保存・復元（static関数呼び出し時）
    pub(super) fn save_outbox_vars(&self) -> HashMap<String, Box<dyn NyashBox>> {
        self.outbox_vars.iter()
            .map(|(k, v)| (k.clone(), (**v).clone_box()))  // Deref Arc to get the Box
            .collect()
    }
    
    pub(super) fn restore_outbox_vars(&mut self, saved: HashMap<String, Box<dyn NyashBox>>) {
        self.outbox_vars = saved.into_iter()
            .map(|(k, v)| (k, Arc::from(v)))  // Convert Box to Arc
            .collect();
    }
    
    /// トップレベル関数をGlobalBoxのメソッドとして登録 - 🔥 暗黙オーバーライド禁止対応
    pub(super) fn register_global_function(&mut self, name: String, func_ast: ASTNode) -> Result<(), RuntimeError> {
        let mut global_box = self.shared.global_box.lock().unwrap();
        global_box.add_method(name, func_ast)
            .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
        Ok(())
    }
    
    
    
    
    
    /// 値が真と評価されるかチェック
    pub(super) fn is_truthy(&self, value: &Box<dyn NyashBox>) -> bool {
        #[allow(unused_imports)]
        use std::any::Any;
        
        if let Some(bool_box) = value.as_any().downcast_ref::<BoolBox>() {
            bool_box.value
        } else if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            int_box.value != 0
        } else if let Some(string_box) = value.as_any().downcast_ref::<StringBox>() {
            !string_box.value.is_empty()
        } else if value.as_any().downcast_ref::<VoidBox>().is_some() {
            false
        } else {
            true // 他のBoxは真とみなす
        }
    }
    
    /// 🌍 革命的変数取得（テスト用）：GlobalBoxのフィールドから取得
    pub fn get_variable(&self, name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let shared_var = self.resolve_variable(name)?;
        Ok((*shared_var).clone_box())  // Convert Arc back to Box for external interface
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::NyashParser;
    
    #[test]
    fn test_simple_execution() {
        let code = r#"
        x = 42
        print(x)
        "#;
        
        let ast = NyashParser::parse_from_string(code).unwrap();
        let mut interpreter = NyashInterpreter::new();
        let result = interpreter.execute(ast);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_arithmetic() {
        let code = r#"
        result = 10 + 32
        "#;
        
        let ast = NyashParser::parse_from_string(code).unwrap();
        let mut interpreter = NyashInterpreter::new();
        interpreter.execute(ast).unwrap();
        
        // 🌍 革命的変数取得：GlobalBoxから
        let result = interpreter.get_variable("result").unwrap();
        assert_eq!(result.to_string_box().value, "42");
    }
    
    #[test]
    fn test_if_statement() {
        let code = r#"
        x = true
        if x {
            y = "success"
        } else {
            y = "failure"
        }
        "#;
        
        let ast = NyashParser::parse_from_string(code).unwrap();
        let mut interpreter = NyashInterpreter::new();
        interpreter.execute(ast).unwrap();
        
        // 🌍 革命的変数取得：GlobalBoxから
        let result = interpreter.get_variable("y").unwrap();
        assert_eq!(result.to_string_box().value, "success");
    }
    
    #[test]
    fn test_box_instance_creation() {
        let code = r#"
        box TestBox {
            value
            
            getValue() {
                return this.value
            }
            
            setValue(newValue) {
                this.value = newValue
            }
        }
        
        obj = new TestBox()
        obj.value = "test123"
        result = obj.getValue()
        "#;
        
        let ast = NyashParser::parse_from_string(code).unwrap();
        let mut interpreter = NyashInterpreter::new();
        interpreter.execute(ast).unwrap();
        
        // 🌍 革命的変数取得：インスタンス作成確認
        let obj = interpreter.get_variable("obj").unwrap();
        assert!(obj.as_any().downcast_ref::<InstanceBox>().is_some());
        
        // 🌍 革命的変数取得：メソッド呼び出し結果確認
        let result = interpreter.get_variable("result").unwrap();
        assert_eq!(result.to_string_box().value, "test123");
    }
}

// ===== 🔥 Static Box管理システム =====

impl NyashInterpreter {
    
    /// Static Box定義を登録
    pub fn register_static_box(&mut self, definition: StaticBoxDefinition) -> Result<(), RuntimeError> {
        let mut definitions = self.shared.static_box_definitions.write()
            .map_err(|_| RuntimeError::RuntimeFailure { 
                message: "Failed to acquire write lock for static box definitions".to_string() 
            })?;
        
        definitions.insert(definition.name.clone(), definition);
        Ok(())
    }
    
    /// Static Box宣言を登録（AST処理から呼ばれる）
    pub fn register_static_box_declaration(
        &mut self, 
        name: String,
        fields: Vec<String>,
        methods: HashMap<String, ASTNode>,
        init_fields: Vec<String>,
        weak_fields: Vec<String>,  // 🔗 weak修飾子が付いたフィールドのリスト
        static_init: Option<Vec<ASTNode>>,
        extends: Vec<String>,  // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
        implements: Vec<String>,
        type_parameters: Vec<String>
    ) -> Result<(), RuntimeError> {
        // 🌍 Static Box定義時にstatics名前空間を確実に作成
        self.ensure_statics_namespace()?;
        
        let definition = StaticBoxDefinition {
            name: name.clone(),
            fields,
            methods,
            init_fields,
            weak_fields,  // 🔗 Add weak_fields to static box definition
            static_init,
            extends,
            implements,
            type_parameters,
            initialization_state: StaticBoxState::NotInitialized,
        };
        
        eprintln!("🔥 Static Box '{}' definition registered in statics namespace", name);
        self.register_static_box(definition)
    }
    
    /// Static Boxの初期化を実行（遅延初期化）
    pub fn ensure_static_box_initialized(&mut self, name: &str) -> Result<(), RuntimeError> {
        // 1. 定義を取得
        let definition = {
            let definitions = self.shared.static_box_definitions.read()
                .map_err(|_| RuntimeError::RuntimeFailure {
                    message: "Failed to acquire read lock for static box definitions".to_string()
                })?;
            
            match definitions.get(name) {
                Some(def) => def.clone(),
                None => return Err(RuntimeError::UndefinedClass { name: name.to_string() }),
            }
        };
        
        // 2. 初期化状態をチェック
        if definition.initialization_state == StaticBoxState::Initialized {
            return Ok(()); // 既に初期化済み
        }
        
        if definition.initialization_state == StaticBoxState::Initializing {
            return Err(RuntimeError::RuntimeFailure {
                message: format!("Circular dependency detected during initialization of static box '{}'", name)
            });
        }
        
        // 3. 初期化開始をマーク
        self.set_static_box_state(name, StaticBoxState::Initializing)?;
        
        // 4. 「statics」名前空間をGlobalBoxに作成（未存在の場合）
        self.ensure_statics_namespace()?;
        
        // 5. シングルトンインスタンスを作成（メソッドも含む）
        let singleton = InstanceBox::new(
            format!("{}_singleton", name),
            definition.init_fields.clone(),
            definition.methods.clone(), // ★ メソッドを正しく設定
        );
        
        // 6. GlobalBox.staticsに登録
        self.set_static_instance(name, singleton)?;
        
        // 7. static初期化ブロックを実行（me変数をバインドして）
        if let Some(ref init_statements) = definition.static_init {
            // statics名前空間からシングルトンインスタンスを取得
            let static_instance = {
                let global_box = self.shared.global_box.lock().unwrap();
                let statics_box = global_box.get_field("statics").unwrap();
                let statics_instance = statics_box.as_any().downcast_ref::<InstanceBox>().unwrap();
                statics_instance.get_field(name).unwrap()
            };
            
            // 🌍 this変数をバインドしてstatic初期化実行（me構文のため）
            self.declare_local_variable("me", (*static_instance).clone_box());
            
            for stmt in init_statements {
                self.execute_statement(stmt)?;
            }
            
            // 🌍 this変数をクリーンアップ
            self.local_vars.remove("me");
        }
        
        // 8. 初期化完了をマーク
        self.set_static_box_state(name, StaticBoxState::Initialized)?;
        
        Ok(())
    }
    
    /// Static Box初期化状態を設定
    fn set_static_box_state(&mut self, name: &str, state: StaticBoxState) -> Result<(), RuntimeError> {
        let mut definitions = self.shared.static_box_definitions.write()
            .map_err(|_| RuntimeError::RuntimeFailure {
                message: "Failed to acquire write lock for static box definitions".to_string()
            })?;
        
        if let Some(definition) = definitions.get_mut(name) {
            definition.initialization_state = state;
        }
        
        Ok(())
    }
    
    /// 「statics」名前空間をGlobalBoxに作成
    fn ensure_statics_namespace(&mut self) -> Result<(), RuntimeError> {
        let global_box = self.shared.global_box.lock()
            .map_err(|_| RuntimeError::RuntimeFailure {
                message: "Failed to acquire global box lock".to_string()
            })?;
        
        // 既に存在する場合はスキップ
        if global_box.get_field("statics").is_some() {
            eprintln!("🌍 statics namespace already exists - skipping creation");
            return Ok(());
        }
        
        // 「statics」用のInstanceBoxを作成
        let statics_box = InstanceBox::new(
            "statics".to_string(),
            vec![],
            HashMap::new(),
        );
        
        // GlobalBoxのfieldsに直接挿入
        {
            let mut fields = global_box.fields.lock().unwrap();
            fields.insert("statics".to_string(), Arc::new(statics_box));
        }
            
        eprintln!("🌍 statics namespace created in GlobalBox successfully");
        Ok(())
    }
    
    /// Static Boxシングルトンインスタンスを設定
    fn set_static_instance(&mut self, name: &str, instance: InstanceBox) -> Result<(), RuntimeError> {
        let global_box = self.shared.global_box.lock()
            .map_err(|_| RuntimeError::RuntimeFailure {
                message: "Failed to acquire global box lock".to_string()
            })?;
        
        // statics名前空間を取得
        let statics_box = global_box.get_field("statics")
            .ok_or(RuntimeError::TypeError {
                message: "statics namespace not found in GlobalBox".to_string()
            })?;
        
        let statics_instance = statics_box.as_any()
            .downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: "statics field is not an InstanceBox".to_string()
            })?;
        
        // statics InstanceBoxのfieldsに直接挿入（動的フィールド追加）
        {
            let mut fields = statics_instance.fields.lock().unwrap();
            fields.insert(name.to_string(), Arc::new(instance));
        }
        
        eprintln!("🔥 Static box '{}' instance registered in statics namespace", name);
        Ok(())
    }
    
    /// 🔥 Static Boxかどうかをチェック
    pub(super) fn is_static_box(&self, name: &str) -> bool {
        if let Ok(definitions) = self.shared.static_box_definitions.read() {
            definitions.contains_key(name)
        } else {
            false
        }
    }
    
    /// 🔗 Trigger weak reference invalidation (expert-validated implementation)
    pub(super) fn trigger_weak_reference_invalidation(&mut self, target_info: &str) {
        eprintln!("🔗 DEBUG: Registering invalidation for: {}", target_info);
        
        // Extract actual object ID from target_info string
        // Format: "<ClassName instance #ID>" -> extract ID
        if let Some(hash_pos) = target_info.find('#') {
            let id_str = &target_info[hash_pos + 1..];
            // Find the end of the ID (before '>')
            let id_end = id_str.find('>').unwrap_or(id_str.len());
            let clean_id_str = &id_str[..id_end];
            
            if let Ok(id) = clean_id_str.parse::<u64>() {
                self.invalidated_ids.lock().unwrap().insert(id);
                eprintln!("🔗 DEBUG: Object with ID {} marked as invalidated", id);
            } else {
                eprintln!("🔗 DEBUG: Failed to parse ID from: {}", clean_id_str);
            }
        } else {
            // Fallback for non-standard target_info format
            eprintln!("🔗 DEBUG: No ID found in target_info, using fallback");
            if target_info.contains("Parent") {
                self.invalidated_ids.lock().unwrap().insert(999); // Fallback marker
                eprintln!("🔗 DEBUG: Parent objects marked as invalidated (fallback ID 999)");
            }
        }
    }
}