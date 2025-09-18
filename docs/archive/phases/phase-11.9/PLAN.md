# Phase 11.9: 統一文法アーキテクチャ — 実装予定（MVP〜段階移行）

## 目的
- Tokenizer/Parser/Interpreter/MIR/VM/JIT の解釈差異を解消し、単一の「文法・意味・実行」定義から各層が参照する構造へ移行する。
- 変更や拡張（予約語/演算子/構文）のコストと不整合リスクを減らす。

## マイルストーン（MVP→段階導入）

### M1: 予約語レジストリの導入（最小）
- 追加: `src/grammar/engine.rs`（`UnifiedGrammarEngine`、`KeywordRegistry` の骨格）
- 追加: `grammar/unified-grammar.toml`（初期エントリ: `me`, `from`, `loop`, `+`）
- 追加: `build.rs` で TOML → `src/grammar/generated.rs` をコード生成（ランタイム I/O 回避）
- Tokenizer 統合（非侵襲）: 従来テーブルの後段に `engine.is_keyword()` を差し込み、`NYASH_GRAMMAR_DIFF=1` で差分ログ
- 成功条件: 既存テストを落とさず、差分ログが 0 or 想定内のみに収束

### M2: 演算子セマンティクスの統一（加算など最小）
- `ExecutionSemantics` に `operators.add` を定義（型規則/コアーション/エラー方針）
- Interpreter/VM/JIT で `execute_semantic("add", …)` による共通実装窓口を追加（従来実装はフォールバック）
- 既存 `hostcall_registry`/JIT ポリシーと接合するインターフェースを用意（型分類/シンボルの参照点を一本化）
- 成功条件: 文字列結合/整数加算/浮動小数加算の3系統で VM/JIT/Interpreter の一致を維持

### M3: 構文規則エンジンの段階導入
- `SyntaxRuleEngine` 追加、`statement`/`expr` の骨格ルールを TOML 側へ切り出し
- Parser 統合（段階的）: 既存パーサ優先＋新ルールでの検証を併走、差分ログで移行安全性を担保
- 成功条件: 代表サンプルで新旧の AST→MIR が一致（スナップショット）

### M4: 並行実行/差分検出・テスト整備
- 並行期間は新旧両系の結果を比較し、スナップショットとファズで回帰防止
- 収束後、旧ルートを段階的に縮退

## 実装順（詳細 TODO）
1) `build.rs` と `src/grammar/mod.rs` の雛形追加（`generated.rs` を `include!`）
2) `KeywordRegistry` の生成コードを実装、Tokenizer に差し込み（環境変数で切り替え）
3) `operators.add` の型規則を TOML 化し、`ExecutionSemantics` で解決
4) Interpreter/VM/JIT へ共通窓口の薄い統合（実行は従来実装と比較可能に）
5) 構文ルール最小セット（statement/expr）を TOML へ移管し、解析の差分をログ化
6) スナップショット/ファズの整備と収束確認

## リスクと対策
- 競合/拡張: プラグイン由来の拡張を名前空間＋優先度でマージ、競合は検知してビルド失敗で気付かせる
- 実行コスト: 生成コード方式でランタイム I/O を避け、起動時間・ホットパスへの影響をゼロに近づける
- 文脈依存: `contextual` のキー粒度を設計（node_kind/context など）し、曖昧解釈を防ぐ

## 成功基準（Exit Criteria）
- 予約語解決の統一（Tokenizer での差分 0）
- 加算に関する VM/JIT/Interpreter のセマンティクス一致（型差分含む）
- 構文最小セットで新旧の AST→MIR が一致（代表ケース）

