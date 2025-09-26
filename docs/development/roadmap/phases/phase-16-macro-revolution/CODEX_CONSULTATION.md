# Codex相談結果 - Macro System実装戦略と技術制約

**日時**: 2025-09-18
**相談内容**: Box-Based Macro Systemの技術的実現可能性と実装ロードマップ

## 🎯 技術的結論（要約）

- ✅ **Property（実行時）+ Macro（コンパイル時）の厳密分離可能**
- ✅ **既存MIR14命令で対応、新命令不要**
- ✅ **Pattern Matching基盤 → Macro実装の順序妥当**
- 📊 **工数見積もり**: 最小2-3週間、充実4-6週間

## 🔧 技術的課題への回答

### 1. Property System（実行時）+ Macro（コンパイル時）の実装分離は可能？

**Codex回答**: **可能だよ。**

#### 分離設計
- **マクロ**: AST→ASTの純関数として実行（副作用なし、決定的）
- **Property評価・状態**: 実行時のみ
- **制約**: マクロが参照できるのはAST上の属性メタデータのみ

#### アーキテクチャ利点
- **フロントエンド（マクロ/構文糖）とバックエンド（MIR/LLVM/Cranelift）が綺麗に分離**
- **既存の実行パスに影響なし**

### 2. 既存MIR命令でマクロ展開をどう表現？新命令必要？

**Codex回答**: **原則不要だよ。**

#### 設計原則
- **展開結果は通常の構文へ還元**
- **既存のloweringでMIR14生成**
- **追加要素**: デバッグ情報/起源（Span/Origin）のメタデータのみ

#### 命令セット維持の利点
- 既存バックエンド（PyVM/LLVM/Cranelift）無変更
- テスト・検証の継続性確保

### 3. Pattern Matching実装 → AST操作ツール → Macro Systemの技術的順序は妥当？

**Codex回答**: **妥当だよ。**

#### 推奨実装順序
```
AST utils（パターン/クオジクオート/リライト）
↓
マクロ（関数風、次に属性）
↓
余裕があれば言語のmatch/desugarを同じ基盤で実装
```

#### Pattern Matching優先の技術的理由
- **ASTは複雑なツリー構造** → Pattern Matchingが最適なツール
- **マクロシステム自体の実装がクリーンになる**
- **膨大なif let、switch、visitorパターンの回避**

### 4. MacroBox<InputAst, OutputAst>型安全マクロの実装コスト？

**Codex回答**: **段階導入を推奨。**

#### 実装戦略
```
Stage 1: 組み込み手続きマクロ（in-proc、型は動的検証）
↓
Stage 2: Rust内製MacroBox（型付きAPI）
↓  
Stage 3: Hygiene/解決文脈まで型モデリング拡張
```

#### コスト分析
- **Rust実装**: Traitベースで比較的低リスク
- **外部マクロ**: JSON AST + スキーマ検証で橋渡し
- **初期**: 境界での厳格スキーマ検証を重視

### 5. 工数見積もりの現実性

**Codex回答**: **現実的工数を提示**

#### 詳細見積もり
- **最小ASTパターン/クオジクオート/リライト**: 2-4日（1-2日は攻めすぎ）
- **マクロシステム最小**: 2-3週間（関数風 + 簡易衛生 + エラーハンドリング）
- **属性マクロ・派生・MacroBox（型付き）**: +2-3週間
- **合計**: 4-6週間が妥当

## 🏗️ アーキテクチャ設計（Box-Based Macroの実像）

### マクロ実行境界
```
Parser → 解決前/中のフェーズでマクロ実行

実行順序:
モジュール/マクロ定義収集 
→ マクロ解決 
→ 展開 
→ 構文糖デシュガ 
→ 型検査 
→ MIR lowering
```

### API設計（Rust側）
```rust
trait Macro { 
    fn expand(&self, ctx: &mut MacroCtx, input: AstNode) -> Result<AstNode>; 
}

// MacroCtx提供機能:
// - fresh_symbol(), resolve_path()
// - emit_error()/warn(), span合成
// - 再帰カウンタ、featureフラグ
```

