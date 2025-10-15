# 論文レビュー用プロンプト

## Gemini用プロンプト（MIR13論文）

```bash
gemini -p "Nyashプロジェクトの論文をレビューしてください。

MIR13論文の日本語版：
docs/papers/active/paper-a-mir13-ir-design/main-paper-jp.md

以下の観点で深く分析してください：
1. 技術的新規性は十分か
2. 13命令という主張の妥当性
3. BoxCall統一の革新性は伝わるか
4. 3バックエンドで十分な証明になるか
5. 査読で突かれそうな弱点
6. 改善すべき点

特に『57命令から削減』の話を完全にカットして『最初から13命令』として見せる戦略は有効か、深く考察してください。"
```

## Gemini用プロンプト（Nyash言語論文）

```bash
gemini -p "Nyashプロジェクトの論文をレビューしてください。

Nyash言語論文の日本語版：
docs/papers/active/paper-b-nyash-execution-model/main-paper-jp.md

以下の観点で深く分析してください：
1. birth/fini対称性の新規性と実用性
2. Everything is Boxの言語設計としての評価
3. GC切替を将来構想に留めた判断の妥当性
4. 実績不足を正直に書いた戦略の是非
5. 査読で指摘されそうな問題点
6. より説得力を高める方法

『初期評価』『実現可能性の実証』という位置付けは適切か、深く考察してください。"
```

## Codex exec用タスク（統合的レビュー）

```bash
codex exec "Nyashプロジェクトの2本の論文を統合的にレビュー

対象：
1. MIR13論文: docs/papers/active/paper-a-mir13-ir-design/main-paper-jp.md
2. Nyash言語論文: docs/papers/active/paper-b-nyash-execution-model/main-paper-jp.md

タスク：
1. 2論文の相互補完性を評価
2. 同時投稿戦略の妥当性を検証
3. それぞれの論文が独立して成立するか確認
4. 国際会議（PLDI、OOPSLA等）での受理可能性を予測
5. より戦略的な投稿先の提案

特に以下を重点的に：
- AI校正を明記した透明性の効果
- 日本語から英訳する戦略の是非
- 15個以上ある論文候補から、なぜこの2本を選んだかの説得力

実用的な改善提案を含む詳細なレポートを作成してください。"
```

## 追加の深堀り質問

### Gemini向け追加質問

```bash
gemini -p "MIR13の『12命令も可能だが可読性のため13命令』という設計判断について、これは弱点になるか強みになるか、コンパイラ研究コミュニティの視点で深く分析してください。"
```

### Codex向け追加タスク

```bash
codex exec "birth/finiモデルの実装パターンとベストプラクティスを、Rustの所有権、SwiftのARC、C++のRAIIと比較しながら体系的に整理し、論文の補強材料を作成してください。"
```