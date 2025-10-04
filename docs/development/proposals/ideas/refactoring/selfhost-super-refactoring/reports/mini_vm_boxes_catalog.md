# Mini-VM 箱カタログ (Box Catalog)

このドキュメントは、Mini-VM内のすべての箱（Box）の詳細仕様をまとめたものです。

---

## 📦 実行器箱 (Executor Boxes)

### MirVmMin (mir_vm_min.hako) - 191行 ⭐主実行器

**責務**: MIR JSON v0形式の実行

**公開API**:
```hakorune
static box MirVmMin {
  run(mjson: String) → Int              // 標準実行モード
  run_thin(mjson: String) → Int         // 簡略retモード

  // 内部ヘルパー（非推奨直接呼び出し）
  _run_min(mjson: String) → Int
  _int_to_str(n: Int) → String
  _is_numeric_str(s: String) → Bool
  _load_reg(regs: MapBox, id: Int) → Int
  _handle_copy(seg: String, regs: MapBox) → Void
  // ... その他ブロック操作ヘルパー
}
```

**対応MIR命令**:
- ✅ const - 定数ロード
- ✅ copy - レジスタコピー
- ✅ binop - 二項演算（Add/Sub/Mul/Div/Mod）
- ✅ compare - 比較演算（Eq/Ne/Lt/Le/Gt/Ge）
- ✅ branch - 条件分岐
- ✅ jump - 無条件ジャンプ
- ✅ ret - 関数リターン

**特殊機能**:
- **thin_mode**: JSON内に `"__thin__":1` があると簡略化されたret解決
- **op_adopt**: JSON内に `"__op_adopt__":1` があるとOperatorBoxとの比較検証
- **無限ループ防止**: 最大200,000ステップ
- **Fail-Fast**: 未定義レジスタは-1を返す

**依存**:
- OpHandlersBox (命令ハンドラ)
- JsonFragBox (JSON断片抽出)
- OperatorBox (演算子API)
- CompareOpsBox (比較演算)

**使用例**:
```hakorune
local mjson = "{\"blocks\":[{\"id\":0,\"instructions\":[...]}}]}"
local result = new MirVmMin().run(mjson)
print(result)  // 42
```

---

### FlowRunner (flow_runner.hako) - 39行

**責務**: AST JSON → MIR → 実行のパイプライン統合

**公開API**:
```hakorune
static box FlowRunner {
  run_vm_min_from_ast(ast_json: String, prefer_cfg: Int, compat: Int) → Int
  main(args: ArrayBox) → Int

  // 内部ヘルパー
  _parse_return_int(ast_json: String) → Int?
}
```

**最適化**:
- **Fast-path**: `Return(Int v)` パターンを直接処理（MIR生成スキップ）
- **互換モード**: `compat=1` で v1→v0 変換経路を使用

**依存**:
- FlowEntryBox (pipeline_v2)
- MirVmMin

**使用例**:
```hakorune
local ast = "{\"type\":\"Return\",\"value\":{\"type\":\"Int\",\"value\":42}}"
local result = new FlowRunner().run_vm_min_from_ast(ast, 0, 0)
print(result)  // 42
```

---

### StepRunnerBox (step_runner.hako) - 74行 ※観測専用

**責務**: MIR JSON v0のステップ観測（実行はしない）

**公開API**:
```hakorune
static box StepRunnerBox {
  parse_compare(seg: String) → MapBox    // compare命令解析
  parse_branch(seg: String) → MapBox     // branch命令解析
  eval_branch_bool(mjson: String) → Bool // branch真偽値評価
  main(args: ArrayBox) → Int
}
```

**特徴**:
- 実行せずに命令を観測・解析
- compare + branch の組み合わせを評価

**依存**:
- CompareOpsBox
- JsonScanBox

---

## 🔧 命令処理箱 (Instruction Handler Boxes)

### OpHandlersBox (op_handlers.hako) - 148行

**責務**: const/binop/compare命令の実装

**公開API**:
```hakorune
static box OpHandlersBox {
  handle_const(seg: String, regs: MapBox) → Void
  handle_compare(seg: String, regs: MapBox) → Void
  handle_binop(seg: String, regs: MapBox) → Void

  // 内部ヘルパー
  _str_to_int(s: String) → Int
  _is_numeric_str(s: String) → Bool
  _find_int_in(seg: String, keypat: String) → Int?
  _find_str_in(seg: String, keypat: String) → String
  _find_kv_int(seg: String, key: String) → Int?
  _find_kv_str(seg: String, key: String) → String
  _load_reg(regs: MapBox, id: Int) → Int
}
```

