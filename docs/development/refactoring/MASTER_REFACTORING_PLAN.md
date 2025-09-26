# Nyash言語 マスターリファクタリング計画

**Version**: 1.0  
**Date**: 2025-09-20  
**Status**: 実行準備完了  
**Duration**: 13-18週間（約3-4ヶ月）  
**Scope**: JIT Cranelift除外、根本品質重視の統一化

## 🎯 戦略概要

### 核心原則
- **Progressive Unification**: 段階的統一による安全な移行
- **Fundamentals First**: 80/20回避、根本からの品質構築
- **Reference Integrity**: PyVMを品質保証の参照点として維持
- **Non-Destructive**: 既存安定機能の破壊を避ける

### 現状の問題構造
```
現在の混在状態:
├── Rust VM (Legacy) - 部分的実装、不安定
├── Python PyVM - 参照実装、安定稼働
├── MIR Interpreter - 最小実装
├── JIT Cranelift - 【除外対象】
└── Mini-VM (Nyash) - 開発中、将来の統一目標
```

### 目標アーキテクチャ
```
統一後の構造:
├── Unified VM Interface
│   ├── PyVM Backend (Reference)
│   ├── Nyash VM Backend (Primary)
│   └── MIR Interpreter Backend (Debug)
├── Common Entry Resolution
├── Unified Control Flow
├── Type Safety Layer
└── Single Macro Expansion Path
```

---

## 📋 Phase 1: VM Core統一 (4-6週間)

### 🎯 Phase 1 目標
**「3つのVM実装を統一インターフェースで管理し、段階的にNyash VMに移行する基盤を確立」**

### Step 1.1: PyVM機能完全化 (Week 1-2)

#### **実装対象**
```python
# src/llvm_py/pyvm/vm_enhanced.py
class EnhancedPyVM:
    def __init__(self):
        self.nested_function_handler = NestedFunctionHandler()
        self.control_flow_manager = ControlFlowManager()
        self.scope_boundary_tracker = ScopeBoundaryTracker()
        
    def execute_with_nested_functions(self, mir_module):
        # ネスト関数の自動リフト処理
        lifted_module = self.nested_function_handler.lift_nested_functions(mir_module)
        return self.execute_standard(lifted_module)
        
    def handle_break_continue(self, instruction, context):
        # Box境界を跨ぐbreak/continue の安全処理
        if context.is_box_boundary():
            return self.control_flow_manager.handle_box_boundary_control(instruction, context)
        return self.control_flow_manager.handle_standard_control(instruction, context)
```

#### **ネスト関数リフト機能**
```python
class NestedFunctionHandler:
    def lift_nested_functions(self, mir_module):
        """
        function outer() {
            function inner() { return 42; }  // ← これを自動リフト
            return inner();
        }
        
        ↓ 変換後
        
        function _lifted_inner_123() { return 42; }
        function outer() {
            return _lifted_inner_123();
        }
        """
        lifted_functions = []
        for func in mir_module.functions:
            nested_funcs, cleaned_func = self.extract_nested_functions(func)
            lifted_functions.extend(nested_funcs)
            lifted_functions.append(cleaned_func)
        
        return MirModule(functions=lifted_functions)
        
    def extract_nested_functions(self, function):
        # 関数内の function 宣言を検出・抽出
        # gensym による名前生成（衝突回避）
        # 呼び出し箇所を生成された名前に置換
        pass
```

#### **制御フロー強化**
```python
class ControlFlowManager:
    def __init__(self):
        self.loop_stack = []  # ループコンテキストスタック
        self.function_stack = []  # 関数コンテキストスタック
        
    def enter_loop(self, loop_id, break_target, continue_target):
        self.loop_stack.append({
            'id': loop_id,
            'break_target': break_target,
            'continue_target': continue_target,
            'in_box_method': self.is_in_box_method()
        })
        
    def handle_break(self, instruction):
        if not self.loop_stack:
            raise RuntimeError("break outside of loop")
            
        current_loop = self.loop_stack[-1]
        if current_loop['in_box_method']:
            # Boxメソッド内のbreak特別処理
            return self.handle_box_method_break(current_loop)
        return self.jump_to_target(current_loop['break_target'])
        
    def handle_continue(self, instruction):
        if not self.loop_stack:
            raise RuntimeError("continue outside of loop")
            
        current_loop = self.loop_stack[-1]
        if current_loop['in_box_method']:
            # Boxメソッド内のcontinue特別処理
            return self.handle_box_method_continue(current_loop)
        return self.jump_to_target(current_loop['continue_target'])
```

#### **受け入れ基準**
```bash
# テストスイート
./tools/test/phase1/pyvm_nested_functions.sh
./tools/test/phase1/pyvm_control_flow.sh
./tools/test/phase1/pyvm_box_boundaries.sh

# 期待する成功例
echo 'function outer() { function inner() { return 42; } return inner(); }' | python pyvm_enhanced.py
# → 出力: 42

echo 'loop(i < 10) { if (i == 5) break; i = i + 1; }' | python pyvm_enhanced.py
# → 正常終了、i = 5
```

### Step 1.2: Unified VM Interface設計 (Week 2-3)

