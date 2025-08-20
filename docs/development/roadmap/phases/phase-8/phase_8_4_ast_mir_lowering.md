# Phase 8.4: AST→MIR Lowering完全実装

## 🎯 Issue概要

**現在の最重要課題**: Phase 8.3のBox操作WASMが実際にテストできない

**根本原因**: AST→MIR Loweringが不完全で、基本的なオブジェクト指向機能が使用不可

**影響範囲**: 
- ユーザー定義Boxが定義・使用できない
- Phase 8.3のRefNew/RefGet/RefSet WASMが実際にテストできない  
- Everything is Box哲学の基盤部分が欠如

## 🚨 現在の具体的問題

### 1. ユーザー定義Box定義不可
```nyash
box DataBox {
    init { value }
}
```
**エラー**: `BoxDeclaration support is currently limited to static box Main`

### 2. オブジェクト生成不可
```nyash
local obj = new DataBox(42)
```
**エラー**: `Unsupported AST node type: New`

### 3. フィールドアクセス不可
```nyash
obj.value
me.field = 10
```
**エラー**: `Unsupported AST node type: Me`

### 4. デリゲーション構文不完全
```nyash
from Parent.method()
override method() { ... }
```
**エラー**: 未対応

## 📋 実装が必要な機能

### Priority 1: 基本オブジェクト操作
- [ ] **BoxDeclaration**: ユーザー定義Box定義
- [ ] **New expression**: `new DataBox(args)` オブジェクト生成
- [ ] **Field access**: `obj.field` フィールド読み取り  
- [ ] **Field assignment**: `obj.field = value` フィールド書き込み
- [ ] **Me expression**: `me.field` 自己参照

### Priority 2: デリゲーション・継承
- [ ] **From expression**: `from Parent.method()` デリゲーション呼び出し
- [ ] **Override declaration**: `override method() { ... }` メソッドオーバーライド
- [ ] **Method calls**: `obj.method(args)` メソッド呼び出し

### Priority 3: 高度な機能
- [ ] **Constructor calls**: `pack()`, `init()` コンストラクタ
- [ ] **Static methods**: `Class.method()` 静的メソッド呼び出し

## 🔧 実装場所・方法

### メインファイル: `src/mir/builder.rs`

#### 1. `build_expression()` メソッド拡張 (行103-)
**現在の対応**: Literal, BinaryOp, UnaryOp, AwaitExpression のみ

**追加が必要**:
```rust
// Line 215付近の _ => Err(...) の前に追加
ASTNode::New { class, arguments, .. } => {
    self.build_new_expression(class, arguments)
},

ASTNode::Me { span } => {
    // 現在のインスタンスへの参照を返す
    self.build_me_expression()
},

ASTNode::FieldAccess { object, field, .. } => {
    self.build_field_access(*object, field)
},

ASTNode::MethodCall { object, method, arguments, .. } => {
    self.build_method_call(*object, method, arguments)
},

ASTNode::From { parent, method, arguments, .. } => {
    self.build_from_expression(parent, method, arguments)
},
```

#### 2. `build_statement()` メソッド拡張
**BoxDeclaration制限解除**:
```rust
// Line 190付近の条件を拡張
ASTNode::BoxDeclaration { name, methods, is_static, fields, .. } => {
    if *is_static && name == "Main" {
        // 既存のstatic box Main処理
    } else {
        // 新規：ユーザー定義Box処理
        self.build_box_declaration(name.clone(), methods.clone(), fields.clone())
    }
}
```

#### 3. 新規メソッド実装が必要

```rust
impl MirBuilder {
    fn build_new_expression(&mut self, class: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // RefNew MIR命令生成
        // Phase 8.3のWASM Box操作と連携
    }
    
    fn build_field_access(&mut self, object: ASTNode, field: String) -> Result<ValueId, String> {
        // RefGet MIR命令生成
    }
    
    fn build_field_assignment(&mut self, object: ASTNode, field: String, value: ASTNode) -> Result<ValueId, String> {
        // RefSet MIR命令生成
    }
    
    fn build_me_expression(&mut self) -> Result<ValueId, String> {
        // 現在のインスタンスへの参照
    }
    
    fn build_box_declaration(&mut self, name: String, methods: Vec<ASTNode>, fields: Vec<String>) -> Result<(), String> {
        // ユーザー定義Box登録
    }
}
```

## 🧪 テストケース（Copilot実装必須）

### Test 1: 基本Box定義・生成
**ファイル**: `test_user_defined_box.nyash`
```nyash
box DataBox {
    init { value }
    
    pack(v) {
        me.value = v
    }
}

static box Main {
    main() {
        local obj = new DataBox(42)
        return obj.value
    }
}
```

