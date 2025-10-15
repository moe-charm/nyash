# JsonCanonicalBox ArrayBox(1) バグ調査レポート

**日付**: 2025-10-16
**ステータス**: 調査中
**優先度**: P0 (Phase 20.5 Gate A)

## 問題の概要

`JsonCanonicalBox.canonicalize(j)` の呼び出し時に、String引数が `ArrayBox(1)` に変換される問題。

### 症状

```bash
[MAIN-DEBUG] Before call: {"b":1,"a":2}  ← Main: String型
[DEBUG] JsonCanonicalBox.canonicalize input value: ArrayBox(1)  ← JsonCanonicalBox内: ArrayBox！
[DEBUG] normalized: ArrayBox(1)
[TRAMPOLINE] extern_invoke: target=nyash.json.canonicalize_h forwarded_len=1 forwarded[0]=Some(String("ArrayBox(1)"))
```

**期待**: String `"{\"b\":1,\"a\":2}"` が渡される
**実際**: `ArrayBox(1)` が渡される

### 影響範囲

- `tools/smokes/v2/profiles/quick-selfhost/json_canonical_box_vm.sh` → SKIP
- `tools/smokes/v2/profiles/quick-selfhost/mirio_canonicalize_vm.sh` → SKIP

---

## 調査プロセス

### Phase 1: Task Agent 4人体制での層別調査

#### 1️⃣ ModuleFunctionCall 引数処理調査
- **調査箇所**: `src/backend/mir_interpreter/handlers/calls/function.rs:606-608`
- **発見**: `materialize_args_in_current_block` → `reg_load` の経路
- **結果**: 正常動作（ValueId → VMValue変換のみ）

#### 2️⃣ GlobalCall 引数処理調査
- **調査箇所**: `src/backend/mir_interpreter/handlers/calls/function.rs:141-166`
- **発見**: GlobalCall も ModuleFunctionCall も最終的に同じ経路
- **結果**: 正常動作

#### 3️⃣ MIR生成時の引数ラップ調査
- **調査箇所**: `src/mir/builder/builder_calls/build.rs`
- **発見**: 引数は `Vec<ValueId>` のまま、ArrayBoxにラップされない
- **結果**: 正常動作

#### 4️⃣ using resolver 関数呼び出し変換調査
- **調査箇所**: `src/runner/modes/common_util/resolve/alias_tools.rs`
- **発見**: Alias変換は正常（`JsonCanonicalBox` → `JsonCanonicalBox_JsonCanonicalBox`）
- **結果**: 正常動作

### Phase 2: 根本原因追跡

#### `materialize_args_in_current_block` 調査
- **ファイル**: `src/backend/mir_interpreter/helpers/materialize.rs:41-53`
- **実装**: ValueId → ValueId の変換のみ（VMValue変換なし）
- **結果**: 問題なし

#### `reg_load` 実装確認
- **ファイル**: `src/backend/mir_interpreter/helpers/eval.rs:147-176`
- **実装**: 単純にレジスタから値を取得するだけ
- **結果**: 問題なし

```rust
pub(in crate::backend::mir_interpreter) fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
    match self.regs.get(&id).cloned() {
        Some(v) => Ok(v),  // ← レジスタから値を取得するだけ
        None => { ... }
    }
}
```

### Phase 3: デバッグログ追加

#### ModuleFunctionCall 引数トレース
- **追加箇所**: `function.rs:607-624`
- **環境変数**: `HAKO_DEBUG_MODULE_FN_ARGS=1`
- **結果**: ログが出力されない → ModuleFunctionCall として呼ばれていない！

---

## 確定した事実

### ✅ 正常動作している箇所

| 箇所 | 確認方法 | 結果 |
|------|----------|------|
| MIR生成 | Task 3 調査 | 引数は `Vec<ValueId>` のまま |
| using resolver | Task 4 調査 | Alias変換も正常 |
| `materialize_args_in_current_block` | 実装確認 | ValueId → ValueId 変換のみ |
| `reg_load` | 実装確認 | レジスタから値を取得するだけ |

### ❌ 問題箇所

**JsonCanonicalBox.canonicalize の `json` パラメータが既に ArrayBox(1) になっている**

