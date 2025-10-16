# LoopBuilder PHI生成とパラメータ保護の詳細分析

**Date**: 2025-10-16
**Investigator**: Claude (Task 3)
**Context**: ParserBox.parse_primary: パラメータがnull化される問題の根本原因調査

---

## 調査結果サマリ

### ❌ **Critical Issue Found: パラメータがループPHIの対象になっている**

**問題**: `prepare_loop_variables` は**すべての変数**（パラメータを含む）に対して不完全PHIを生成している。パラメータ保護機構は**存在しない**。

**影響**: ループ開始時にパラメータ（me, json_text, path等）もPHI対象になり、seal_block時にlatchからの値で上書きされる可能性がある。

---

## 詳細調査結果

### 1. prepare_loop_variables の動作 (phi.rs:14-33)

```rust
// src/mir/loop_builder/phi.rs:14-33
pub(super) fn prepare_loop_variables(
    &mut self,
    header_id: BasicBlockId,
    preheader_id: BasicBlockId,
) -> Result<(), String> {
    let current_vars = self.get_current_variable_map();  // ← ⚠️ すべての変数を取得
    crate::mir::phi_core::loop_phi::save_block_snapshot(
        &mut self.block_var_maps,
        preheader_id,
        &current_vars,
    );
    let incs = crate::mir::phi_core::loop_phi::prepare_loop_variables_with(
        self,
        header_id,
        preheader_id,
        &current_vars,  // ← ⚠️ すべての変数を渡す
    )?;
    self.incomplete_phis.insert(header_id, incs);
    Ok(())
}
```

**証拠**:
- **Line 19**: `self.get_current_variable_map()` - パラメータを含む全変数を取得
- **Line 25-30**: `prepare_loop_variables_with` に全変数を渡す
- **フィルタリング機構なし**: パラメータを除外する処理が存在しない

---

### 2. get_current_variable_map の動作 (mod.rs:151-153)

```rust
// src/mir/loop_builder/mod.rs:151-153
pub(super) fn get_current_variable_map(&self) -> HashMap<String, ValueId> {
    self.parent_builder.variable_map.clone()  // ← ⚠️ 全変数をclone
}
```

**証拠**: `parent_builder.variable_map` を**そのままclone**。パラメータも含まれる。

---

### 3. variable_map にパラメータが含まれる証拠 (lowering.rs:40-52)

```rust
// src/mir/builder/builder_calls/lowering.rs:40-52
if let Some(ref mut f) = self.current_function {
    f.metadata
        .optimization_hints
        .push("static_singleton_me".to_string());
    let me_id = self.value_gen.next();  // ← v%0
    me_origin = Some(me_id);
    f.params.push(me_id);
    self.variable_map.insert("me".to_string(), me_id);  // ← ⚠️ me を登録
    for p in &params {
        let pid = self.value_gen.next();  // ← v%1, v%2, ...
        f.params.push(pid);
        self.variable_map.insert(p.clone(), pid);  // ← ⚠️ パラメータを登録
    }
}
```

**証拠**:
- **Line 47**: `variable_map.insert("me", me_id)` - v%0 を登録
- **Line 51**: `variable_map.insert(param_name, pid)` - v%1, v%2 を登録

**結論**: ループ開始時の `variable_map` には以下が含まれる:
```
{
  "me": v%0,           // パラメータ（static box singleton）
  "json_text": v%1,    // パラメータ（引数1）
  "path": v%2,         // パラメータ（引数2）
  "local_var1": v%X,   // ローカル変数
  ...
}
```

---

### 4. prepare_loop_variables_with の動作 (loop_phi.rs:127-145)

