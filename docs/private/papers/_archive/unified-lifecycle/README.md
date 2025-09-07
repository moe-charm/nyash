# Nyash: A Box-Centric Language with Unified Plugin Lifecycle

論文執筆プロジェクトのホームディレクトリです。

## 📁 構成

- `abstract.md` - アブストラクト（要約）
- `main-paper.md` - 論文本体
- `technical-details.md` - 技術的詳細
- `evaluation-plan.md` - 評価計画
- `related-work.md` - 関連研究
- `figures/` - 図表ディレクトリ

## 🎯 主要貢献

1. **統一ライフサイクル契約**: ユーザーBoxとプラグインBoxを同一契約で管理
2. **GCオン/オフの意味論等価**: 開発時と本番時で完全に同じ動作
3. **C ABI v0とプラグイン一本化**: VM/JIT/AOT/WASMを共通化
4. **小さな実装での完全系**: ~4K LoCで実用パイプライン実現

## 📊 投稿先候補

- **arXiv** (まず公開)
- **PLDI** (Programming Language Design and Implementation)
- **OOPSLA** (Object-Oriented Programming, Systems, Languages & Applications) 
- **ECOOP** (European Conference on Object-Oriented Programming)
- **ASPLOS** (Architectural Support for Programming Languages and Operating Systems)

## ✍️ 執筆状況

- [x] 基本構想
- [x] 技術的詳細の整理
- [ ] アブストラクト作成
- [ ] 本文執筆
- [ ] 図表作成
- [ ] 評価実験
- [ ] 査読対応準備