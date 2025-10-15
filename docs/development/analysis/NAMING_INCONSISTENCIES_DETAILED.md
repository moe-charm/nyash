# 命名不統一箇所 詳細リスト

**分析日**: 2025-10-15
**対象**: selfhost/ 全体 (165 files)

---

## 🎯 優先度: High - Box名 `Box` 接尾辞なし

### 1. StringHelpers → StringHelpersBox

**ファイル**: `selfhost/shared/common/string_helpers.hako:5`
**現在の定義**:
```hakorune
static box StringHelpers {
    int_to_str(n) { ... }
    to_i64(x) { ... }
    json_quote(s) { ... }
    // ... 他20+ メソッド
}
```

**推奨変更**:
```hakorune
static box StringHelpersBox {
    // 内容は同じ
}
```

**影響範囲** (using文・静的呼び出し):
```bash
# 検索コマンド
grep -rn "StringHelpers\." selfhost/ --include="*.hako"
grep -rn "as StringHelpers" selfhost/ --include="*.hako"
```

**推定影響**: 50+ 箇所

---

### 2. StringOps → StringOpsBox

**ファイル**: `selfhost/shared/common/string_ops.hako:5`
**現在の定義**:
```hakorune
static box StringOps {
    substring_safe(s, a, b) { ... }
    char_at(s, i) { ... }
    ends_with(s, suffix) { ... }
}
```

**推奨変更**: → `StringOpsBox`

**影響範囲**: 10+ 箇所

---

### 3. MiniVm 系 (3ファイルで重複定義!)

#### 3.1 `selfhost/vm/mini_vm_lib.hako:5`
```hakorune
static box MiniVm {
    run(json) { ... }
    // ライブラリ版
}
```

#### 3.2 `selfhost/vm/mini_vm_if_branch.hako:4`
```hakorune
static box MiniVm {
    run_branch_trace(mir_json, trace) { ... }
    // 条件分岐特化版
}
```

#### 3.3 `selfhost/vm/boxes/mini_vm_core.hako:7`
```hakorune
static box MiniVm {
    run_min(mir_json) { ... }
    // コア版
}
```

**問題**: **同一Box名が3ファイルで重複定義** ⚠️⚠️⚠️
**推奨変更**:
- `MiniVm` → `MiniVmLibBox` (lib版)
- `MiniVm` → `MiniVmBranchBox` (branch版)
- `MiniVm` → `MiniVmCoreBox` (core版)

**影響範囲**: 30+ 箇所 (using文・静的呼び出し)

---

### 4. DepTree 系

#### 4.1 `selfhost/tools/dep_tree.hako`
```hakorune
static box DepTree {
    // 依存関係ツリー本体
}
```

**推奨変更**: → `DepTreeBox`

#### 4.2 `selfhost/tools/dep_tree_core.hako`
```hakorune
static box DepTreeCore {
    // コア実装
}
```

**推奨変更**: → `DepTreeCoreBox`

**影響範囲**: 5+ 箇所

---

### 5. FlowRunner → FlowRunnerBox

**ファイル**: `selfhost/vm/flow_runner.hako:5`
**現在の定義**:
```hakorune
static box FlowRunner {
    run_flow(source, backend) { ... }
}
```

**推奨変更**: → `FlowRunnerBox`

**影響範囲**: 10+ 箇所

---

### 6. VM実装系

#### 6.1 MirVmMin → MirVmMinBox
**ファイル**: `selfhost/vm/boxes/mir_vm_min.hako:10`
**影響**: 20+ 箇所

#### 6.2 MirVmM2 → MirVmM2Box
**ファイル**: `selfhost/vm/boxes/mir_vm_m2.hako:8`
**影響**: 15+ 箇所

#### 6.3 HakoruneVmCore → HakoruneVmCoreBox
**ファイル**: `selfhost/hakorune-vm/hakorune_vm_core.hako:12`
**影響**: 40+ 箇所 (最多!)

