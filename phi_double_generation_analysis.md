# PHI二重生成問題の根本原因分析

**発見日**: 2025-10-02
**影響**: LLVM IR構文エラー（空PHI生成）
**回避策**: `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`

---

## 🔍 **問題の症状**

### LLVM IR出力
```llvm
bb1:
  %"phi_2.1" = phi  i64              ← 空のPHI（incoming なし）
  %"phi_2" = phi  i64 [0, %"bb0"], [%"add_4", %"bb1"]  ← 正しいPHI
```

### エラーメッセージ
```
RuntimeError: LLVM IR parsing error
<string>:11:3: error: expected '[' in phi value list
  %"phi_2" = phi  i64 [0, %"bb0"], [%"add_4", %"bb1"]
  ^
```

**注**: `.1` サフィックスは llvmlite の自動リネーム（同名PHI衝突回避）

---

## 🎯 **根本原因：2つのPHIシステムが競合**

### システム1: PhiHandler（block_lower.py）
**担当**: MIR JSON の PHI命令を直接処理
**処理**: `PhiHandler.process_phi_instructions()` → 正しいPHI生成 + incoming設定

```python
# block_lower.py:161-176
phi_handler = PhiHandler(builder, verbose=phi_verbose)
phi_ops, non_phi_insts = phi_handler.collect_phi_instructions(insts)
phi_handler.process_phi_instructions(phi_ops, bb, func)  # ✅ 正しいPHI
```

### システム2: finalize_phis（phi_wiring/wiring.py）
**担当**: `block_phi_incomings` メタデータから PHI生成
**処理**: `ensure_phi()` → 空のPHI生成（既存PHIチェック不完全）

```python
# phi_wiring/wiring.py:16-39
def ensure_phi(builder, block_id, dst_vid, bb):
    # 既存PHIチェック（30-35行）
    cur = builder.vmap.get(dst_vid)
    if cur is not None and hasattr(cur, "add_incoming"):
        if getattr(getattr(cur, "basic_block", None), "name", None) == bb.name:
            return cur  # ✅ 同じブロックなら再利用

    # ❌ PhiHandler生成のPHIが vmap に登録されてない場合、空PHI生成
    ph = b.phi(builder.i64, name=f"phi_{dst_vid}")
    builder.vmap[dst_vid] = ph
    return ph
```

---

## 📋 **実行フロー（function_lower.py）**

```python
# 1. PHIメタデータ収集
181: _setup_phi_placeholders(builder, blocks)
     # MIR JSON PHI命令 → block_phi_incomings 登録

# 2. multi-pred自動検出PHI登録
212-266: # ⚠️ 無効化フラグなし！
         for bid, blk in block_by_id.items():
             preds_list = [...]  # multi-pred判定
             if len(preds_list) <= 1: continue

             defs = _collect_defs(blk)
             uses = _collect_uses(blk)
             need = [u for u in uses if u not in defs]

             # ❌ 使用値を自動登録（PHI命令がなくても登録される）
             for vid in need:
                 builder.block_phi_incomings.setdefault(bid, {})[vid] = [
                     (p, vid) for p in preds_list
                 ]

# 3. ブロック処理（PhiHandler）
279: _lower_blocks(builder, func, block_by_id, order, loop_plan)
     # ✅ PhiHandler が正しいPHI生成

# 4. PHI finalize（空PHI生成）
303: _finalize_phis(builder)
     # ❌ ensure_phi() が空のPHI生成
```

---

## ⚠️ **なぜ ensure_phi() のチェックが失敗するか**

### タイミング問題
```python
# block_lower.py:167-170
self.builder.vmap[dst] = phi
if hasattr(self.builder, '_current_vmap'):
    self.builder._current_vmap[dst] = phi  # ブロックローカル
```

**問題**: PhiHandler は `_current_vmap` に登録するが、ensure_phi() は `builder.vmap` をチェック
**結果**: グローバル vmap に登録されてない場合、ensure_phi() が検出できず空PHI生成

---

## ✅ **現在の回避策**

### NYASH_LLVM_SANITIZE_EMPTY_PHI=1
```python
# llvm_builder.py:673-683
if os.environ.get('NYASH_LLVM_SANITIZE_EMPTY_PHI') == '1':
    fixed_lines = []
    for line in ir_text.splitlines():
        if (" = phi  i64" in line or " = phi i64" in line) and ("[" not in line):
            continue  # 空PHIをスキップ
        fixed_lines.append(line)
    ir_text = "\n".join(fixed_lines)
```

**効果**: 空PHI削除 → LLVM IRパース成功 → WASM実行成功 ✅

---

## 🔧 **根本解決策（検討中）**

### 案1: finalize_phis() を無効化
```python
# function_lower.py:303
# _finalize_phis(builder)  # コメントアウト
```
**影響**: PhiHandler のみでPHI処理（block_lower経路）

### 案2: 自動検出PHI登録を無効化
```python
# function_lower.py:212-266
if os.environ.get('NYASH_LLVM_AUTO_PHI_DETECT') != '0':
    # multi-pred自動検出処理
    ...
```
**影響**: 明示的なPHI命令のみ処理

### 案3: ensure_phi() のチェック強化
```python
# phi_wiring/wiring.py:30-35
# グローバル vmap + _current_vmap の両方チェック
cur = builder.vmap.get(dst_vid)
if cur is None and hasattr(builder, '_current_vmap'):
    cur = builder._current_vmap.get(dst_vid)
```
**影響**: PhiHandler生成のPHIを正しく検出

---

## 📊 **ChatGPT推奨環境変数の効果**

```bash
NYASH_LLVM_PREPASS_IFMERGE=0  # ✅ 実装済み（function_lower.py:185）
NYASH_LLVM_PREPASS_LOOP=0     # ✅ 実装済み（function_lower.py:271）
```

**効果**: if-merge plan と loop plan を無効化
**問題**: 212-266行の「自動検出PHI登録」は無効化されない
**結果**: 空PHI生成は継続

---

## 🎉 **動作確認済み**

### ループPHI WASM実行
```bash
export NYASH_LLVM_USE_HARNESS=1
export NYASH_LLVM_SANITIZE_EMPTY_PHI=1
export NYASH_LLVM_PREPASS_IFMERGE=0
export NYASH_LLVM_PREPASS_LOOP=0

python3 llvm_builder.py test_phi_loop.json --target wasm32 -o /tmp/test.wasm
python3 tools/wasm_add_export.py /tmp/test.wasm /tmp/test_ex.wasm ny_main
node tools/wasm_runner.js /tmp/test_ex.wasm

# 出力: ✅ ny_main() returned: 10
```

**成功**: WASM実行成功、ループPHI動作確認 ✅

---

## 📋 **次のステップ**

1. 根本解決策の選択（案1/2/3）
2. テストケース作成（if-PHI/loop-PHI）
3. ドキュメント更新（CURRENT_TASK_WASM.md）
4. コミット・PR作成

---

**更新日**: 2025-10-02
**担当**: Claude Code + ユーザー協働
**参照**: ChatGPT推奨環境変数セット