#### **共通インターフェース実装**
```rust
// src/backend/vm_unified.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VMResult {
    pub return_value: Option<VMValue>,
    pub stdout: String,
    pub stderr: String,
    pub execution_stats: VMStats,
}

#[derive(Debug, Clone)]
pub struct VMStats {
    pub instructions_executed: u64,
    pub memory_used: usize,
    pub execution_time_ms: u64,
    pub function_calls: u64,
}

pub trait UnifiedVM {
    fn name(&self) -> &str;
    fn execute_mir(&mut self, module: &MirModule) -> Result<VMResult, VMError>;
    fn set_debug_mode(&mut self, enabled: bool);
    fn set_trace_mode(&mut self, enabled: bool);
    fn get_current_stats(&self) -> VMStats;
    fn reset_stats(&mut self);
}

// PyVM実装
pub struct PyVMBackend {
    python_path: PathBuf,
    debug_enabled: bool,
    trace_enabled: bool,
    stats: VMStats,
}

impl UnifiedVM for PyVMBackend {
    fn name(&self) -> &str { "PyVM" }
    
    fn execute_mir(&mut self, module: &MirModule) -> Result<VMResult, VMError> {
        // 1. MIRをJSON形式でシリアライズ
        let mir_json = serde_json::to_string(module)?;
        
        // 2. Python VMを呼び出し
        let mut cmd = Command::new("python");
        cmd.arg(&self.python_path)
           .arg("--mode").arg("execute")
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        if self.debug_enabled {
            cmd.arg("--debug");
        }
        if self.trace_enabled {
            cmd.arg("--trace");
        }
        
        let mut child = cmd.spawn()?;
        child.stdin.as_mut().unwrap().write_all(mir_json.as_bytes())?;
        
        let output = child.wait_with_output()?;
        
        // 3. 結果をパース
        let result: VMResult = serde_json::from_slice(&output.stdout)?;
        self.stats = result.execution_stats.clone();
        
        Ok(result)
    }
    
    fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }
    
    fn set_trace_mode(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
    }
    
    fn get_current_stats(&self) -> VMStats {
        self.stats.clone()
    }
    
    fn reset_stats(&mut self) {
        self.stats = VMStats::default();
    }
}

// Nyash VM実装（将来）
pub struct NyashVMBackend {
    vm_script_path: PathBuf,
    debug_enabled: bool,
    interpreter: NyashInterpreter,
}

impl UnifiedVM for NyashVMBackend {
    fn name(&self) -> &str { "NyashVM" }
    
    fn execute_mir(&mut self, module: &MirModule) -> Result<VMResult, VMError> {
        // Nyashスクリプトで実装されたVMを実行
        // 現在は開発中、将来的にはこれが主力
        todo!("Nyash VM implementation in progress")
    }
    
    // ... 他のメソッド実装
}
```

#### **VM管理システム**
```rust
// src/backend/vm_manager.rs
pub struct VMManager {
    backends: HashMap<String, Box<dyn UnifiedVM>>,
    default_backend: String,
    fallback_chain: Vec<String>,
}

impl VMManager {
    pub fn new() -> Self {
        let mut manager = VMManager {
            backends: HashMap::new(),
            default_backend: "PyVM".to_string(),
            fallback_chain: vec!["PyVM".to_string(), "MIRInterpreter".to_string()],
        };
        
        // 利用可能なバックエンドを登録
        manager.register_backend("PyVM", Box::new(PyVMBackend::new()));
        manager.register_backend("MIRInterpreter", Box::new(MIRInterpreterBackend::new()));
        
        // NyashVMが利用可能な場合のみ登録
        if NyashVMBackend::is_available() {
            manager.register_backend("NyashVM", Box::new(NyashVMBackend::new()));
            manager.default_backend = "NyashVM".to_string();
        }
        
        manager
    }
    
    pub fn execute_with_fallback(&mut self, module: &MirModule) -> Result<VMResult, VMError> {
        // デフォルトバックエンドを試行
        if let Some(backend) = self.backends.get_mut(&self.default_backend) {
            match backend.execute_mir(module) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Warning: {} failed: {}", self.default_backend, e);
                }
            }
        }
        
        // フォールバックチェーンを順次試行
        for backend_name in &self.fallback_chain {
            if backend_name == &self.default_backend {
                continue; // 既に試行済み
            }
            
            if let Some(backend) = self.backends.get_mut(backend_name) {
                match backend.execute_mir(module) {
                    Ok(result) => {
                        eprintln!("Fallback to {} succeeded", backend_name);
                        return Ok(result);
                    }
                    Err(e) => {
                        eprintln!("Warning: {} failed: {}", backend_name, e);
                    }
                }
            }
        }
        
        Err(VMError::AllBackendsFailed)
    }
    
    pub fn register_backend(&mut self, name: &str, backend: Box<dyn UnifiedVM>) {
        self.backends.insert(name.to_string(), backend);
    }
    
    pub fn list_available_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }
    
    pub fn get_backend_stats(&self, name: &str) -> Option<VMStats> {
        self.backends.get(name).map(|b| b.get_current_stats())
    }
}
```

### Step 1.3: Mini-VM段階的拡張 (Week 3-4)

#### **JSON v0 完全ローダー**
```nyash
// apps/selfhost/vm/json_loader.nyash
static box JSONLoader {
    parse_mir_module(json_string: StringBox) -> MirModuleBox {
        local root = me.parse_json_object(json_string)
        
        local functions = new ArrayBox()
        local funcs_array = root.get("functions")
        local i = 0
        loop(i < funcs_array.length()) {
            local func_data = funcs_array.get(i)
            local parsed_func = me.parse_function(func_data)
            functions.push(parsed_func)
            i = i + 1
        }
        
        local module = new MirModuleBox()
        module.set_functions(functions)
        return module
    }
    
    parse_function(func_obj: MapBox) -> FunctionBox {
        local name = func_obj.get("name")
        local params = me.parse_parameters(func_obj.get("parameters"))
        local body = me.parse_basic_blocks(func_obj.get("body"))
        
        local function = new FunctionBox()
        function.set_name(name)
        function.set_parameters(params)
        function.set_body(body)
        return function
    }
    
    parse_basic_blocks(blocks_array: ArrayBox) -> ArrayBox {
        local blocks = new ArrayBox()
        local i = 0
        loop(i < blocks_array.length()) {
            local block_data = blocks_array.get(i)
            local block = me.parse_basic_block(block_data)
            blocks.push(block)
            i = i + 1
        }
        return blocks
    }
    
    parse_basic_block(block_obj: MapBox) -> BasicBlockBox {
        local id = block_obj.get("id")
        local instructions = me.parse_instructions(block_obj.get("instructions"))
        
        local block = new BasicBlockBox()
        block.set_id(id)
        block.set_instructions(instructions)
        return block
    }
    
    parse_instructions(instrs_array: ArrayBox) -> ArrayBox {
        local instructions = new ArrayBox()
        local i = 0
        loop(i < instrs_array.length()) {
            local instr_data = instrs_array.get(i)
            local instruction = me.parse_instruction(instr_data)
            instructions.push(instruction)
            i = i + 1
        }
        return instructions
    }
    
    parse_instruction(instr_obj: MapBox) -> InstructionBox {
        local opcode = instr_obj.get("opcode")
        local operands = instr_obj.get("operands")
        
        local instruction = new InstructionBox()
        instruction.set_opcode(opcode)
        instruction.set_operands(operands)
        return instruction
    }
}
```

