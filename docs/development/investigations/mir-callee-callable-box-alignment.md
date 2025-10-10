# MIR Callee統一との整合性調査報告

**調査日**: 2025-10-10
**調査者**: Claude Code
**ソース**: ChatGPT提案「CallableBoxは MIRの Callee と同型の表現を保持」

---

## 📋 概要

ChatGPT提案の「CallableBox」とMIR Calleeシステムの同型性を徹底調査し、実装可能性と課題を評価した。

---

## 1. MIR Callee定義の詳細

### 1-1. Callee型の完全定義

**ソース**: `/home/tomoaki/git/hakorune-selfhost/src/mir/definitions/call_unified.rs`

```rust
/// Call target specification for type-safe function resolution
#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    /// Global function call (e.g., nyash.builtin.print)
    Global(String),

    /// Module function call (e.g., ParserBox.starts_with/3)
    ModuleFunction(String),

    /// Box method call with explicit receiver
    Method {
        box_name: String,           // "StringBox", "ConsoleStd", etc.
        method: String,             // "upper", "print", etc.
        receiver: Option<ValueId>,  // Some(obj) for instance, None for static/constructor
        certainty: TypeCertainty,   // Phase 3: known vs union
    },

    /// Constructor call (NewBox equivalent)
    Constructor {
        box_type: String,           // "StringBox", "ArrayBox", etc.
    },

    /// Closure creation (NewClosure equivalent)
    Closure {
        params: Vec<String>,
        captures: Vec<(String, ValueId)>,
        me_capture: Option<ValueId>,
    },

    /// Dynamic function value call
    Value(ValueId),

    /// External C ABI function call
    Extern(String),
}
```

### 1-2. 各バリアントの説明

| Callee variant | 説明 | データ構造 |
|---------------|------|----------|
| **Global** | 組み込み/グローバル関数 | `String` (関数名) |
| **ModuleFunction** | モジュール関数（静的Box） | `String` (完全修飾名) |
| **Method** | Boxメソッド呼び出し | box_name + method + receiver + certainty |
| **Constructor** | Box生成（NewBox相当） | box_type |
| **Closure** | クロージャ生成 | params + captures + me_capture |
| **Value** | 動的関数値呼び出し | ValueId |
| **Extern** | 外部C ABI呼び出し | String (extern名) |

### 1-3. MirCall命令の構造

```rust
pub struct MirCall {
    pub dst: Option<ValueId>,      // 戻り値
    pub callee: Callee,             // 呼び出しターゲット
    pub args: Vec<ValueId>,         // 引数（receiverは含まない）
    pub flags: CallFlags,           // フラグ
    pub effects: EffectMask,        // 副作用マスク
}
```

**重要**: `Method` の receiver は `Callee::Method` 内に含まれ、`args` には含まれない。

---

## 2. CallableBox↔Callee対応分析

### 2-1. 1対1対応表

| Callee variant | CallableBox実現 | 実装難易度 | 備考 |
|---------------|----------------|----------|------|
| **Global** | ✅ 可能 | 低 | 関数名を保持して呼び出し |
| **ModuleFunction** | ✅ 可能 | 低 | Global同様に関数名保持 |
| **Method** | ⚠️ 部分的 | 中 | receiverの扱いが課題 |
| **Constructor** | ✅ 可能 | 低 | box_typeを保持 |
| **Closure** | ✅ 可能 | 中 | FunctionBox既存実装あり |
| **Value** | ✅ 可能 | 低 | 既にValueId呼び出し対応 |
| **Extern** | ✅ 可能 | 低 | extern名を保持 |

### 2-2. receiverの扱い（最大の課題）

#### 問題点

**ChatGPT提案**:
```hakorune
// モジュール関数参照
local cb = ref Module.func/2

// 呼び出し
local result = cb.call(args)
```

