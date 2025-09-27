# MIR Callee型革新 - 関数呼び出しアーキテクチャの根本改良

## 概要

シャドウイングバグから発見された根本的問題を解決するため、ChatGPT5 Pro提案に基づくMIR Call命令の完全リアーキテクチャを実施。実行時文字列解決からコンパイル時型付き解決への移行により、パフォーマンス・安全性・保守性を根本改善。

## 問題の本質

### 現在のMIR Call命令の構造的欠陥

```rust
// ❌ 現在の問題構造
Call {
    dst: Option<ValueId>,
    func: ValueId,  // ← Const(String("print"))のみ！スコープ情報なし
    args: Vec<ValueId>,
}
```

**根本的問題**:
1. **実行時文字列解決**: 全ての関数呼び出しが実行時に文字列で解決
2. **スコープ情報欠落**: どこから呼び出すかの情報がMIRに残らない
3. **シャドウイング脆弱性**: 同名メソッドが意図せず自己再帰を引き起こす
4. **最適化阻害**: コンパイル時に呼び出し先が確定できない
5. **デバッグ困難**: MIRダンプに"誰を呼ぶか"が不明瞭

## 解決策: Callee型の導入

### 新しいMIR Call命令

```rust
// ✅ 革新後の構造
pub enum Callee {
    /// グローバル関数（nyash.builtin.print等）
    Global(String),

    /// ボックスメソッド（obj.method()）
    Method {
        box_name: String,
        method: String,
        receiver: Option<ValueId>
    },

    /// 関数値（動的呼び出し、最小限）
    Value(ValueId),

    /// 外部関数（C ABI）
    Extern(String),
}

pub struct Call {
    pub dst: Option<ValueId>,
    pub callee: Callee,  // ← 型付き呼び出し先！
    pub args: Vec<ValueId>,
}
```

## 実装戦略（3段階）

### Phase 1: 最小変更（即実装可能）

**目標**: 破壊的変更なしで基本構造導入

```rust
// 段階移行用構造
pub struct Call {
    pub dst: Option<ValueId>,
    pub func: ValueId,              // 既存（廃止予定）
    pub callee: Option<Callee>,     // 新型（優先）
    pub args: Vec<ValueId>,
}
```

**変更箇所**:
- `src/mir/mod.rs`: Callee型定義追加
- `src/mir/builder/builder_calls.rs`: build_function_call()修正
- `src/backend/*/`: Callee対応実行器追加

**優先度**:
1. グローバル関数（print, error等）のCallee::Global化
2. ボックスメソッドのCallee::Method化
3. 動的呼び出しのCallee::Value明示化

### Phase 2: 中期構造化（HIR導入）

**目標**: コンパイル時名前解決の確立

```rust
// バインダによる名前解決
pub struct ResolvedName {
    pub binding: BindingId,
    pub kind: BindingKind,
}

pub enum BindingKind {
    Local(ValueId),
    Global(FunctionId),
    Method { box_id: BoxId, method_id: MethodId },
    Extern(HostFunctionId),
}
```

**実装内容**:
- AST→HIR変換でSymbol Table構築
- 各識別子にBindingId付与
- MIRビルダが文字列を一切参照しない構造

### Phase 3: 長期完成（言語仕様統合）

**目標**: 完全修飾名と明示的スコープシステム

```nyash
// 明示的スコープ演算子
::print("global")           // グローバルスコープ
global::print("explicit")  // グローバル修飾
ConsoleStd::print("static") // 静的メソッド

// 完全修飾名
nyash.builtin.print("full") // 完全修飾名
std.console.log("module")   // モジュールシステム
```

## 技術仕様詳細

### Callee型の詳細定義

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    /// グローバル関数
    /// 例: "print", "error", "panic", "exit"
    Global(String),

    /// ボックスメソッド呼び出し
    /// 例: StringBox.upper(), obj.method()
    Method {
        box_name: String,           // "StringBox", "ConsoleStd"
        method: String,             // "upper", "print"
        receiver: Option<ValueId>,  // レシーバオブジェクト（Someの場合）
        certainty: TypeCertainty,   // 追加: Known/Union（型確度）
    },

    /// 関数値による動的呼び出し
    /// 例: let f = print; f("hello")
    Value(ValueId),

    /// 外部関数（C ABI）
    /// 例: "nyash.console.log"
    Extern(String),
}
```

補足: 型確度（TypeCertainty）

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCertainty {
    Known,  // 受信クラスが一意に既知（origin 伝播・静的文脈）
    Union,  // 分岐合流などで非一意（VM などのルータに委譲）
}
```