```rust
// src/mir/phi_core/loop_phi.rs:127-145
pub fn prepare_loop_variables_with<O: LoopPhiOps>(
    ops: &mut O,
    _header_id: BasicBlockId,
    preheader_id: BasicBlockId,
    current_vars: &std::collections::HashMap<String, ValueId>,
) -> Result<Vec<IncompletePhi>, String> {
    let mut incomplete_phis: Vec<IncompletePhi> = Vec::new();
    for (var_name, &value_before) in current_vars.iter() {  // ← ⚠️ すべての変数をループ
        let phi_id = ops.new_value();  // ← 新しいPHI結果を生成
        let inc = IncompletePhi {
            phi_id,
            var_name: var_name.clone(),
            known_inputs: vec![(preheader_id, value_before)],  // ← preheaderからの入力のみ
        };
        incomplete_phis.push(inc);
        ops.update_var(var_name.clone(), phi_id);  // ← ⚠️ 変数を新PHI結果で上書き
    }
    Ok(incomplete_phis)
}
```

**証拠**:
- **Line 134**: `current_vars.iter()` - すべての変数（パラメータ含む）を処理
- **Line 135**: 各変数に対して新しいPHI結果 `phi_id` を生成
- **Line 142**: `ops.update_var(var_name, phi_id)` - **変数を新PHI結果で上書き**

**結果**: ループヘッダーに入った時点で、以下のように上書きされる:
```
Before (preheader):
  me: v%0 (param)
  json_text: v%1 (param)
  path: v%2 (param)

After (header, incomplete PHI):
  me: v%10 (PHI: preheader=v%0, latch=?)
  json_text: v%11 (PHI: preheader=v%1, latch=?)
  path: v%12 (PHI: preheader=v%2, latch=?)
```

---

### 5. seal_incomplete_phis_with の動作 (loop_phi.rs:97-122)

```rust
// src/mir/phi_core/loop_phi.rs:97-122
pub fn seal_incomplete_phis_with<O: LoopPhiOps>(
    ops: &mut O,
    block_id: BasicBlockId,
    latch_id: BasicBlockId,
    mut incomplete_phis: Vec<IncompletePhi>,
    continue_snapshots: &[(BasicBlockId, VarSnapshot)],
) -> Result<(), String> {
    for mut phi in incomplete_phis.drain(..) {
        // from continue points
        for (cid, snapshot) in continue_snapshots.iter() {
            if let Some(&v) = snapshot.get(&phi.var_name) {
                phi.known_inputs.push((*cid, v));
            }
        }
        // from latch
        let value_after = ops
            .get_variable_at_block(&phi.var_name, latch_id)  // ← ⚠️ latchから値取得
            .ok_or_else(|| format!("Variable {} not found at latch block", phi.var_name))?;
        phi.known_inputs.push((latch_id, value_after));  // ← latchの値を追加

        ops.debug_verify_phi_inputs(block_id, &phi.known_inputs);
        ops.emit_phi_at_block_start(block_id, phi.phi_id, phi.known_inputs)?;  // ← PHI完成
        ops.update_var(phi.var_name.clone(), phi.phi_id);  // ← 変数を最終PHI結果で更新
    }
    Ok(())
}
```

**証拠**:
- **Line 112-114**: `get_variable_at_block` でlatchからの値を取得
- **Line 115**: latchの値をPHI入力に追加
- **Line 118**: PHIを完成させる
- **Line 119**: 変数を最終PHI結果で更新

**問題**: latchでパラメータが存在しない/nullの場合、PHIは以下のようになる:
```
PHI v%10 = [preheader: v%0 (param), latch: v%null]
```

---

### 6. get_variable_at_block の動作 (mod.rs:210-219)

```rust
// src/mir/loop_builder/mod.rs:210-219
pub(super) fn get_variable_at_block(&self, name: &str, block_id: BasicBlockId) -> Option<ValueId> {
    // まずブロックごとのスナップショットを優先
    if let Some(map) = self.block_var_maps.get(&block_id) {
        if let Some(v) = map.get(name) {
            return Some(*v);
        }
    }
    // フォールバック：現在の変数マップ（単純ケース用）
    self.parent_builder.variable_map.get(name).copied()
}
```

**証拠**:
- **Line 212-216**: `block_var_maps` (latch snapshot) を優先
- **Line 218**: スナップショットにない場合は `parent_builder.variable_map` からフォールバック

