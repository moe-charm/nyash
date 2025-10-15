# Mini-VM 改善アクションプラン

**目的**: Mini-VMのコード品質と保守性を向上させる具体的な実行計画

**調査結果サマリー**:
- 総ファイル数: 52 (.hako: 23, .nyash: 29)
- 総行数: 約3,400行
- コード重複: 約250行（削減可能）
- 循環依存: 3箇所確認
- 命名不統一: 3箇所確認

---

## 🎯 Phase 1: 緊急対応（1-2日）

### 優先度A: コード重複削減

#### Task 1.1: StringUtilsBox新設 ⭐最重要

**目的**: 5箇所で重複する文字列・数値変換を統一

**新規ファイル**: `selfhost/shared/common/utils/string_utils.hako`

**実装内容**:
```hakorune
// selfhost/shared/common/utils/string_utils.hako
static box StringUtilsBox {
  // 文字列 → 整数変換（負数対応）
  str_to_int(s: String) → Int {
    local i = 0
    local n = s.length()
    local acc = 0
    local neg = 0
    if s.substring(0,1) == "-" { neg = 1  i = 1 }
    loop (i < n) {
      local ch = s.substring(i, i+1)
      local d = match ch {
        "0"=>0,"1"=>1,"2"=>2,"3"=>3,"4"=>4,
        "5"=>5,"6"=>6,"7"=>7,"8"=>8,"9"=>9,
        _=>-1
      }
      if d < 0 { break }
      acc = acc * 10 + d
      i = i + 1
    }
    if neg == 1 { return 0 - acc }
    return acc
  }

  // 整数 → 文字列変換
  int_to_str(n: Int) → String {
    if n == 0 { return "0" }
    if n < 0 { return "-" + me.int_to_str(0 - n) }
    local v = n
    local out = ""
    local digits = "0123456789"
    loop (v > 0) {
      local d = v % 10
      local ch = digits.substring(d, d+1)
      out = ch + out
      v = v / 10
    }
    return out
  }

  // 数値文字列判定
  is_numeric_str(s: String) → Bool {
    if s == null { return 0 }
    local n = s.length()
    if n == 0 { return 0 }
    local i = 0
    if s.substring(0,1) == "-" {
      if n == 1 { return 0 }
      i = 1
    }
    loop (i < n) {
      local ch = s.substring(i, i+1)
      if ch < "0" || ch > "9" { return 0 }
      i = i + 1
    }
    return 1
  }

  // 数字1文字 → 整数
  digit_char_to_int(ch: String) → Int {
    return match ch {
      "0"=>0,"1"=>1,"2"=>2,"3"=>3,"4"=>4,
      "5"=>5,"6"=>6,"7"=>7,"8"=>8,"9"=>9,
      _=>-1
    }
  }

  // 整数 → 数字1文字
  int_to_digit_char(d: Int) → String {
    if d < 0 || d > 9 { return "0" }
    return "0123456789".substring(d, d+1)
  }
}
```

**移行対象**:
1. `arithmetic.hako` (115-130行) → StringUtilsBox.str_to_int
2. `op_handlers.hako` (18-33行) → StringUtilsBox.str_to_int
3. `json_frag.hako` (12行) → StringUtilsBox.str_to_int
4. `json_scan.hako` (8行) → StringUtilsBox.str_to_int
5. `step_runner.hako` (8行) → StringUtilsBox.str_to_int
6. `mir_vm_min.hako` (30-36行) → StringUtilsBox.int_to_str
7. `mini_vm_scan.hako` (112-123行) → StringUtilsBox.int_to_str

**削減見込み**: 約120行

**検証方法**:
```bash
# 1. StringUtilsBox単体テスト
./target/release/hakorune apps/selfhost/tests/test_string_utils.hako

# 2. 既存テスト実行（回帰確認）
./tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"
```

---

#### Task 1.2: ScanUtilsBox新設

**目的**: 5箇所で重複する文字列スキャン機能を統一

**新規ファイル**: `selfhost/shared/common/utils/scan_utils.hako`

