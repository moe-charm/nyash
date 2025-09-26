# MIR Callee型実装ロードマップ - Phase 15.4

## 概要

ChatGPT5 Pro設計案に基づく、MIR Call命令の根本的改良実装計画。3段階の段階的実装により、破壊的変更を回避しながら設計革新を実現。

## 実装優先度マトリックス

| 段階 | 実装コスト | 効果 | リスク | 期間 | 優先度 |
|------|----------|------|--------|------|--------|
| Phase 1: 最小変更 | ⭐ | ⭐⭐⭐ | ⭐ | 2-3日 | 🟢 **最高** |
| Phase 2: HIR導入 | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | 1-2週間 | 🟡 **高** |
| Phase 3: 言語仕様 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | 1ヶ月 | 🟠 **中** |

## Phase 1: 最小変更実装（即実装可能）

### 🎯 **目標**: 破壊的変更なしでCallee型基盤確立

### 📋 **実装チェックリスト**

#### Step 1.1: MIR型定義追加
- [ ] `src/mir/mod.rs`: Callee列挙型定義
- [ ] `src/mir/mod.rs`: Call構造体にcalleeフィールド追加
- [ ] 互換性テスト: 既存MIRテストの全面パス確認

#### Step 1.2: ビルダー修正
- [ ] `src/mir/builder/builder_calls.rs`: resolve_call_target()実装
- [ ] `src/mir/builder/builder_calls.rs`: build_function_call()修正
- [ ] ビルトイン関数リスト作成: is_builtin_function()
- [ ] 警告システム追加: emit_warning()

#### Step 1.3: 実行器対応
- [ ] `src/backend/vm/`: Callee対応実行器
- [ ] `src/backend/llvm/`: LLVM Callee変換
- [ ] `src/backend/pyvm/`: PyVM Callee処理
- [ ] フォールバック処理: 旧func使用時の警告

#### Step 1.4: テスト・検証
- [ ] 基本テスト: `print("hello")`→Callee::Global変換
- [ ] ボックステスト: `obj.method()`→Callee::Method変換
- [ ] 互換性テスト: 全既存テストのパス確認
- [ ] MIRダンプ確認: Callee情報の正確な出力

### 📂 **具体的ファイル変更**

#### `src/mir/mod.rs`
```rust
// 追加: Callee型定義
#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    Global(String),
    Method {
        box_name: String,
        method: String,
        receiver: Option<ValueId>,
    },
    Value(ValueId),
    Extern(String),
}

// 修正: Call構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub dst: Option<ValueId>,
    pub func: ValueId,              // 既存（廃止予定）
    pub callee: Option<Callee>,     // 新規（優先）
    pub args: Vec<ValueId>,
    pub effects: EffectMask,
}
```

#### `src/mir/builder/builder_calls.rs`
```rust
impl MirBuilder {
    // 新規: 呼び出し先解決
    fn resolve_call_target(&self, name: &str) -> Result<Callee, String> {
        // 1. ビルトイン関数チェック
        if self.is_builtin_function(name) {
            return Ok(Callee::Global(name.to_string()));
        }

        // 2. 現在のボックスメソッドチェック
        if let Some(box_name) = &self.current_static_box {
            if self.has_method(box_name, name) {
                self.emit_warning(Warning::PotentialSelfRecursion {
                    method: name.to_string()
                });
                return Ok(Callee::Method {
                    box_name: box_name.clone(),
                    method: name.to_string(),
                    receiver: None,
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

    // 新規: ビルトイン関数判定
    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(name, "print" | "error" | "panic" | "exit" | "now")
    }

    // 修正: 関数呼び出しビルド
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

        // 新型使用
        self.emit_instruction(MirInstruction::Call {
            dst: Some(dst),
            func: self.value_gen.next(), // ダミー（互換性）
            callee: Some(callee),
            args: arg_values,
            effects: EffectMask::READ,
        })?;

        Ok(dst)
    }
}
```

