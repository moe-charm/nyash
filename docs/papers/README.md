# Nyash論文プロジェクト

このディレクトリはNyashに関する学術論文の執筆プロジェクトを管理します。

## 📁 ディレクトリ構造

```
papers/
├── README.md                      # このファイル
├── active/                        # 現在執筆中の論文
│   ├── mir15-fullstack/          # MIR15フルスタック論文（二本柱戦略）★NEW
│   └── unified-lifecycle/         # 統一ライフサイクル論文（LLVM待ち）
├── archive/                       # 過去の検討・下書き
│   ├── initial-proposals/         # 初期提案資料
│   └── mir15-implementation/      # 旧MIR15論文（統合済み）
└── resources/                     # 共通リソース
    ├── bibliography/              # 参考文献
    └── templates/                 # 論文テンプレート
```

## 📊 現在の論文プロジェクト

### 1. MIR15フルスタック論文（二本柱戦略）🎯
**状態**: 執筆開始  
**投稿先**: arXiv → POPL/PLDI/ICFP 2026  
**締切**: 2025年9月（arXiv速報）→ 2025年11月（本投稿）  
**内容**: 
- **実証**: MIR15でUbuntu/Windows GUI動作
- **理論**: Everything is Box - The Atomic Theory
- 30日実装、4000行、性能評価

### 2. 統一ライフサイクル論文（本格版）
**状態**: LLVM実装待ち  
**投稿先**: OOPSLA 2026 / PLDI 2026  
**締切**: 2025年10月（OOPSLA）  
**内容**: 全バックエンド等価性、GCオン/オフ、プラグイン統一

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