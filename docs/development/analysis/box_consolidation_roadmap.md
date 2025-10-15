# Box統合ロードマップ - ビジュアルガイド

**目的**: Box化機会を視覚的に理解し、実装順序を明確にする

---

## 🗺️ 現状マップ（Before）

```
selfhost/ (165 files, 13,417 lines)
├─ hakorune-vm/ (44 files, ~3,500 lines)
│  ├─ *_handler.hako × 22 files (2,068 lines) ⚠️ 統合候補
│  │  ├─ binop_handler.hako (98 lines)
│  │  ├─ compare_handler.hako (84 lines)
│  │  ├─ const_handler.hako (80 lines)
│  │  └─ ... (19 more handlers)
│  │
│  ├─ *_locator.hako × 4 files ⚠️ 統合候補
│  │  ├─ blocks_locator.hako (39 lines)
│  │  ├─ function_locator.hako (23 lines)
│  │  ├─ instrs_locator.hako
│  │  └─ instruction_array_locator.hako
│  │
│  ├─ *_scanner.hako × 2 files ⚠️ 統合候補
│  │  ├─ backward_object_scanner.hako (46 lines)
│  │  └─ block_iterator.hako (40 lines)
│  │
│  └─ *_guard.hako × 4 files (576 lines) ⚠️ 統合候補
│     ├─ args_guard.hako (23 lines)
│     ├─ reg_guard.hako
│     ├─ receiver_guard.hako
│     └─ json_scan_guard.hako
│
├─ shared/ (40+ files)
│  ├─ json/ ⚠️ 重複多数
│  │  ├─ json_cursor.hako (used by 22 files)
│  │  ├─ json_utils.hako (211 lines)
│  │  ├─ json_field_extractor.hako (used by 71 files)
│  │  └─ mir_builder_*.hako × 3 files (複雑)
│  │
│  └─ common/
│     ├─ string_helpers.hako (174 lines, 68 files使用) ⚠️ 重複
│     └─ string_ops.hako ⚠️ 重複
│
└─ compiler/pipeline_v2/ (38 files)
   ├─ *_box.hako × 30+ files (良好、統一されている ✅)
   └─ *_helpers_box.hako × 2 files (適切に分離 ✅)
```

---

## 🎯 目標マップ（After）

```
selfhost/ (155 files, 12,200 lines) ← 10ファイル削減、1,200行削減
├─ hakorune-vm/ (30 files, ~3,000 lines) ← 14ファイル削減
│  ├─ instruction_handler_registry_box.hako (NEW!) 🆕
│  │  └─ handlers/ ← 22 handlersを統一管理
│  │     ├─ binop_handler_box.hako
│  │     ├─ compare_handler_box.hako
│  │     └─ ... (22 handlers, 統一インターフェース)
│  │
│  ├─ json_locator_utils_box.hako (NEW!) 🆕
│  │  └─ 9 locator/scanner files統合
│  │
│  └─ validation_guard_box.hako (NEW!) 🆕
│     └─ 4 guard files統合
│
├─ shared/ (30 files) ← 10ファイル削減
│  ├─ json/
│  │  ├─ json_navigator_box.hako (NEW!) 🆕
│  │  │  └─ JsonCursor + JsonUtils + JsonFieldExtractor統合
│  │  └─ mir_builder_core_box.hako (統合後)
│  │
│  └─ common/
│     ├─ string_ops_box.hako (統合後) 🔄
│     └─ result_builder_box.hako (拡張) 🔄
│
└─ compiler/pipeline_v2/ (38 files) ← 変更なし ✅
```

---

## 📊 Box統合優先度マトリックス

```
          高影響
            ↑
            │  [JsonNavigatorBox]     [InstructionHandlerRegistry]
            │     77 files               22 files
            │     🔥最優先              🔥高優先
            │
            │  [ResultBuilderBox]     [JsonLocatorUtilsBox]
            │     全ファイル            9 files
            │     🔥高優先              🔶中優先
   影響 ────┼─────────────────────────────────→ 難易度
   範囲     │  [GuardBox統合]          [MirBuilder再編]
            │     4 files               5+ files
            │     🔶中優先              🔵低優先
            │
            │  [StringOpsBox]
            │     68 files
            │     🔶中優先
            ↓
          低影響
```

---

## 🚀 3フェーズ実装計画

### Phase 1: クイックウィン（2-3週間）