---

## 📋 自動修正スクリプト (詳細版)

### Phase 1: StringHelpers → StringHelpersBox

```bash
#!/bin/bash
# Phase 1: StringHelpers → StringHelpersBox

OLD="StringHelpers"
NEW="StringHelpersBox"

echo "=== Phase 1: $OLD → $NEW ==="

# 1. Box定義の変更
echo "Step 1: Box定義変更"
sed -i "s/^static box $OLD {/static box $NEW {/" selfhost/shared/common/string_helpers.hako

# 2. using文の変更
echo "Step 2: using文変更"
find selfhost -name "*.hako" -type f -exec sed -i "s/ as $OLD\$/ as $NEW/" {} +

# 3. 静的呼び出しの変更
echo "Step 3: 静的呼び出し変更"
find selfhost -name "*.hako" -type f -exec sed -i "s/\b$OLD\./$NEW./g" {} +

# 4. 検証
echo "Step 4: 検証"
echo "Box定義数 (should be 1):"
grep -r "^static box $NEW {" selfhost/ --include="*.hako" | wc -l

echo "旧Box名残存 (should be 0):"
grep -r "^static box $OLD {" selfhost/ --include="*.hako" | wc -l
grep -r " as $OLD\$" selfhost/ --include="*.hako" | wc -l
grep -r "\b$OLD\." selfhost/ --include="*.hako" | wc -l

echo "=== Phase 1 完了 ==="
```

---

### Phase 2: MiniVm系 (重複解消 + Box接尾辞)

```bash
#!/bin/bash
# Phase 2: MiniVm系の重複解消

echo "=== Phase 2: MiniVm系重複解消 ==="

# 2.1 MiniVm (lib) → MiniVmLibBox
OLD="MiniVm"
NEW="MiniVmLibBox"
FILE="selfhost/vm/mini_vm_lib.hako"

echo "Step 2.1: $OLD → $NEW (lib)"
sed -i "s/^static box $OLD {/static box $NEW {/" "$FILE"

# using文・呼び出しの変更 (mini_vm_lib.hako 使用箇所のみ)
# 注: 他のMiniVmと区別するため、慎重に実施
grep -rn "using \"selfhost/vm/mini_vm_lib.hako\" as $OLD" selfhost/ --include="*.hako" | \
  cut -d: -f1 | sort -u | while read f; do
    echo "Updating: $f"
    sed -i "s/ as $OLD\$/ as $NEW/" "$f"
    sed -i "s/\b$OLD\./$NEW./g" "$f"
done

# 2.2 MiniVm (branch) → MiniVmBranchBox
OLD="MiniVm"
NEW="MiniVmBranchBox"
FILE="selfhost/vm/mini_vm_if_branch.hako"

echo "Step 2.2: $OLD → $NEW (branch)"
sed -i "s/^static box $OLD {/static box $NEW {/" "$FILE"

grep -rn "using \"selfhost/vm/mini_vm_if_branch.hako\" as $OLD" selfhost/ --include="*.hako" | \
  cut -d: -f1 | sort -u | while read f; do
    echo "Updating: $f"
    sed -i "s/ as $OLD\$/ as $NEW/" "$f"
    sed -i "s/\b$OLD\./$NEW./g" "$f"
done

# 2.3 MiniVm (core) → MiniVmCoreBox
OLD="MiniVm"
NEW="MiniVmCoreBox"
FILE="selfhost/vm/boxes/mini_vm_core.hako"

echo "Step 2.3: $OLD → $NEW (core)"
sed -i "s/^static box $OLD {/static box $NEW {/" "$FILE"

grep -rn "using \"selfhost/vm/boxes/mini_vm_core.hako\" as $OLD" selfhost/ --include="*.hako" | \
  cut -d: -f1 | sort -u | while read f; do
    echo "Updating: $f"
    sed -i "s/ as $OLD\$/ as $NEW/" "$f"
    sed -i "s/\b$OLD\./$NEW./g" "$f"
done

echo "=== Phase 2 完了 ==="
```