```
[MAIN-DEBUG] Before call: {"b":1,"a":2}  ← Main: String
[DEBUG] JsonCanonicalBox.canonicalize input value: ArrayBox(1)  ← JsonCanonicalBox内: ArrayBox
```

---

## 推定原因

### 仮説 1: static box のメソッド呼び出しが特殊な経路を通っている

- `[MODULE-FN-ARGS]` ログが出力されない
- ModuleFunctionCall として呼ばれていない可能性
- 別の経路（GlobalCall? HostBridge?）を通っている

### 仮説 2: Hakoruneコンパイラ（selfhost）が生成するMIRで引数がArrayBoxにラップされている

- Hakoruneで書かれたコンパイラが間違ったMIRを生成
- `JsonCanonicalBox.canonicalize(j)` が `JsonCanonicalBox.canonicalize([j])` のような形に変換されている

### 仮説 3: static box の引数渡しの実装バグ

- static box のメソッド呼び出しで引数が自動的にArrayBoxにラップされる仕様
- または、実装ミスで引数がArrayBoxにラップされている

---

## 次のステップ

### 1. JsonCanonicalBox の birth() 確認
- `selfhost/shared/json/json_canonical_box.hako` の実装確認
- static box の初期化処理を確認

### 2. Main.main のMIR確認
- `JsonCanonicalBox.canonicalize(j)` の呼び出しがどのようなMIRに変換されているか
- 引数がArrayBoxにラップされているか確認

### 3. static box のメソッド呼び出し経路確認
- `function.rs` の `handle_callee_module_function` 以外の経路を確認
- GlobalCall / HostBridge / BoxCall のいずれかを通っているか

---

## 修正案候補

### Option A: JsonCanonicalBox 側で ArrayBox を展開 (簡易修正)

```hakorune
canonicalize(json) {
    local normalized = "" + json
    // ← ここで json が ArrayBox の場合は展開
    if normalized.get != null {
        normalized = normalized.get(0)
    }
    normalized = "" + normalized
    args.push(normalized)
    // ...
}
```

**メリット**: 影響範囲が小さい
**デメリット**: 根本原因を解決していない

### Option B: 呼び出し側を修正 (根本修正)

MIR生成またはVM実行時の引数処理を修正

**メリット**: 根本原因を解決
**デメリット**: 影響範囲が大きい、調査が必要

---

## 関連ファイル

### 調査済みファイル
- `src/backend/mir_interpreter/handlers/calls/function.rs` (ModuleFunctionCall)
- `src/backend/mir_interpreter/handlers/calls/legacy/extern_handler.rs` (extern_invoke)
- `src/backend/mir_interpreter/extern_adapter/extern_env.rs` (nyash.json.canonicalize_h)
- `src/backend/mir_interpreter/helpers/materialize.rs` (materialize)
- `src/backend/mir_interpreter/helpers/eval.rs` (reg_load)
- `selfhost/shared/json/json_canonical_box.hako` (JsonCanonicalBox)

### 未調査ファイル
- `src/backend/mir_interpreter/handlers/calls/method.rs` (BoxCall)
- `src/backend/mir_interpreter/handlers/calls/legacy/mod.rs` (handle_call)
- `src/mir/builder/` (MIR生成)

---

## 失敗・問題点の記録

### ❌ 失敗 1: extern_handler.rs の修正は効果なし

**実施内容**: `extern_handler.rs:183-186` で ArrayBox要素を直接 String に変換する修正を追加

**結果**: 引数が既に "ArrayBox(1)" という文字列になっているため、効果なし

**学び**: 問題は extern_handler より前の段階で発生している

### ❌ 失敗 2: MIR dump ができない

**実施内容**: `--emit-mir-json` でMIRをダンプしようとした

**結果**: using文があるとパースエラーで失敗

**学び**: using文のあるコードではMIR出力機能が使えない（バグ？）

### ❌ 失敗 3: デバッグログが出力されない

**実施内容**: `HAKO_DEBUG_MODULE_FN_ARGS=1` でModuleFunctionCallの引数をトレース

**結果**: ログが全く出力されない

**学び**: JsonCanonicalBox.canonicalize は ModuleFunctionCall として呼ばれていない

---

## 参考情報