### AST操作ツール
- **パターン**: 変数束縛、ワイルドカード、可変長（…）
- **準引用/脱引用**: ASTをコード片として安全に構築
- **リライト**: 訪問/置換の汎用器（Span伝播対応）

### 衛生（Hygiene）設計
```
Stage 1: gensymベースの簡易衛生（捕捉回避）
Stage 2: SyntaxContext/Scope Markによる本格衛生
```

## 📋 実装フェーズとタスク

### Phase A（基盤・1週）
- ✅ AST Pattern/Unifier（変数/ワイルドカード/variadic）
- ✅ Quasi-quote/unquote、AST Builder、Span連鎖
- ✅ Rewriter（停止条件/置換/環境）

### Phase B（最小マクロ・1-2週）
- ✅ マクロ定義/登録/解決（関数風）
- ✅ 簡易衛生（gensym）+ 再帰上限
- ✅ エラー設計（Span指向、補助メッセージ）
- ✅ 展開トレース `NYASH_MACRO_TRACE=1`

### Phase C（拡張・1-2週）
- ✅ 属性マクロ（宣言/プロパティ/関数）
- ✅ MacroBox（Rust in-proc型付きAPI）
- ✅ デシュガ（pattern matching構文等）を基盤上で実装

### Phase D（高機能・以降）
- ✅ 本格衛生（SyntaxContext）
- ✅ 外部手続きマクロ（AST JSON v0）試験的
- ✅ キャッシュ/インクリメンタル展開

## ✅ 受け入れ基準（各段階）

### Phase A完了基準
- AST Pattern/クオジクオートのユニットテスト
- Span一貫性の確保

### Phase B完了基準
- マクロ→通常構文→MIR14が既存スモークと一致
- PyVM/LLVM両方で差分なし

### Phase C完了基準
- 属性マクロでProperty宣言の糖衣実装
- MacroBoxで実例1つ動作

### Phase D完了基準
- 再帰/衛生の難ケース（名前捕捉/別スコープ）で期待通り動作

## 🎯 制約とリスク対応

### Phase-15方針との整合
- **優先度**: AST操作基盤 → 最小マクロ（in-proc）→ デシュガ → 属性マクロ
- **制約**: Rust VM/JIT拡張は最小化、フロントエンド完結
- **影響**: 既存実行経路への影響なし

### リスク対応策
- **衛生の落とし穴**: 段階導入（gensym→context）で抑制
- **エラーレポート品質**: Span合成と補助ヒント初期設計
- **外部マクロ不安定化**: Phase-15中はin-proc限定
- **無限展開**: 再帰上限と循環検出（展開履歴）

## 🚀 次アクション（Codex推奨）

### 即座に着手すべき
1. **AST utils（Pattern/Quote/Rewrite）の最小設計確定**
2. **MacroCtx/Registryの最小Trait草案作成**
3. **関数風マクロ + 簡易衛生 + トレースでスモーク1本**
4. **属性マクロでProperty糖衣（例: #[once]）実装**

### 実装方針
> この方針なら、Property実行系に手を入れず、安全にマクロを導入できるよ。必要ならMacroCtx/Registryの最小APIスケッチもすぐ出すよ。

## 📊 工数見積もり詳細

| フェーズ | 内容 | 期間 | 累積 |
|----------|------|------|------|
| Phase A | AST基盤 | 3-5日 | 1週間 |
| Phase B | 最小マクロ | 7-10日 | 2-3週間 |
| Phase C | Box化・属性 | 7-10日 | 4-6週間 |
| Phase D | 高機能化 | 1-2週間 | 6-8週間 |

### 最小ルート（急ぎ）
- **最小を急げば**: 2-3週間
- **充実まで**: 4-6週間

---

**結論**: Codexの技術分析により、**Box-Based Macro System**は既存アーキテクチャを壊すことなく、段階的に実装可能であることが確認された。

*技術的制約とリスクを明確化し、現実的な実装ロードマップを提示。*