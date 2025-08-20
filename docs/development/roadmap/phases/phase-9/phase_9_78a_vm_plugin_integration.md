# Phase 9.78a: VM Plugin System統合計画

## 🎯 目標

VMバックエンドからプラグインシステム（BID-FFI v1）を呼び出し可能にする。FileBoxプラグインをVMから実行できることを実証する。

## 📊 現状分析

### 既存のVM実装

1. **箱変換メカニズム**
   ```rust
   // VMValue ↔ NyashBox間の相互変換が既に存在
   pub fn to_nyash_box(&self) -> Box<dyn NyashBox>
   pub fn from_nyash_box(nyash_box: Box<dyn NyashBox>) -> VMValue
   ```

2. **BoxCall実装**
   - VMValue → NyashBox変換
   - メソッド呼び出し（call_box_method）
   - 結果をVMValueに戻す

3. **制限事項**
   - 基本型（Integer, String, Bool）のみサポート
   - ユーザー定義Box・プラグインBox未対応
   - ExternCallがprintlnスタブのみ

### プラグインシステムの現状

1. **PluginBoxV2**
   - 完全なNyashBoxトレイト実装
   - birth/finiライフサイクル対応
   - TLV通信プロトコル確立

2. **統一Box管理**
   - InstanceBoxでのラップ可能
   - 演算子オーバーロード対応
   - メソッドディスパッチ統一

## 🚀 実装計画

### Phase 1: VMValue拡張（優先度：最高）

**1. BoxRef型の追加**
```rust
pub enum VMValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Future(FutureBox),
    Void,
    BoxRef(Arc<dyn NyashBox>), // ← 新規追加
}
```

**2. 変換関数の拡張**
```rust
impl VMValue {
    pub fn from_nyash_box(nyash_box: Box<dyn NyashBox>) -> VMValue {
        // 基本型チェック（既存）
        if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
            return VMValue::Integer(int_box.value);
        }
        // ... 他の基本型
        
        // すべての他のBoxはBoxRefとして保持
        VMValue::BoxRef(Arc::from(nyash_box))
    }
    
    pub fn to_nyash_box(&self) -> Box<dyn NyashBox> {
        match self {
            // 既存の基本型処理
            VMValue::BoxRef(arc_box) => {
                // Arc<dyn NyashBox>をBox<dyn NyashBox>に変換
                arc_box.clone_box()
            }
        }
    }
}
```

### Phase 2: プラグインローダー統合（優先度：高）

**1. VM構造体の拡張**
```rust
pub struct VM {
    // 既存フィールド
    registers: HashMap<RegisterId, VMValue>,
    memory: HashMap<MemoryLocation, VMValue>,
    
    // 新規追加
    plugin_loader: Option<Arc<PluginLoaderV2>>,
}
```

**2. VM初期化時の統合**
```rust
impl VM {
    pub fn new_with_plugins(plugin_loader: Arc<PluginLoaderV2>) -> Self {
        VM {
            // ... 既存の初期化
            plugin_loader: Some(plugin_loader),
        }
    }
}
```

### Phase 3: ExternCall実装（優先度：高）

**1. プラグインBox作成**
```rust
MirInstruction::ExternCall { dst, iface_name, method_name, args, effects } => {
    // プラグインBox作成の場合
    if iface_name == "plugin" && method_name == "new" {
        // args[0] = Box型名（例："FileBox"）
        // args[1..] = コンストラクタ引数
        
        let box_type = self.get_value(args[0])?.to_string();
        let ctor_args: Vec<Box<dyn NyashBox>> = args[1..]
            .iter()
            .map(|id| self.get_value(*id)?.to_nyash_box())
            .collect();
        
        // プラグインローダーでBox作成
        if let Some(loader) = &self.plugin_loader {
            let plugin_box = loader.create_box(&box_type, ctor_args)?;
            let vm_value = VMValue::from_nyash_box(plugin_box);
            
            if let Some(dst_id) = dst {
                self.set_value(*dst_id, vm_value);
            }
        }
    }
    // 既存の処理...
}
```

**2. 統一メソッド呼び出し**
```rust
// BoxCallの拡張
MirInstruction::BoxCall { dst, box_val, method, args, effects } => {
    let box_vm_value = self.get_value(*box_val)?;
    
    // BoxRefの場合も透過的に処理
    match box_vm_value {
        VMValue::BoxRef(arc_box) => {
            // プラグインBoxも含めて統一的に処理
            let result = self.call_unified_method(
                arc_box.as_ref(), 
                method, 
                args
            )?;
            // ...
        }
        _ => {
            // 既存の基本型処理
        }
    }
}
```

