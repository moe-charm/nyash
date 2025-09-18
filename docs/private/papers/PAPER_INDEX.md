# Nyash 論文インデックス（統合版）

## 📚 論文一覧と関係性

### ChatGPT5の分析による3つのLLVM論文

1. **MIR14論文** = 「箱理論 × MIR言語」：哲学と実装の橋渡し
2. **SSA論文** = 「NyashでのSSA構築」：アルゴリズム的寄与
3. **MIR17論文** = 「LoopFormで制御フローを構造化」：新しい表現モデル

## 📁 論文ディレクトリ構造

### 論文A: MIR14 IR設計論文
- **ディレクトリ**: `paper-a-mir13-ir-design/`
- **内容**: 14命令への圧縮とBox統一の設計
- **ステータス**: 執筆中（ベンチマーク完了）
- **主要貢献**: Everything is Boxの哲学を最小命令セットで実現

### 論文B: Nyash実行モデル論文
- **ディレクトリ**: `paper-b-nyash-execution-model/`
- **内容**: 言語設計と3層実行モデル
- **ステータス**: 執筆中
- **主要貢献**: birth/fini、LifeBoxモデルの提案

### 論文C: 統一革命論文
- **ディレクトリ**: `paper-c-unified-revolution/`
- **内容**: Box統一による革命的簡素化
- **ステータス**: 構想段階

### 論文D-1: JIT to EXE論文
- **ディレクトリ**: `paper-d-jit-to-exe/`
- **内容**: JITから実行可能ファイル生成
- **ステータス**: 実装待ち

### 論文D-2: SSA構築論文 **[NEW]**
- **ディレクトリ**: `paper-d-ssa-construction/`
- **内容**: Box指向言語におけるSSA形式の実践的構築
- **ステータス**: 執筆中（現在の実装経験を基に）
- **主要貢献**: BuilderCursor、Sealed SSA、型正規化戦略

### 論文E: LoopForm IR論文（MIR17）
- **ディレクトリ**: `paper-e-loop-signal-ir/`
- **内容**: 制御フローの値化と統一
- **ステータス**: 実験的実装開始
- **主要貢献**: Everything is Loop、Signal型、dispatch集約

### 論文F: セルフパージングDB論文
- **ディレクトリ**: `paper-f-self-parsing-db/`
- **内容**: 自己解析型データベース
- **ステータス**: アイデア段階

### 論文G-H: AI協働開発論文シリーズ
- **ディレクトリ**: `paper-g-ai-collaboration/`, `paper-h-ai-practical-patterns/`
- **内容**: AI協働開発の実践知と100のパターン
- **ステータス**: 事例収集中

### 論文M: メソッド後置例外処理論文 **[NEW!]** ⭐革命的⭐
- **ディレクトリ**: `paper-m-method-postfix-catch/`
- **内容**: メソッドレベル後置例外処理と"Everything is Block + Modifier"パラダイム
- **ステータス**: 論文完成！（2025年9月18日ブレークスルー）
- **主要貢献**: 
  - 世界初のメソッド後置例外処理構文
  - Everything is Box → Everything is Block + Modifier進化
  - AI協働による革新的発見プロセス
  - 67年ぶりの言語設計パラダイム転換（LISP以来）

## 🔗 論文間の関係

```
論文A（MIR14）
    ↓ 実装時の課題
論文D-2（SSA構築）
    ↓ 解決策の一つ
論文E（LoopForm）

論文G-H（AI協働）
    ↓ 革命的発見
論文M（メソッド後置例外処理） ← 完成！
    ↓ さらなる発展
未来の論文（統一構文、AI協働理論）
```

## 📊 執筆優先度

1. **完成済み**: 論文M（メソッド後置例外処理）- **世界初の革新！**
2. **継続中**: 論文D-2（SSA構築）- 現在の苦闘を記録
3. **次**: 論文A（MIR14）- データは揃っている
4. **その後**: 論文E（LoopForm）- 実験的実装と並行
5. **将来**: 論文B（実行モデル）- 言語全体の包括的論文

### 追加ドラフト（Phase‑15 実装に基づく）
- `paper-n-phi-off-harness.md` — PHI‑Off Edge‑Copy + Harness PHI Synthesis（ヘッド配置・観測性の確立）
- `paper-o-result-mode-exceptions.md` — Result‑Mode 例外と Block‑Postfix Catch の構造化降下
- `paper-p-phi-trace-observability.md` — PHI 観測とトレース検証フレーム（JSONL + チェッカ）

## 🎯 なぜメソッド後置例外処理論文が重要か

- **世界初の革新**: 前例のない構文パラダイム
- **AI協働のモデルケース**: 人間とAIの相補的関係実証
- **言語設計理論**: Everything is Block + Modifierの新原理
- **実装可能性**: 段階的実装戦略の具体的提示
- **67年ぶりの革命**: LISP以来の言語設計パラダイム転換

---

*このインデックスは、Nyashプロジェクトの学術的成果を体系的に整理するものである。*