### デバッグコマンド

```bash
# 基本実行
HAKO_JSON_CANON=1 ./target/release/hakorune --backend vm test.hako

# デバッグログ有効化
HAKO_DEBUG_TRAMPOLINE=1 HAKO_DEBUG_MODULE_FN_ARGS=1 ./target/release/hakorune --backend vm test.hako

# スモークテスト
HAKO_JSON_CANON=1 tools/smokes/v2/profiles/quick-selfhost/json_canonical_box_vm.sh
```

### テストコード

```hakorune
using "selfhost/shared/json/json_canonical_box.hako" as JsonCanonicalBox

static box Main {
  main(args) {
    local j = "{\"b\":1,\"a\":2}"
    print("[MAIN-DEBUG] Before call: " + j)
    local out = JsonCanonicalBox.canonicalize(j)
    print("[MAIN-DEBUG] After call: " + out)
    return 0
  }
}
```

---

## ✅ **根本原因確定**

### 🎯 **問題箇所**

**ファイル**: `src/backend/mir_interpreter/handlers/boxes/legacy/mod.rs`
**行**: Line 235-236 (修正前)

```rust
// Build argv: pass receiver as first arg (me)
let recv_vm = self.reg_load(box_val)?;
let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
argv.push(recv_vm);  // ← 無条件にレシーバーを先頭に追加！
for a in args { argv.push(self.reg_load(*a)?); }
```

### 🐛 **問題の流れ**

1. `JsonCanonicalBox.canonicalize(j)` を呼び出し
2. `box_val` = JsonCanonicalBox インスタンス、`args` = `[j]`
3. **Line 235**: `argv[0]` = JsonCanonicalBox インスタンス（レシーバー）
4. **Line 236**: `argv[1]` = `j`（JSON文字列）
5. `canonicalize(json)` は引数1個を期待 → `argv[0]` を `json` として受け取る
6. `argv[0]` は JsonCanonicalBox インスタンス
7. インスタンスが "ArrayBox(1)" と表示される

### ✅ **修正内容**

**修正後のコード** (Line 257-266):

```rust
// Build argv: decide whether to pass receiver as first arg
// - If func.params.len() == args.len(): static box method, don't pass receiver
// - If func.params.len() == args.len() + 1: instance method with 'me', pass receiver
let recv_vm = self.reg_load(box_val)?;
let pass_receiver = func.params.len() == args.len() + 1;
let mut argv: Vec<VMValue> = Vec::with_capacity(if pass_receiver { 1 + args.len() } else { args.len() });
if pass_receiver {
    argv.push(recv_vm.clone());  // ← instance メソッドの場合のみ追加
}
for a in args { argv.push(self.reg_load(*a)?); }
```

**判定ロジック**:
- `func.params.len() == args.len()` → **static box メソッド**、レシーバーを含めない
- `func.params.len() == args.len() + 1` → **instance メソッド**、レシーバーを含める

### 🎯 **修正後の動作**

1. `JsonCanonicalBox.canonicalize(j)` を呼び出し
2. `func.params.len() == 1`, `args.len() == 1` → `pass_receiver = false`
3. **新コード**: `argv = [j]`（レシーバーを含めない）
4. `canonicalize(json)` が正しく `j` を受け取る
5. 正しい出力: `{"a":2,"b":1}` ✅

---

## ステータス

**現在の状態**: ✅ **修正完了**

**完了したタスク**:
- ✅ Task Agent 4人体制での層別調査
- ✅ materialize_args_in_current_block 調査
- ✅ reg_load 実装確認
- ✅ デバッグログ追加
- ✅ static box のメソッド呼び出し経路特定
- ✅ 根本原因の特定
- ✅ 修正案実装

**検証ブロック**:
- ⚠️ スモークテスト実行が `MirIoBox.validate/1` の欠落でブロック中
- ⚠️ セルフホストコンパイラの依存関係問題

**次のステップ**:
- `MirIoBox.validate/1` の実装または依存関係の解決
- スモークテスト PASS 確認

---

## 更新履歴

- **2025-10-16 (19:00)**: 根本原因確定、修正完了
- **2025-10-16 (12:00)**: 初版作成（Task Agent 4人体制調査完了）
