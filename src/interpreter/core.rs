/*!
 * Nyash Interpreter - Rust Implementation
 * 
 * Python版nyashc_v4.pyのインタープリターをRustで完全再実装
 * Everything is Box哲学に基づくAST実行エンジン
 */

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, SharedNyashBox};
use crate::instance_v2::InstanceBox;
use crate::parser::ParseError;
use super::BuiltinStdlib;
use crate::runtime::{NyashRuntime, NyashRuntimeBuilder};
use crate::box_factory::BoxFactory;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use super::{ControlFlow, BoxDeclaration, ConstructorContext, StaticBoxDefinition, StaticBoxState};
use super::{RuntimeError, SharedState};
use std::fs::OpenOptions;
use std::io::Write;

// ファイルロガー（expressions.rsと同じ）
pub(crate) fn debug_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/mnt/c/git/nyash/development/debug_hang_issue/debug_trace.log") 
    {
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

// Conditional debug macro - unified with utils::debug_on()
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        if crate::interpreter::utils::debug_on() { eprintln!($($arg)*); }
    };
}

macro_rules! idebug {
    ($($arg:tt)*) => {
        if crate::interpreter::utils::debug_on() { eprintln!($($arg)*); }
    };
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
    #[allow(dead_code)]
    pub(super) evaluation_stack: Vec<usize>,
    
    /// 🔗 Invalidated object IDs for weak reference system
    pub invalidated_ids: Arc<Mutex<HashSet<u64>>>,
    
    /// 📚 組み込み標準ライブラリ
    pub(super) stdlib: Option<BuiltinStdlib>,

    /// 共有ランタイム（Boxレジストリ等）
    #[allow(dead_code)]
    pub(super) runtime: NyashRuntime,

    /// 現在の文脈で式結果が破棄されるか（must_use警告用）
    pub(super) discard_context: bool,
}

impl NyashInterpreter {
    /// 新しいインタープリターを作成
    pub fn new() -> Self {
        let shared = SharedState::new();

        // 先にランタイムを構築（UDFは後から同一SharedStateで注入）
        let runtime = NyashRuntimeBuilder::new().build();

        // Runtimeのbox_declarationsを共有状態に差し替え、同一の参照を保つ
        let mut shared = shared; // 可変化
        shared.box_declarations = runtime.box_declarations.clone();

        // ユーザー定義Boxファクトリを、差し替え済みSharedStateで登録
        use crate::box_factory::user_defined::UserDefinedBoxFactory;
        let udf = Arc::new(UserDefinedBoxFactory::new(shared.clone()));
        if let Ok(mut reg) = runtime.box_registry.lock() {
            reg.register(udf);
        }

        let mut this = Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None, // 遅延初期化
            runtime,
            discard_context: false,
        };

        // Register MethodBox invoker once (idempotent)
        self::register_methodbox_invoker();