---

### Phase 3: 残りのBox接尾辞追加

```bash
#!/bin/bash
# Phase 3: 残りのBox接尾辞追加

echo "=== Phase 3: 残りのBox接尾辞追加 ==="

declare -A RENAMES=(
    ["StringOps"]="StringOpsBox"
    ["DepTree"]="DepTreeBox"
    ["DepTreeCore"]="DepTreeCoreBox"
    ["FlowRunner"]="FlowRunnerBox"
    ["MirVmMin"]="MirVmMinBox"
    ["MirVmM2"]="MirVmM2Box"
    ["HakoruneVmCore"]="HakoruneVmCoreBox"
)

for old in "${!RENAMES[@]}"; do
    new="${RENAMES[$old]}"
    echo "Processing: $old → $new"

    # Box定義の変更
    find selfhost -name "*.hako" -type f -exec sed -i "s/^static box $old {/static box $new {/" {} +
    find selfhost -name "*.hako" -type f -exec sed -i "s/^box $old {/box $new {/" {} +

    # using文の変更
    find selfhost -name "*.hako" -type f -exec sed -i "s/ as $old\$/ as $new/" {} +

    # 静的呼び出しの変更
    find selfhost -name "*.hako" -type f -exec sed -i "s/\b$old\./$new./g" {} +

    echo "✓ $old → $new 完了"
done

echo "=== Phase 3 完了 ==="
```

---

## ✅ テスト実行計画

### Phase 1後テスト

```bash
# Phase 1: StringHelpers → StringHelpersBox 実施後
./tools/smokes/v2/run.sh --profile quick

# 期待結果: 全テスト PASS
# 失敗時: git revert
```

### Phase 2後テスト

```bash
# Phase 2: MiniVm系重複解消 実施後
./tools/smokes/v2/run.sh --profile quick

# 特に VM関連テストを重点確認
./tools/smokes/v2/run.sh --profile quick-selfhost

# 期待結果: 全テスト PASS
```

### Phase 3後テスト

```bash
# Phase 3: 残りBox接尾辞追加 実施後
./tools/smokes/v2/run.sh --profile quick
./tools/smokes/v2/run.sh --profile integration

# 期待結果: 170 PASS (現状維持)
```

---

## 📊 影響範囲マトリックス

| Box名 | ファイル数 | 影響箇所 (推定) | リスク | 優先度 |
|-------|----------|----------------|--------|--------|
| **StringHelpers** | 1 | 50+ | **High** | 🔴 High |
| **HakoruneVmCore** | 1 | 40+ | **High** | 🔴 High |
| **MiniVm (3重複)** | 3 | 30+ | **Very High** | 🔴🔴 Critical |
| **MirVmMin** | 1 | 20+ | Medium | 🟡 Medium |
| **MirVmM2** | 1 | 15+ | Medium | 🟡 Medium |
| **FlowRunner** | 1 | 10+ | Low | 🟢 Low |
| **StringOps** | 1 | 10+ | Low | 🟢 Low |
| **DepTree** | 1 | 5+ | Low | 🟢 Low |
| **DepTreeCore** | 1 | 5+ | Low | 🟢 Low |

---

## 🚨 リスク分析

### Critical Risk: MiniVm 3重複定義

**問題**:
- 同一Box名 `MiniVm` が3ファイルで定義
- Hakorune実行時、どのMiniVmが使われるか不明確
- 現状は using文の順序依存 (非常に脆弱)

**影響**:
- テスト失敗の原因になりうる
- 将来のリファクタで混乱を招く

**推奨対応**:
1. **即座に修正** (Phase 2優先実施)
2. 3つのMiniVmを明確に分離:
   - `MiniVmLibBox` (lib版)
   - `MiniVmBranchBox` (branch版)
   - `MiniVmCoreBox` (core版)

---

