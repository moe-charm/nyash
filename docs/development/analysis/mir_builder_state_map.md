# MirBuilder 状態マップ（Box化候補一覧）

## 視覚的全体像

```
MirBuilder (735 lines)
├── SSA・ブロック管理 ─────────────────────────────────────
│   ├── local_ssa_map           ✅ Box化候補1 (LocalSSAStateBox)
│   ├── schedule_mat_map         ↑ 同Box内に統合
│   └── current_block            ← 基本フィールド（そのまま）
│
├── Origin・メタデータ追跡 ────────────────────────────────
│   ├── value_origin_newbox      ✅ 既にTrackerBox経由でアクセス
│   ├── value_types              ← MetadataBox候補（将来）
│   └── field_origin_class       🔥 Box化候補2 (FieldOriginRegistryBox)
│       field_origin_by_box      🔥 ↑ 同Box内に統合
│
├── Weak・Property管理 ────────────────────────────────────
│   ├── weak_fields_by_box       📦 Box化候補4 (WeakFieldRegistryBox)
│   └── property_getters_by_box  📦 ↑ 同Box内に統合
│
├── メソッドインデックス ──────────────────────────────────
│   ├── static_method_index      🔥 Box化候補3 (MethodIndexBox)
│   ├── instance_method_index    🔥 ↑ 同Box内に統合
│   ├── method_tail_index        🔥 ↑ 同Box内に統合
│   └── method_tail_index_source_len  ↑ 同Box内に統合
│
├── 制御フロー・スタック ──────────────────────────────────
│   ├── loop_header_stack        ← 軽量スタック（Box化不要）
│   ├── loop_exit_stack          ← 軽量スタック（Box化不要）
│   └── if_merge_stack           ← 軽量スタック（Box化不要）
│
├── 変数・スコープ管理 ────────────────────────────────────
│   ├── variable_map             ← 基本構造（Box化不要）
│   ├── user_defined_boxes       ← 設定類（Box化不要）
│   └── static_box_names         ← 設定類（Box化不要）
│
└── プラグイン・設定 ──────────────────────────────────────
    ├── plugin_method_sigs       ← 読み取り専用（Box化不要）
    ├── current_static_box       ← 軽量フラグ（Box化不要）
    └── その他フラグ類           ← 軽量フラグ（Box化不要）
```

---

## Box化優先順位マトリックス

| 優先度 | Box名 | 削減行数 | 独立性 | 実装難易度 | ROI | アクション |
|--------|------|---------|--------|----------|-----|-----------|
| 🔥 **1位** | FieldOriginRegistryBox | 60-80行 | ⭐⭐⭐ | 低 | 最大 | **即座に実装** |
| 🔥 **2位** | MethodIndexBox | 40-50行 | ⭐⭐⭐ | 中 | 高 | **Week 2実装** |
| ✅ **3位** | LocalSSAStateBox | 20-30行 | ⭐⭐ | 低 | 中 | **Week 3完成** |
| 📦 **4位** | WeakFieldRegistryBox | 10-15行 | ⭐⭐ | 低 | 小 | **Week 1実装** |
| 🔄 **5位** | NormalizeStateBox | 0-10行 | ⭐⭐⭐ | 低 | 極小 | **将来検討** |

---

## 使用頻度ヒートマップ

```
[高頻度] ◆◆◆◆◆◆◆◆◆◆
local_ssa_map         ◆◆◆◆◆◆◆◆◆   (9箇所) → LocalSSAStateBox
method_tail_index     ◆◆◆◆◆◆◆◆    (8箇所) → MethodIndexBox
field_origin_*        ◆◆◆◆◆◆◆     (7箇所) → FieldOriginRegistryBox 🔥

[中頻度] ◆◆◆◆◆
value_origin_newbox   ◆◆◆◆◆       (5箇所) ✅ 既にTrackerBox経由
instance_method_index ◆◆◆         (3箇所) → MethodIndexBox

[低頻度] ◆◆
weak_fields_by_box    ◆◆          (2箇所) → WeakFieldRegistryBox
property_getters      ◆           (1箇所) → WeakFieldRegistryBox
```

---

## 重複コード分布

### FieldOriginRegistryBox（最大の重複）

```
src/mir/builder/fields.rs:
  L68-76:   field_origin_class.insert(...) + field_origin_by_box.insert(...)  [15行]
  L99:      field_origin_class.get(...)                                        [5行]
  L190-194: field_origin_class.insert(...) + field_origin_by_box.insert(...)  [15行]

src/mir/builder/decls.rs:
  (間接参照のみ)

合計: 35行 × 2箇所 = 70行の重複 🔥
```

### MethodIndexBox（インデックス管理の重複）

```
src/mir/builder.rs:
  L304-322: rebuild_method_tail_index()     [19行]
  L324-332: ensure_method_tail_index()      [9行]
  L334-341: method_candidates()             [8行]
  L343-349: method_candidates_tail()        [7行]

合計: 43行の統合候補
```

### LocalSSAStateBox（既に分離済み）

```
src/mir/builder/ssa/local.rs:
  L30-67:   ensure() - 核心ロジック         [38行] ✅ 既に分離済み
  L69-147:  ヘルパー関数群                  [79行] ✅ 既に分離済み

src/mir/builder/utils.rs:
  L103-105: clear() - 重複処理              [3行] → 統合候補
```