#### **MIR14命令完全実装**
```nyash
// apps/selfhost/vm/mir_executor.nyash
static box MirExecutor {
    stack: ArrayBox
    heap: MapBox
    call_stack: ArrayBox
    current_function: FunctionBox
    current_block: IntegerBox
    instruction_pointer: IntegerBox
    
    birth() {
        me.stack = new ArrayBox()
        me.heap = new MapBox()
        me.call_stack = new ArrayBox()
        me.current_block = 0
        me.instruction_pointer = 0
    }
    
    execute_module(module: MirModuleBox) -> VMResultBox {
        // エントリポイント解決
        local entry_func = me.resolve_entry_point(module)
        
        // メイン実行ループ
        me.current_function = entry_func
        local result = me.execute_function(entry_func, new ArrayBox())
        
        local vm_result = new VMResultBox()
        vm_result.set_return_value(result)
        vm_result.set_stdout(me.get_stdout())
        vm_result.set_stderr(me.get_stderr())
        return vm_result
    }
    
    execute_function(function: FunctionBox, args: ArrayBox) -> ValueBox {
        // 関数フレーム設定
        me.push_call_frame(function, args)
        
        // 基本ブロック実行
        local blocks = function.get_body()
        me.current_block = 0
        
        loop(me.current_block < blocks.length()) {
            local block = blocks.get(me.current_block)
            local result = me.execute_basic_block(block)
            
            if (result.is_return()) {
                me.pop_call_frame()
                return result.get_value()
            }
            
            if (result.is_jump()) {
                me.current_block = result.get_target_block()
            } else {
                me.current_block = me.current_block + 1
            }
        }
        
        // デフォルトリターン
        return new NullValueBox()
    }
    
    execute_basic_block(block: BasicBlockBox) -> ExecutionResultBox {
        local instructions = block.get_instructions()
        local i = 0
        
        loop(i < instructions.length()) {
            local instruction = instructions.get(i)
            local result = me.execute_instruction(instruction)
            
            if (result.is_control_flow()) {
                return result
            }
            
            i = i + 1
        }
        
        return new ContinueResultBox()
    }
    
    execute_instruction(instruction: InstructionBox) -> ExecutionResultBox {
        local opcode = instruction.get_opcode()
        local operands = instruction.get_operands()
        
        return peek opcode {
            "const" => me.execute_const(operands),
            "load" => me.execute_load(operands),
            "store" => me.execute_store(operands),
            "binop" => me.execute_binop(operands),
            "compare" => me.execute_compare(operands),
            "branch" => me.execute_branch(operands),
            "jump" => me.execute_jump(operands),
            "call" => me.execute_call(operands),
            "ret" => me.execute_ret(operands),
            "phi" => me.execute_phi(operands),
            "newbox" => me.execute_newbox(operands),
            "boxcall" => me.execute_boxcall(operands),
            "externcall" => me.execute_externcall(operands),
            "typeop" => me.execute_typeop(operands),
            else => panic("Unknown instruction: " + opcode)
        }
    }
    
    // 各命令の実装
    execute_const(operands: ArrayBox) -> ExecutionResultBox {
        local value = operands.get(0)
        local target = operands.get(1)
        me.set_variable(target, value)
        return new ContinueResultBox()
    }
    
    execute_binop(operands: ArrayBox) -> ExecutionResultBox {
        local op = operands.get(0)
        local left = me.get_variable(operands.get(1))
        local right = me.get_variable(operands.get(2))
        local target = operands.get(3)
        
        local result = peek op {
            "add" => left + right,
            "sub" => left - right,
            "mul" => left * right,
            "div" => left / right,
            else => panic("Unknown binary operation: " + op)
        }
        
        me.set_variable(target, result)
        return new ContinueResultBox()
    }
    
    execute_call(operands: ArrayBox) -> ExecutionResultBox {
        local func_name = operands.get(0)
        local args = me.evaluate_arguments(operands.slice(1))
        local target = operands.get_last()
        
        local function = me.resolve_function(func_name)
        local result = me.execute_function(function, args)
        me.set_variable(target, result)
        
        return new ContinueResultBox()
    }
    
    execute_ret(operands: ArrayBox) -> ExecutionResultBox {
        local value = if (operands.length() > 0) {
            me.get_variable(operands.get(0))
        } else {
            new NullValueBox()
        }
        
        local result = new ReturnResultBox()
        result.set_value(value)
        return result
    }
    
    execute_branch(operands: ArrayBox) -> ExecutionResultBox {
        local condition = me.get_variable(operands.get(0))
        local true_block = operands.get(1)
        local false_block = operands.get(2)
        
        local target_block = if (condition.to_boolean()) {
            true_block
        } else {
            false_block
        }
        
        local result = new JumpResultBox()
        result.set_target_block(target_block)
        return result
    }
    
    execute_jump(operands: ArrayBox) -> ExecutionResultBox {
        local target_block = operands.get(0)
        local result = new JumpResultBox()
        result.set_target_block(target_block)
        return result
    }
}
```

#### **受け入れ基準**
```bash
# Mini-VM基本機能テスト
./tools/test/phase1/mini_vm_basic.sh

# PyVMとの出力比較テスト
./tools/test/phase1/mini_vm_vs_pyvm.sh

# 期待する成功例
echo '{"functions": [{"name": "main", "body": [...]}]}' | ./mini_vm.nyash
# → PyVMと同一の出力
```

---

## 📋 Phase 2: エントリ解決統一 (2-3週間)

### 🎯 Phase 2 目標
**「すべてのVM実装で一貫したエントリポイント解決を実現し、実行時の不整合を根絶」**

### Step 2.1: エントリ解決ライブラリ設計 (Week 1)