### MIRビルダの変更

```rust
impl MirBuilder {
    pub fn build_function_call(
        &mut self,
        name: String,
        args: Vec<ASTNode>
    ) -> Result<ValueId, String> {
        let callee = self.resolve_call_target(&name)?;

        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.build_expression(arg)?);
        }

        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Call {
            dst: Some(dst),
            callee,     // ← 新型使用
            args: arg_values,
        })?;

        Ok(dst)
    }

    fn resolve_call_target(&self, name: &str) -> Result<Callee, String> {
        // 1. グローバル関数チェック
        if self.is_builtin_function(name) {
            return Ok(Callee::Global(name.to_string()));
        }

        // 2. 現在のボックスメソッドチェック（警告付き）
        if let Some(box_name) = &self.current_static_box {
            if self.has_method(box_name, name) {
                self.emit_warning(Warning::PotentialSelfRecursion {
                    method: name.to_string()
                });
                return Ok(Callee::Method {
                    box_name: box_name.clone(),
                    method: name.to_string(),
                    receiver: None  // static method
                });
            }
        }

        // 3. ローカル変数として関数値
        if self.variable_map.contains_key(name) {
            let value_id = self.variable_map[name];
            return Ok(Callee::Value(value_id));
        }

        // 4. 解決失敗
        Err(format!("Unresolved function: {}", name))
    }
}
```

## 段階的移行戦略

### 1. 互換性保持

```rust
// 実行器での段階的処理
fn execute_call(call: &Call) -> Result<Value, Error> {
    if let Some(callee) = &call.callee {
        // 新型優先
        match callee {
            Callee::Global(name) => execute_global_function(name, &call.args),
            Callee::Method { box_name, method, receiver } => {
                execute_method_call(box_name, method, receiver, &call.args)
            },
            Callee::Value(func_val) => execute_dynamic_call(func_val, &call.args),
            Callee::Extern(name) => execute_extern_call(name, &call.args),
        }
    } else {
        // 旧型フォールバック（廃止予定）
        execute_string_based_call(&call.func, &call.args)
    }
}
```

### 2. 段階的警告システム

```rust
// コンパイル時警告
#[derive(Debug)]
pub enum Warning {
    PotentialSelfRecursion { method: String },
    DeprecatedStringCall { function: String },
    AmbiguousScope { name: String, candidates: Vec<String> },
}
```

## 予期される効果

### パフォーマンス向上
- **コンパイル時解決**: 実行時オーバーヘッド削減
- **最適化機会**: インライン化・特殊化が可能
- **キャッシュ効率**: 仮想呼び出し削減

### 安全性向上
- **シャドウイング排除**: 意図しない再帰の完全防止
- **静的検証**: コンパイル時エラー検出
- **型安全性**: 呼び出し先の型情報保持

### 保守性向上
- **明確な意図**: MIRダンプで呼び出し先が一目瞭然
- **デバッグ支援**: スタックトレースの改善
- **リファクタリング支援**: 影響範囲の正確な特定

## Phase 15との統合

### セルフホスティング安定化への寄与

1. **using system連携**: built-in namespaceとCallee::Globalの統合
2. **PyVM最適化**: 型付き呼び出しによる実行高速化
3. **LLVM最適化**: 静的解決による最適化機会拡大
4. **コード削減**: 実行時解決ロジックの簡略化

### 80k→20k行目標への寄与

- 実行時解決ロジック削減: ~2000行
- 名前解決の一元化: ~1500行
- デバッグ・エラー処理の簡略化: ~1000行
- **合計予想削減**: ~4500行（目標の22.5%）

## リスク管理

### 互換性リスク
- **対策**: Option<Callee>による段階移行
- **検証**: 既存テストの全面グリーン維持

### 実装複雑性
- **対策**: 3段階の明確な分離
- **検証**: 各段階でのスモークテスト実施

### パフォーマンスリスク
- **対策**: ベンチマークによる検証
- **フォールバック**: 旧実装の維持（警告付き）

## 結論

ChatGPT5 Proの洞察により、単純なバグ修正から根本的アーキテクチャ改良への昇華を実現。この変更により、Nyashの関数呼び出しシステムが現代的な言語実装に匹敵する堅牢性と性能を獲得し、Phase 15セルフホスティング目標の重要な基盤となる。

---

*この設計は2025-09-23にChatGPT5 Proとの協働により策定されました。*