---

## 依存関係グラフ

```
MirBuilder
    │
    ├─── LocalSSAStateBox ────────────────┐
    │       ├── ensure()                   │
    │       ├── recv()                     │
    │       └── clear()                    │
    │                                      │
    ├─── FieldOriginRegistryBox ──────────┤
    │       ├── register_value_field()     │  独立性が高い
    │       ├── register_box_field()       │  （相互依存なし）
    │       └── infer_field_origin()       │
    │                                      │
    ├─── MethodIndexBox ──────────────────┤
    │       ├── register_*()               │
    │       ├── find_candidates()          │
    │       └── rebuild_tail_index()       │
    │                                      │
    └─── WeakFieldRegistryBox ────────────┘
            ├── register_weak_fields()
            └── is_weak_field()
```

**重要**: すべての Box は相互依存なし！並行実装可能

---

## 実装タイムライン（4週間計画）

```
Week 1: 小粒Box実装（即効性重視）
┌─────────────────────────────────────┐
│ Day 1-2: FieldOriginRegistryBox     │ 🔥 最大ROI
│ Day 3:   WeakFieldRegistryBox       │ 📦 安全な練習
│ Day 4-5: テスト・統合               │
├─────────────────────────────────────┤
│ 成果: 70-95行削減 ✅                 │
└─────────────────────────────────────┘

Week 2: 中粒Box実装（独立性重視）
┌─────────────────────────────────────┐
│ Day 1-2: MethodIndexBox 基本実装    │ 🔥 独立性高い
│ Day 3:   API統合                     │
│ Day 4-5: テスト・検証               │
├─────────────────────────────────────┤
│ 成果: 40-50行削減 ✅                 │
└─────────────────────────────────────┘

Week 3: 既存Box完成（完成度重視）
┌─────────────────────────────────────┐
│ Day 1-2: LocalSSAStateBox Box化     │ ✅ 既に80%完了
│ Day 3:   API最終化                   │
│ Day 4-5: テスト・ドキュメント       │
├─────────────────────────────────────┤
│ 成果: 20-30行削減 ✅                 │
└─────────────────────────────────────┘

Week 4: 統合・最終調整
┌─────────────────────────────────────┐
│ Day 1-2: 統合テスト                 │
│ Day 3-4: パフォーマンス測定         │
│ Day 5:   ドキュメント整備           │
├─────────────────────────────────────┤
│ 成果: 全Box協調動作確認 ✅           │
└─────────────────────────────────────┘

合計削減行数: 130-185行
新規テストコード: 100-200行
```

---

## メトリクス目標

### 定量目標
- ✅ **コード削減**: 130-185行 → **目標150行以上**
- ✅ **重複削減率**: 70% 以上（70行重複 → 20行以下）
- ✅ **テストカバレッジ**: 各Box 80% 以上

### 定性目標
- ✅ **可読性**: 新規開発者が責務を理解しやすい
- ✅ **デバッグ容易性**: `dump()` で状態確認容易
- ✅ **保守性**: 変更影響範囲がBox内に閉じる

---

## 箱理論4原則チェックリスト

### ✅ 1. 箱にする
- [x] LocalSSAStateBox: SSA状態をBox内に閉じ込め
- [x] FieldOriginRegistryBox: フィールド起源をBox内に閉じ込め
- [x] MethodIndexBox: メソッドインデックスをBox内に閉じ込め
- [x] WeakFieldRegistryBox: Weak field情報をBox内に閉じ込め

### ✅ 2. 境界を作る
- [x] 各Boxが明確な責務を持つ（単一責任原則）
- [x] Box間の相互依存なし（独立性保証）
- [x] 統一API: `register_*()`, `find_*()`, `infer_*()`, `dump()`

### ✅ 3. 戻せる
- [x] 各Boxを独立してロールバック可能
- [x] 段階的実装: 1 Box ずつテスト・統合
- [x] 回帰テスト: 既存スモークテスト全PASS

### ✅ 4. 見える化
- [x] すべてのBoxに `dump()` メソッド実装
- [x] トレース機能: `NYASH_*_TRACE=1` で詳細ログ
- [x] デバッグ容易: 状態を可視化して問題特定

---

## 次のステップ

### 即座に実行（今日中）
1. ✅ **FieldOriginRegistryBox 設計レビュー**
   - 既存の `fields.rs` / `decls.rs` を詳細分析
   - API設計を最終化

2. ✅ **WeakFieldRegistryBox クイック実装**
   - 最も小粒で安全 → ウォームアップに最適
   - 1-2時間で完成可能

### Week 1 開始（明日〜）
3. ✅ **FieldOriginRegistryBox 実装開始**
   - 最大のROI（60-80行削減）
   - 3-4日で完成予定

---

## 参考リンク

- **詳細レポート**: [mir_builder_box_refactoring_task1.md](./mir_builder_box_refactoring_task1.md)
- **箱理論**: [CLAUDE.md](../../CLAUDE.md#箱理論-box-first)
- **MirBuilder本体**: [src/mir/builder.rs](../../src/mir/builder.rs)

---

**最終更新**: 2025-10-15
**作成者**: Claude (Task 1 調査)