**対応命令**:
- **const**: `{"op":"const","dst":1,"value":42}` または `{"op":"const","dst":1,"value":{"type":"i64","value":42}}`
- **binop**: `{"op":"binop","op_kind":"Add","lhs":1,"rhs":2,"dst":3}`
  - 対応演算: Add, Sub, Mul, Div, Mod
- **compare**: `{"op":"compare","cmp":"Eq","lhs":1,"rhs":2,"dst":3}`

**依存**:
- ArithmeticBox (算術演算)
- CompareOpsBox (比較演算)

---

### CompareOpsBox (compare_ops.hako) - 24行 ⭐最小箱

**責務**: 比較演算のマッピングと評価

**公開API**:
```hakorune
static box CompareOpsBox {
  map_symbol(sym: String) → String  // "==" → "Eq"
  eval(kind: String, a: Int, b: Int) → Bool
}
```

**対応演算**:
| シンボル | Kind | 意味 |
|---------|------|------|
| `==` | `Eq` | 等しい |
| `!=` | `Ne` | 等しくない |
| `<` | `Lt` | 未満 |
| `<=` | `Le` | 以下 |
| `>` | `Gt` | より大きい |
| `>=` | `Ge` | 以上 |

**依存**: なし（完全自立）

---

### ArithmeticBox (arithmetic.hako) - 136行

**責務**: 安全な10進数演算（オーバーフロー回避）

**公開API**:
```hakorune
static box ArithmeticBox {
  add_i64(a: Int, b: Int) → Int
  sub_i64(a: Int, b: Int) → Int  // 負数対応
  mul_i64(a: Int, b: Int) → Int

  // 内部ヘルパー（文字列ベース演算）
  _to_dec_str(x) → String
  _cmp_dec(a: String, b: String) → Int
  _add_dec(a: String, b: String) → String
  _sub_dec(a: String, b: String) → String
  _mul_dec(a: String, b: String) → String
  _str_to_int(s: String) → Int
}
```

**特徴**:
- 文字列ベース演算によりオーバーフロー回避
- 負数対応（subのみ）
- ArrayBoxを使った桁演算

**依存**: なし（ArrayBoxのみ使用）

---

### OperatorBox (operator_box.hako) - 36行

**責務**: 演算子の統一API提供（デバッグ/パリティチェック用）

**公開API**:
```hakorune
static box OperatorBox {
  apply2(kind: String, a: Int, b: Int) → Int  // 二項演算
  unary(kind: String, a: Int) → Int           // 単項演算
  compare(kind: String, a: Int, b: Int) → Bool // 比較演算
}
```

**対応演算**:
- **二項**: Add, Sub, Mul, Div, Mod, BitAnd, BitOr, BitXor, Shl, Shr
- **単項**: Neg, Not, BitNot
- **比較**: CompareOpsBoxに委譲

**依存**:
- ArithmeticBox
- CompareOpsBox

**用途**: MirVmMin内で `__op_adopt__:1` 時のパリティチェック

---

## 📄 JSON処理箱 (JSON Processing Boxes)

### JsonFragBox (json_frag.hako) - 51行

**責務**: JSON文字列から key:int / key:str を簡便に取り出す

**公開API**:
```hakorune
static box JsonFragBox {
  get_int(seg: String, key: String) → Int?
  get_str(seg: String, key: String) → String
  block0_segment(mjson: String) → String

  // 内部ヘルパー
  index_of_from(hay: String, needle: String, pos: Int) → Int
  read_digits(text: String, pos: Int) → String
  _str_to_int(s: String) → Int
}
```

**使用例**:
```hakorune
local seg = "{\"op\":\"const\",\"dst\":1,\"value\":42}"
local dst = JsonFragBox.get_int(seg, "dst")   // → 1
local val = JsonFragBox.get_int(seg, "value") // → 42
```

**依存**:
- StringScanBox
- JsonScanBox

---

### JsonScanBox (json_scan.hako) - 71行

**責務**: エスケープ対応のJSON構造解析