**問題**: latchでパラメータが再定義されていない場合、`block_var_maps` に存在しない可能性がある。この場合、フォールバックが効くが、**latchのスナップショット作成時にパラメータが含まれているかは不明**。

---

## パラメータ保護が必要な箇所

### 📍 **修正箇所1: prepare_loop_variables_with (loop_phi.rs:127-145)**

**問題**: すべての変数にPHIを生成している。

**修正方針**:
```rust
pub fn prepare_loop_variables_with<O: LoopPhiOps>(
    ops: &mut O,
    _header_id: BasicBlockId,
    preheader_id: BasicBlockId,
    current_vars: &std::collections::HashMap<String, ValueId>,
) -> Result<Vec<IncompletePhi>, String> {
    let mut incomplete_phis: Vec<IncompletePhi> = Vec::new();
    for (var_name, &value_before) in current_vars.iter() {
        // ⚠️ 修正: パラメータをフィルタ
        if is_function_parameter(ops, var_name, value_before) {
            continue;  // パラメータはPHI対象外
        }

        let phi_id = ops.new_value();
        let inc = IncompletePhi {
            phi_id,
            var_name: var_name.clone(),
            known_inputs: vec![(preheader_id, value_before)],
        };
        incomplete_phis.push(inc);
        ops.update_var(var_name.clone(), phi_id);
    }
    Ok(incomplete_phis)
}
```

---

### 📍 **修正箇所2: LoopPhiOps トレイト拡張**

**問題**: パラメータ判定機能がない。

**修正方針**:
```rust
pub trait LoopPhiOps {
    fn new_value(&mut self) -> ValueId;
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String>;
    fn update_var(&mut self, name: String, value: ValueId);
    fn get_variable_at_block(&mut self, name: &str, block: BasicBlockId) -> Option<ValueId>;
    fn debug_verify_phi_inputs(&mut self, _merge_bb: BasicBlockId, _inputs: &[(BasicBlockId, ValueId)]) {}

    // ⚠️ 追加: パラメータ判定機能
    fn is_function_parameter(&self, name: &str, value: ValueId) -> bool;
}
```

---

### 📍 **修正箇所3: LoopBuilder でのトレイト実装**

**問題**: パラメータ判定機能の実装が必要。

**修正方針**:
```rust
impl crate::mir::phi_core::loop_phi::LoopPhiOps for LoopBuilder<'_> {
    fn new_value(&mut self) -> ValueId { self.new_value() }
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        self.emit_phi_at_block_start(block, dst, inputs)
    }
    fn update_var(&mut self, name: String, value: ValueId) { self.update_variable(name, value) }
    fn get_variable_at_block(&mut self, name: &str, block: BasicBlockId) -> Option<ValueId> {
        LoopBuilder::get_variable_at_block(self, name, block)
    }
    fn debug_verify_phi_inputs(&mut self, merge_bb: BasicBlockId, inputs: &[(BasicBlockId, ValueId)]) {
        if let Some(ref func) = self.parent_builder.current_function {
            crate::mir::phi_core::common::debug_verify_phi_inputs(func, merge_bb, inputs);
        }
    }

    // ⚠️ 追加実装
    fn is_function_parameter(&self, _name: &str, value: ValueId) -> bool {
        if let Some(ref func) = self.parent_builder.current_function {
            func.params.contains(&value)
        } else {
            false
        }
    }
}
```

---

## パラメータ判定の実装戦略

### 戦略A: ValueId ベース判定（推奨）

**方法**: 関数パラメータリスト (`func.params`) に含まれるValueIdをチェック

**利点**:
- 正確（パラメータとして登録されたValueIdのみ対象）
- 高速（HashSetルックアップ）
- 変数名に依存しない（`me` の特別扱い不要）

**実装**:
```rust
fn is_function_parameter(&self, _name: &str, value: ValueId) -> bool {
    if let Some(ref func) = self.parent_builder.current_function {
        func.params.contains(&value)  // O(n) だがparamsは通常3-5個
    } else {
        false
    }
}
```

---

### 戦略B: 変数名ベース判定（非推奨）

**方法**: 変数名が `"me"` や関数シグネチャのパラメータ名に一致するかチェック

