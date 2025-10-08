# Strategy C（段階的統合）採用決定

**決定日**: 2025-10-08
**決定者**: ユーザー判断（「全ての開発にかかわってきますにゃ」）
**分析**: ultrathink 長期コード品質分析（10年視点）

---

## 🎯 採用戦略

**Strategy C（段階的統合）**: 25-35人日（5-7週間）

```
Step 1: enum MVP実装（3-5人日）
  ├─ Option<T> 基本実装
  ├─ Result<T,E> 基本実装
  └─ 基本パターンマッチング

Step 2: Mini-VM実装 with enum MVP（10-15人日）
  ├─ 新規コードのみ Option<T>/Result<T,E> 使用
  ├─ 既存コードは最小限の修正
  └─ 技術的負債の新規追加を防ぐ

Step 3: セルフホスト達成（3-5人日）

Step 4: enum完全実装（Phase 20、10-15人日）
  ├─ @enum/@matchマクロ実装
  └─ 既存コード段階的リファクタリング
```

---

## 📊 3戦略比較（決定根拠）

### 短期視点（1ヶ月）
- **Strategy A（enum-first）**: 28-42人日、★★☆☆☆（遅い）
- **Strategy B（Mini-VM-first）**: 13-20人日、★★★★★（最速）← 当初提案
- **Strategy C（段階統合）**: 25-35人日、★★★☆☆（中間）

### 長期視点（10年）
- **Strategy A（enum-first）**: 品質★★★★★、技術的負債100 → 200
- **Strategy B（Mini-VM-first）**: 品質★☆☆☆☆、技術的負債100 → 800-1000
- **Strategy C（段階統合）**: 品質★★★★☆、技術的負債100 → 200-300 ← **採用**

### 総合評価（10年ROI）
```
Strategy B: 初期13-20人日節約、但し10年後 +50-100人日リファクタ
Strategy C: 初期+12-15人日追加、但し10年後 -50-100人日節約

→ Strategy C の方が10年累計で 60-115人日有利
```

---

## 🔍 技術的負債分析

### 現状（Phase 15.7時点）
**既存Mini-VM（2,379行、38ファイル）**:
- null チェック: 66箇所
- error コード（-1/-2/0）: 34箇所
- 品質スコア: 5/10（中〜高レベルの技術的負債）

### 10年累積モデル

#### Strategy B（Mini-VM-first）の場合
```
Phase 15.7:  100 debt points
Phase 20:    200 points（+新規nullチェック50、+新規errorコード50）
Phase 25:    400 points（複雑性の複利増加）
10年後:      800-1000 points

リファクタリングコスト: 50-100人日（遅延するほど増大）
```

#### Strategy C（段階統合）の場合
```
Phase 15.7:  100 debt points
Step 1:      +10 points（enum MVP実装）
Step 2:      +20 points（新規null/error禁止により50%削減）
Step 3:      +10 points（統合作業）
Step 4:      -60 points（段階的リファクタ）
10年後:      200-300 points

技術的負債削減: 70%（800-1000 → 200-300）
```

---

## 💡 決定の転機

### ユーザーの重要発言

> **「hakoruneセルフホスティング　コードは　綺麗にするのとても大切とおもいますにゃ　一番大本のrust vmからの立上げで　何かあったとき　ここからビルドする事も想定しますにゃ　全ての開発にかかわってきますにゃ」**

### この発言の意味

1. **Bootstrap Chainの信頼性が最優先**
   - Rust VM → Hakorune Selfhost Compiler → MIR JSON → VM実行
   - 10年以上メンテナンスする可能性

2. **技術的負債の複利的増加を回避**
   - 早期の悪習慣は後の全コードに伝播
   - "全ての開発にかかわってくる"

3. **短期的スピードより長期的品質**
   - 1ヶ月の差（13-20 vs 25-35人日）は許容範囲
   - 10年で50-100人日節約の方が重要

---

## 🚀 次のアクション

### 最優先タスク: Step 1（enum MVP実装）

**期間**: 3-5人日

**成果物**:
- `apps/lib/boxes/option.hako` - Option<T> 基本実装
- `apps/lib/boxes/result.hako` - Result<T,E> 基本実装
- `apps/tests/test_option_basic.hako` - テストスイート
- `apps/tests/test_result_basic.hako` - テストスイート
- `docs/guides/option-result-usage.md` - 使用ガイド

**成功基準**:
- [ ] Option<T> 基本操作すべて動作
- [ ] Result<T,E> 基本操作すべて動作
- [ ] スモークテスト PASS
- [ ] Mini-VMコードで使用可能な状態

---

## 📋 関連ドキュメント

- **実行計画**: [mini_vm_migration_plan.md](mini_vm_migration_plan.md)
- **進捗記録**: [mini_vm_progress.md](mini_vm_progress.md)
- **失敗記録**: [mini_vm_lessons.md](mini_vm_lessons.md)
- **VariantBox設計**: [docs/development/roadmap/phases/phase-20-variant-box/](../../roadmap/phases/phase-20-variant-box/)

---

**結論**: Strategy C（段階的統合）により、短期的コスト（+12-15人日）を払って長期的品質（10年で-50-100人日）を獲得する戦略を採用。

**Bootstrap Chainの信頼性 = プロジェクトの10年生存率**
