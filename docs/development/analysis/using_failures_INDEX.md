# using系11件失敗分析 - INDEX

**作成日**: 2025-10-16
**コンテキスト**: Phase 1-3統合後のスモークテスト結果分析
**調査者**: Claude (Task Agent #4)

---

## 📚 ドキュメント一覧

### 1. クイックサマリー（最初に読む）
**[using_failures_quick_summary.md](using_failures_quick_summary.md)**

- ⚡ 最も簡潔な要約
- 4つのエラーパターン概要
- 修正優先度表
- 次のアクション

**読むべき人**: 全員（必読）
**読む時間**: 2分

---

### 2. 詳細分類レポート（完全版）
**[using_failures_classification_report.md](using_failures_classification_report.md)**

- 📊 エグゼクティブサマリー
- 詳細分類表（11件すべて）
- 各パターンの根本原因分析
- legacy-boxes除外との関連性検証
- 修正優先度と推奨アクション
- 統計サマリー

**読むべき人**: 修正担当者、アーキテクト
**読む時間**: 10-15分

---

### 3. フローチャート（視覚的理解）
**[using_failures_flowchart.md](using_failures_flowchart.md)**

- 🔄 エラー分類フローチャート
- パターン別エラー連鎖図
- 修正の依存関係図
- テスト優先度マップ
- 検証チェックリスト

**読むべき人**: 全体像を把握したい人
**読む時間**: 5-10分

---

### 4. 再現手順ガイド（実装者向け）
**[using_failures_reproduction_guide.md](using_failures_reproduction_guide.md)**

- 🔬 各パターンの最小再現例
- デバッグ手順（コマンド付き）
- 修正案（コード例付き）
- 検証チェックリスト
- 回帰テストコマンド

**読むべき人**: 修正実装者
**読む時間**: 30分（実際にコマンド実行する場合）

---

## 🎯 読み方ガイド

### 🚀 急いでいる人（5分）
1. [クイックサマリー](using_failures_quick_summary.md) を読む
2. 修正優先度 P0 の項目を確認
3. 次のアクションを実行

### 📖 全体を理解したい人（20分）
1. [クイックサマリー](using_failures_quick_summary.md) を読む
2. [フローチャート](using_failures_flowchart.md) で視覚的に理解
3. [詳細レポート](using_failures_classification_report.md) で深掘り

### 🔧 修正を実装する人（1時間）
1. [クイックサマリー](using_failures_quick_summary.md) で全体把握
2. [詳細レポート](using_failures_classification_report.md) で背景理解
3. [再現手順ガイド](using_failures_reproduction_guide.md) で実装
4. [フローチャート](using_failures_flowchart.md) で検証計画確認

---

## 🔑 キーファインディング

### ✅ 重要な発見

1. **legacy-boxes除外は無関係**
   - 11件すべてが using/module resolution の問題
   - kernel-embedded boxes は正常動作
   - 疑惑は完全に晴れた ✨

2. **4つの明確なパターン**
   - A: Parser Error (低優先度ログノイズ)
   - B: Type Error (高優先度、実行不可)
   - C: Static Singleton未具現化 (高優先度)
   - D: Expected Failure誤検出 (混在)

3. **修正の優先度が明確**
   - P0: 4件（B: 3件, C: 1件） - 即座に修正
   - P1: 1件（D-1: 循環依存） - 高優先度
   - P2: 6件（A: 5件, D-2: 2件） - 低優先度

---

## 📊 統計サマリー

| 項目 | 値 |
|------|-----|
| **総失敗件数** | 11件 |
| **legacy-boxes関連** | 0件 ✅ |
| **using/module関連** | 11件 (100%) |
| **P0 (即座修正)** | 4件 |
| **P1 (高優先度)** | 1件 |
| **P2 (低優先度)** | 6件 |

---

## 🚀 次のステップ

### 1. P0修正（最優先）
- **パターンB**: workspace module resolution 修正
  - ファイル: `src/frontend/using_resolver.rs`
  - 影響: 3件のテスト
  - 詳細: [再現手順ガイド - パターンB](using_failures_reproduction_guide.md#パターンb-type-error-voidunknownbox)

- **パターンC**: static box singleton materialization
  - ファイル: `src/frontend/mir_builder.rs`
  - 影響: 1件のテスト
  - 詳細: [再現手順ガイド - パターンC](using_failures_reproduction_guide.md#パターンc-static-singleton未具現化)

### 2. 回帰テスト
- P0修正後、11件すべてを再実行
- 既存の170件 passing tests が break していないことを確認

### 3. P1修正（高優先度）
- **パターンD-1**: 循環依存検出
  - ファイル: `src/frontend/using_resolver.rs`
  - 影響: セキュリティ/安定性
  - 詳細: [再現手順ガイド - パターンD-1](using_failures_reproduction_guide.md#パターンd-1-循環依存検出失敗)

### 4. P2修正（低優先度）
- **パターンA**: TOML parse error ログ抑制
- **パターンD-2**: デバッグログ漏出防止

---

## 🔗 関連ドキュメント

### プロジェクト内
- [Phase 1-3統合計画](../roadmap/phases/phase-p1-p3-integration/)
- [using system reference](../../reference/language/using.md)
- [MIR static box handling](../../reference/mir/static-boxes.md)
- [Test debugging guide](../../guides/smoke-test-debugging.md)

### 外部参照
- workspace module resolution
- MIR Builder architecture
- VM interpreter architecture

---

## 📝 メタ情報

| 項目 | 値 |
|------|-----|
| **作成日** | 2025-10-16 |
| **調査者** | Claude (Task Agent #4) |
| **調査時間** | 約1時間 |
| **ドキュメント数** | 5件（この INDEX 含む） |
| **総文字数** | 約30,000文字 |
| **コード例** | 15個以上 |
| **フローチャート** | 6個 |

---

## 🎓 学び

### 成功要因
1. **体系的な分類**: 4つの明確なパターンに分類できた
2. **具体例の提示**: 各パターンに再現コード・デバッグ手順を付与
3. **優先度の明確化**: P0/P1/P2 の明確な区分
4. **視覚化**: フローチャートで理解を促進

### 今後の改善点
1. **自動化**: 失敗パターン分類を自動化できるか？
2. **予防**: 同様の問題を事前に検出できるか？
3. **テスト**: より細かい単体テストで早期発見できるか？

---

**最終更新**: 2025-10-16
**バージョン**: 1.0
**ステータス**: Complete ✅