**欠点**:
- 不正確（同名のローカル変数があると誤判定）
- 変数名リネームに弱い
- `me` の特別扱いが必要

---

## 期待される修正効果

### Before（現状）:
```
preheader:
  me: v%0 (param)
  json_text: v%1 (param)
  path: v%2 (param)

header (after prepare_loop_variables):
  me: v%10 (PHI: preheader=v%0, latch=?)  ← ⚠️ パラメータが上書き
  json_text: v%11 (PHI: preheader=v%1, latch=?)
  path: v%12 (PHI: preheader=v%2, latch=?)

header (after seal_block):
  me: v%10 (PHI: [preheader=v%0, latch=v%null])  ← ⚠️ null混入
  json_text: v%11 (PHI: [preheader=v%1, latch=v%null])
  path: v%12 (PHI: [preheader=v%2, latch=v%null])
```

### After（修正後）:
```
preheader:
  me: v%0 (param)
  json_text: v%1 (param)
  path: v%2 (param)

header (after prepare_loop_variables):
  me: v%0 (param)  ← ✅ そのまま保持（PHI対象外）
  json_text: v%1 (param)
  path: v%2 (param)
  local_var: v%10 (PHI: preheader=v%X, latch=?)  ← ローカル変数のみPHI

header (after seal_block):
  me: v%0 (param)  ← ✅ 変わらず
  json_text: v%1 (param)
  path: v%2 (param)
  local_var: v%10 (PHI: [preheader=v%X, latch=v%Y])
```

---

## 追加調査が必要な項目

### 1. latch でのスナップショット保存 (build.rs:139-146)

```rust
// src/mir/loop_builder/build.rs:139-146
let latch_id = self.current_block()?;
let latch_snapshot = self.get_current_variable_map();  // ← ⚠️ パラメータ含む？
crate::mir::phi_core::loop_phi::save_block_snapshot(
    &mut self.block_var_maps,
    latch_id,
    &latch_snapshot,
);
```

**疑問**: `latch_snapshot` にパラメータが含まれているか？
- 含まれている場合: `seal_block` 時に正しいパラメータ値が取得される
- 含まれていない場合: `get_variable_at_block` のフォールバックが効く可能性

**調査方法**: `NYASH_LOOP_TRACE=1` でスナップショット内容を出力

---

### 2. continue文でのスナップショット (control_flow.rs:33, 46)

```rust
// src/mir/loop_builder/control_flow.rs:33
let snapshot = self.get_current_variable_map();
self.continue_snapshots.push((current_block, snapshot));
```

**疑問**: continue時のスナップショットにパラメータが含まれているか？

---

## 結論

### ❌ **パラメータ保護は現在存在しない**

**証拠まとめ**:
1. `prepare_loop_variables` はすべての変数を処理（Line: phi.rs:19, loop_phi.rs:134）
2. `variable_map` にパラメータが含まれる（Line: lowering.rs:47, 51）
3. パラメータフィルタ機構は存在しない
4. すべての変数に不完全PHIが生成される（Line: loop_phi.rs:135-142）

### ✅ **修正方針**

**戦略**: ValueIdベースのパラメータ判定を `prepare_loop_variables_with` に追加

**修正箇所**:
- `/home/tomoaki/git/hakorune-selfhost/src/mir/phi_core/loop_phi.rs:127-145` - パラメータフィルタ追加
- `/home/tomoaki/git/hakorune-selfhost/src/mir/phi_core/loop_phi.rs:32-43` - トレイト拡張
- `/home/tomoaki/git/hakorune-selfhost/src/mir/loop_builder/mod.rs:254-275` - トレイト実装

**影響範囲**: LoopBuilder のみ（IF-PHI システムには影響なし）

---

## 次のステップ

1. **Task 4**: パラメータ保護機能の実装
2. **Task 5**: スモークテスト実行・検証
3. **Task 6**: latch/continue スナップショット内容の詳細調査（必要に応じて）

---

**レポート作成者**: Claude (Task Agent 3)
**調査完了時刻**: 2025-10-16