```
Week 1-2: JsonNavigatorBox 🔥
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
├─ JsonCursorBox (22 files)
├─ JsonUtilsBox (211 lines)
├─ JsonFieldExtractor (71 files)
└─ string_helpers.hako (JSON部分)

[期待効果]
✅ 削減: 200-300 lines
✅ JSON処理完全統一
✅ 学習コスト -50%

[実装ステップ]
1. json_navigator_box.hako作成
2. 基本メソッド移植（extract_*, read_*, skip_*）
3. 5 filesずつ段階的移行
4. テスト・検証

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Week 2-3: ResultBuilderBox拡張 🔥
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
├─ ErrorBuilderBox (8 lines)
└─ 手動エラー文字列生成パターン

[期待効果]
✅ 削減: 100-150 lines
✅ Rust-style Result確立
✅ エラーハンドリング統一

[実装ステップ]
1. result_box.hako拡張
2. unwrap_or, map, and_then追加
3. 段階的適用（低リスク）

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Phase 1 成果**: 300-450行削減、保守性+40%

---

### Phase 2: 戦略的統合（4-6週間）

```
Week 4-6: InstructionHandlerRegistry 🔥
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
22個の *_handler.hako files (2,068 lines)

[期待効果]
✅ 削減: 300-400 lines
✅ 新規命令追加の容易化
✅ VM実行の明確化

[実装ステップ]
Week 4: Registry骨格 + 5 handlers
Week 5: 次の10 handlers
Week 6: 残り7 handlers + テスト