**MIR Callee::Method構造**:
```rust
Method {
    box_name: String,
    method: String,
    receiver: Option<ValueId>,  // ← receiverはCallee内に含まれる
    certainty: TypeCertainty,
}
```

**課題**:
1. **Methodの場合、receiverは`Callee`内に含まれる**
   - `cb.call([arg1, arg2])` では receiver が指定できない
   - `ref obj.method` 時点で receiver を捕捉する必要がある

2. **部分適用（partial application）の必要性**
   ```hakorune
   // receiverを捕捉したメソッド参照
   local cb = ref myString.upper/0
   local result = cb.call([])  // myString.upper() 相当
   ```

#### 解決策候補

**Option A: receiverを内部に捕捉**
```hakorune
box CallableBox {
    _callee_type: StringBox      // "Method", "Global", etc.
    _function_name: StringBox
    _box_name: StringBox         // Method用
    _method_name: StringBox      // Method用
    _receiver: any               // Method用（部分適用）
    _params: ArrayBox
    _captures: MapBox

    birth() { /* フィールド初期化 */ }

    call(args: ArrayBox) {
        // _callee_typeで分岐
        if me._callee_type == "Method" {
            // receiverは既に me._receiver に捕捉されている
            return /* MirCall with Callee::Method */
        }
        // ...
    }
}
```

**Option B: receiver付きcall()メソッド**
```hakorune
box CallableBox {
    // ...

    // 静的メソッド参照用（receiverなし）
    call_static(args: ArrayBox)

    // インスタンスメソッド用（receiver指定）
    call_with_receiver(receiver: any, args: ArrayBox)
}
```

---

## 3. VM/LLVM/Pluginでの一経路維持

### 3-1. 現在の実装状況

#### VM Backend

**ソース**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/legacy/callee_dispatcher.rs`

```rust
impl MirInterpreter {
    pub(crate) fn execute_callee_call(
        &mut self,
        callee: &Callee,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match callee {
            Callee::Global(func_name) => self.handle_callee_global(func_name, args),
            Callee::ModuleFunction(func_name) => self.handle_callee_module_function(func_name, args),
            Callee::Method { box_name, method, receiver, .. } => {
                self.handle_method_call_legacy(box_name, method, *receiver, args)
            }
            Callee::Constructor { box_type } => Err(VMError::InvalidInstruction(...)),  // ❌ 未実装
            Callee::Closure { .. } => Err(VMError::InvalidInstruction(...)),            // ❌ 未実装
            Callee::Value(func_val_id) => Err(VMError::InvalidInstruction(...)),        // ❌ 未実装
            Callee::Extern(extern_name) => self.handle_callee_extern(extern_name, args),
        }
    }
}
```

**実装状況**:
- ✅ Global, ModuleFunction, Method, Extern: 実装済み
- ❌ Constructor, Closure, Value: **未実装**

#### LLVM Backend

**ソース**: `/home/tomoaki/git/hakorune-selfhost/src/llvm_py/instructions/mir_call.py`

```python
def lower_mir_call(owner, builder, mir_call, dst_vid, vmap, resolver):
    callee = mir_call.get("callee", {})
    callee_type = callee.get("type")

    if callee_type == "Global":
        lower_global_call(...)
    elif callee_type == "ModuleFunction":
        lower_global_call(...)  # ModuleFunctionもGlobalと同じ経路
    elif callee_type == "Method":
        lower_method_call(...)
    elif callee_type == "Constructor":
        lower_constructor_call(...)
    elif callee_type == "Closure":
        lower_closure_creation(...)
    elif callee_type == "Value":
        lower_value_call(...)
    elif callee_type == "Extern":
        lower_extern_call(...)
