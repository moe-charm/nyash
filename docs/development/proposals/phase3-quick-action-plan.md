# Phase 3: Box統合 Quick Action Plan

**即座に実装できる2つのBox統合**（2時間で46行削減）

---

## 🚀 Action 1: CompareOpsBox統合（1時間、26行削減）

### 修正ファイル: `apps/selfhost/vm/boxes/mir_vm_min.hako`

**削除箇所1**（行228-233）:
```hako
# 現在（6行）
if cmp == "Eq" { if lv == rv { cv = 1 } else { cv = 0 } }
else if cmp == "Ne" { if lv != rv { cv = 1 } else { cv = 0 } }
else if cmp == "Gt" { if lv > rv { cv = 1 } else { cv = 0 } }
else if cmp == "Ge" { if lv >= rv { cv = 1 } else { cv = 0 } }
else if cmp == "Lt" { if lv < rv { cv = 1 } else { cv = 0 } }
else if cmp == "Le" { if lv <= rv { cv = 1 } else { cv = 0 } }

# 置換後（1行）
local cv = CompareOpsBox.eval(cmp, lv, rv)
```

**削除箇所2**（行299-304）:
```hako
# 同じ重複コードを同様に1行に置換
local cv = CompareOpsBox.eval(cmp, lv, rv)
```

**using追加**:
```hako
# 既に using 済み（行6）なので追加不要
using "apps/selfhost/vm/boxes/compare_ops.hako" as CompareOpsBox
```

### 修正ファイル: `apps/selfhost/vm/boxes/op_handlers.hako`

**削除箇所**（行67-76）:
```hako
# 削除: _map_cmp_symbol メソッド（既に CompareOpsBox.map_symbol が存在）
# 削除: _eval_cmp メソッド（既に CompareOpsBox.eval が存在）

# 行112の呼び出しを変更:
# 現在: kind = CompareOpsBox.map_symbol(sym)  # 既に正しい
# 行122の呼び出しを変更:
# 現在: local r = CompareOpsBox.eval(kind, a, b)  # 既に正しい
```

**注**: op_handlers.hakoは既にCompareOpsBoxを使用中（行5, 112, 122）。
行67-76の重複メソッドは単純に削除するだけ（呼び出し側は変更不要）。

### テスト
```bash
tools/smokes/v2/run.sh --profile quick
```

### コミット
```bash
git add apps/selfhost/vm/boxes/mir_vm_min.hako apps/selfhost/vm/boxes/op_handlers.hako
git commit -m "refactor(vm): consolidate comparison operators into CompareOpsBox (-26 lines)"
```

---

## 🚀 Action 2: JsonScannerBox統合（1時間、20行削除）

### 修正ファイル: `apps/selfhost/vm/boxes/mir_vm_min.hako`

**using追加**（冒頭に追加）:
```hako
using "selfhost/shared/json/json_cursor.hako" as JsonCursorBox
```

**削除箇所**（行47-49）:
```hako
# 削除: _seek_array_end メソッド（20行）
_seek_array_end(text, pos){ ... }

# 削除: _block_insts_end メソッド（1行）
_block_insts_end(mjson,insts_start){ return me._seek_array_end(mjson,insts_start) }
```

**置換箇所**（行78付近）:
```hako
# 現在
local endp = me._block_insts_end(mjson, start)

# 置換後
local endp = JsonCursorBox.seek_array_end(mjson, start)
```

**注**: JsonCursorBox.seek_array_end() はmir_vm_min の _seek_array_end と完全に同一機能。

### テスト
```bash
tools/smokes/v2/run.sh --profile quick
```

### コミット
```bash
git add apps/selfhost/vm/boxes/mir_vm_min.hako
git commit -m "refactor(vm): use JsonCursorBox.seek_array_end instead of local impl (-20 lines)"
```

---

## 📊 即座実行の効果

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| mir_vm_min.hako | 319行 | 273行 | **-46行** |
| op_handlers.hako | 143行 | 133行 | **-10行** |
| **合計** | 462行 | 406行 | **-56行** |

**実装時間**: 2時間
**リスク**: 極小（既存Box使用、重複削除のみ）
**テスト**: スモークテスト（quick profile、約30秒）

---

## 🎯 実行手順（2時間で完了）

```bash
# 1. ブランチ作成
git checkout -b refactor/phase3-box-consolidation

# 2. CompareOpsBox統合（1時間）
# - mir_vm_min.hako: 行228-233, 299-304 を CompareOpsBox.eval() に置換
# - op_handlers.hako: 行67-76 削除
tools/smokes/v2/run.sh --profile quick
git add apps/selfhost/vm/boxes/{mir_vm_min,op_handlers}.hako
git commit -m "refactor(vm): consolidate comparison operators into CompareOpsBox (-26 lines)"

# 3. JsonScannerBox統合（1時間）
# - mir_vm_min.hako: using JsonCursorBox 追加、行47-49削除、行78置換
tools/smokes/v2/run.sh --profile quick
git add apps/selfhost/vm/boxes/mir_vm_min.hako
git commit -m "refactor(vm): use JsonCursorBox.seek_array_end instead of local impl (-20 lines)"

# 4. 統合テスト
tools/smokes/v2/run.sh --profile integration

# 5. マージ
git checkout selfhost
git merge refactor/phase3-box-consolidation
git branch -d refactor/phase3-box-consolidation
```

---

## ✅ 完了基準

- [ ] mir_vm_min.hako: 319行 → 273行（-46行）
- [ ] op_handlers.hako: 143行 → 133行（-10行）
- [ ] スモークテスト（quick）: すべて通過
- [ ] スモークテスト（integration）: すべて通過
- [ ] コミット2件作成

---

**準備完了！今すぐ実行可能です。**