        this
    }

    /// グループ構成を指定して新しいインタープリターを作成
    pub fn new_with_groups(groups: crate::box_factory::builtin::BuiltinGroups) -> Self {
        let shared = SharedState::new();

        // 先にランタイム（組み込みグループのみ）を構築
        let runtime = NyashRuntimeBuilder::new()
            .with_builtin_groups(groups)
            .build();

        // Runtimeのbox_declarationsを共有状態に差し替え
        let mut shared = shared; // 可変化
        shared.box_declarations = runtime.box_declarations.clone();

        // 差し替え済みSharedStateでUDFを登録
        use crate::box_factory::user_defined::UserDefinedBoxFactory;
        let udf = Arc::new(UserDefinedBoxFactory::new(shared.clone()));
        if let Ok(mut reg) = runtime.box_registry.lock() {
            reg.register(udf);
        }

        let mut this = Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None,
            runtime,
            discard_context: false,
        };
        self::register_methodbox_invoker();
        this
    }
    
    /// 共有状態から新しいインタープリターを作成（非同期実行用）
    pub fn with_shared(shared: SharedState) -> Self {
        // 共有状態に紐づいたランタイムを先に構築
        let runtime = NyashRuntimeBuilder::new().build();

        // Runtimeのbox_declarationsに寄せ替え
        let mut shared = shared; // 可変化
        shared.box_declarations = runtime.box_declarations.clone();

        // 差し替え済みSharedStateでUDFを登録
        use crate::box_factory::user_defined::UserDefinedBoxFactory;
        let udf = Arc::new(UserDefinedBoxFactory::new(shared.clone()));
        if let Ok(mut reg) = runtime.box_registry.lock() {
            reg.register(udf);
        }

        let mut this = Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None, // 遅延初期化
            runtime,
            discard_context: false,
        };
        self::register_methodbox_invoker();
        this
    }

    /// 共有状態＋グループ構成を指定して新しいインタープリターを作成（非同期実行用）
    pub fn with_shared_and_groups(shared: SharedState, groups: crate::box_factory::builtin::BuiltinGroups) -> Self {
        // 先にランタイム（組み込みグループのみ）を構築
        let runtime = NyashRuntimeBuilder::new()
            .with_builtin_groups(groups)
            .build();

        let mut shared = shared; // 可変化
        shared.box_declarations = runtime.box_declarations.clone();

        // 差し替え済みSharedStateでUDFを登録
        use crate::box_factory::user_defined::UserDefinedBoxFactory;
        let udf = Arc::new(UserDefinedBoxFactory::new(shared.clone()));
        if let Ok(mut reg) = runtime.box_registry.lock() {
            reg.register(udf);
        }

        let mut this = Self {
            shared,
            local_vars: HashMap::new(),
            outbox_vars: HashMap::new(),
            control_flow: ControlFlow::None,
            current_constructor_context: None,
            evaluation_stack: Vec::new(),
            invalidated_ids: Arc::new(Mutex::new(HashSet::new())),
            stdlib: None,
            runtime,
            discard_context: false,
        };
        self::register_methodbox_invoker();
        this
    }
    

    /// Register an additional BoxFactory into this interpreter's runtime registry.
    /// This allows tests or embedders to inject custom factories without globals.
    pub fn register_box_factory(&mut self, factory: Arc<dyn BoxFactory>) {
        if let Ok(mut reg) = self.runtime.box_registry.lock() {
            reg.register(factory);
        }
    }
    
    
    // ========== 🌍 GlobalBox変数解決システム ==========
    
    /// 革命的変数解決: local変数 → GlobalBoxフィールド → エラー
    pub(super) fn resolve_variable(&self, name: &str) -> Result<SharedNyashBox, RuntimeError> {
        let log_msg = format!("resolve_variable: name='{}', local_vars={:?}", 
                             name, self.local_vars.keys().collect::<Vec<_>>());
        debug_log(&log_msg);
        // 1. outbox変数を最初にチェック（static関数内で優先）
        if let Some(outbox_value) = self.outbox_vars.get(name) {
            // 🔧 修正：clone_box() → Arc::clone() で参照共有
            let shared_value = Arc::clone(outbox_value);
            return Ok(shared_value);
        }
        
        // 2. local変数をチェック
        if let Some(local_value) = self.local_vars.get(name) {
            // 🔧 修正：clone_box() → Arc::clone() で参照共有
            let shared_value = Arc::clone(local_value);
            return Ok(shared_value);
        }
        
        // 3. GlobalBoxのフィールドをチェック
        let global_box = self.shared.global_box.lock().unwrap();
        if let Some(field_value) = global_box.get_field(name) {
            return Ok(field_value);
        }
        
        // 4. statics名前空間内のstatic boxをチェック
        if let Some(statics_namespace) = global_box.get_field("statics") {
            
            // MapBoxとして試す
            if let Some(map_box) = statics_namespace.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                let key_box: Box<dyn NyashBox> = Box::new(StringBox::new(name));
                let static_box_result = map_box.get(key_box);
                
                // NullBoxでないかチェック（MapBoxは見つからない場合NullBoxを返す）
                if static_box_result.type_name() != "NullBox" {
                    return Ok(Arc::from(static_box_result));
                }
            } else if let Some(instance) = statics_namespace.as_any().downcast_ref::<InstanceBox>() {
                if let Some(static_box) = instance.get_field(name) {
                    return Ok(static_box);
                }
            }
        }
        
        drop(global_box); // lockを解放してからstdlibチェック
        
        // 5. nyashstd標準ライブラリ名前空間をチェック  
        if let Some(ref stdlib) = self.stdlib {
            if let Some(nyashstd_namespace) = stdlib.namespaces.get("nyashstd") {
                if let Some(_static_box) = nyashstd_namespace.static_boxes.get(name) {
                    // BuiltinStaticBoxをInstanceBoxとしてラップ
                    let static_instance = InstanceBox::new(
                        format!("{}_builtin", name),
                        vec![], // フィールドなし
                        HashMap::new(), // メソッドは動的に解決される
                    );
                    
                    return Ok(Arc::new(static_instance));
                }
            }
        }
        
        // 6. エラー：見つからない
        idebug!("🔍 DEBUG: '{}' not found anywhere!", name);
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
                global_box.set_field_dynamic_legacy(name.to_string(), shared_value);
                return Ok(());
            }
        }
        
        // 4. グローバル変数として新規作成（従来の緩い挙動に合わせる）
        {
            let mut global_box = self.shared.global_box.lock().unwrap();
            global_box.set_field_dynamic_legacy(name.to_string(), shared_value);
        }
        Ok(())
    }
    
    /// local変数を宣言（関数内でのみ有効）
    pub(super) fn declare_local_variable(&mut self, name: &str, value: Box<dyn NyashBox>) {
        // Pass-by-share for plugin handle types; by-value (clone) semantics can be applied at call sites
        #[allow(unused_mut)]
        let mut store_value = value;
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            if store_value.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                store_value = store_value.share_box();
            }
        }
        self.local_vars.insert(name.to_string(), Arc::from(store_value));
    }
    
    /// outbox変数を宣言（static関数内で所有権移転）
    pub(super) fn declare_outbox_variable(&mut self, name: &str, value: Box<dyn NyashBox>) {
        #[allow(unused_mut)]
        let mut store_value = value;
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            if store_value.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                store_value = store_value.share_box();
            }
        }
        self.outbox_vars.insert(name.to_string(), Arc::from(store_value));
    }
    
    /// local変数スタックを保存・復元（関数呼び出し時）
    pub(super) fn save_local_vars(&self) -> HashMap<String, Box<dyn NyashBox>> {
        self.local_vars.iter()
            .map(|(k, v)| {
                let b: &dyn NyashBox = &**v;
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                    return (k.clone(), b.share_box());
                }
                (k.clone(), b.clone_box())
            })
            .collect()
    }
    
    pub(super) fn restore_local_vars(&mut self, saved: HashMap<String, Box<dyn NyashBox>>) {
        // 🎯 スコープ離脱時：現在のローカル変数に対してfiniを呼ぶ
        // ただし「me」は特別扱い（インスタンス自身なのでfiniしない）
        for (name, value) in &self.local_vars {
            // 「me」はインスタンス自身なのでスコープ離脱時にfiniしない
            if name == "me" {
                continue;
            }
            
            // ユーザー定義Box（InstanceBox）の場合
            if let Some(instance) = (**value).as_any().downcast_ref::<InstanceBox>() {
                let _ = instance.fini();
                idebug!("🔄 Scope exit: Called fini() on local variable '{}' (InstanceBox)", name);
            }
            // プラグインBoxは共有ハンドルの可能性が高いため自動finiしない（明示呼び出しのみ）
            // ビルトインBoxは元々finiメソッドを持たないので呼ばない
            // （StringBox、IntegerBox等はリソース管理不要）
        }
        
        // その後、保存されていた変数で復元
        self.local_vars = saved.into_iter()
            .map(|(k, v)| (k, Arc::from(v)))  // Convert Box to Arc
            .collect();
    }
    
    /// outbox変数スタックを保存・復元（static関数呼び出し時）
    pub(super) fn save_outbox_vars(&self) -> HashMap<String, Box<dyn NyashBox>> {
        self.outbox_vars.iter()
            .map(|(k, v)| {
                let b: &dyn NyashBox = &**v;
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                    return (k.clone(), b.share_box());
                }
                (k.clone(), b.clone_box())
            })
            .collect()
    }
    
    pub(super) fn restore_outbox_vars(&mut self, saved: HashMap<String, Box<dyn NyashBox>>) {
        // 🎯 スコープ離脱時：現在のoutbox変数に対してもfiniを呼ぶ
        for (name, value) in &self.outbox_vars {
            // ユーザー定義Box（InstanceBox）の場合
            if let Some(instance) = (**value).as_any().downcast_ref::<InstanceBox>() {
                let _ = instance.fini();
                idebug!("🔄 Scope exit: Called fini() on outbox variable '{}' (InstanceBox)", name);
            }
            // プラグインBoxは共有ハンドルの可能性が高いため自動finiしない
            // ビルトインBoxは元々finiメソッドを持たないので呼ばない（要修正）
        }
        
        // その後、保存されていた変数で復元
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
        
        idebug!("🔥 Static Box '{}' definition registered in statics namespace", name);
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
            self.declare_local_variable("me", (*static_instance).clone_or_share());
            
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
            idebug!("🌍 statics namespace already exists - skipping creation");
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
            let fields = global_box.get_fields();
            let mut fields_locked = fields.lock().unwrap();
            fields_locked.insert("statics".to_string(), Arc::new(statics_box));
        }
            
        idebug!("🌍 statics namespace created in GlobalBox successfully");
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
            let fields = statics_instance.get_fields();
            let mut fields_locked = fields.lock().unwrap();
            fields_locked.insert(name.to_string(), Arc::new(instance));
        }
        
        idebug!("🔥 Static box '{}' instance registered in statics namespace", name);
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
        idebug!("🔗 DEBUG: Registering invalidation for: {}", target_info);
        
        // Extract actual object ID from target_info string
        // Format: "<ClassName instance #ID>" -> extract ID
        if let Some(hash_pos) = target_info.find('#') {
            let id_str = &target_info[hash_pos + 1..];
            // Find the end of the ID (before '>')
            let id_end = id_str.find('>').unwrap_or(id_str.len());
            let clean_id_str = &id_str[..id_end];
            
            if let Ok(id) = clean_id_str.parse::<u64>() {
                self.invalidated_ids.lock().unwrap().insert(id);
                idebug!("🔗 DEBUG: Object with ID {} marked as invalidated", id);
            } else {
                idebug!("🔗 DEBUG: Failed to parse ID from: {}", clean_id_str);
            }
        } else {
            // Fallback for non-standard target_info format
            idebug!("🔗 DEBUG: No ID found in target_info, using fallback");
            if target_info.contains("Parent") {
                self.invalidated_ids.lock().unwrap().insert(999); // Fallback marker
                idebug!("🔗 DEBUG: Parent objects marked as invalidated (fallback ID 999)");
            }
        }
    }
}
// ==== MethodBox Invoker Bridge ==========================================
fn register_methodbox_invoker() {
    use crate::method_box::{MethodBox, MethodInvoker, FunctionDefinition, set_method_invoker};
    use crate::box_trait::{VoidBox};
    use std::sync::Arc;

    struct SimpleMethodInvoker;
    impl MethodInvoker for SimpleMethodInvoker {
        fn invoke(&self, method: &MethodBox, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, String> {
            // 1) 取得: メソッド定義
            let def: FunctionDefinition = if let Some(def) = &method.method_def {
                def.clone()
            } else {
                let inst_guard = method.get_instance();
                let inst_locked = inst_guard.lock().map_err(|_| "MethodBox instance lock poisoned".to_string())?;
                if let Some(inst) = inst_locked.as_any().downcast_ref::<InstanceBox>() {
                    if let Some(ast) = inst.get_method(&method.method_name) {
                        if let ASTNode::FunctionDeclaration { name, params, body, is_static, .. } = ast {
                            FunctionDefinition { name: name.clone(), params: params.clone(), body: body.clone(), is_static: *is_static }
                        } else {
                            return Err("Method AST is not a function declaration".to_string());
                        }
                    } else {
                        return Err(format!("Method '{}' not found on instance", method.method_name));
                    }
                } else {
                    return Err("MethodBox instance is not an InstanceBox".to_string());
                }
            };

            // 2) 新しいInterpreterでメソッド本体を実行（簡易）
            let mut interp = NyashInterpreter::new();
            // me = instance
            let me_box = {
                let inst_guard = method.get_instance();
                let inst_locked = inst_guard.lock().map_err(|_| "MethodBox instance lock poisoned".to_string())?;
                inst_locked.clone_or_share()
            };
            interp.declare_local_variable("me", me_box);

            // 引数をローカルへ
            if def.params.len() != args.len() {
                return Err(format!("Argument mismatch: expected {}, got {}", def.params.len(), args.len()));
            }
            for (p, v) in def.params.iter().zip(args.into_iter()) {
                interp.declare_local_variable(p, v);
            }

            // 本体実行
            let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
            for st in &def.body {
                match interp.execute_statement(st) {
                    Ok(val) => {
                        result = val;
                        if let super::ControlFlow::Return(ret) = &interp.control_flow {
                            result = ret.clone_box();
                            interp.control_flow = super::ControlFlow::None;
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(format!("Invoke error: {:?}", e));
                    }
                }
            }
            Ok(result)
        }
    }

    // 登録（複数回new()されてもOnceCellなので一度だけ）
    let _ = set_method_invoker(Arc::new(SimpleMethodInvoker));
}