```

**実装状況**:
- ✅ **全バリアント実装済み**（LLVM Pythonバックエンドは完全実装）

#### Plugin Backend

- Plugin経由の呼び出しは `BoxCall` 経由で実行
- MirCall統一は進行中だが、Pluginは既存のBoxCall経路を使用

### 3-2. CallableBox導入の影響

#### 新しいMIR命令が必要か？

**回答: NO（既存MirCall命令で実現可能）**

**理由**:
1. CallableBox.call() は内部で `Callee::Value(callback_id)` を生成
2. 既存の `MirInstruction::Call` で `Callee::Value` をサポート（定義済み）
3. VMが `Callee::Value` の実装を追加すれば動作する

**実装イメージ**:
```rust
// VM Backend追加実装
Callee::Value(func_val_id) => {
    let func_val = self.reg_load(*func_val_id)?;

    // FunctionBoxの場合
    if let Some(func_box) = func_val.as_function_box() {
        return self.execute_function_box(func_box, args);
    }

    // CallableBoxの場合（将来実装）
    if let Some(callable) = func_val.as_callable_box() {
        return callable.call(args);  // CallableBox.call() を呼ぶ
    }

    Err(VMError::InvalidFunctionValue)
}
```

#### 各バックエンドの改修規模

| Backend | 改修箇所 | 見積もり工数 | 優先度 |
|---------|---------|------------|--------|
| **VM** | `execute_callee_call` に `Callee::Value` 実装追加 | 1-2人日 | 高 |
| **LLVM** | 既に実装済み（`lower_value_call`） | 0人日 | - |
| **Plugin** | BoxCall経由で動作（改修不要） | 0人日 | - |

---

## 4. 実装例の検証

### 4-1. ref構文のパース

**現状**: `ref` キーワードは**存在しない**

**検証**:
```bash
$ grep -r "TokenType::Ref" src/
# → 結果なし
```

**必要な実装**:
1. **Lexer**: `ref` キーワード追加
2. **Parser**: `ref Module.func/2` 構文のパース
3. **AST**: `ASTNode::FunctionReference` ノード追加
4. **MIR Builder**: CallableBox生成命令への変換

**見積もり**: 2-3人日

### 4-2. CallableBox生成（MIRレベル）

**Option A: 新規MIR命令 `NewCallable`**
```rust
NewCallable {
    dst: ValueId,
    callee: Callee,  // 参照するCallee
}
```

**Option B: 既存 `NewBox` + 初期化**
```rust
// NewBox で CallableBox 生成
NewBox {
    dst: v%1,
    box_type: "CallableBox",
    args: [],
}

// フィールド初期化（BoxCallで設定）
BoxCall {
    box_val: v%1,
    method: "set_callee",
    args: [/* Callee情報のシリアライズ */],
}
```

**推奨**: **Option B**（既存命令の組み合わせで実現可能）

### 4-3. Map格納とcall()呼び出し

```hakorune
// Map経由の例
local handlers = new MapBox()
handlers.set("double", ref Math.double/1)

// 取得と呼び出し
local cb = handlers.get("double")
local result = cb.call([10])
```

**MIR変換**:
```rust
// handlers.get("double")
v%3 = BoxCall {
    box_val: v%1,  // handlers
    method: "get",
    args: [v%2],   // "double"
}

// cb.call([10])
v%5 = BoxCall {
    box_val: v%3,  // cb (CallableBox)
    method: "call",
    args: [v%4],   // [10]
}