**実装内容**:
```hakorune
// selfhost/shared/common/utils/scan_utils.hako
using selfhost.common.utils.string_utils as StringUtilsBox

static box ScanUtilsBox {
  // 部分文字列検索（位置指定）
  index_of_from(hay: String, needle: String, pos: Int) → Int {
    if pos < 0 { pos = 0 }
    local n = hay.length()
    if pos >= n { return -1 }
    local m = needle.length()
    if m <= 0 { return pos }
    local i = pos
    local limit = n - m
    loop (i <= limit) {
      if hay.substring(i, i + m) == needle { return i }
      i = i + 1
    }
    return -1
  }

  // 連続する数字列の読み込み
  read_digits(text: String, pos: Int) → String {
    local out = ""
    local i = pos
    loop (true) {
      local ch = text.substring(i, i+1)
      if ch == "" { break }
      if ch >= "0" && ch <= "9" {
        out = out + ch
        i = i + 1
      } else {
        break
      }
    }
    return out
  }

  // 連続する数字列の読み込み＋変換
  read_int(text: String, pos: Int) → Int {
    local digits = me.read_digits(text, pos)
    if digits == "" { return 0 }
    return StringUtilsBox.str_to_int(digits)
  }

  // 空白スキップ（次の非空白文字位置を返す）
  skip_whitespace(s: String, pos: Int) → Int {
    local i = pos
    local n = s.length()
    loop (i < n) {
      local ch = s.substring(i, i+1)
      if ch != " " && ch != "\n" && ch != "\r" && ch != "\t" {
        return i
      }
      i = i + 1
    }
    return -1
  }
}
```

**移行対象**:
1. `mini_vm_scan.hako` (4-19行) → ScanUtilsBox.index_of_from
2. `json_frag.hako` (10行) → ScanUtilsBox.index_of_from
3. `flow_runner.hako` (7行) → ScanUtilsBox.index_of_from
4. `mir_vm_min.hako` (41行) → ScanUtilsBox.index_of_from
5. `step_runner.hako` (6行) → ScanUtilsBox.index_of_from
6. `mini_vm_scan.hako` (126-144行) → ScanUtilsBox.read_digits
7. `json_frag.hako` (11行) → ScanUtilsBox.read_digits
8. `json_cur.hako` (36-59行) → ScanUtilsBox.read_digits
9. `flow_runner.hako` (8行) → ScanUtilsBox.read_digits

**削減見込み**: 約130行

---

### 優先度B: 循環依存の解消

#### Task 1.3: common → vm/boxes 依存の削除

**問題箇所**:
1. `mini_vm_binop.hako` (common) → `selfhost.vm.scan` (存在不明)
2. `mini_vm_binop.hako` (common) → `selfhost.vm.boxes.json_cur`
3. `mini_vm_compare.hako` (common) → `selfhost.vm.scan`

**解決策**:
```hakorune
// mini_vm_binop.hako の修正
// Before:
using selfhost.vm.scan as MiniVmScan
using selfhost.vm.boxes.json_cur as MiniJson

// After:
using selfhost.common.utils.scan_utils as ScanUtilsBox
using selfhost.common.utils.string_utils as StringUtilsBox
// json_cur.hakoの機能を直接実装（重複削減後）
```

**実施手順**:
1. `mini_vm_scan.hako` の機能を `ScanUtilsBox` に移行
2. `json_cur.hako` の機能を `ScanUtilsBox` に統合（必要な機能のみ）
3. `mini_vm_binop.hako` を `ScanUtilsBox` 使用に変更
4. `mini_vm_compare.hako` を `ScanUtilsBox` 使用に変更

**検証**:
```bash
# 循環依存チェック
grep -r "using.*selfhost.vm" selfhost/shared/common/
# → 結果が0件であることを確認
```

---

## 🔧 Phase 2: 品質改善（3-5日）

### 優先度C: 命名統一化

#### Task 2.1: 箱名の統一

**修正対象**:

| 現在のファイル名 | 現在の箱名 | 推奨名 |
|---------------|-----------|--------|
| json_cur.hako | MiniJsonCur | JsonCursorBox |
| mini_vm_core.hako | MiniVm | MiniVmCoreBox |
| mini_vm_prints.hako | MiniVmPrints | MiniVmPrintsBox |

**実施方法**:
```bash
# 1. 箱名変更
# json_cur.hako内: static box MiniJsonCur → static box JsonCursorBox

# 2. 使用箇所の一括置換
grep -rl "MiniJsonCur" apps/selfhost/ | xargs sed -i 's/MiniJsonCur/JsonCursorBox/g'

# 3. ビルド確認
cargo build --release

# 4. テスト実行
./tools/smokes/v2/run.sh --profile quick
```

---

### 優先度D: レイヤー分離

#### Task 2.2: ディレクトリ再編成

**現状**:
```
apps/selfhost/
├── common/
│   ├── mini_vm_scan.hako
│   ├── mini_vm_binop.hako
│   ├── mini_vm_compare.hako
│   └── json_adapter.hako
└── vm/boxes/
    └── (23箇所のhako)
```

