# Pipeline v2 重複コード詳細比較

**分析日**: 2025-10-12
**補足資料**: pipeline_v2_boxification_analysis.md の詳細版

---

## 📋 重複パターン完全マップ

### 🔴 重複度: **極めて高い（85%以上）**

---

## 1️⃣ Extract系の完全重複（最優先）

### ファイル比較

| ファイル | 行数 | 重複行数 | 重複率 |
|---------|-----|---------|-------|
| call_extract_box.hako | 54 | 46 | 85% |
| method_extract_box.hako | 51 | 43 | 84% |
| new_extract_box.hako | 51 | 43 | 84% |

### 行単位比較表

| 行番号範囲 | call_extract | method_extract | new_extract | 差異 |
|----------|-------------|---------------|-------------|-----|
| 1-7 | 検索パターン: `"type":"Call"` | 検索パターン: `"type":"Method"` | 検索パターン: `"type":"New"` | **3行のみ異なる** |
| 8-18 | ラベル抽出: `"name":"` (8文字) | ラベル抽出: `"method":"` (10文字) | ラベル抽出: `"class":"` (9文字) | **オフセット値のみ異なる** |
| 19-48 | **完全一致** | **完全一致** | **完全一致** | **30行完全重複** |
| 49-51 | 返却: `{ name: ..., args: ... }` | 返却: `{ method: ..., args: ... }` | 返却: `{ class: ..., args: ... }` | **キー名のみ異なる** |

---

### 重複コード詳細（30行完全一致部分）

#### 🔴 ブロック1: args配列検索（7行完全一致）
```hako
// ファイル: call_extract_box.hako:19-25
// ファイル: method_extract_box.hako:19-25
// ファイル: new_extract_box.hako:19-25
local ak = RegexFlow.find_from(ast_json, "\"args\":[", q)
local vals = []  // ← 変数名だけ call: vals, method: args, new: args
if ak >= 0 {
  // bracket-aware end
  local lb = RegexFlow.find_from(ast_json, "[", ak)
  local rb = ast_json.size()
  if lb >= 0 {
```

#### 🔴 ブロック2: 括弧深度追跡（10行完全一致）
```hako
// ファイル: call_extract_box.hako:26-35
// ファイル: method_extract_box.hako:25-34
// ファイル: new_extract_box.hako:24-33
    local i2 = lb + 1
    local depth = 1
    loop(true) {
      local ch = ast_json.substring(i2, i2+1)
      if ch == "" { break }
      if ch == "[" { depth = depth + 1 } else { if ch == "]" { depth = depth - 1 } }
      if depth == 0 { rb = i2  break }
      i2 = i2 + 1
    }
  }
```

#### 🔴 ブロック3: Int値抽出（13行完全一致）
```hako
// ファイル: call_extract_box.hako:37-48
// ファイル: method_extract_box.hako:35-45
// ファイル: new_extract_box.hako:35-45
  // scan ints within ak..rb
  local i = ak
  loop(true) {
    local tpos = RegexFlow.find_from(ast_json, "\"type\":\"Int\"", i)
    if tpos < 0 || tpos >= rb { break }
    local vpos = RegexFlow.find_from(ast_json, "\"value\":", tpos)
    if vpos < 0 || vpos >= rb { i = tpos + 1  continue }
    local ds = RegexFlow.digits_from(ast_json, vpos + 8)
    if ds != "" { vals.push(RegexFlow.to_int(ds)) }  // ← vals/args
    i = vpos + 8 + ds.size()
  }
}
```

---

### 統合後のコード（提案）

#### 新規Box: `Stage1IntArgsExtractBox`
```hako
// stage1_int_args_extract_box.hako
using "selfhost/compiler/pipeline_v2/regex_flow.hako" as RegexFlow

static box Stage1IntArgsExtractBox {
  // 汎用: JSON args配列からInt値リストを抽出
  // Returns: [Int,...] or []
  extract_int_args(ast_json, start_pos) {
    if ast_json == null { return [] }
    local s = "" + ast_json
    local ak = RegexFlow.find_from(s, "\"args\":[", start_pos)
    if ak < 0 { return [] }

    // bracket-aware end position
    local rb = me.find_bracket_end(s, ak)

    // scan all Int values within [ak, rb)
    local vals = []
    local i = ak
    loop(true) {
      local tpos = RegexFlow.find_from(s, "\"type\":\"Int\"", i)
      if tpos < 0 || tpos >= rb { break }
      local vpos = RegexFlow.find_from(s, "\"value\":", tpos)
      if vpos < 0 || vpos >= rb { i = tpos + 1  continue }
      local ds = RegexFlow.digits_from(s, vpos + 8)
      if ds != "" { vals.push(RegexFlow.to_int(ds)) }
      i = vpos + 8 + ds.size()
    }
    return vals
  }

  // ヘルパー: 括弧の終端位置検出（深度追跡）
  find_bracket_end(s, start_pos) {
    local lb = RegexFlow.find_from(s, "[", start_pos)
    local rb = s.size()
    if lb >= 0 {
      local i = lb + 1
      local depth = 1
      loop(true) {
        local ch = s.substring(i, i+1)
        if ch == "" { break }
        if ch == "[" { depth = depth + 1 } else { if ch == "]" { depth = depth - 1 } }
        if depth == 0 { rb = i  break }
        i = i + 1
      }
    }
    return rb
  }
}

static box Stage1IntArgsExtractStub { main(args) { return 0 } }
```

