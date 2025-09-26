# Phase 16 次期実装タスク - 実装開始への具体的ステップ

**策定日**: 2025-09-18
**ステータス**: 実装準備完了
**優先度**: 最高（全AI相談結果に基づく統合計画）

## 🎯 実装開始の準備状況

### ✅ 完了済み（計画・設計フェーズ）
- **AI三賢者相談完了**: ChatGPT・Gemini・Codex全ての技術検証済み
- **統合実装ロードマップ**: 4-6週間の詳細計画策定済み
- **Pattern Matching基盤計画**: 2週間の実装詳細完成
- **マクロ実例集**: 6つの革命的マクロタイプの具体例完成
- **技術アーキテクチャ**: HIRパッチ式エンジン設計確定

## 🚀 即座実装タスク（今週着手推奨）

### Task 1: Pattern Matching実装開始（最優先）
**期間**: 2週間  
**理由**: Gemini・Codex共に「マクロ実装の必須基盤」と明言

#### Week 1: 基本構文
```bash
# 実装ファイル
src/parser/pattern_matching.rs     # パターン構文解析
src/ast/pattern.rs                 # Pattern AST定義
src/mir/lowering/pattern.rs        # Pattern → MIR14変換
```

#### Week 2: 高度機能
```bash
# 実装ファイル
src/parser/destructuring.rs        # Box destructuring
src/type_checker/pattern.rs        # パターン型検査
src/mir/optimization/pattern.rs    # パターン最適化
```

#### 受け入れ基準
```nyash
// 動作必須テスト
local result = match value {
    0 => "zero",
    1..10 => "small", 
    UserBox(name, age) => "user: " + name,
    _ => "other"
}
```

### Task 2: AST操作基盤構築（Pattern Matching並行）
**期間**: 1週間  
**依存**: Pattern Matching Week 1完了後

#### 実装内容
```bash
# 新規ファイル
src/macro_system/ast_pattern.rs    # AST用Pattern Matching
src/macro_system/quote.rs          # 準引用/脱引用
src/macro_system/rewriter.rs       # AST書き換え器
```

#### API設計例
```rust
// マクロでのAST操作
fn expand_derive(input: &AstNode) -> Result<AstNode> {
    match input {
        BoxDef { name, fields } => {
            let equals_method = quote! {
                method equals(other: #name) -> BoolBox {
                    #(generate_field_comparisons(fields))
                }
            };
            Ok(equals_method)
        }
    }
}
```

### Task 3: @derive(Equals)最小実装（MVP）
**期間**: 1週間  
**依存**: Task 1・2完了

#### 実装目標
```nyash
// 入力
@derive(Equals)
box UserBox {
    name: StringBox
    age: IntegerBox
}

// 自動生成
method equals(other: UserBox) -> BoolBox {
    return me.name == other.name && me.age == other.age
}
```

#### 実装ファイル
```bash
src/macro_system/derive/mod.rs     # derive マクロシステム
src/macro_system/derive/equals.rs # Equals実装ジェネレーター
src/macro_system/registry.rs      # マクロ登録システム
```

## 📋 並行作業可能タスク

### A系列: コア機能実装
- [ ] Pattern Matching parser
- [ ] AST manipulation tools
- [ ] HIR patch engine
- [ ] @derive(Equals) generator

### B系列: 品質保証
- [ ] Unit tests for pattern matching
- [ ] Integration tests for macro expansion
- [ ] Performance benchmarks
- [ ] Error message quality

### C系列: 開発体験
- [ ] `nyash --expand` コマンド
- [ ] `NYASH_MACRO_TRACE=1` デバッグ
- [ ] マクロ展開可視化ツール
- [ ] 開発者ドキュメント

## 🎯 2週間後の目標状態

### 動作するコード例
```nyash
// Pattern Matching が動作
local greeting = match user {
    UserBox(name, age) => "Hello " + name + "!",
    AdminBox(name) => "Hello Admin " + name + "!",
    _ => "Hello stranger!"
}

// @derive(Equals) が動作  
@derive(Equals, ToString)
box PersonBox {
    name: StringBox
    age: IntegerBox
}

local person1 = new PersonBox("Alice", 25)
local person2 = new PersonBox("Alice", 25)
assert person1.equals(person2)  // 自動生成されたequalsメソッド
```

### 技術達成目標
- ✅ Pattern Matching基本動作（リテラル・変数・構造パターン）
- ✅ @derive(Equals)自動生成動作
- ✅ HIRパッチエンジン基盤完成
- ✅ マクロ展開デバッグツール動作

## 🔧 実装時の技術指針

### 安全な実装戦略
1. **既存MIR14命令不変**: 新命令追加なし
2. **段階的機能追加**: 最小動作から開始
3. **回帰テスト重視**: 既存機能への影響なし
4. **エラーハンドリング**: 明確なエラーメッセージ

### パフォーマンス考慮
1. **Pattern最適化**: Jump table、Decision tree
2. **マクロキャッシュ**: 展開結果のキャッシュ
3. **漸進的コンパイル**: 変更部分のみ再処理
4. **メモリ効率**: AST操作の最適化

## 🚨 実装時の注意点

### Hygiene（衛生）問題
```nyash
// 生成されるメソッド名が既存と衝突しないように
method __generated_equals_#unique_id(other: UserBox) -> BoolBox {
    // 安全な名前空間での生成
}
```

### エラー報告品質
```nyash
// 良いエラーメッセージ例
Error: @derive(Equals) cannot be applied to UserBox
  → UserBox contains field 'callback' of type FunctionBox
  → Equals comparison is not supported for FunctionBox
  Help: Consider implementing custom equals method or excluding this field
```

### デバッグ支援
```bash
# マクロ展開の可視化
nyash --expand program.nyash

# ステップバイステップ追跡
NYASH_MACRO_TRACE=1 nyash program.nyash
```

## 📊 成功指標（2週間後）

### 機能的成功
- [ ] Pattern Matching基本テスト全通過
- [ ] @derive(Equals)生成コード動作確認
- [ ] 既存テストスイート全通過（回帰なし）
- [ ] 実用サンプルアプリでの動作確認

### 技術的成功
- [ ] MIR14出力の妥当性確認
- [ ] PyVM・LLVMバックエンド両対応
- [ ] パフォーマンス基準クリア
- [ ] メモリ使用量の適正範囲

### 開発者体験
- [ ] 明確なエラーメッセージ提供
- [ ] デバッグツールの実用性確認
- [ ] ドキュメントの完成度
- [ ] サンプルコードの動作確認

---

**Phase 16実装開始！世界最強マクロ言語への第一歩。**

*「Pattern Matching → @derive(Equals) → 世界征服」- 明確な実装経路の確立完了。*