### High Risk: StringHelpers / HakoruneVmCore

**問題**:
- 使用頻度が最も高い (50+, 40+ 箇所)
- 自動置換ミスの影響が大きい

**推奨対応**:
1. **慎重な自動修正** + git diff全確認
2. テスト実行 (quick + integration)
3. 失敗時の即座のrevert準備

---

## 📅 実施スケジュール

### Week 1: 準備・検証

**Day 1-2**: 影響範囲の完全調査
```bash
# 各Box名の使用箇所を完全リストアップ
for box in StringHelpers MiniVm HakoruneVmCore; do
    echo "=== $box ==="
    grep -rn "\b$box\." selfhost/ --include="*.hako"
    grep -rn " as $box\$" selfhost/ --include="*.hako"
done > naming_impact_full.txt
```

**Day 3-4**: スクリプトのドライラン検証
```bash
# dry-run モード追加
bash naming_phase1_dryrun.sh > phase1_preview.txt
# 変更内容を確認、問題ないか検証
```

**Day 5**: バックアップ・ブランチ作成
```bash
git checkout -b feature/naming-unification
git add -A
git commit -m "Backup before naming unification"
```

---

### Week 2: Phase 1-2実施 (Critical)

**Day 1**: Phase 1 (StringHelpers)
```bash
bash naming_phase1.sh
git diff | less  # 全変更確認
./tools/smokes/v2/run.sh --profile quick
git commit -m "refactor: StringHelpers → StringHelpersBox"
```

**Day 2-3**: Phase 2 (MiniVm系重複解消)
```bash
bash naming_phase2.sh
git diff | less  # 全変更確認
./tools/smokes/v2/run.sh --profile quick
./tools/smokes/v2/run.sh --profile quick-selfhost  # VM重点テスト
git commit -m "refactor: MiniVm系重複解消 (3ファイル → 3Box)"
```

**Day 4**: HakoruneVmCore → HakoruneVmCoreBox
```bash
# Phase 3の一部を前倒し (影響大)
bash naming_phase3_hakorune_vm.sh
git diff | less
./tools/smokes/v2/run.sh --profile quick
git commit -m "refactor: HakoruneVmCore → HakoruneVmCoreBox"
```

**Day 5**: 統合テスト
```bash
./tools/smokes/v2/run.sh --profile integration
# 170 PASS 確認
```

---

### Week 3: Phase 3実施 (残り)

**Day 1-3**: 残りBox接尾辞追加
```bash
bash naming_phase3_remaining.sh
git diff | less
./tools/smokes/v2/run.sh --profile quick
git commit -m "refactor: 残りBox接尾辞統一 (7 Boxes)"
```

**Day 4-5**: 統合テスト・文書更新
```bash
./tools/smokes/v2/run.sh --profile integration
# ドキュメント更新 (README, using例)
```

---

## 🎯 完了条件

### ✅ Phase 1完了条件
- [ ] StringHelpers → StringHelpersBox 変更完了
- [ ] using文・静的呼び出し全更新
- [ ] quick スモークテスト全PASS
- [ ] git commit完了

### ✅ Phase 2完了条件
- [ ] MiniVm 3ファイルの重複解消
- [ ] 3Box (Lib/Branch/Core) 独立
- [ ] quick + quick-selfhost スモークテスト全PASS
- [ ] git commit完了

### ✅ Phase 3完了条件
- [ ] 残り7Box の接尾辞統一
- [ ] integration スモークテスト全PASS (170 PASS維持)
- [ ] 関連ドキュメント更新

### ✅ 最終完了条件
- [ ] selfhost/ 内の全Box名が `*Box` 接尾辞で統一
- [ ] 例外: `*Main`, `*Stub`, `*Adapter` のみ
- [ ] スモークテスト 170 PASS 維持
- [ ] 命名規約ドキュメント更新

---

**作成日**: 2025-10-15
**最終更新**: 2025-10-15
**ステータス**: 📋 Ready for execution