#### `src/backend/vm/mod.rs`
```rust
// Callee対応実行器
fn execute_call(
    vm: &mut VM,
    dst: Option<ValueId>,
    func: ValueId,
    callee: Option<&Callee>,
    args: &[ValueId],
) -> Result<(), VMError> {
    if let Some(callee) = callee {
        match callee {
            Callee::Global(name) => {
                execute_global_function(vm, name, args, dst)
            },
            Callee::Method { box_name, method, receiver } => {
                execute_method_call(vm, box_name, method, receiver, args, dst)
            },
            Callee::Value(func_val) => {
                execute_dynamic_call(vm, *func_val, args, dst)
            },
            Callee::Extern(name) => {
                execute_extern_call(vm, name, args, dst)
            },
        }
    } else {
        // フォールバック: 旧実装（警告付き）
        eprintln!("Warning: Using deprecated string-based function call");
        execute_string_based_call(vm, func, args, dst)
    }
}
```

### 🧪 **テスト戦略**

#### 基本機能テスト
```nyash
// Test 1: グローバル関数
print("Hello World")  // → Callee::Global("print")

// Test 2: ボックスメソッド
static box Test {
    method() {
        print("from method")  // → 警告 + Callee::Method
    }
}

// Test 3: 関数値
local f = print
f("dynamic")  // → Callee::Value
```

#### MIRダンプ検証
```
# 期待される出力
call global "print" ["Hello World"]
call method Test::method() ["from method"]  # 警告付き
call value %42 ["dynamic"]
```

## Phase 2: HIR導入（中期）

### 🎯 **目標**: コンパイル時名前解決の確立

### 📋 **実装計画**
- AST→HIR変換層追加
- Symbol Table構築
- BindingId→FunctionId/MethodIdマッピング
- MIRビルダの文字列依存完全排除

### 🗓️ **実装期間**: 1-2週間

## Phase 3: 言語仕様統合（長期）

### 🎯 **目標**: 明示的スコープと完全修飾名

### 📋 **実装計画**
- パーサー拡張: `::print`, `global::print`
- 完全修飾名システム
- import/moduleシステム
- 静的解析・リンタ統合

### 🗓️ **実装期間**: 1ヶ月

## Phase 15統合戦略

### セルフホスティング安定化への直接寄与

1. **using system連携**
   - `using nyashstd`→Callee::Global統合
   - built-in namespace解決の最適化

2. **PyVM最適化**
   - 型付き呼び出しによる実行高速化
   - 動的解決オーバーヘッド削減

3. **LLVM最適化**
   - 静的解決による最適化機会拡大
   - インライン化・特殊化の実現

### 80k→20k行目標への寄与

#### 削減予想（Phase 1のみ）
- 実行時解決ロジック削減: ~800行
- エラー処理の簡略化: ~400行
- デバッグコードの削減: ~300行
- **Phase 1合計**: ~1500行（目標の7.5%）

#### 削減予想（全Phase完了時）
- 名前解決の一元化: ~2000行
- 実行時解決完全排除: ~1500行
- デバッグ・エラー処理: ~1000行
- **全Phase合計**: ~4500行（目標の22.5%）

## リスク管理

### 実装リスク
- **互換性破損**: Option<Callee>による段階移行で回避
- **パフォーマンス劣化**: ベンチマークによる継続監視
- **複雑性増大**: 明確な段階分離とドキュメント化

### 検証方法
- 各段階でのスモークテスト実施
- 既存テストスイートの全面グリーン維持
- パフォーマンスベンチマークの継続実行

## 成功指標

### Phase 1成功基準
- [ ] 全既存テストパス（グリーン維持）
- [ ] シャドウイング無限再帰の完全排除
- [ ] MIRダンプにCallee情報正確表示
- [ ] 警告システムの適切な動作
- [ ] パフォーマンス劣化なし（±5%以内）

### 最終成功基準
- [ ] 実行時文字列解決の完全排除
- [ ] コンパイル時エラー検出の実現
- [ ] デバッグ体験の劇的改善
- [ ] 80k→20k行目標への明確な寄与
- [ ] Phase 15セルフホスティング安定化

---

**実装開始**: 2025-09-23
**Phase 1完了予定**: 2025-09-26
**最終完了予定**: 2025-10-23

*この計画はChatGPT5 Proとの協働により策定され、段階的実装により確実な成功を目指します。*