**公開API**:
```hakorune
static box JsonScanBox {
  seek_obj_end(text: String, start: Int) → Int
  seek_array_end(text: String, start: Int) → Int
  find_key_dual(text: String, plain: String, escaped: String, pos: Int) → Int

  // 内部ヘルパー
  _str_to_int(s: String) → Int
}
```

**特徴**:
- エスケープ文字（`\"`）を正しく処理
- ネストした構造に対応
- 括弧の深さを追跡

**依存**:
- StringScanBox

---

### JsonCursorBox (json_cur.hako) - 61行

**責務**: 低レベルJSON走査

**公開API**:
```hakorune
static box MiniJsonCur {  // ※命名がMiniJsonCur
  next_non_ws(s: String, pos: Int) → Int
  read_quoted_from(s: String, pos: Int) → String
  read_digits_from(s: String, pos: Int) → String

  // 内部ヘルパー
  _is_digit(ch: String) → Bool
}
```

**特徴**:
- 空白スキップ機能
- エスケープ対応の引用符読み込み
- null/負数ガード

**依存**: なし

**使用例**:
```hakorune
local json = "  \"hello\""
local cur = new MiniJsonCur()
local pos = cur.next_non_ws(json, 0)  // → 2
local str = cur.read_quoted_from(json, pos)  // → "hello"
```

---

### StringScanBox (string_scan.hako) - 42行

**責務**: 文字列スキャン基本機能

**公開API**:
```hakorune
static box StringScanBox {
  scan_string_end(text: String, quote_pos: Int) → Int
}
```

**特徴**:
- エスケープシーケンス（`\"`, `\\`）対応
- 引用符の終端を正確に検出

**依存**: なし

---

## 🖨️ Print処理箱 (Print Handler Boxes)

### MiniVmPrints (mini_vm_prints.hako) - 115行

**責務**: Print命令の実装（AST JSONからの出力）

**公開API**:
```hakorune
static box MiniVmPrints {
  try_print_string_value_at(json: String, end: Int, print_pos: Int) → Int
  try_print_int_value_at(json: String, end: Int, print_pos: Int) → Int
  try_print_functioncall_at(json: String, end: Int, print_pos: Int) → Int
  print_prints_in_slice(json: String, start: Int, end: Int) → Int
  process_if_once(json: String) → ?
  print_all_print_literals(json: String) → ?

  // 内部フラグ
  _trace_enabled() → Bool
  _fallback_enabled() → Bool
}
```

**対応パターン**:
- `Print(Literal("hello"))` → 文字列出力
- `Print(Literal(42))` → 整数出力（型付き）
- `Print(FunctionCall("echo", [Literal("x")]))` → echo/itoa関数呼び出し

**依存**:
- MiniVmScan
- MiniVmBinOp
- MiniVmCompare
- MiniJsonLoader (json_cur.hako)

---

## 🧮 共通モジュール箱 (Common Module Boxes)

### MiniVmScan (mini_vm_scan.hako) - 206行 [common]

**責務**: スキャンと数値ヘルパー

**公開API**:
```hakorune
static box MiniVmScan {
  index_of_from(hay: String, needle: String, pos: Int) → Int
  find_balanced_array_end(json: String, idx: Int) → Int
  find_balanced_object_end(json: String, idx: Int) → Int
  _str_to_int(s: String) → Int
  _int_to_str(n: Int) → String
  _digit_char(d: Int) → String
  read_digits(json: String, pos: Int) → String
  sum_numbers_no_quotes(json: String) → String
  sum_all_digits_naive(json: String) → String
  sum_first_two_numbers(json: String) → String
}
```

**特徴**:
- エスケープ対応の括弧バランス検出
- 無限ループ防止（guard > 50000で中断）
- 複数の数値集計戦略

**依存**: なし

---

### MiniVmBinOp (mini_vm_binop.hako) - 277行 [common]

**責務**: BinaryOp処理（Print内の二項演算）

**公開API**:
```hakorune
static box MiniVmBinOp {
  try_print_binop_at(json: String, end: Int, print_pos: Int) → Int
  try_print_binop_int_greedy(json: String, end: Int, print_pos: Int) → Int  // 無効化
  try_print_binop_sum_any(json: String, end: Int, print_pos: Int) → Int
  try_print_binop_sum_expr_values(json: String, end: Int, print_pos: Int) → Int
  try_print_binop_sum_after_bop(json: String) → Int
  parse_first_binop_sum(json: String) → String
}
```