// CallableBox.call() 内部でMirCallに変換
MirInstruction::Call {
    dst: Some(v%6),
    callee: Some(Callee::ModuleFunction("Math.double/1")),
    args: [v%4],
}
```

**実現可能**: ✅ YES

---

## 5. 総合評価

### 5-1. 整合性

**ChatGPT提案とMIR Calleeの整合性**: ⭐⭐⭐⭐☆ (4/5 高)

**理由**:
- ✅ Callee型は完全に同型変換可能
- ✅ 7バリアント全てをCallableBoxで表現可能
- ⚠️ **receiverの扱いに設計上の課題あり**
- ✅ 既存MIR命令で実装可能（新規命令不要）

### 5-2. 実現可能性

**総合評価**: ✅ **実現可能（ただし段階的実装が必要）**

**実装規模見積もり**: **8-12人日**

内訳:
1. **ref構文実装** (2-3人日)
   - Lexer/Parser/AST拡張
   - MIR Builder変換
2. **CallableBox実装** (2-3人日)
   - Box定義（7バリアント対応）
   - call()メソッド実装
3. **VM Backend拡張** (2-3人日)
   - `Callee::Value` 実装
   - `Callee::Constructor` 実装
   - `Callee::Closure` 実装
4. **テスト・スモーク** (2-3人日)
   - 7バリアント×3Backend = 21ケース
   - Map経由の動的呼び出しテスト

### 5-3. 段階的実装推奨

**Phase 1: 基盤実装** (4-5人日)
- CallableBox定義（Global/ModuleFunction/Externのみ）
- VM Backend: `Callee::Value` 実装
- 基本テスト

**Phase 2: Method対応** (2-3人日)
- receiver捕捉機能実装
- 部分適用（partial application）実装
- Method呼び出しテスト

**Phase 3: Constructor/Closure** (2-3人日)
- Constructor呼び出し実装
- Closure生成・呼び出し実装
- 統合テスト

**Phase 4: ref構文** (2-3人日)
- Lexer/Parser拡張
- 構文糖衣の実装
- E2Eテスト

---

## 6. 主要な課題と解決策

### 課題1: receiverの扱い

**問題**: `Callee::Method` は receiver を内部に持つが、CallableBox.call() では receiver を指定できない

**解決策**:
```hakorune
box CallableBox {
    _receiver: any  // receiverを捕捉（部分適用）

    birth_method(receiver: any, box_name: StringBox, method: StringBox) {
        me._callee_type = "Method"
        me._receiver = receiver      // ← receiverを捕捉
        me._box_name = box_name
        me._method_name = method
    }

    call(args: ArrayBox) {
        if me._callee_type == "Method" {
            // 既に捕捉された receiver を使用
            return /* MirCall with receiver=me._receiver */
        }
    }
}

// 使用例
local cb = CallableBox.from_method(myString, "StringBox", "upper")
local result = cb.call([])  // myString.upper() 相当
```

### 課題2: VM Backend未実装バリアント

**現状**: Constructor, Closure, Value が未実装

**解決策**: 優先度付けで段階実装
1. **Value** (最優先): CallableBox実装の前提
2. **Constructor**: Box生成の一般化に重要
3. **Closure**: 既存FunctionBox実装を活用

### 課題3: ref構文の非存在

**現状**: `ref` キーワードが存在しない

**解決策**:
- **短期**: CallableBox.from_*() メソッドで手動生成
- **中期**: ref構文実装（ユーザビリティ向上）

---

## 7. 推奨実装順序

### Step 1: VM Backend補完 (2-3人日) ⭐最優先
```rust
// src/backend/mir_interpreter/handlers/calls/legacy/callee_dispatcher.rs
Callee::Value(func_val_id) => {
    let func_val = self.reg_load(*func_val_id)?;
    // FunctionBox/CallableBox 実行
    self.execute_dynamic_call(func_val, args)
}

Callee::Constructor { box_type } => {
    // NewBox相当の実装
    self.create_box_instance(box_type, args)
}

Callee::Closure { params, captures, me_capture } => {
    // FunctionBox生成
    self.create_function_box(params, captures, me_capture)
}
```

### Step 2: CallableBox基本実装 (2-3人日)
```hakorune
box CallableBox {
    _callee_type: StringBox
    _function_name: StringBox
    // ... (7バリアント対応フィールド)

    birth() { /* 初期化 */ }

    // Factory methods
    static from_global(name: StringBox) -> CallableBox
    static from_module_function(name: StringBox) -> CallableBox
    static from_method(receiver: any, box_name: StringBox, method: StringBox) -> CallableBox

    // 呼び出し
    call(args: ArrayBox) -> any
}
```

### Step 3: Map統合テスト (1人日)
```hakorune
local handlers = new MapBox()
handlers.set("add", CallableBox.from_global("Math.add/2"))
handlers.set("mul", CallableBox.from_global("Math.mul/2"))