**期待MIR出力例**:
```mir
define void @main() {
bb0:
    0: safepoint
    1: %0 = const 42
    2: %1 = ref_new "DataBox", %0
    3: %2 = ref_get %1, "value" 
    4: ret %2
}
```

**実行期待結果**: `42`

### Test 2: フィールドアクセス・代入
**ファイル**: `test_field_operations.nyash`
```nyash
box Counter {
    init { count }
    
    pack() {
        me.count = 0
    }
    
    increment() {
        me.count = me.count + 1
        return me.count
    }
}

static box Main {
    main() {
        local c = new Counter()
        return c.increment()
    }
}
```

**期待結果**: `1`

### Test 3: デリゲーション基本
**ファイル**: `test_delegation_basic.nyash`  
```nyash
box Parent {
    init { name }
    
    pack(n) {
        me.name = n
    }
    
    greet() {
        return "Hello " + me.name
    }
}

box Child from Parent {
    init { age }
    
    pack(n, a) {
        from Parent.pack(n)
        me.age = a
    }
    
    override greet() {
        local base = from Parent.greet()
        return base + " (age " + me.age + ")"
    }
}

static box Main {
    main() {
        local c = new Child("Alice", 25)
        return c.greet()
    }
}
```

**期待結果**: `"Hello Alice (age 25)"`

### Test 4: WASM Box操作統合テスト
**ファイル**: `test_wasm_box_integration.nyash`
```nyash
box SimpleData {
    init { x, y }
    
    pack(a, b) {
        me.x = a
        me.y = b
    }
    
    sum() {
        return me.x + me.y
    }
}

static box Main {
    main() {
        local data = new SimpleData(10, 20)
        return data.sum()
    }
}
```

**テスト方法**:
```bash
# MIR生成テスト
./target/release/nyash --dump-mir test_wasm_box_integration.nyash

# WASM生成テスト  
./target/release/nyash --compile-wasm test_wasm_box_integration.nyash

# WASM実行テスト（wasmtime）
./target/release/nyash --compile-wasm test_wasm_box_integration.nyash > test.wat
sed -n '4,$p' test.wat > clean_test.wat
$HOME/.wasmtime/bin/wasmtime run clean_test.wat --invoke main
```

**期待結果**: 全プロセスでエラーなし、最終結果 `30`

## ✅ 成功基準

### 必須基準
- [ ] 上記4つのテストケースがすべて成功
- [ ] `cargo build --release` でエラーなし
- [ ] 既存のstatic box Main機能が破損していない
- [ ] Phase 8.3のWASM Box操作が実際に動作確認

### 理想基準  
- [ ] MIR→WASM→wasmtime実行の完全パイプライン動作
- [ ] ベンチマーク性能が劣化していない
- [ ] 複雑なデリゲーション・継承チェーンが動作

## 🤖 Copilot向け実装ガイド

### 実装順序推奨
1. **Phase 1**: `build_new_expression()` - オブジェクト生成
2. **Phase 2**: `build_field_access()` - フィールド読み取り
3. **Phase 3**: Field assignment - フィールド書き込み  
4. **Phase 4**: `build_me_expression()` - 自己参照
5. **Phase 5**: `build_box_declaration()` - Box定義
6. **Phase 6**: デリゲーション構文

### 既存コードとの統合注意点
- **MIR命令**: 既存のRefNew/RefGet/RefSet MIR命令を活用
- **型システム**: 既存のValueId/BasicBlockId体系を維持
- **エラーハンドリング**: 既存のResult<ValueId, String>パターンを踏襲

### デバッグ支援
```bash
# MIR生成確認
./target/release/nyash --dump-mir --mir-verbose test_file.nyash

# パーサー確認
./target/release/nyash --debug-fuel unlimited test_file.nyash
```

## 📊 期待される効果

### 技術的効果
- Phase 8.3のBox操作WASMが実際に使用可能
- Everything is Box哲学の実用レベル実現
- 真のオブジェクト指向プログラミング対応

### 開発効率向上
- Nyashプログラムの実用性大幅向上
- 実際のアプリケーション開発が可能
- ベンチマーク・テストの精度向上

## 🔗 関連リンク

- **Phase 8.3実装**: RefNew/RefGet/RefSet WASM対応
- **MIR設計**: `docs/説明書/reference/mir-reference.md`
- **AST定義**: `src/ast.rs`
- **既存MIR実装**: `src/mir/instruction.rs`

---

**優先度**: Critical
**担当**: Copilot + Claude協調実装
**最終目標**: test_wasm_box_integration.nyash が完全動作