**対応パターン**:
- `BinaryOp(+, Literal("a"), Literal("b"))` → "ab" (文字列連結)
- `BinaryOp(+, Literal(1), Literal(2))` → "3" (整数加算)

**依存**:
- MiniVmScan
- MiniJson (json_cur.hako) ※上位層への依存！

---

### MiniVmCompare (mini_vm_compare.hako) - 48行 [common]

**責務**: Compare処理（Print内の比較演算）

**公開API**:
```hakorune
static box MiniVmCompare {
  try_print_compare_at(json: String, end: Int, print_pos: Int) → Int
}
```

**対応演算**: <, ==, <=, >, >=, !=

**依存**:
- MiniVmScan ※上位層への依存！

---

### MiniVm (mini_vm_core.hako) - 28行

**責務**: コア機能（委譲ファサード）

**公開API**:
```hakorune
static box MiniVm {
  _is_digit(ch: String) → Bool
  _str_to_int(s: String) → Int
  _int_to_str(n: Int) → String
  read_digits(json: String, pos: Int) → String
  read_json_string(json: String, pos: Int) → String
  index_of_from(hay: String, needle: String, pos: Int) → Int
  next_non_ws(json: String, pos: Int) → Int
}
```

**特徴**:
- 全メソッドが他箱への委譲
- 統一インターフェース提供

**依存**:
- MiniJsonCur (json_cur.hako)
- MiniVmScan [common]
- MiniVmBinOp [common]
- MiniVmCompare [common]
- MiniVmPrints

---

## 🔍 診断・デバッグ箱 (Diagnostic Boxes)

### SeamInspectorBox (seam_inspector.hako) - 202行

**責務**: 継ぎ目（Seam）検査器

**注**: 詳細未調査（行数のみ確認）

---

### FlowDebuggerBox (flow_debugger.hako) - 95行

**責務**: フロー診断器

**注**: 詳細未調査（行数のみ確認）

---

### InstructionScannerBox (instruction_scanner.hako) - 112行

**責務**: 命令スキャナ

**注**: 詳細未調査（行数のみ確認）

---

### MinivmProbeBox (minivm_probe.hako) - 57行

**責務**: Mini-VMプローブ（診断）

**注**: 詳細未調査（行数のみ確認）

---

## 📊 箱サイズ一覧（降順）

| 箱名 | ファイル | 行数 | カテゴリ |
|------|---------|------|---------|
| MiniVmBinOp | mini_vm_binop.hako | 277 | 共通 |
| MiniVmScan | mini_vm_scan.hako | 206 | 共通 |
| SeamInspectorBox | seam_inspector.hako | 202 | 診断 |
| MirVmMin | mir_vm_min.hako | 191 | 実行器 |
| OpHandlersBox | op_handlers.hako | 148 | 命令処理 |
| ArithmeticBox | arithmetic.hako | 136 | 演算 |
| MiniVmPrints | mini_vm_prints.hako | 115 | Print処理 |
| InstructionScannerBox | instruction_scanner.hako | 112 | 診断 |
| FlowDebuggerBox | flow_debugger.hako | 95 | 診断 |
| StepRunnerBox | step_runner.hako | 74 | 実行器 |
| JsonScanBox | json_scan.hako | 71 | JSON処理 |
| MiniJsonCur | json_cur.hako | 61 | JSON処理 |
| MinivmProbeBox | minivm_probe.hako | 57 | 診断 |
| JsonFragBox | json_frag.hako | 51 | JSON処理 |
| MiniVmCompare | mini_vm_compare.hako | 48 | 共通 |
| StringScanBox | string_scan.hako | 42 | JSON処理 |
| FlowRunner | flow_runner.hako | 39 | 実行器 |
| OperatorBox | operator_box.hako | 36 | 演算 |
| MiniVm | mini_vm_core.hako | 28 | コア |
| CompareOpsBox | compare_ops.hako | 24 | 演算 |
| (JsonAdapter) | json_adapter.hako | 19 | 共通 |

---

## 🎯 箱の責務マトリックス