**提案**:
```
apps/selfhost/
├── common/
│   ├── utils/              # Layer 1: 基本ユーティリティ
│   │   ├── string_utils.hako   ⭐新規
│   │   └── scan_utils.hako     ⭐新規
│   │
│   ├── json/               # Layer 2: JSON処理
│   │   ├── json_cursor.hako    (json_cur.hako移動)
│   │   ├── json_scan.hako      (移動)
│   │   ├── json_frag.hako      (移動)
│   │   └── string_scan.hako    (移動)
│   │
│   └── ops/                # Layer 3: 演算処理
│       ├── arithmetic.hako      (移動)
│       ├── compare_ops.hako     (移動)
│       ├── operator_box.hako    (移動)
│       ├── mini_vm_scan.hako    (残留・統合検討)
│       ├── mini_vm_binop.hako   (残留)
│       └── mini_vm_compare.hako (残留)
│
└── vm/                     # Layer 4: VM実装
    ├── boxes/
    │   ├── mir_vm_min.hako
    │   ├── mini_vm_core.hako
    │   ├── mini_vm_prints.hako
    │   ├── step_runner.hako
    │   └── op_handlers.hako
    │
    ├── flow_runner.hako
    └── tests/              ⭐新規テストディレクトリ
        ├── test_mir_vm_min.hako
        ├── test_arithmetic.hako
        └── test_json_frag.hako
```

**実施手順**:
```bash
# 1. ディレクトリ作成
mkdir -p selfhost/shared/common/{utils,json,ops}
mkdir -p apps/selfhost/vm/tests

# 2. ファイル移動
# StringUtilsBox/ScanUtilsBox新設（Task 1.1, 1.2）
# 既存ファイル移動
mv selfhost/vm/boxes/json_cur.hako selfhost/shared/json/json_cursor.hako
mv selfhost/vm/boxes/json_scan.hako selfhost/shared/json/
mv selfhost/vm/boxes/json_frag.hako selfhost/shared/json/
mv selfhost/vm/boxes/string_scan.hako selfhost/shared/json/
mv selfhost/vm/boxes/arithmetic.hako selfhost/shared/common/ops/
mv selfhost/vm/boxes/compare_ops.hako selfhost/shared/common/ops/
mv selfhost/vm/boxes/operator_box.hako selfhost/shared/common/ops/

# 3. using文の一括修正
# 例: using "selfhost/vm/boxes/json_scan.hako"
#  → using "selfhost/shared/json/json_scan.hako"
```

---

## 📝 Phase 3: テスト整備（5-7日）

### 優先度E: 単体テスト追加

#### Task 3.1: 基本ユーティリティのテスト

**新規ファイル**: `apps/selfhost/vm/tests/test_string_utils.hako`

```hakorune
using selfhost.common.utils.string_utils as StringUtilsBox

static box TestStringUtils {
  main(args) {
    // str_to_int テスト
    local r1 = StringUtilsBox.str_to_int("42")
    if r1 != 42 { print("FAIL: str_to_int(42)") return 1 }

    local r2 = StringUtilsBox.str_to_int("-10")
    if r2 != -10 { print("FAIL: str_to_int(-10)") return 1 }

    local r3 = StringUtilsBox.str_to_int("0")
    if r3 != 0 { print("FAIL: str_to_int(0)") return 1 }

    // int_to_str テスト
    local s1 = StringUtilsBox.int_to_str(42)
    if s1 != "42" { print("FAIL: int_to_str(42)") return 1 }

    local s2 = StringUtilsBox.int_to_str(-10)
    if s2 != "-10" { print("FAIL: int_to_str(-10)") return 1 }

    local s3 = StringUtilsBox.int_to_str(0)
    if s3 != "0" { print("FAIL: int_to_str(0)") return 1 }

    // is_numeric_str テスト
    local n1 = StringUtilsBox.is_numeric_str("123")
    if n1 != 1 { print("FAIL: is_numeric_str(123)") return 1 }

    local n2 = StringUtilsBox.is_numeric_str("abc")
    if n2 != 0 { print("FAIL: is_numeric_str(abc)") return 1 }

    print("PASS: All StringUtils tests")
    return 0
  }
}
```

**実行**:
```bash
./target/release/hakorune apps/selfhost/vm/tests/test_string_utils.hako
# Expected output: PASS: All StringUtils tests
```

---

#### Task 3.2: JSON処理のテスト

**新規ファイル**: `apps/selfhost/vm/tests/test_json_frag.hako`