#### 統合後: `call_extract_box.hako`（54行 → 24行）
```hako
// CallExtractBox — Stage‑1 JSON から Return(Call name(args...)) を抽出
using "selfhost/compiler/pipeline_v2/regex_flow.hako" as RegexFlow
using "selfhost/compiler/pipeline_v2/stage1_int_args_extract_box.hako" as IntArgsExtract

static box CallExtractBox {
  // Returns { name: String, args: [Int,...] } or null
  extract_return_call_ints(ast_json) {
    if ast_json == null { return null }
    // Return → Call
    local rq = RegexFlow.find_from(ast_json, "\"type\":\"Return\"", 0)
    if rq < 0 { return null }
    local q = RegexFlow.find_from(ast_json, "\"type\":\"Call\"", rq)
    if q < 0 { return null }
    // name
    local nk = RegexFlow.find_from(ast_json, "\"name\":\"", q)
    if nk < 0 { return null }
    local nk_end = RegexFlow.find_from(ast_json, "\"", nk + 8)
    if nk_end < 0 { return null }
    local name = ast_json.substring(nk + 8, nk_end)
    // args via shared extractor ⭐ここだけ変更
    local vals = IntArgsExtract.extract_int_args(ast_json, q)
    return { name: name, args: vals }
  }
}

static box CallExtractStub { main(args) { return 0 } }
```

#### 統合後: `method_extract_box.hako`（51行 → 22行）
```hako
// MethodExtractBox — Stage‑1 JSON から Return(Method ...) を抽出
using "selfhost/compiler/pipeline_v2/regex_flow.hako" as RegexFlow
using "selfhost/compiler/pipeline_v2/stage1_int_args_extract_box.hako" as IntArgsExtract

static box MethodExtractBox {
  extract_return_method_ints(ast_json) {
    if ast_json == null { return null }
    local rq = RegexFlow.find_from(ast_json, "\"type\":\"Return\"", 0)
    if rq < 0 { return null }
    local q = RegexFlow.find_from(ast_json, "\"type\":\"Method\"", rq)
    if q < 0 { return null }
    // method name (offset 10 for "method":")
    local mk = RegexFlow.find_from(ast_json, "\"method\":\"", q)
    if mk < 0 { return null }
    local mk_end = RegexFlow.find_from(ast_json, "\"", mk + 10)
    if mk_end < 0 { return null }
    local mname = ast_json.substring(mk + 10, mk_end)
    // args via shared extractor ⭐
    local args = IntArgsExtract.extract_int_args(ast_json, q)
    return { method: mname, args: args }
  }
}

static box MethodExtractStub { main(args) { return 0 } }
```

#### 統合後: `new_extract_box.hako`（51行 → 22行）
```hako
// NewExtractBox — Stage‑1 JSON から Return(New ...) を抽出
using "selfhost/compiler/pipeline_v2/regex_flow.hako" as RegexFlow
using "selfhost/compiler/pipeline_v2/stage1_int_args_extract_box.hako" as IntArgsExtract

static box NewExtractBox {
  extract_return_new_ints(ast_json) {
    if ast_json == null { return null }
    local rq = RegexFlow.find_from(ast_json, "\"type\":\"Return\"", 0)
    if rq < 0 { return null }
    local q = RegexFlow.find_from(ast_json, "\"type\":\"New\"", rq)
    if q < 0 { return null }
    // class name (offset 9 for "class":")
    local ck = RegexFlow.find_from(ast_json, "\"class\":\"", q)
    if ck < 0 { return null }
    local ck_end = RegexFlow.find_from(ast_json, "\"", ck + 9)
    if ck_end < 0 { return null }
    local cname = ast_json.substring(ck + 9, ck_end)
    // args via shared extractor ⭐
    local args = IntArgsExtract.extract_int_args(ast_json, q)
    return { class: cname, args: args }
  }
}

static box NewExtractStub { main(args) { return 0 } }
```