#### **統一エントリ解決システム**
```rust
// src/runtime/entry_resolver.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntryResolverConfig {
    pub allow_toplevel_main: bool,
    pub strict_main_signature: bool,
    pub fallback_to_first_function: bool,
    pub require_main_box: bool,
}

impl Default for EntryResolverConfig {
    fn default() -> Self {
        Self {
            allow_toplevel_main: false,  // デフォルトは strict
            strict_main_signature: true,
            fallback_to_first_function: false,
            require_main_box: true,
        }
    }
}

pub struct EntryResolver {
    config: EntryResolverConfig,
}

impl EntryResolver {
    pub fn new(config: EntryResolverConfig) -> Self {
        Self { config }
    }
    
    pub fn resolve_entry_point(&self, module: &MirModule) -> Result<FunctionId, EntryResolutionError> {
        // 1. Main.main() を最優先で検索
        if let Some(func_id) = self.find_main_box_main(module) {
            return Ok(func_id);
        }
        
        // 2. 設定によってtoplevel main() を試行
        if self.config.allow_toplevel_main {
            if let Some(func_id) = self.find_toplevel_main(module) {
                return Ok(func_id);
            }
        }
        
        // 3. フォールバック戦略
        if self.config.fallback_to_first_function && !module.functions.is_empty() {
            return Ok(FunctionId(0));
        }
        
        // 4. エラー
        Err(EntryResolutionError::NoValidEntryPoint {
            found_functions: self.list_available_functions(module),
            config: self.config.clone(),
        })
    }
    
    fn find_main_box_main(&self, module: &MirModule) -> Option<FunctionId> {
        for (i, function) in module.functions.iter().enumerate() {
            // "Main.main" または "main" (Boxメソッドとして)
            if function.name == "Main.main" || 
               (function.is_box_method && function.name == "main" && function.box_name == Some("Main")) {
                
                if self.config.strict_main_signature {
                    if self.validate_main_signature(function) {
                        return Some(FunctionId(i));
                    }
                } else {
                    return Some(FunctionId(i));
                }
            }
        }
        None
    }
    
    fn find_toplevel_main(&self, module: &MirModule) -> Option<FunctionId> {
        for (i, function) in module.functions.iter().enumerate() {
            if function.name == "main" && !function.is_box_method {
                if self.config.strict_main_signature {
                    if self.validate_main_signature(function) {
                        return Some(FunctionId(i));
                    }
                } else {
                    return Some(FunctionId(i));
                }
            }
        }
        None
    }
    
    fn validate_main_signature(&self, function: &MirFunction) -> bool {
        // main() または main(args: ArrayBox) を受け入れ
        match function.parameters.len() {
            0 => true,  // main()
            1 => {
                // main(args: ArrayBox) を確認
                function.parameters[0].type_hint.as_ref()
                    .map(|t| t == "ArrayBox")
                    .unwrap_or(true)  // 型ヒントない場合は許可
            }
            _ => false,  // 2個以上の引数は不正
        }
    }
    
    fn list_available_functions(&self, module: &MirModule) -> Vec<String> {
        module.functions.iter()
            .map(|f| format!("{}({})", f.name, f.parameters.len()))
            .collect()
    }
    
    pub fn create_detailed_error_message(&self, error: &EntryResolutionError) -> String {
        match error {
            EntryResolutionError::NoValidEntryPoint { found_functions, config } => {
                let mut msg = String::new();
                msg.push_str("No valid entry point found.\n\n");
                
                msg.push_str("Expected entry points (in order of preference):\n");
                if config.require_main_box {
                    msg.push_str("  1. static box Main { main() { ... } }  (最推奨)\n");
                    msg.push_str("  2. static box Main { main(args) { ... } }\n");
                }
                if config.allow_toplevel_main {
                    msg.push_str("  3. function main() { ... }  (非推奨)\n");
                    msg.push_str("  4. function main(args) { ... }\n");
                }
                
                msg.push_str("\nFound functions:\n");
                for func in found_functions {
                    msg.push_str(&format!("  - {}\n", func));
                }
                
                msg.push_str("\nRecommendation:\n");
                msg.push_str("Add this to your code:\n");
                msg.push_str("static box Main {\n");
                msg.push_str("    main() {\n");
                msg.push_str("        // Your code here\n");
                msg.push_str("    }\n");
                msg.push_str("}\n");
                
                msg
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EntryResolutionError {
    NoValidEntryPoint {
        found_functions: Vec<String>,
        config: EntryResolverConfig,
    },
}

impl std::fmt::Display for EntryResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryResolutionError::NoValidEntryPoint { .. } => {
                write!(f, "No valid entry point found")
            }
        }
    }
}

impl std::error::Error for EntryResolutionError {}
```

#### **環境変数による設定**
```rust
// src/runtime/entry_resolver.rs に追加
impl EntryResolverConfig {
    pub fn from_environment() -> Self {
        Self {
            allow_toplevel_main: std::env::var("NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            strict_main_signature: std::env::var("NYASH_ENTRY_STRICT_SIGNATURE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true),
            fallback_to_first_function: std::env::var("NYASH_ENTRY_FALLBACK_FIRST")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            require_main_box: std::env::var("NYASH_ENTRY_REQUIRE_MAIN_BOX")
                .map(|v| v != "0" && v.to_lowercase() != "false")
                .unwrap_or(true),
        }
    }
}
```

### Step 2.2: 全バックエンド統合 (Week 2-3)

#### **Unified VM Interface に統合**
```rust
// src/backend/vm_unified.rs に追加
impl UnifiedVM for PyVMBackend {
    fn execute_mir(&mut self, module: &MirModule) -> Result<VMResult, VMError> {
        // 1. エントリ解決
        let resolver = EntryResolver::new(EntryResolverConfig::from_environment());
        let entry_func_id = resolver.resolve_entry_point(module)
            .map_err(|e| VMError::EntryResolution(e))?;
        
        // 2. モジュールにエントリ情報を付加
        let mut module_with_entry = module.clone();
        module_with_entry.entry_point = Some(entry_func_id);
        
        // 3. PyVMに渡すJSONに含める
        let mir_json = serde_json::to_string(&module_with_entry)?;
        
        // ... 以下既存のPyVM実行ロジック
    }
}
```

#### **PyVM側での対応**
```python
# src/llvm_py/pyvm/vm_enhanced.py に追加
class EnhancedPyVM:
    def execute_mir_module(self, mir_json_str):
        mir_module = json.loads(mir_json_str)
        
        # エントリポイント解決（Rust側で解決済みを使用）
        if 'entry_point' in mir_module:
            entry_func_id = mir_module['entry_point']
            entry_function = mir_module['functions'][entry_func_id]
        else:
            # フォールバック（古い形式互換性）
            entry_function = self.resolve_entry_legacy(mir_module)
        
        # メイン実行
        result = self.execute_function(entry_function, [])
        
        return {
            'return_value': result,
            'stdout': self.get_stdout(),
            'stderr': self.get_stderr(),
            'execution_stats': self.get_stats(),
        }
    
    def resolve_entry_legacy(self, mir_module):
        """レガシー互換性のためのエントリ解決"""
        # "Main.main" を検索
        for func in mir_module['functions']:
            if func['name'] == 'Main.main':
                return func
        
        # "main" を検索
        for func in mir_module['functions']:
            if func['name'] == 'main':
                return func
        
        # 最初の関数を使用
        if mir_module['functions']:
            return mir_module['functions'][0]
        
        raise RuntimeError("No executable function found")
```