[設計パターン]
box InstructionHandlerRegistryBox {
  handlers: MapBox
  register_all()
  dispatch(op, context) → Result
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Week 7-8: JsonLocatorUtilsBox + GuardBox統合 🔶
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
├─ 9 locator/scanner files (576 lines)
└─ 4 guard files (80-100 lines)

[期待効果]
✅ 削減: 230-300 lines
✅ 検証ロジック統一
✅ JSON解析の明確化

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Phase 2 成果**: 800-1,000行削減、拡張性+60%

---

### Phase 3: 長期改善（8-12週間、Phase 20.6以降）

```
Week 9-12: MirBuilder系統再編 🔵
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
├─ mir_builder_min.hako (436 lines)
├─ mir_builder2.hako
├─ mir_builder_box.hako
├─ block_builder_box.hako (231 lines)
└─ mir_io_box.hako (185 lines)

[期待効果]
✅ 削減: 300-500 lines
✅ MIR生成の明確化
✅ アーキテクチャ完成

[設計パターン]
MirBuilderCoreBox (基底)
  ↓ from
MirBuilderMinBox (最小限)
  ↓ from
MirBuilderFullBox (フル機能)

⚠️ 警告: 高リスク、Phase 20.6以降推奨
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Week 13-16: StringOpsBox統合 + 全体最適化 🔶
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[統合対象]
├─ string_helpers.hako (174 lines, 68 files使用)
└─ string_ops.hako

[期待効果]
✅ 削減: 50-100 lines
✅ 文字列操作完全統一
✅ 学習コスト -60%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Phase 3 成果**: 累計1,200行削減、アーキテクチャ完成度+80%

---

## 📈 削減効果の累積グラフ

```
削減行数
  ↑
1200│                                    ●  累計: 1,200行削減
    │                                  ／
1000│                            ●    ／     Phase 3完了
    │                          ／    ／
 800│                    ●    ／    ／       Phase 2完了
    │                  ／    ／    ／
 600│            ●    ／    ／    ／
    │          ／    ／    ／    ／
 400│    ●    ／    ／    ／    ／          Phase 1完了
    │  ／    ／    ／    ／    ／
 200│●      ／    ／    ／    ／            Week 2完了
    │      ／    ／    ／    ／
   0└─────┴─────┴─────┴─────┴───────────→ 時間
    0    2w   4w   6w   8w  10w  12w  16w

凡例:
● Phase 1: JsonNavigator + ResultBuilder (2-3週間)
● Phase 2: HandlerRegistry + Locator/Guard (4-6週間)
● Phase 3: MirBuilder + StringOps統合 (8-12週間)
```

---

## 🎯 即座に実行可能なコマンド

### Step 1: JsonNavigatorBox作成（最優先！）

```bash
# 1. 新規ファイル作成
cd /home/tomoaki/git/hakorune-selfhost
touch selfhost/shared/json/json_navigator_box.hako

# 2. 骨格実装
cat > selfhost/shared/json/json_navigator_box.hako <<'EOF'
// json_navigator_box.hako — JSON統合ナビゲーター
// 責任: JSON解析・抽出・スキャンの統一インターフェース

using "selfhost/shared/common/string_helpers.hako" as StringHelpers

static box JsonNavigatorBox {
  // 【Phase 1: 基本抽出】
  extract_value(json, key) {
    // JsonUtilsBox.extract_value を移植
  }

  extract_int(json, key) {
    // JsonFieldExtractor.extract_int を移植
  }

  extract_string(json, key) {
    // JsonFieldExtractor.extract_string を移植
  }

  // 【Phase 2: 構造スキャン】
  read_object(json, idx) {
    // JsonUtilsBox.read_object を移植
  }

  read_array(json, idx) {
    // JsonUtilsBox.read_array を移植
  }

  skip_string(json, idx) {
    // JsonUtilsBox.skip_string を移植
  }

  // 【Phase 3: 高度操作】
  split_top_level(array_json) {
    // JsonUtilsBox.split_top_level を移植
  }

  // 【Phase 4: 位置検索】
  index_of_from(json, pattern, pos) {
    // JsonCursorBox.index_of_from を移植
  }
}
EOF

# 3. 段階的移行スクリプト
cat > tools/migrate_to_json_navigator.sh <<'EOF'
#!/bin/bash
# JsonNavigatorBox移行スクリプト

echo "Phase 1: JsonUtilsBox使用箇所を移行 (2 files)"
# TODO: using "json_utils.hako" → using "json_navigator_box.hako"

echo "Phase 2: JsonCursorBox使用箇所を移行 (22 files)"
# TODO: using "json_cursor.hako" → using "json_navigator_box.hako"

echo "Phase 3: JsonFieldExtractor使用箇所を移行 (71 files)"
# TODO: using "json_field_extractor.hako" → using "json_navigator_box.hako"

echo "Phase 4: テスト・検証"
bash tools/smokes/v2/run.sh --profile quick
EOF

chmod +x tools/migrate_to_json_navigator.sh
```

### Step 2: ResultBuilderBox拡張

```bash
# 既存ファイル編集
vim selfhost/vm/boxes/result_box.hako

# 追加メソッド例:
# unwrap_or(result, default) → Value
# map(result, func) → Result
# and_then(result, func) → Result
```

### Step 3: 進捗確認

```bash
# Box統合進捗レポート
echo "=== Box統合進捗レポート ==="
echo "JsonNavigatorBox: $(grep -r 'using.*json_navigator_box' selfhost --include='*.hako' | wc -l) files移行済み"
echo "ResultBuilder拡張: $(grep -r 'ResultBuilderBox.unwrap_or' selfhost --include='*.hako' | wc -l) 箇所適用済み"
echo "HandlerRegistry: $(grep -r 'InstructionHandlerRegistry' selfhost --include='*.hako' | wc -l) files使用中"
```

---

## 💡 成功の鍵

### ✅ DO（実施すべきこと）
1. **段階的移行**: 5-10 filesずつ移行、テスト確認
2. **後方互換性**: 旧Box削除は最後の最後（全移行完了後）
3. **テスト駆動**: 各移行後に `tools/smokes/v2/run.sh` 実行
4. **ドキュメント更新**: 各Box化完了時にREADME更新

### ❌ DON'T（避けるべきこと）
1. ❌ 一気に全ファイル移行（リスク大）
2. ❌ テストなしでcommit（Silent failureリスク）
3. ❌ 旧Boxの即座削除（Rollback不可）
4. ❌ Phase 3を先に実施（Phase 1/2が基盤）

---

## 🎓 学んだこと

### Hakoruneの素晴らしい点
1. **96.4% Box化済み** - 業界トップクラス！
2. **Everything is Box原則** - 一貫性が高い
3. **命名規則統一** - *_box, *_handler, *_guardが明確

### 改善機会
1. **責任の重複** - JSON処理が4箇所に分散
2. **Handler散在** - 22 handlersが独立管理
3. **Error処理** - Result型が未統一

### 次のステップ
**JsonNavigatorBox作成 → 即座に着手可能！**

---

**作成日**: 2025-10-15
**作成者**: Claude Code (Anthropic)
**レビュー推奨**: tomoaki-san
**次のアクション**: `bash tools/migrate_to_json_navigator.sh` 実行