### 削減効果まとめ

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| **ファイル数** | 3 | 4 (共通Box追加) | +1 |
| **総行数** | 156 | 94 | **-62行 (-40%)** |
| **call_extract** | 54 | 24 | -30行 |
| **method_extract** | 51 | 22 | -29行 |
| **new_extract** | 51 | 22 | -29行 |
| **新規: int_args_extract** | 0 | 52 | +52行 |
| **純削減** | - | - | **-62行** ⭐ |

---

## 2️⃣ Normalizer系の重複

### ファイル: `normalizer_box.hako`

#### 重複箇所比較表

| メソッド | 開始行 | 終了行 | 行数 | 重複内容 |
|---------|-------|-------|-----|---------|
| normalize_call_ints | 32 | 51 | 20 | 引数配列処理 |
| normalize_method_ints | 54 | 72 | 19 | 引数配列処理（完全一致） |
| normalize_new_ints | 75 | 93 | 19 | 引数配列処理（完全一致） |

#### 🔴 重複コード（16行完全一致）
```hako
// normalizer_box.hako:39-48 (normalize_call_ints内)
// normalizer_box.hako:60-69 (normalize_method_ints内)
// normalizer_box.hako:82-91 (normalize_new_ints内)

local arr = new ArrayBox()
local src = raw.get("args")
if src != null && src.size != null {
  local n = src.size()
  local i = 0
  loop (i < n) {
    arr.push(me._to_i64(src.get(i)))
    i = i + 1
  }
}
out.set("args", arr)
```

### 統合後のコード（提案）

```hako
// normalizer_box.hako (96行 → 74行)
using "selfhost/shared/common/string_helpers.hako" as StringHelpers

static box NormalizerBox {
  _to_i64(v) { return StringHelpers.to_i64(v) }
  _to_string(v) {
    if v == null { return "" }
    return "" + v
  }

  // ⭐ 新規: 汎用引数配列正規化
  _normalize_int_array(src) {
    local arr = new ArrayBox()
    if src != null && src.size != null {
      local n = src.size()
      local i = 0
      loop (i < n) {
        arr.push(me._to_i64(src.get(i)))
        i = i + 1
      }
    }
    return arr
  }

  // ⭐ 新規: 汎用label+args正規化
  _normalize_with_label_and_args(raw, label_key) {
    if raw == null { return null }
    local out = new MapBox()
    local label = me._to_string(raw.get(label_key))
    if label == null || label == "" { return null }
    out.set(label_key, label)
    out.set("args", me._normalize_int_array(raw.get("args")))
    return out
  }

  // 既存: cmp専用（引数なし）
  normalize_cmp(raw) {
    if raw == null { return null }
    local out = new MapBox()
    local cmp = "" + raw.get("cmp")
    if cmp == null || cmp == "" { return null }
    out.set("cmp", cmp)
    local lhs = me._to_i64(raw.get("lhs"))
    local rhs = me._to_i64(raw.get("rhs"))
    out.set("lhs", lhs)
    out.set("rhs", rhs)
    return out
  }

  // ⭐ 簡略化: 薄いラッパー
  normalize_call_ints(raw) {
    return me._normalize_with_label_and_args(raw, "name")
  }

  normalize_method_ints(raw) {
    return me._normalize_with_label_and_args(raw, "method")
  }

  normalize_new_ints(raw) {
    return me._normalize_with_label_and_args(raw, "class")
  }
}

static box NormalizerStub { main(args) { return 0 } }
```

### 削減効果

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| **総行数** | 96 | 74 | **-22行 (-23%)** |
| normalize_call_ints | 20行 | 3行 | -17行 |
| normalize_method_ints | 19行 | 3行 | -16行 |
| normalize_new_ints | 19行 | 3行 | -16行 |
| 新規ヘルパー | 0 | 27行 | +27行 |
| **純削減** | - | - | **-22行** ⭐ |

---

## 3️⃣ Emit系の重複

### ファイル比較

| ファイル | 行数 | 重複行数 | 重複箇所 |
|---------|-----|---------|---------|
| emit_call_box.hako | 56 | 12 | L18-21, L39-43 |
| emit_method_box.hako | 54 | 12 | L18-20, L38-41 |
| emit_newbox_box.hako | 54 | 12 | L18-21, L38-41 |

