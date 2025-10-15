# 分析レポート一覧

このディレクトリには、Hakoruneプロジェクトの各種分析レポートが格納されています。

---

## 🚀 セルフホストコンパイラー 横断的最適化分析（2025-10-12）⭐NEW

**概要**: セルフホストコンパイラー全体 (71ファイル、5,733行) の横断的重複コード分析と箱化・最適化機会の特定

### ドキュメント（推奨読み順）

1. **[最適化ロードマップ](./selfhost-compiler-optimization-roadmap.md)** ⭐開始点
   - 1分でわかる核心
   - 即座の推奨アクション (クイックウィン)
   - 優先度付き施策
   - チェックリスト
   - 所要時間: 5分

2. **[重複コードヒートマップ](./selfhost-compiler-duplication-heatmap.md)**
   - 重複度ヒートマップ (視覚的)
   - ファイルサイズ分布
   - 削減ポテンシャルマップ
   - Everything is Box準拠度の内訳
   - 所要時間: 15分

3. **[横断的重複コード分析](./selfhost-compiler-cross-cutting-analysis.md)**
   - 完全な統計データ
   - 重複パターン詳細分析 (6カテゴリ)
   - 箱化機会の定量分析
   - 技術的深掘り
   - 学びと教訓
   - 所要時間: 30分

### 主要発見事項

**重複コード量**: 325-540行 (5.7-9.4%) 削減可能

**Everything is Box準拠度**: 65% → 85-90% (目標)

**優先実装候補**（ROI最大）:

| 優先度 | 施策 | 削減行数 | 工数 | ROI |
|--------|------|---------|------|-----|
| **P0 超優先** | StringUtilsBox統合 | 220-370行 | 20-30h | ★★★★★ |
| **P1 優先** | JsonUtilsBox抽出 | 50-80行 | 10-15h | ★★★★☆ |
| **P2 中** | MapHelpersBox拡張 | 50-80行 | 10-15h | ★★★☆☆ |
| **P3 低** | DebugBox改善 | 5-10行 | 5-10h | ★☆☆☆☆ |

**合計**: 325-540行削減、45-70時間

### クイックウィン（即座実行可能）

| ファイル | 削減行数 | 工数 |
|---------|---------|------|
| ParserStringUtilsBox | 63行 (76%!) | 2-3h |
| MirEmitterBox | 30行 | 3-4h |
| regex_flow.hako | 15-20行 | 2-3h |
| builder/ssa/local.hako | 10-15行 | 2-3h |
| builder/ssa/cond_inserter.hako | 10-15行 | 2-3h |
| **合計** | **128-143行** | **11-16h** |

### 推奨アクション

1. **StringUtilsBox統合**（最優先、P0）
   - 30+ファイルの文字列操作を統一
   - 220-370行削減
   - クイックウィンから段階的実施

2. **JsonUtilsBox抽出**（優先、P1）
   - JsonProgramBox (531行) を責務分離
   - 50-80行削減

3. **MapHelpersBox拡張**（中優先、P2）
   - 型安全アクセサの標準化
   - 50-80行削減

---

## 📊 MIR Builder系 箱化・最適化分析（2025-10-12）

**概要**: セルフホストコンパイラのMIR Builder関連コードの箱化・最適化機会を分析

### ドキュメント

1. **[サマリー](./mir-builder-boxification-summary.md)** ⭐開始点
   - 一言まとめ、優先実装候補、期待効果
   - 所要時間: 5分

2. **[詳細分析](./mir-builder-boxification-analysis.md)**
   - 重複コード分析、状態管理分析、パフォーマンス分析
   - Box化候補詳細、実装ロードマップ
   - 所要時間: 30分

3. **[アーキテクチャ図](./mir-builder-architecture-diagram.md)**
   - 現状アーキテクチャ vs 推奨アーキテクチャ
   - データフロー図、重複コード削減マップ
   - 所要時間: 15分

### 主要発見事項

**重複コード量**: 200-280行（3.5-4.9%）削減可能

**優先実装候補**（ROI最大）:

| 候補 | 期待効果 | 工数 |
|------|---------|------|
| JsonStringParserBox | 80-100行削減 | 8-12時間 |
| MirBuilderContext | 40-60行削減 | 6-8時間 |

**合計**: 120-160行削減、14-20時間

### 推奨アクション

1. **JsonStringParserBox実装**（最優先）
   - JSON文字列パース処理を統一
   - 3ファイルの重複削減

2. **MirBuilderContext統一**（2番目）
   - Builder状態管理を統一
   - 2つのBuilder実装の一貫性向上

---

## 📝 分析レポート作成ガイドライン

### レポート構成

1. **サマリー版** (`*-summary.md`)
   - 1ページ以内
   - 一言まとめ、優先実装候補、期待効果

2. **詳細版** (`*-analysis.md`)
   - 10-30ページ
   - 背景、問題分析、提案詳細、実装ロードマップ

3. **図解版** (`*-diagram.md`)（Optional）
   - アーキテクチャ図、データフロー図
   - 視覚的理解を助ける

### 命名規則

```
{対象}-{分析種類}-{形式}.md

例:
- mir-builder-boxification-summary.md
- plugin-system-performance-analysis.md
- memory-management-security-diagram.md
```

### テンプレート

```markdown
# {タイトル}

**分析日**: YYYY-MM-DD
**対象範囲**: {ファイル/モジュール}
**分析者**: {名前}

## 📊 一言まとめ

{1-2文で要約}

## 🎯 優先実装候補

### 1. {候補名}（{工数見積もり}）

**効果**: {期待効果}
**理由**: {なぜ優先するか}

## 📈 期待効果

{表形式でサマリー}

## 🔍 詳細分析

{セクション分け}

## 💡 推奨アクション

{具体的なアクション}
```

---

## 📚 関連ドキュメント

- **開発ガイド**: [docs/guides/development-practices.md](../../guides/development-practices.md)
- **Box理論**: [docs/guides/box-theory-guide.md](../../guides/box-theory-guide.md)
- **マスタープラン**: [docs/development/roadmap/phases/00_MASTER_ROADMAP.md](../roadmap/phases/00_MASTER_ROADMAP.md)

---

**最終更新**: 2025-10-12