#### **受け入れ基準**
```bash
# エントリ解決統一テスト
./tools/test/phase2/entry_resolution_unified.sh

# 設定による動作変更テスト
NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 ./tools/test/phase2/toplevel_main_test.sh
NYASH_ENTRY_STRICT_SIGNATURE=0 ./tools/test/phase2/relaxed_signature_test.sh

# 期待する成功例
# Main.main() パターン
echo 'static box Main { main() { print("Hello"); } }' | ./nyash --backend pyvm
# → Hello

# toplevel main() パターン（環境変数有効時）
NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 echo 'function main() { print("World"); }' | ./nyash --backend pyvm
# → World

# エラーメッセージ品質確認
echo 'function foo() { print("test"); }' | ./nyash --backend pyvm
# → 詳細なエラーメッセージと修正提案
```

---

## 📋 Phase 3: 制御フロー根本修正 (3-4週間)

### 🎯 Phase 3 目標
**「ネスト関数、break/continue、Box境界での制御フローを完全に統一し、言語機能の完全性を達成」**

### Step 3.1: 制御フロー統一設計 (Week 1)

#### **制御フローコンテキスト管理**
```rust
// src/mir/control_flow_unified.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ControlFlowContext {
    loop_stack: Vec<LoopContext>,
    function_stack: Vec<FunctionContext>,
    break_targets: HashMap<LoopId, BasicBlockId>,
    continue_targets: HashMap<LoopId, BasicBlockId>,
    scope_stack: Vec<ScopeContext>,
}

#[derive(Debug, Clone)]
pub struct LoopContext {
    id: LoopId,
    break_label: String,
    continue_label: String,
    is_in_box_method: bool,
    parent_function: FunctionId,
}

#[derive(Debug, Clone)]
pub struct FunctionContext {
    id: FunctionId,
    name: String,
    is_box_method: bool,
    box_name: Option<String>,
    return_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeContext {
    scope_type: ScopeType,
    depth: usize,
    parent_function: FunctionId,
    variables: HashMap<String, VariableInfo>,
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Function,
    Loop,
    If,
    Block,
    BoxMethod,
}

impl ControlFlowContext {
    pub fn new() -> Self {
        Self {
            loop_stack: Vec::new(),
            function_stack: Vec::new(),
            break_targets: HashMap::new(),
            continue_targets: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }
    
    pub fn enter_function(&mut self, func_id: FunctionId, name: String, is_box_method: bool) {
        let context = FunctionContext {
            id: func_id,
            name,
            is_box_method,
            box_name: None, // TODO: 実際のBox名を取得
            return_type: None,
        };
        
        self.function_stack.push(context);
        
        self.scope_stack.push(ScopeContext {
            scope_type: if is_box_method { ScopeType::BoxMethod } else { ScopeType::Function },
            depth: self.scope_stack.len(),
            parent_function: func_id,
            variables: HashMap::new(),
        });
    }
    
    pub fn exit_function(&mut self) {
        self.function_stack.pop();
        
        // 関数スコープを削除
        while let Some(scope) = self.scope_stack.last() {
            if matches!(scope.scope_type, ScopeType::Function | ScopeType::BoxMethod) {
                self.scope_stack.pop();
                break;
            }
            self.scope_stack.pop();
        }
    }
    
    pub fn enter_loop(&mut self, loop_id: LoopId, break_target: BasicBlockId, continue_target: BasicBlockId) {
        let current_function = self.current_function_id();
        let is_in_box_method = self.is_in_box_method();
        
        let context = LoopContext {
            id: loop_id,
            break_label: format!("break_{}", loop_id.0),
            continue_label: format!("continue_{}", loop_id.0),
            is_in_box_method,
            parent_function: current_function,
        };
        
        self.loop_stack.push(context);
        self.break_targets.insert(loop_id, break_target);
        self.continue_targets.insert(loop_id, continue_target);
        
        self.scope_stack.push(ScopeContext {
            scope_type: ScopeType::Loop,
            depth: self.scope_stack.len(),
            parent_function: current_function,
            variables: HashMap::new(),
        });
    }
    
    pub fn exit_loop(&mut self) {
        if let Some(loop_context) = self.loop_stack.pop() {
            self.break_targets.remove(&loop_context.id);
            self.continue_targets.remove(&loop_context.id);
        }
        
        // ループスコープを削除
        while let Some(scope) = self.scope_stack.last() {
            if matches!(scope.scope_type, ScopeType::Loop) {
                self.scope_stack.pop();
                break;
            }
            self.scope_stack.pop();
        }
    }
    
    pub fn handle_break(&self) -> Result<BasicBlockId, ControlFlowError> {
        let current_loop = self.loop_stack.last()
            .ok_or(ControlFlowError::BreakOutsideLoop)?;
        
        if current_loop.is_in_box_method {
            // Boxメソッド内のbreak特別処理
            self.handle_box_method_break(current_loop)
        } else {
            // 通常のbreak処理
            Ok(self.break_targets[&current_loop.id])
        }
    }
    
    pub fn handle_continue(&self) -> Result<BasicBlockId, ControlFlowError> {
        let current_loop = self.loop_stack.last()
            .ok_or(ControlFlowError::ContinueOutsideLoop)?;
        
        if current_loop.is_in_box_method {
            // Boxメソッド内のcontinue特別処理
            self.handle_box_method_continue(current_loop)
        } else {
            // 通常のcontinue処理
            Ok(self.continue_targets[&current_loop.id])
        }
    }
    
    fn handle_box_method_break(&self, loop_context: &LoopContext) -> Result<BasicBlockId, ControlFlowError> {
        // Boxメソッド境界を超えるbreak処理
        // TODO: Boxメソッドの特別な制御フロー処理
        Ok(self.break_targets[&loop_context.id])
    }
    
    fn handle_box_method_continue(&self, loop_context: &LoopContext) -> Result<BasicBlockId, ControlFlowError> {
        // Boxメソッド境界を超えるcontinue処理
        // TODO: Boxメソッドの特別な制御フロー処理
        Ok(self.continue_targets[&loop_context.id])
    }
    
    pub fn current_function_id(&self) -> FunctionId {
        self.function_stack.last()
            .map(|ctx| ctx.id)
            .unwrap_or(FunctionId(0))
    }
    
    pub fn is_in_box_method(&self) -> bool {
        self.function_stack.last()
            .map(|ctx| ctx.is_box_method)
            .unwrap_or(false)
    }
    
    pub fn validate_break(&self) -> Result<(), ControlFlowError> {
        if self.loop_stack.is_empty() {
            return Err(ControlFlowError::BreakOutsideLoop);
        }
        
        // Box境界チェック
        let current_loop = &self.loop_stack[self.loop_stack.len() - 1];
        let current_function = self.current_function_id();
        
        if current_loop.parent_function != current_function {
            return Err(ControlFlowError::BreakAcrossFunctionBoundary);
        }
        
        Ok(())
    }
    
    pub fn validate_continue(&self) -> Result<(), ControlFlowError> {
        if self.loop_stack.is_empty() {
            return Err(ControlFlowError::ContinueOutsideLoop);
        }
        
        // Box境界チェック
        let current_loop = &self.loop_stack[self.loop_stack.len() - 1];
        let current_function = self.current_function_id();
        
        if current_loop.parent_function != current_function {
            return Err(ControlFlowError::ContinueAcrossFunctionBoundary);
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ControlFlowError {
    BreakOutsideLoop,
    ContinueOutsideLoop,
    BreakAcrossFunctionBoundary,
    ContinueAcrossFunctionBoundary,
    InvalidLoopNesting,
}

impl std::fmt::Display for ControlFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlowError::BreakOutsideLoop => 
                write!(f, "break statement outside of loop"),
            ControlFlowError::ContinueOutsideLoop => 
                write!(f, "continue statement outside of loop"),
            ControlFlowError::BreakAcrossFunctionBoundary => 
                write!(f, "break statement cannot cross function boundary"),
            ControlFlowError::ContinueAcrossFunctionBoundary => 
                write!(f, "continue statement cannot cross function boundary"),
            ControlFlowError::InvalidLoopNesting => 
                write!(f, "invalid loop nesting structure"),
        }
    }
}

impl std::error::Error for ControlFlowError {}
```

