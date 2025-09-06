# Nyash論文プロジェクト

このディレクトリはNyashに関する学術論文の執筆プロジェクトを管理します。

## 📁 ディレクトリ構造（ChatGPT5提案による再編成済み）

```
papers/
├── README.md                          # このファイル（全候補への索引）
├── active/                            # 現在執筆中の論文
│   ├── paper-a-mir13-ir-design/      # 論文A: MIR13命令とIR設計
│   ├── paper-b-nyash-execution-model/ # 論文B: Nyash言語と実行モデル
│   ├── paper-c-ancp-compression/     # 論文C: ANCP 90%圧縮技法（世界記録）
│   ├── paper-d-jit-to-exe/          # 論文D: JIT→EXE統合パイプライン
│   ├── three-papers-strategy.md      # 3論文戦略の統合計画
│   └── WHICH_PAPER_FIRST.md         # 論文優先順位の検討（15個候補）
├── archive/                           # 過去の検討・下書き
│   ├── initial-proposals/             # 初期提案資料
│   ├── mir15-implementation/          # 旧MIR15論文
│   ├── mir15-fullstack/              # MIR15フルスタック論文（論文Aに統合）
│   └── unified-lifecycle/             # 統一ライフサイクル論文（論文Bに統合）
└── resources/                         # 共通リソース
    ├── bibliography/                  # 参考文献
    └── templates/                     # 論文テンプレート
```

## 📊 現在の論文プロジェクト（主要2本 + 追加候補多数）

### 論文A: MIR13命令とIR設計 🎯
**主題**: 中間表現（MIR）の統合設計  
**対象読者**: コンパイラ・言語処理系の研究者、PL実装者  
**ポイント**:
- ArrayGet/Set などを BoxCall に吸収する思想
- IC, AOT, TypedArray 最適化
- 「Everything is Box」哲学が MIR にどう落ちるか

**投稿先**: arXiv → POPL/PLDI 2026  
**締切**: 2025年9月（arXiv速報）→ 2025年11月（本投稿）

### 論文B: Nyash言語と実行モデル 🚀
**主題**: Nyash言語そのものの設計と実装  
**対象読者**: 言語理論・分散システム・アプリ開発寄り  
**ポイント**:
- init/fini 対称性によるメモリ管理
- P2P Intent モデルと Box 構造
- VM → JIT → AOT の多層アーキテクチャ
- 実験例（NyashCoin、プラグインストア）

**投稿先**: OOPSLA 2026 / Onward! 2026  
**締切**: 2025年10月（OOPSLA）

## 🎯 投稿戦略

1. **Phase 1（2025年9月）**: MIR15速報論文をarXiv投稿
2. **Phase 2（2025年10月）**: LLVM完成後、統一論文をOOPSLA投稿
3. **Phase 3（2026年春）**: 設計哲学論文をOnward!投稿

## 📝 執筆ガイドライン

- 各論文は独立したディレクトリで管理
- README.md、abstract.md、main.mdは必須
- 図表は figures/ サブディレクトリに配置
- 参考文献は BibTeX 形式で管理

## 🔗 関連ドキュメント

### 📝 論文候補への索引（15個以上！）
- **[15個の論文候補一覧](active/WHICH_PAPER_FIRST.md)** - すべての候補リスト
- **[3論文戦略](active/three-papers-strategy.md)** - 段階的発表計画
- **[Paper A: MIR13](active/paper-a-mir13-ir-design/)** - 13命令IR設計
- **[Paper B: Nyash](active/paper-b-nyash-execution-model/)** - 言語実行モデル
- **[Paper C: ANCP](active/paper-c-ancp-compression/)** - 90%圧縮技法
- **[Paper D: JIT-EXE](active/paper-d-jit-to-exe/)** - 統合パイプライン

### 🎯 他の論文アイデア所在地
- **[研究フォルダ](../research/)** - Box理論JIT、1ヶ月実装記録など5個以上
- **[アイデアフォルダ](../ideas/)** - 新規提案候補
- **[AI相談記録](../../sessions/)** - WebBox革命、AI協働方法論など

### 📊 執筆支援ドキュメント
- [論文執筆戦略](active/PAPER_WRITING_STRATEGY.md)
- [論文分割戦略](active/PAPER_DIVISION_STRATEGY.md)
- [ベンチマークアプリ推奨](active/BENCHMARK_APP_RECOMMENDATIONS.md)

### 🔧 開発関連
- [開発ロードマップ](../development/roadmap/)
- [技術仕様](../reference/)
- [現在のタスク](../../CURRENT_TASK.md)