### Phase 4: テスト実装（優先度：中）

**1. FileBoxテストケース**
```nyash
// test_vm_filebox.nyash
local file = new FileBox("test.txt")
file.write("Hello from VM!")
local content = file.read()
print(content)
file.close()
```

**2. MIR生成確認**
```
%1 = const "test.txt"
%2 = extern_call plugin.new("FileBox", %1)
%3 = const "Hello from VM!"
%4 = box_call %2.write(%3)
%5 = box_call %2.read()
print %5
%6 = box_call %2.close()
```

**3. パフォーマンス比較**
- インタープリター実行時間
- VM実行時間
- オーバーヘッド測定

## 🔧 技術的課題と解決策

### 課題1: メモリ管理

**問題**: Arc<dyn NyashBox>のライフタイム管理

**解決策**:
- BoxRefで参照カウント管理
- スコープ離脱時の自動解放
- WeakRef対応も将来的に追加

### 課題2: 型安全性

**問題**: ダウンキャストの失敗処理

**解決策**:
- MIR生成時の型チェック強化
- 実行時エラーの適切なハンドリング
- TypeCheckインストラクションの活用

### 課題3: パフォーマンス

**問題**: Box変換のオーバーヘッド

**解決策**:
- 基本型は直接VMValueで保持（既存通り）
- BoxRefは参照のみ（コピーコスト削減）
- インライン最適化の検討

## 📈 期待される成果

1. **統一アーキテクチャ**
   - すべてのBox型（ビルトイン、ユーザー定義、プラグイン）が同じ扱い
   - Everything is Box哲学の完全実現

2. **高速実行**
   - VM最適化の恩恵を受けられる
   - プラグイン呼び出しも高速化

3. **拡張性**
   - 新しいプラグインBox追加が容易
   - 将来的なJIT/AOT対応も視野に

## 🎯 成功基準

1. FileBoxプラグインがVMから呼び出し可能
2. インタープリターと同じ実行結果
3. パフォーマンス劣化が10%以内
4. 既存のテストがすべてパス

## 🔧 実装箇所の詳細分析

### 1. MIR生成部分（mir/builder.rs）

**現在の実装**：
```rust
fn build_new_expression(&mut self, class: String, arguments: Vec<ASTNode>) {
    match class.as_str() {
        "IntegerBox" | "StringBox" | "BoolBox" => {
            // 基本型は最適化（直接値を返す）
            emit(MirInstruction::Const { ... })
        }
        _ => {
            // その他はRefNew（不適切）
            emit(MirInstruction::RefNew { ... })
        }
    }
}
```

**必要な修正**：
```rust
// すべてのnew式に対してNewBox命令を生成
let arg_values = arguments.iter()
    .map(|arg| self.build_expression(arg))
    .collect::<Result<Vec<_>, _>>()?;

emit(MirInstruction::NewBox {
    dst,
    box_type: class,
    args: arg_values
})
```

### 2. VM実行部分（backend/vm.rs）

**主要な修正箇所**：
- `NewBox`処理 - BoxFactory統合、birth実行
- `BoxCall`処理 - 統一メソッドディスパッチ
- スコープ管理 - ScopeTracker実装
- VM初期化 - BoxFactory、PluginLoader注入

### 3. 共有コンポーネント

**VMでも使用する既存コンポーネント**：
- `BoxFactory` - src/box_factory.rs
- `BoxDeclaration` - src/ast.rs
- `PluginLoaderV2` - src/runtime/plugin_loader_v2.rs
- `InstanceBox` - src/instance_v2.rs

## 📅 実装スケジュール

1. **Step 1**: MIR生成修正（NewBox命令）
2. **Step 2**: VM構造体拡張（BoxFactory統合）
3. **Step 3**: NewBox実装（birth実行含む）
4. **Step 4**: BoxCall統一実装
5. **Step 5**: スコープ管理とfini実装
6. **Step 6**: テストとデバッグ

---

**作成日**: 2025-08-21  
**更新日**: 2025-08-21（実装箇所詳細追加）
**優先度**: 高（Phase 8.4 AST→MIR Loweringの次）  
**前提条件**: Phase 9.78 BoxFactory統一実装完了