#### 🔴 重複コード1: 引数materialize（v0版）
```hako
// emit_call_box.hako:18-21
// emit_method_box.hako:18-20 (receiverが+1オフセット)
// emit_newbox_box.hako:18-21

local insts = []
local vals = Stage1ArgsParserBox.parse_ints(args)
local n = 0
{ local i = 0  local m = 0  if vals != null && vals.size != null { m = vals.size() }  loop(i < m) { insts.push(MirEmitBox.make_const((i + 1), vals.get(i)))  i = i + 1 }  n = m }
```

#### 🔴 重複コード2: arg_ids配列構築
```hako
// emit_call_box.hako:22-25
// emit_method_box.hako:22-24
// emit_newbox_box.hako:23-25

local arg_ids = []
local i = 0
loop (i < n) { arg_ids.push(i + 1)  i = i + 1 }
```

### 統合後のコード（提案）

#### 新規Box: `ArgsConstEmitBox`
```hako
// args_const_emit_box.hako
using "selfhost/compiler/pipeline_v2/stage1_args_parser_box.hako" as ArgsParser
using "apps/selfhost-compiler/common/mir_emit_box.hako" as MirEmitBox

static box ArgsConstEmitBox {
  // 引数値を連番レジスタにmaterialize
  // Returns: { insts: [MapBox...], count: Int, arg_ids: [Int...] }
  materialize_int_args(args, start_reg) {
    if start_reg == null { start_reg = 1 }
    local insts = []
    local vals = ArgsParser.parse_ints(args)
    local n = 0
    if vals != null && vals.size != null {
      n = vals.size()
      local i = 0
      loop(i < n) {
        insts.push(MirEmitBox.make_const(start_reg + i, vals.get(i)))
        i = i + 1
      }
    }
    // Build arg_ids array
    local arg_ids = new ArrayBox()
    local k = 0
    loop(k < n) {
      arg_ids.push(start_reg + k)
      k = k + 1
    }
    return { insts: insts, count: n, arg_ids: arg_ids }
  }
}

static box ArgsConstEmitStub { main(args) { return 0 } }
```

#### 統合後: `emit_call_box.hako`（56行 → 43行）
```hako
// EmitCallBox — Return(Call name(int_args...)) を MIR(JSON v0) に変換
using "apps/selfhost-compiler/common/json_emit_box.hako" as JsonEmitBox
using "apps/selfhost-compiler/common/mir_emit_box.hako" as MirEmitBox
using "apps/selfhost-compiler/common/call_emit_box.hako" as CallEmitBox
using "apps/selfhost-compiler/common/header_emit_box.hako" as HeaderEmitBox
using "selfhost/compiler/pipeline_v2/args_const_emit_box.hako" as ArgsConstEmit

static box EmitCallBox {
  emit_call_int_args(name, args) {
    name = match name { null => "", _ => name }
    args = match args { null => [], _ => args }

    // ⭐ 共通Box利用
    local result = ArgsConstEmit.materialize_int_args(args, 1)
    local insts = result.get("insts")
    local n = result.get("count")
    local arg_ids = result.get("arg_ids")

    local dst_ret = n + 1
    insts.push(CallEmitBox.make_call(name, arg_ids, dst_ret))
    insts.push(MirEmitBox.make_ret(dst_ret))

    local blocks = [HeaderEmitBox.make_block(0, insts)]
    local fns = [HeaderEmitBox.make_function_main(blocks)]
    return JsonEmitBox.to_json(HeaderEmitBox.make_module_with_functions(fns))
  }

  // v1版も同様に簡略化
  emit_call_int_args_v1(name, args) {
    name = match name { null => "", _ => name }
    args = match args { null => [], _ => args }
    local result = ArgsConstEmit.materialize_int_args(args, 1)
    local insts = result.get("insts")
    local n = result.get("count")
    local dst = n + 1
    insts.push(CallEmitBox.make_mir_call_global(name, result.get("arg_ids"), dst))
    insts.push(MirEmitBox.make_ret(dst))
    local blocks = [HeaderEmitBox.make_block(0, insts)]
    local fns = [HeaderEmitBox.make_function_main(blocks)]
    return JsonEmitBox.to_json(HeaderEmitBox.make_module_with_functions(fns))
  }
}

static box EmitCallStub { main(args) { return 0 } }
```