### Step 3.2: ネスト関数リフト標準化 (Week 2)

#### **ネスト関数自動リフトシステム**
```rust
// src/mir/nested_function_lifter.rs
use std::collections::{HashMap, HashSet};

pub struct NestedFunctionLifter {
    symbol_generator: SymbolGenerator,
    captured_variables: HashMap<FunctionId, HashSet<String>>,
}

impl NestedFunctionLifter {
    pub fn new() -> Self {
        Self {
            symbol_generator: SymbolGenerator::new(),
            captured_variables: HashMap::new(),
        }
    }
    
    pub fn lift_nested_functions(&mut self, module: &mut MirModule) -> Result<(), LiftError> {
        let mut lifted_functions = Vec::new();
        let mut modified_functions = Vec::new();
        
        for (func_id, function) in module.functions.iter().enumerate() {
            let (nested_funcs, modified_func) = self.extract_and_lift_nested(
                FunctionId(func_id), 
                function
            )?;
            
            lifted_functions.extend(nested_funcs);
            modified_functions.push(modified_func);
        }
        
        // 元の関数を置換
        module.functions = modified_functions;
        
        // リフトされた関数を追加
        module.functions.extend(lifted_functions);
        
        Ok(())
    }
    
    fn extract_and_lift_nested(&mut self, parent_id: FunctionId, function: &MirFunction) 
        -> Result<(Vec<MirFunction>, MirFunction), LiftError> {
        
        let mut nested_functions = Vec::new();
        let mut modified_body = function.body.clone();
        let mut replacement_map = HashMap::new();
        
        // 1. ネスト関数を検出・抽出
        for (block_idx, block) in function.body.iter().enumerate() {
            for (instr_idx, instruction) in block.instructions.iter().enumerate() {
                if let Some(nested_func) = self.extract_function_declaration(instruction)? {
                    // 新しい関数名を生成
                    let lifted_name = self.symbol_generator.generate_function_name(
                        &function.name, 
                        &nested_func.name
                    );
                    
                    // キャプチャ変数を分析
                    let captured = self.analyze_captured_variables(&nested_func, function)?;
                    
                    if !captured.is_empty() {
                        return Err(LiftError::CapturedVariablesNotSupported {
                            function_name: nested_func.name.clone(),
                            captured_variables: captured,
                        });
                    }
                    
                    // リフトされた関数を作成
                    let mut lifted_func = nested_func.clone();
                    lifted_func.name = lifted_name.clone();
                    lifted_func.is_nested = false;
                    
                    nested_functions.push(lifted_func);
                    
                    // 置換マップに記録
                    replacement_map.insert(nested_func.name.clone(), lifted_name);
                    
                    // 元の命令を削除（no-op に置換）
                    modified_body[block_idx].instructions[instr_idx] = MirInstruction::noop();
                }
            }
        }
        
        // 2. 関数呼び出しを置換
        self.replace_function_calls(&mut modified_body, &replacement_map)?;
        
        let mut modified_function = function.clone();
        modified_function.body = modified_body;
        
        Ok((nested_functions, modified_function))
    }
    
    fn extract_function_declaration(&self, instruction: &MirInstruction) 
        -> Result<Option<MirFunction>, LiftError> {
        
        match instruction {
            MirInstruction::FunctionDeclaration { function } => {
                Ok(Some(function.clone()))
            }
            _ => Ok(None)
        }
    }
    
    fn analyze_captured_variables(&self, nested_func: &MirFunction, parent_func: &MirFunction) 
        -> Result<HashSet<String>, LiftError> {
        
        let mut captured = HashSet::new();
        let mut nested_locals = HashSet::new();
        let parent_locals = self.collect_local_variables(parent_func);
        
        // ネスト関数のローカル変数を収集
        for param in &nested_func.parameters {
            nested_locals.insert(param.name.clone());
        }
        
        // ネスト関数内で使用される変数を分析
        for block in &nested_func.body {
            for instruction in &block.instructions {
                for used_var in self.extract_used_variables(instruction) {
                    if !nested_locals.contains(&used_var) && parent_locals.contains(&used_var) {
                        captured.insert(used_var);
                    }
                }
            }
        }
        
        Ok(captured)
    }
    
    fn collect_local_variables(&self, function: &MirFunction) -> HashSet<String> {
        let mut locals = HashSet::new();
        
        // パラメータを追加
        for param in &function.parameters {
            locals.insert(param.name.clone());
        }
        
        // ローカル変数宣言を検索
        for block in &function.body {
            for instruction in &block.instructions {
                if let Some(var_name) = self.extract_local_declaration(instruction) {
                    locals.insert(var_name);
                }
            }
        }
        
        locals
    }
    
    fn extract_used_variables(&self, instruction: &MirInstruction) -> Vec<String> {
        match instruction {
            MirInstruction::Load { source, .. } => vec![source.clone()],
            MirInstruction::Store { target, source } => vec![target.clone(), source.clone()],
            MirInstruction::BinOp { left, right, .. } => vec![left.clone(), right.clone()],
            MirInstruction::Call { args, .. } => args.clone(),
            // ... 他の命令タイプ
            _ => Vec::new(),
        }
    }
    
    fn extract_local_declaration(&self, instruction: &MirInstruction) -> Option<String> {
        match instruction {
            MirInstruction::LocalDeclaration { name } => Some(name.clone()),
            _ => None,
        }
    }
    
    fn replace_function_calls(&self, body: &mut Vec<BasicBlock>, replacement_map: &HashMap<String, String>) 
        -> Result<(), LiftError> {
        
        for block in body.iter_mut() {
            for instruction in block.instructions.iter_mut() {
                if let MirInstruction::Call { function_name, .. } = instruction {
                    if let Some(new_name) = replacement_map.get(function_name) {
                        *function_name = new_name.clone();
                    }
                }
            }
        }
        
        Ok(())
    }
}

pub struct SymbolGenerator {
    counter: u32,
}

impl SymbolGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
    
    pub fn generate_function_name(&mut self, parent_name: &str, nested_name: &str) -> String {
        self.counter += 1;
        format!("_lifted_{}_{}__{}", parent_name, nested_name, self.counter)
    }
}

#[derive(Debug, Clone)]
pub enum LiftError {
    CapturedVariablesNotSupported {
        function_name: String,
        captured_variables: HashSet<String>,
    },
    InvalidNestedFunction {
        function_name: String,
        reason: String,
    },
}

impl std::fmt::Display for LiftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiftError::CapturedVariablesNotSupported { function_name, captured_variables } => {
                write!(f, 
                    "Nested function '{}' captures variables from parent scope: {:?}. \
                    Closure capture is not yet supported. \
                    Please move the function to top level or box level.",
                    function_name, captured_variables
                )
            }
            LiftError::InvalidNestedFunction { function_name, reason } => {
                write!(f, "Invalid nested function '{}': {}", function_name, reason)
            }
        }
    }
}

impl std::error::Error for LiftError {}
```

