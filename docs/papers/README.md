# Nyash論文プロジェクト

このディレクトリはNyashに関する学術論文の執筆プロジェクトを管理します。

## 📁 ディレクトリ構造（ChatGPT5提案による再編成済み）

```
papers/
├── README.md                          # このファイル
├── active/                            # 現在執筆中の論文
│   ├── paper-a-mir13-ir-design/      # 論文A: MIR13命令とIR設計
│   └── paper-b-nyash-execution-model/ # 論文B: Nyash言語と実行モデル
├── archive/                           # 過去の検討・下書き
│   ├── initial-proposals/             # 初期提案資料
│   ├── mir15-implementation/          # 旧MIR15論文
│   ├── mir15-fullstack/              # MIR15フルスタック論文（論文Aに統合）
│   └── unified-lifecycle/             # 統一ライフサイクル論文（論文Bに統合）
└── resources/                         # 共通リソース
    ├── bibliography/                  # 参考文献
    └── templates/                     # 論文テンプレート
```

## 📊 現在の論文プロジェクト（2本立て戦略）

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

- [開発ロードマップ](../development/roadmap/)
- [技術仕様](../reference/)
- [現在のタスク](../development/current/CURRENT_TASK.md)