#### 統合後: `emit_method_box.hako`（54行 → 41行）
```hako
// EmitMethodBox — Return(Method recv, method, args[int...]) → MIR(JSON v0)
using "apps/selfhost-compiler/common/mir_emit_box.hako" as MirEmitBox
using "apps/selfhost-compiler/common/call_emit_box.hako" as CallEmitBox
using "apps/selfhost-compiler/common/json_emit_box.hako" as JsonEmitBox
using "apps/selfhost-compiler/common/header_emit_box.hako" as HeaderEmitBox
using "selfhost/compiler/pipeline_v2/args_const_emit_box.hako" as ArgsConstEmit

static box EmitMethodBox {
  emit_method_int_args(method, recv_val, args) {
    method = match method { null => "", _ => method }
    args = match args { null => [], _ => args }

    // recv at r1
    local insts = [MirEmitBox.make_const(1, recv_val)]

    // ⭐ 共通Box利用（r2から開始）
    local result = ArgsConstEmit.materialize_int_args(args, 2)
    local arg_insts = result.get("insts")
    local n = result.get("count")
    local arg_ids = result.get("arg_ids")

    { local i = 0  loop(i < arg_insts.size()) { insts.push(arg_insts.get(i))  i = i + 1 } }

    local dst = n + 2
    insts.push(CallEmitBox.make_boxcall(method, 1, arg_ids, dst))
    insts.push(MirEmitBox.make_ret(dst))
    local blocks = [HeaderEmitBox.make_block(0, insts)]
    local fns = [HeaderEmitBox.make_function_main(blocks)]
    return JsonEmitBox.to_json(HeaderEmitBox.make_module_with_functions(fns))
  }

  emit_method_int_args_v1(method, recv_val, args) {
    // 同様に簡略化
  }
}

static box EmitMethodStub { main(args) { return 0 } }
```

### 削減効果まとめ

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| **ファイル数** | 3 | 4 | +1 |
| **総行数** | 164 | 137 | **-27行 (-16%)** |
| emit_call_box | 56 | 43 | -13行 |
| emit_method_box | 54 | 41 | -13行 |
| emit_newbox_box | 54 | 41 | -13行 |
| 新規: args_const_emit | 0 | 31 | +31行 |
| **純削減** | - | - | **-27行** ⭐ |

---

## 📊 総合削減効果

### Phase 1実装後の削減

| 施策 | 対象ファイル数 | 削減行数 | 削減率 |
|------|--------------|---------|-------|
| **Extract系統合** | 3 → 4 | -62行 | -40% |
| **Normalizer共通化** | 1 | -22行 | -23% |
| **Emit系統合** | 3 → 4 | -27行 | -16% |
| **合計** | 7 → 9 (+2) | **-111行** | **-30%** |

### 全体への影響

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| **pipeline_v2/総行数** | 2,840 | 2,729 | -111行 (-3.9%) |
| **対象7ファイル** | 316 | 205 | -111行 (-35%) |

---

## 🎯 実装チェックリスト

### Step 1: Extract系統合
- [ ] `stage1_int_args_extract_box.hako` 新規作成（52行）
- [ ] `call_extract_box.hako` リファクタ（54→24行）
- [ ] `method_extract_box.hako` リファクタ（51→22行）
- [ ] `new_extract_box.hako` リファクタ（51→22行）
- [ ] スモークテスト実行（call/method/new全パターン）

### Step 2: Normalizer共通化
- [ ] `normalizer_box.hako` リファクタ（96→74行）
  - [ ] `_normalize_int_array()` 追加
  - [ ] `_normalize_with_label_and_args()` 追加
  - [ ] 既存3メソッドを薄いラッパーに変更
- [ ] スモークテスト実行（normalize全パターン）

### Step 3: Emit系統合
- [ ] `args_const_emit_box.hako` 新規作成（31行）
- [ ] `emit_call_box.hako` リファクタ（56→43行）
- [ ] `emit_method_box.hako` リファクタ（54→41行）
- [ ] `emit_newbox_box.hako` リファクタ（54→41行）
- [ ] スモークテスト実行（emit全パターン）

### Step 4: 統合テスト
- [ ] `tools/smokes/v2/run.sh --profile quick` 全PASS
- [ ] セルフホストビルド確認
- [ ] パフォーマンス計測（before/after）

---

## 💡 実装時の注意点

### 1. **変数名の統一**
Extract系で `vals` / `args` が混在 → 統一推奨

### 2. **オフセット値の明示**
```hako
// ❌ マジックナンバー
local name = ast_json.substring(nk + 8, nk_end)

// ✅ 定数化（将来的に）
local KEY_OFFSET = "\"name\":\"".size()
local name = ast_json.substring(nk + KEY_OFFSET, nk_end)
```

### 3. **エラーハンドリング統一**
現状 `null` 返却 → 将来的に `Result<T, E>` 型導入検討

### 4. **テストカバレッジ**
各統合後、以下のパターンをテスト：
- 引数0個
- 引数1個
- 引数3個以上
- 不正JSON
- null入力

---

**次のステップ**: Step 1実装開始（`stage1_int_args_extract_box.hako` 新規作成）