```hakorune
using selfhost.common.json.json_frag as JsonFragBox

static box TestJsonFrag {
  main(args) {
    local json = "{\"op\":\"const\",\"dst\":1,\"value\":42}"

    // get_int テスト
    local dst = JsonFragBox.get_int(json, "dst")
    if dst != 1 { print("FAIL: get_int dst") return 1 }

    local val = JsonFragBox.get_int(json, "value")
    if val != 42 { print("FAIL: get_int value") return 1 }

    // get_str テスト
    local op = JsonFragBox.get_str(json, "op")
    if op != "const" { print("FAIL: get_str op") return 1 }

    print("PASS: All JsonFrag tests")
    return 0
  }
}
```

---

#### Task 3.3: MIR実行器のテスト

**新規ファイル**: `apps/selfhost/vm/tests/test_mir_vm_min.hako`

```hakorune
using selfhost.vm.boxes.mir_vm_min as MirVmMin

static box TestMirVmMin {
  main(args) {
    local vm = new MirVmMin()

    // Test 1: const + ret
    local json1 = "{\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":42},{\"op\":\"ret\",\"value\":1}]}]}"
    local r1 = vm.run(json1)
    if r1 != 42 { print("FAIL: const+ret") return 1 }

    // Test 2: binop (Add)
    local json2 = "{\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":5},{\"op\":\"const\",\"dst\":2,\"value\":3},{\"op\":\"binop\",\"op_kind\":\"Add\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"ret\",\"value\":3}]}]}"
    local r2 = vm.run(json2)
    if r2 != 8 { print("FAIL: binop Add") return 1 }

    // Test 3: compare + branch
    local json3 = "{\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":10},{\"op\":\"const\",\"dst\":2,\"value\":20},{\"op\":\"compare\",\"cmp\":\"Lt\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}]},{\"id\":1,\"instructions\":[{\"op\":\"const\",\"dst\":4,\"value\":100},{\"op\":\"ret\",\"value\":4}]},{\"id\":2,\"instructions\":[{\"op\":\"const\",\"dst\":5,\"value\":200},{\"op\":\"ret\",\"value\":5}]}]}"
    local r3 = vm.run(json3)
    if r3 != 100 { print("FAIL: compare+branch") return 1 }

    print("PASS: All MirVmMin tests")
    return 0
  }
}
```

---

### 優先度F: 統合テスト追加

#### Task 3.4: スモークテストの体系化

**新規ファイル**: `tools/smokes/v2/profiles/quick/selfhost/mini_vm_suite.sh`

```bash
#!/bin/bash
# Mini-VM統合テストスイート

set -e

echo "=== Mini-VM Test Suite ==="

# 1. ユーティリティテスト
echo "[1/5] StringUtils tests..."
./target/release/hakorune apps/selfhost/vm/tests/test_string_utils.hako

echo "[2/5] ScanUtils tests..."
./target/release/hakorune apps/selfhost/vm/tests/test_scan_utils.hako

# 2. JSON処理テスト
echo "[3/5] JsonFrag tests..."
./target/release/hakorune apps/selfhost/vm/tests/test_json_frag.hako

# 3. 演算処理テスト
echo "[4/5] Arithmetic tests..."
./target/release/hakorune apps/selfhost/vm/tests/test_arithmetic.hako

# 4. MIR実行器テスト
echo "[5/5] MirVmMin tests..."
./target/release/hakorune apps/selfhost/vm/tests/test_mir_vm_min.hako

echo "=== All tests PASSED ==="
```

**実行**:
```bash
chmod +x tools/smokes/v2/profiles/quick/selfhost/mini_vm_suite.sh
./tools/smokes/v2/profiles/quick/selfhost/mini_vm_suite.sh
```

---

## 📚 Phase 4: ドキュメント整備（2-3日）

### 優先度G: README追加

#### Task 4.1: vm/README.md作成

**新規ファイル**: `selfhost/vm/README.md`

```markdown
# Mini-VM - Hakorune MIR実行器

## 概要

Mini-VMは、Hakoruneのセルフホスティング実装における**JSON v0形式のMIR実行器**です。

## アーキテクチャ

- **実行器コア**: mir_vm_min.hako (191行)
- **命令処理**: op_handlers.hako, compare_ops.hako, arithmetic.hako
- **JSON解析**: json_frag.hako, json_scan.hako, json_cursor.hako
- **Print処理**: mini_vm_prints.hako

## 対応MIR命令

- const, copy, binop, compare, branch, jump, ret

## 使用方法

```hakorune
using selfhost.vm.boxes.mir_vm_min as MirVmMin

local mjson = "{\"blocks\":[...]}"
local vm = new MirVmMin()
local result = vm.run(mjson)
print(result)
```

## テスト実行

```bash
./tools/smokes/v2/profiles/quick/selfhost/mini_vm_suite.sh
```

## 詳細

- アーキテクチャ設計: docs/development/mini-vm-architecture.md
- 箱カタログ: /tmp/mini_vm_boxes_catalog.md
```

