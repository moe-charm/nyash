# クイックリファレンス - 開発者向け重要資料

迷子になりやすい重要な設計書・仕様書をここに集約。

## 🏗️ アーキテクチャ設計

### [**名前空間・using system**](../reference/language/using.md) ⭐重要
- ドット記法（`plugin.StringBox`）
- 修飾名・namespace解決
- Phase 15.5での実装予定

### [**MIR Callee革新**](../development/architecture/mir-callee-revolution.md)
- 関数呼び出しの型安全化
- シャドウイング問題解決
- `Callee::Global`/`Method`/`Value`/`Extern`

### [**Box Factory設計**](../reference/architecture/box-factory-design.md)
- builtin vs plugin優先順位
- Phase 15.5 Core Box統一問題

## 📋 実装ガイド

### [構文早見表](syntax-cheatsheet.md)
- 基本構文・よくある間違い
- birth構文・match式・loop構文

### [アーキテクチャマップ](architecture-map.md)
- 全体構成図
- MIR→VM/LLVM フロー
- プラグインシステム

## 🔗 関連ドキュメント

- [完全言語リファレンス](../reference/language/LANGUAGE_REFERENCE_2025.md)
- [Phase 15 ロードマップ](../development/roadmap/phases/phase-15/README.md)
- [using system詳細](../reference/language/using.md)

---

**💡 迷ったらまずここを見る！**