### Step 3.3: Box境界制御フロー (Week 3-4)

#### **Box境界制御フロー処理**
```rust
// src/mir/box_boundary_control_flow.rs
use std::collections::HashMap;

pub struct BoxBoundaryControlFlowHandler {
    box_method_contexts: HashMap<FunctionId, BoxMethodContext>,
    active_loops: HashMap<LoopId, LoopBoundaryInfo>,
}

#[derive(Debug, Clone)]
pub struct BoxMethodContext {
    box_name: String,
    method_name: String,
    parent_scope: Option<ScopeId>,
    boundary_type: BoxBoundaryType,
}

#[derive(Debug, Clone)]
pub enum BoxBoundaryType {
    Isolated,        // Box内で完結
    Transparent,     // 外部ループを継承
    Controlled,      // 特別な制御が必要
}

#[derive(Debug, Clone)]
pub struct LoopBoundaryInfo {
    loop_id: LoopId,
    parent_function: FunctionId,
    crossing_box_methods: Vec<FunctionId>,
    break_handling: BreakHandlingStrategy,
    continue_handling: ContinueHandlingStrategy,
}

#[derive(Debug, Clone)]
pub enum BreakHandlingStrategy {
    Standard,           // 通常のbreak処理
    BoxMethodExit,      // Boxメソッド終了として処理
    ParentLoopBreak,    // 親ループのbreakに変換
    Error,              // エラーとして処理
}

#[derive(Debug, Clone)]
pub enum ContinueHandlingStrategy {
    Standard,           // 通常のcontinue処理
    BoxMethodReturn,    // Boxメソッドreturnとして処理
    ParentLoopContinue, // 親ループのcontinueに変換
    Error,              // エラーとして処理
}

impl BoxBoundaryControlFlowHandler {
    pub fn new() -> Self {
        Self {
            box_method_contexts: HashMap::new(),
            active_loops: HashMap::new(),
        }
    }
    
    pub fn register_box_method(&mut self, func_id: FunctionId, box_name: String, method_name: String) {
        let context = BoxMethodContext {
            box_name,
            method_name,
            parent_scope: None,
            boundary_type: self.determine_boundary_type(&box_name),
        };
        
        self.box_method_contexts.insert(func_id, context);
    }
    
    fn determine_boundary_type(&self, box_name: &str) -> BoxBoundaryType {
        // Boxの種類に応じて境界タイプを決定
        match box_name {
            // システムBoxは透明
            "ArrayBox" | "StringBox" | "MapBox" => BoxBoundaryType::Transparent,
            
            // ユーザー定義Boxは分離
            _ => BoxBoundaryType::Isolated,
        }
    }
    
    pub fn handle_loop_entry(&mut self, loop_id: LoopId, current_function: FunctionId) {
        let boundary_info = LoopBoundaryInfo {
            loop_id,
            parent_function: current_function,
            crossing_box_methods: Vec::new(),
            break_handling: self.determine_break_strategy(current_function),
            continue_handling: self.determine_continue_strategy(current_function),
        };
        
        self.active_loops.insert(loop_id, boundary_info);
    }
    
    pub fn handle_loop_exit(&mut self, loop_id: LoopId) {
        self.active_loops.remove(&loop_id);
    }
    
    fn determine_break_strategy(&self, function_id: FunctionId) -> BreakHandlingStrategy {
        if let Some(context) = self.box_method_contexts.get(&function_id) {
            match context.boundary_type {
                BoxBoundaryType::Isolated => BreakHandlingStrategy::BoxMethodExit,
                BoxBoundaryType::Transparent => BreakHandlingStrategy::Standard,
                BoxBoundaryType::Controlled => BreakHandlingStrategy::ParentLoopBreak,
            }
        } else {
            BreakHandlingStrategy::Standard
        }
    }
    
    fn determine_continue_strategy(&self, function_id: FunctionId) -> ContinueHandlingStrategy {
        if let Some(context) = self.box_method_contexts.get(&function_id) {
            match context.boundary_type {
                BoxBoundaryType::Isolated => ContinueHandlingStrategy::BoxMethodReturn,
                BoxBoundaryType::Transparent => ContinueHandlingStrategy::Standard,
                BoxBoundaryType::Controlled => ContinueHandlingStrategy::ParentLoopContinue,
            }
        } else {
            ContinueHandlingStrategy::Standard
        }
    }
    
    pub fn process_break(&self, loop_id: LoopId, current_function: FunctionId) 
        -> Result<ControlFlowAction, BoxBoundaryError> {
        
        let loop_info = self.active_loops.get(&loop_id)
            .ok_or(BoxBoundaryError::LoopNotFound(loop_id))?;
        
        match loop_info.break_handling {
            BreakHandlingStrategy::Standard => {
                Ok(ControlFlowAction::Break(loop_info.loop_id))
            }
            
            BreakHandlingStrategy::BoxMethodExit => {
                // Boxメソッド終了として処理
                Ok(ControlFlowAction::Return(None))
            }
            
            BreakHandlingStrategy::ParentLoopBreak => {
                // 親ループのbreakに変換
                if loop_info.parent_function != current_function {
                    self.find_parent_loop_break(current_function)
                } else {
                    Ok(ControlFlowAction::Break(loop_info.loop_id))
                }
            }
            
            BreakHandlingStrategy::Error => {
                Err(BoxBoundaryError::BreakAcrossBoundary {
                    loop_id,
                    function_id: current_function,
                })
            }
        }
    }
    
    pub fn process_continue(&self, loop_id: LoopId, current_function: FunctionId) 
        -> Result<ControlFlowAction, BoxBoundaryError> {
        
        let loop_info = self.active_loops.get(&loop_id)
            .ok_or(BoxBoundaryError::LoopNotFound(loop_id))?;
        
        match loop_info.continue_handling {
            ContinueHandlingStrategy::Standard => {
                Ok(ControlFlowAction::Continue(loop_info.loop_id))
            }
            
            ContinueHandlingStrategy::BoxMethodReturn => {
                // Boxメソッドreturnとして処理（継続的な意味で）
                Ok(ControlFlowAction::Return(None))
            }
            
            ContinueHandlingStrategy::ParentLoopContinue => {
                // 親ループのcontinueに変換
                if loop_info.parent_function != current_function {
                    self.find_parent_loop_continue(current_function)
                } else {
                    Ok(ControlFlowAction::Continue(loop_info.loop_id))
                }
            }
            
            ContinueHandlingStrategy::Error => {
                Err(BoxBoundaryError::ContinueAcrossBoundary {
                    loop_id,
                    function_id: current_function,
                })
            }
        }
    }
    
    fn find_parent_loop_break(&self, current_function: FunctionId) 
        -> Result<ControlFlowAction, BoxBoundaryError> {
        
        // 現在の関数の親スコープでアクティブなループを検索
        for (loop_id, loop_info) in &self.active_loops {
            if loop_info.crossing_box_methods.contains(&current_function) {
                return Ok(ControlFlowAction::Break(*loop_id));
            }
        }
        
        Err(BoxBoundaryError::NoParentLoop(current_function))
    }
    
    fn find_parent_loop_continue(&self, current_function: FunctionId) 
        -> Result<ControlFlowAction, BoxBoundaryError> {
        
        // 現在の関数の親スコープでアクティブなループを検索
        for (loop_id, loop_info) in &self.active_loops {
            if loop_info.crossing_box_methods.contains(&current_function) {
                return Ok(ControlFlowAction::Continue(*loop_id));
            }
        }
        
        Err(BoxBoundaryError::NoParentLoop(current_function))
    }
}

#[derive(Debug, Clone)]
pub enum ControlFlowAction {
    Break(LoopId),
    Continue(LoopId),
    Return(Option<String>),
    Jump(BasicBlockId),
}

#[derive(Debug, Clone)]
pub enum BoxBoundaryError {
    LoopNotFound(LoopId),
    BreakAcrossBoundary { loop_id: LoopId, function_id: FunctionId },
    ContinueAcrossBoundary { loop_id: LoopId, function_id: FunctionId },
    NoParentLoop(FunctionId),
    InvalidBoundaryType(String),
}

impl std::fmt::Display for BoxBoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxBoundaryError::LoopNotFound(loop_id) => 
                write!(f, "Loop {} not found in active loop contexts", loop_id.0),
            BoxBoundaryError::BreakAcrossBoundary { loop_id, function_id } => 
                write!(f, "break statement in function {} cannot reach loop {}", function_id.0, loop_id.0),
            BoxBoundaryError::ContinueAcrossBoundary { loop_id, function_id } => 
                write!(f, "continue statement in function {} cannot reach loop {}", function_id.0, loop_id.0),
            BoxBoundaryError::NoParentLoop(function_id) => 
                write!(f, "No parent loop found for function {}", function_id.0),
            BoxBoundaryError::InvalidBoundaryType(msg) => 
                write!(f, "Invalid box boundary type: {}", msg),
        }
    }
}

impl std::error::Error for BoxBoundaryError {}
```

#### **受け入れ基準**
```bash
# 制御フロー統合テスト
./tools/test/phase3/control_flow_unified.sh

# ネスト関数リフトテスト
./tools/test/phase3/nested_function_lift.sh

# Box境界制御フローテスト
./tools/test/phase3/box_boundary_control_flow.sh

# 期待する成功例
# ネスト関数の自動リフト
echo 'function outer() { function inner() { return 42; } return inner(); }' | ./nyash --backend pyvm
# → 42 (正常実行)

# Box境界でのbreak/continue
echo 'local arr = new ArrayBox(); loop(i < 10) { arr.forEach(x => { if (x > 5) break; }); }' | ./nyash --backend pyvm
# → 適切なエラーメッセージまたは正常実行

# 複雑な制御フロー
echo 'loop(i < 10) { loop(j < 5) { if (condition) continue; if (other) break; } }' | ./nyash --backend pyvm
# → 正常実行
```

---

この詳細計画により、Nyashの根本的な品質問題を段階的に解決し、真の意味でのセルフホスティング成功への基盤を確立できます。