local op = handlers.get("add")
local result = op.call([1, 2])  // → 3
```

### Step 4: ref構文実装 (2-3人日) ⭐ユーザビリティ向上
```hakorune
// 構文糖衣
local cb = ref Math.add/2
// ↓ 展開
local cb = CallableBox.from_global("Math.add/2")
```

---

## 8. 一経路維持の検証

### 8-1. 現状の経路

**VM**:
```
MirInstruction::Call
  → execute_callee_call(callee, args)
    → match callee { ... }
      → handle_callee_global / handle_callee_module_function / etc.
```

**LLVM**:
```
MirCall JSON
  → lower_mir_call(mir_call)
    → match callee_type { ... }
      → lower_global_call / lower_method_call / etc.
```

### 8-2. CallableBox導入後の経路

**変更なし**: 既存の `MirInstruction::Call` + `Callee` システムをそのまま使用

**新規追加**:
```
CallableBox.call(args)
  → (内部でCalleeを生成)
  → MirInstruction::Call { callee: Callee::Value(callback_id) }
  → 既存の execute_callee_call 経路
```

**結論**: ✅ **一経路維持可能**（既存経路に乗せるだけ）

---

## 9. 結論

### 9-1. ChatGPT提案の評価

**整合性**: ⭐⭐⭐⭐☆ (4/5)
- MIR Calleeと同型変換可能
- receiver捕捉に設計課題あり（解決可能）

**実現可能性**: ⭐⭐⭐⭐⭐ (5/5)
- 既存MIR命令で実装可能
- VM Backend補完が必要（2-3人日）
- 段階的実装で低リスク

**実装規模**: 8-12人日（段階実装）

### 9-2. 推奨アクション

#### 優先度1: VM Backend補完 (2-3人日)
- `Callee::Value` 実装（CallableBox前提）
- `Callee::Constructor` 実装（NewBox統一）
- `Callee::Closure` 実装（FunctionBox統合）

#### 優先度2: CallableBox基本実装 (2-3人日)
- Global/ModuleFunction/Extern対応
- Factory methods実装
- call()メソッド実装

#### 優先度3: Method対応 (2-3人日)
- receiver捕捉機能
- 部分適用実装

#### 優先度4: ref構文 (2-3人日)
- Lexer/Parser拡張
- 構文糖衣実装

### 9-3. リスク評価

**技術リスク**: 🟢 低
- 既存システムへの追加のみ（破壊的変更なし）
- LLVM Backendは既に全実装済み
- VM Backendの未実装部分は明確

**実装リスク**: 🟡 中
- receiver捕捉の設計に注意必要
- テストケースが多い（7バリアント×3Backend）

**運用リスク**: 🟢 低
- 段階的導入可能
- 既存コードへの影響なし

---

## 10. 参考資料

### ソースコード
- `/home/tomoaki/git/hakorune-selfhost/src/mir/definitions/call_unified.rs` - Callee定義
- `/home/tomoaki/git/hakorune-selfhost/src/mir/instruction.rs` - MIR命令セット
- `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/legacy/callee_dispatcher.rs` - VM実装
- `/home/tomoaki/git/hakorune-selfhost/src/llvm_py/instructions/mir_call.py` - LLVM実装
- `/home/tomoaki/git/hakorune-selfhost/src/boxes/function_box.rs` - FunctionBox実装

### ドキュメント
- `/home/tomoaki/git/hakorune-selfhost/docs/development/current/function_values_and_captures.md` - 関数値の現状
- `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase-20-python-integration/design/enhanced-architecture-v2.md` - PyCallableBox参考

---

**最終更新**: 2025-10-10
**作成者**: Claude Code
**ステータス**: 調査完了・実装判断待ち