---

#### Task 4.2: common/README.md作成

**新規ファイル**: `selfhost/shared/common/README.md`

```markdown
# 共通モジュール (Common Modules)

## 構成

### utils/ - 基本ユーティリティ (Layer 1)
- string_utils.hako - 文字列・数値変換
- scan_utils.hako - 文字列スキャン

### json/ - JSON処理 (Layer 2)
- json_cursor.hako - JSONカーソル
- json_scan.hako - JSON構造スキャン
- json_frag.hako - JSON断片抽出
- string_scan.hako - 文字列スキャン基本

### ops/ - 演算処理 (Layer 3)
- arithmetic.hako - 安全算術演算
- compare_ops.hako - 比較演算
- operator_box.hako - 演算子統一API

## 依存関係

Layer 1 (utils) ← Layer 2 (json) ← Layer 3 (ops) ← Layer 4 (vm)

（下位層のみに依存）
```

---

## 📊 実施スケジュール

| Phase | Task | 見込み | 担当 | 期限 |
|-------|------|--------|------|------|
| 1 | StringUtilsBox新設 | 4h | - | Day 1 |
| 1 | ScanUtilsBox新設 | 4h | - | Day 1 |
| 1 | 循環依存解消 | 4h | - | Day 2 |
| 2 | 命名統一化 | 3h | - | Day 3 |
| 2 | レイヤー分離 | 5h | - | Day 3-4 |
| 3 | 単体テスト追加 | 8h | - | Day 5-6 |
| 3 | 統合テスト追加 | 4h | - | Day 7 |
| 4 | README追加 | 3h | - | Day 8 |
| 4 | アーキテクチャ図作成 | 2h | - | Day 8 |

**合計見込み**: 8日間（1日4時間作業想定）

---

## ✅ チェックリスト

### Phase 1完了条件
- [ ] StringUtilsBox実装完了
- [ ] ScanUtilsBox実装完了
- [ ] 5箇所以上の重複削除完了
- [ ] 循環依存0件
- [ ] 既存テスト全PASS

### Phase 2完了条件
- [ ] 箱名が全て `*Box` 形式
- [ ] ディレクトリ構造が4層に分離
- [ ] using文のパス更新完了
- [ ] ビルド成功
- [ ] 既存テスト全PASS

### Phase 3完了条件
- [ ] 各層に単体テストが存在
- [ ] 統合テストスイート作成
- [ ] 全テストPASS
- [ ] カバレッジ50%以上

### Phase 4完了条件
- [ ] 各ディレクトリにREADME.md存在
- [ ] アーキテクチャ図作成
- [ ] 依存関係図作成
- [ ] API仕様書完成

---

## 🎯 成功指標 (KPI)

### コード品質
- **重複削減率**: 250行 / 3,400行 = **7.4%削減**
- **循環依存**: 3箇所 → **0箇所**
- **テストカバレッジ**: 0% → **50%以上**

### 保守性
- **ファイル構造**: フラット → **4層階層化**
- **命名統一率**: 87% (20/23) → **100%**
- **ドキュメント化率**: 0% → **100%**

### 開発効率
- **ビルド時間**: 変化なし（想定）
- **テスト実行時間**: +30秒（許容）
- **新規機能追加時間**: **30%削減**（見込み）

---

## 🚨 リスクと対策

### リスク1: 既存機能の破壊
**対策**:
- 段階的な移行（1箱ずつテスト）
- 既存テストの継続実行
- コミット単位での検証

### リスク2: 作業時間の超過
**対策**:
- Phase 1のみ優先実施
- Phase 2以降は必要に応じて延期可能
- 最小限の改善で効果を確認

### リスク3: using文のパス不一致
**対策**:
- grep -r での全箇所確認
- ビルドエラー時の即座な修正
- テスト実行でのエンドツーエンド検証

---

## 📝 まとめ

**即座に実施すべき**:
1. ✅ StringUtilsBox新設（120行削減）
2. ✅ ScanUtilsBox新設（130行削減）
3. ✅ 循環依存解消

**短期的に実施すべき**:
4. 命名統一化
5. レイヤー分離

**中長期的に実施すべき**:
6. テスト整備
7. ドキュメント整備

**効果**:
- コード重複: 約250行削減（7.4%）
- 保守性: 大幅向上
- 開発効率: 30%向上（見込み）

---

**作成日**: 2025-10-04
**対象**: apps/selfhost/vm/ および関連モジュール
**推奨開始日**: 即時
