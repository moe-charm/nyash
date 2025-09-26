# 論文A: MIR14で作る万能実行系

> Scope (2025-09-19): 本稿の実験・評価は PyVM（意味論リファレンス）と LLVM/llvmlite（AOT/EXEハーネス）に限定する。MIR は PHI‑off（合流はエッジコピー）で、PHI 形成は LLVM 側で合成する。詳細は SCOPE.md を参照。

## 📚 概要

**タイトル**: From Interpreter to Native GUI Apps: Universal Execution with 14 Core Instructions

**主題**: 14命令のミニマルIRで実現する5つの実行形態（インタープリター/VM/JIT/AOT/GUI）

**対象読者**: システム研究者、言語実装者、実用性重視の開発者

## 🌱 MIR命令数の進化史

**初期**: 27命令前後（汎用的にあれもこれも欲しい状態）
↓
**削減会議**: Box哲学と抽象化を突き詰めて13命令まで削減
↓
**復活**: 「最低限の算術演算は直接あった方が良い」という判断で UnaryOp を追加
↓
**MIR14**: 現在のコア命令セット（Core-13 + UnaryOp = 14命令）

## 🎯 研究ポイント

### 1. 実装の完全性
- **インタープリター**: 開発・デバッグ用（500行）
- **VM**: 高速実行（1000行）  
- **JIT/AOT**: Cranelift統合でネイティブ性能
- **EXE生成**: lld内蔵で完全自立
- **Windows GUIアプリ**: EguiBoxで実用アプリ

### 2. MIR14の威力
- 14命令（Core-13 + UnaryOp）ですべての実行形態をサポート
- 27命令 → 13命令 → 14命令への実践的な進化
- BoxCallへの統一と必要最小限の算術演算

### 3. 実用性の証明
- サイコロRPG（ゲーム）
- 統計計算ツール（数値計算）
- LISPインタープリター（言語処理系）
- ファイルエクスプローラー（GUIアプリ）

## 📊 実験計画

### 実行形態の比較
- **起動時間**: Interpreter < VM < JIT < AOT < GUI
- **実行速度**: Interpreter < VM < JIT ≈ AOT
- **バイナリサイズ**: Script < VM < JIT < AOT < GUI
- **メモリ使用量**: 各形態での比較

### 実アプリケーション評価
- **サイコロRPG**: ゲームループ性能（60fps達成）
- **統計計算**: 大規模データ処理（100万件）
- **GUIレスポンス**: ユーザー操作の遅延（<16ms）
- **コンパイル時間**: ソース→EXEの所要時間

## 📁 ディレクトリ構造

```
paper-a-mir13-ir-design/
├── README.md              # このファイル
├── abstract.md           # 論文概要
├── main-paper.md         # 本文
├── chapters/             # 章別ファイル
│   ├── 01-introduction.md
│   ├── 02-mir-evolution.md
│   ├── 03-boxcall-unification.md
│   ├── 04-optimization-techniques.md
│   ├── 05-evaluation.md
│   └── 06-conclusion.md
├── figures/              # 図表
│   ├── mir-instruction-reduction.png
│   ├── performance-comparison.png
│   └── boxcall-architecture.svg
├── data/                 # 実験データ
│   ├── benchmark-results/
│   └── mir-statistics/
└── related-work.md       # 関連研究

```

## 🗓️ スケジュール

- **2025年9月前半**: 実験実施・データ収集
- **2025年9月中旬**: 執筆開始
- **2025年9月末**: arXiv投稿（速報版）
- **2025年11月**: POPL/PLDI 2026投稿

## 📝 執筆メモ

### 強調すべき貢献
1. **実装の幅広さ**: 1つのIRで5つの実行形態を実現
2. **完全な自立性**: 外部コンパイラ・リンカー不要
3. **実用アプリ動作**: GUIアプリまで実際に動く

### 新規性
- 13命令で実用GUIアプリまで動かした初の事例
- インタープリターからネイティブまでの統一パイプライン
- Cranelift + lld内蔵による完全自己完結型言語

## 🔗 関連ドキュメント

- [MIR Instruction Set](../../../../reference/mir/INSTRUCTION_SET.md)
- [Phase 11.8 MIR Cleanup](../../../../development/roadmap/phases/phase-11.8_mir_cleanup/)
- [Phase 12 TypeBox統合](../../../../development/roadmap/phases/phase-12/)