| 箱 | 実行 | JSON解析 | 演算 | 診断 | ヘルパー |
|----|------|----------|------|------|---------|
| MirVmMin | ✅ | ✅ | ✅ | - | ✅ |
| FlowRunner | ✅ | - | - | - | ✅ |
| StepRunnerBox | - | ✅ | ✅ | ✅ | - |
| OpHandlersBox | ✅ | ✅ | ✅ | - | ✅ |
| CompareOpsBox | - | - | ✅ | - | - |
| ArithmeticBox | - | - | ✅ | - | ✅ |
| OperatorBox | - | - | ✅ | - | - |
| JsonFragBox | - | ✅ | - | - | ✅ |
| JsonScanBox | - | ✅ | - | - | ✅ |
| JsonCursorBox | - | ✅ | - | - | ✅ |
| StringScanBox | - | ✅ | - | - | - |
| MiniVmPrints | ✅ | ✅ | - | - | - |
| MiniVmScan | - | ✅ | - | - | ✅ |
| MiniVmBinOp | ✅ | ✅ | ✅ | - | - |
| MiniVmCompare | ✅ | ✅ | ✅ | - | - |
| MiniVm | - | - | - | - | ✅ |

---

## 🔗 使用例（エンドツーエンド）

### 基本的なMIR実行

```hakorune
// MIR JSON v0 (const + ret)
local mjson = "{
  \"blocks\": [
    {
      \"id\": 0,
      \"instructions\": [
        {\"op\":\"const\",\"dst\":1,\"value\":42},
        {\"op\":\"ret\",\"value\":1}
      ]
    }
  ]
}"

local vm = new MirVmMin()
local result = vm.run(mjson)
print(result)  // Output: 42
```

### 条件分岐の実行

```hakorune
// MIR JSON v0 (compare + branch)
local mjson = "{
  \"blocks\": [
    {
      \"id\": 0,
      \"instructions\": [
        {\"op\":\"const\",\"dst\":1,\"value\":10},
        {\"op\":\"const\",\"dst\":2,\"value\":20},
        {\"op\":\"compare\",\"cmp\":\"Lt\",\"lhs\":1,\"rhs\":2,\"dst\":3},
        {\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}
      ]
    },
    {
      \"id\": 1,
      \"instructions\": [
        {\"op\":\"const\",\"dst\":4,\"value\":100},
        {\"op\":\"ret\",\"value\":4}
      ]
    },
    {
      \"id\": 2,
      \"instructions\": [
        {\"op\":\"const\",\"dst\":5,\"value\":200},
        {\"op\":\"ret\",\"value\":5}
      ]
    }
  ]
}"

local vm = new MirVmMin()
local result = vm.run(mjson)
print(result)  // Output: 100 (10 < 20 → block 1)
```

### 算術演算の実行

```hakorune
// MIR JSON v0 (binop)
local mjson = "{
  \"blocks\": [
    {
      \"id\": 0,
      \"instructions\": [
        {\"op\":\"const\",\"dst\":1,\"value\":5},
        {\"op\":\"const\",\"dst\":2,\"value\":3},
        {\"op\":\"binop\",\"op_kind\":\"Add\",\"lhs\":1,\"rhs\":2,\"dst\":3},
        {\"op\":\"ret\",\"value\":3}
      ]
    }
  ]
}"

local vm = new MirVmMin()
local result = vm.run(mjson)
print(result)  // Output: 8
```

---

## 📝 命名規則の問題

以下の箱は命名が不統一です:

| ファイル名 | 箱名 | 推奨名 |
|-----------|------|--------|
| json_cur.hako | MiniJsonCur | JsonCursorBox |
| mini_vm_core.hako | MiniVm | MiniVmCoreBox |
| mini_vm_prints.hako | MiniVmPrints | MiniVmPrintsBox |

**推奨命名規則**: すべての箱名を `*Box` 形式に統一

---

## ✅ 完全自立箱（依存なし）

以下の箱は外部依存がありません:

1. **CompareOpsBox** (24行) - 比較演算
2. **ArithmeticBox** (136行) - 算術演算（ArrayBoxのみ使用）
3. **JsonCursorBox** (61行) - JSONカーソル
4. **StringScanBox** (42行) - 文字列スキャン
5. **MiniVmScan** (206行) - スキャンヘルパー

これらの箱は他の箱から独立しており、最も再利用しやすい設計です。

---

**カタログ作成日**: 2025-10-04
**対象ブランチ**: selfhost
**調査コミット**: 51f7e9f1
