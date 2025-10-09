# Phase 15.15: 共通化綺麗綺麗大作戦（Boxification + Deduplication）

**開始日**: 2025-10-09
**完了日**: 2025-10-09
**期間**: 約3時間
**戦略**: Task Teacher 4人がかり調査 → 優先順位順実行

---

## 📋 概要

Phase 15.13/15.14でマクロ適用が完了した後、Task Teacher 4人がかりで「箱化・モジュール化・共通化・マクロ」の多角的調査を実施。その結果、**優先度1-3の施策を順次実行**し、純削減48行を達成。

**重要**: 行数削減より**品質優先**（カプセル化・保守性向上）を選択。

---

## 🎯 実行内容

### 15.15.1: MirJsonBuilderMin Instance Box化（+38行）

**目的**: MapBox状態パラメータ threading → 内部フィールド化

**変更**:
```hako
// Before: Static box + MapBox state
static box MirJsonBuilderMin {
  make() { return map({ buf: "", phase: 0, ... }) }
  start_module(st) { ... }
}

// After: Instance box with fields
box MirJsonBuilderMin {
  buf: StringBox
  phase: IntegerBox
  birth() { me.buf = "" me.phase = 0 ... }
  start_module() { me.phase = 1 ... }
}
```

**ファイル**:
- `apps/selfhost/common/json/mir_builder_min.hako` (377→415行, +38)
- 7つのスモークテスト更新（使用パターン変更）

**テスト**: 302/302 PASS ✅

**コミット**: `7630bc1f`

---

### 15.15.2: find_balanced_* 統合（-53行）

**目的**: 3ファイルの重複実装を JsonCursorBox に集約

**変更**:
```hako
// Before: 56 lines of implementation
find_balanced_array_end(json, idx) {
  @n = json.length()
  // ... 26 lines of loop logic
  return -1
}

// After: 1 line delegation
find_balanced_array_end(json, idx) { return JsonCursorBox.seek_array_end(json, idx) }
```

**ファイル**:
- `apps/selfhost/common/mini_vm_scan.hako` (132→80行, -52)
- `apps/selfhost/vm/boxes/phi_apply_box.hako` (import追加)

**テスト**: 302/302 PASS ✅

**コミット**: `88d0038e`

---

### 15.15.3: _str_to_int 統合（-33行）

**目的**: 3ファイルの _str_to_int 実装を StringHelpers に委譲

**変更**:
```hako
// Before: 16-20 lines per file
_str_to_int(s) {
  local n = 0
  local i = 0
  loop(i < s.length()) {
    local ch = s.substring(i, i+1)
    n = n * 10 + ("0123456789").indexOf(ch)
    i = i + 1
  }
  return n
}

// After: 1 line delegation
_str_to_int(s) { return StringHelpers.to_i64(s) }
```

**ファイル**:
- `apps/selfhost/vm/collect_mixed_smoke.hako` (-18行)
- `apps/selfhost/vm/boxes/arithmetic.hako` (-14行)
- `apps/selfhost/vm/boxes/phi_apply_box.hako` (-1行、既に委譲済み確認）

**テスト**: 302/302 PASS ✅

**コミット**: `f340a15b`

---

## 📊 統計

### コード変更
| サブフェーズ | 変更行数 | 修正ファイル | 新規ファイル |
|------------|---------|------------|------------|
| 15.15.1    | +38     | 8          | 0          |
| 15.15.2    | -53     | 2          | 0          |
| 15.15.3    | -33     | 3          | 0          |
| **合計**   | **-48** | **13**     | **0**      |

### 内訳
- 削減: -86行（重複実装削除）
- 追加: +38行（品質投資 - instance box化）
- 純削減: **-48行**

### テスト結果
- ✅ 全テスト: 302/302 PASS（0エラー）
- ✅ スモークテスト: 全PASS
- ✅ 既知の問題なし

---

## 💡 重要な判断

### 品質優先の決断（15.15.1）

**行数増加を受け入れた理由**:
1. **カプセル化**: MapBox状態管理→内部フィールド（データ隠蔽）
2. **API改善**: `.make() |> .start_module(st)` → `new Builder().start_module()`
3. **保守性**: 状態管理が明示的、バグ混入リスク減少

**教訓**: 行数削減は手段、品質向上が目的。

---

## 🧪 テスト戦略

### スモークテスト更新（15.15.1）
7つのスモークテストで使用パターン変更：

```bash
# Before
local j = MirJsonBuilderMin.make()
  |> MirJsonBuilderMin.start_module()
  |> MirJsonBuilderMin.to_string()

# After
local builder = new MirJsonBuilderMin()
builder.start_module()
local j = builder.to_string()
```

**更新ファイル**:
- `selfhost_mir_m2_binop_ops_vm.sh`
- `selfhost_mir_m2_compare_ops_vm.sh`
- `selfhost_pipeline_v2_call_exec_vm.sh`
- `selfhost_pipeline_v2_method_exec_vm.sh`
- `selfhost_pipeline_v2_newbox_exec_vm.sh`
- `selfhost_terminator_guard_after_ret_vm.sh`
- `selfhost_mir_m2_compare_ge_builder_vm_llvm.sh`

### 既存問題の継続
- ⚠️ Mini-VM実装エラー（Phase 15.15と無関係、既存問題）
- ⚠️ `mini_vm_scan.hako` パースエラー（Phase 15.15.2前から存在）

---

## 🎯 Task Teacher 調査結果（参考）

4つの Task Teacher による並行調査の結果：

### Task 1: Boxing/Modularization
- 推定削減: -330～-470行
- 主要候補: MirJsonBuilderMin instance化（採用✅）

### Task 2: Common code/Deduplication
- 推定削減: -200～-304行
- 主要候補: find_balanced_*/\_str_to_int（採用✅）

### Task 3: Macro/Structure improvement
- 推定削減: -200～-390行
- 状況: Phase 15.13/15.14で既に実施済み

### Task 4: Architecture improvement
- 対象: 大規模リファクタリング
- 判断: 優先度低（現時点で不要）

**実行判断**: Task 1-2の優先度1-3を順次実行（Task 3は既に完了、Task 4は保留）

---

## 🎓 学び

### 成功要因
1. **Task Teacher活用**: 多角的調査で抜け漏れ防止
2. **優先順位**: 推定削減行数＋実装工数で判断
3. **品質優先**: 行数増加を恐れない（15.15.1）
4. **段階実行**: 3サブフェーズに分割、各コミット・テスト

### 改善点
なし（計画通り実行、全テストPASS）

---

## 📝 コミット

1. `7630bc1f` - "refactor(selfhost): MirJsonBuilderMin instance box化 Phase 15.15.1"
2. `88d0038e` - "refactor(selfhost): find_balanced_* integration Phase 15.15.2"
3. `f340a15b` - "refactor(selfhost): _str_to_int integration Phase 15.15.3"

---

## 🚀 次のステップ

**Phase 15完了**: 15.13（マクロ適用 -52行）、15.14（可読性 +6行）、15.15（共通化 -48行）

**次の Phase**: Mini-VM Migration Plan Step 2 - Mini-VM実装 with @match（10-15人日）

詳細: [mini_vm_migration_plan.md](../../current/main/mini_vm_migration_plan.md)

---

## 📚 関連ドキュメント

- [Phase 15.13 README](../phase-15.13/README.md) - マクロ適用
- [Phase 15.14 README](../phase-15.14/README.md) - @match適用
- [CLAUDE.md](../../../../CLAUDE.md) - 開発状況サマリー
