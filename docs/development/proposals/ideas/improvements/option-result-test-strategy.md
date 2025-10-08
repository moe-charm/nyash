# Option/Result テスト戦略 + スモークテスト統合

**作成日**: 2025-10-08
**目的**: 包括的テストスイート設計とスモークテスト統合方法の標準化

---

## 📋 エグゼクティブサマリー

**Option/Result実装への総合テスト戦略**

- **テスト総数**: 25パターン（Option:10, Result:10, 組み合わせ:5）
- **スモークテスト統合**: `tools/smokes/v2/profiles/quick/core/` に2本追加
- **テスト駆動開発**: Phase 15.11成功事例を適用（テスト先行→実装→確認）

---

## 🎯 1. テストケース設計

### 1.1 Option<T> テストケース（10パターン）

#### 基本パターン
```nyash
// apps/tests/test_option_basic.hako
using "apps/lib/boxes/option_std.hako" as Opt

static box TestOptionBasic {
  main() {
    // TC1: Some作成と値取得
    local opt1 = Opt.some(42)
    print("TC1: is_some=" + opt1.is_some())           // → 1
    print("TC1: is_none=" + opt1.is_none())           // → 0
    print("TC1: value=" + opt1.value())               // → 42

    // TC2: None作成と確認
    local opt2 = Opt.none()
    print("TC2: is_some=" + opt2.is_some())           // → 0
    print("TC2: is_none=" + opt2.is_none())           // → 1

    // TC3: unwrap_or（Some時）
    local opt3 = Opt.some(100)
    print("TC3: unwrap_or=" + opt3.unwrap_or(0))      // → 100

    // TC4: unwrap_or（None時）
    local opt4 = Opt.none()
    print("TC4: unwrap_or=" + opt4.unwrap_or(999))    // → 999

    // TC5: map（Some時）
    local opt5 = Opt.some(5)
    local mapped = opt5.map(fn(x) { x * 2 })
    print("TC5: mapped=" + mapped.unwrap_or(0))       // → 10

    // TC6: map（None時）
    local opt6 = Opt.none()
    local mapped2 = opt6.map(fn(x) { x * 2 })
    print("TC6: mapped=" + mapped2.unwrap_or(-1))     // → -1

    // TC7: and_then（Some時）
    local opt7 = Opt.some(3)
    local chained = opt7.and_then(fn(x) { Opt.some(x + 10) })
    print("TC7: chained=" + chained.unwrap_or(0))     // → 13

    // TC8: and_then（None時）
    local opt8 = Opt.none()
    local chained2 = opt8.and_then(fn(x) { Opt.some(x + 10) })
    print("TC8: chained=" + chained2.unwrap_or(-1))   // → -1

    // TC9: filter（条件一致）
    local opt9 = Opt.some(10)
    local filtered = opt9.filter(fn(x) { x > 5 })
    print("TC9: filtered=" + filtered.unwrap_or(0))   // → 10

    // TC10: filter（条件不一致）
    local opt10 = Opt.some(3)
    local filtered2 = opt10.filter(fn(x) { x > 5 })
    print("TC10: filtered=" + filtered2.unwrap_or(-1)) // → -1

    return 0
  }
}
```

**期待出力**:
```
TC1: is_some=1
TC1: is_none=0
TC1: value=42
TC2: is_some=0
TC2: is_none=1
TC3: unwrap_or=100
TC4: unwrap_or=999
TC5: mapped=10
TC6: mapped=-1
TC7: chained=13
TC8: chained=-1
TC9: filtered=10
TC10: filtered=-1
```

### 1.2 Result<T,E> テストケース（10パターン）

```nyash
// apps/tests/test_result_basic.hako
using "apps/lib/boxes/result_std.hako" as Res

static box TestResultBasic {
  main() {
    // TC1: Ok作成と値取得
    local res1 = Res.ok(42)
    print("TC1: is_ok=" + res1.is_ok())               // → 1
    print("TC1: value=" + res1.value())               // → 42

    // TC2: Err作成と確認
    local res2 = Res.err("error message")
    print("TC2: is_ok=" + res2.is_ok())               // → 0
    print("TC2: error=" + res2.error())               // → "error message"

    // TC3: unwrap_or（Ok時）
    local res3 = Res.ok(100)
    print("TC3: unwrap_or=" + res3.unwrap_or(0))      // → 100

    // TC4: unwrap_or（Err時）
    local res4 = Res.err("fail")
    print("TC4: unwrap_or=" + res4.unwrap_or(999))    // → 999

    // TC5: map（Ok時）
    local res5 = Res.ok(5)
    local mapped = res5.map(fn(x) { x * 2 })
    print("TC5: mapped=" + mapped.unwrap_or(0))       // → 10

    // TC6: map（Err時）
    local res6 = Res.err("error")
    local mapped2 = res6.map(fn(x) { x * 2 })
    print("TC6: mapped=" + mapped2.unwrap_or(-1))     // → -1

    // TC7: map_err（Err時）
    local res7 = Res.err("original")
    local mapped_err = res7.map_err(fn(e) { e + "_modified" })
    print("TC7: error=" + mapped_err.error())         // → "original_modified"

    // TC8: and_then（Ok時）
    local res8 = Res.ok(3)
    local chained = res8.and_then(fn(x) { Res.ok(x + 10) })
    print("TC8: chained=" + chained.unwrap_or(0))     // → 13

    // TC9: or_else（Err時）
    local res9 = Res.err("fail")
    local fallback = res9.or_else(fn(e) { Res.ok(999) })
    print("TC9: fallback=" + fallback.unwrap_or(0))   // → 999

    // TC10: 境界条件（空文字列エラー）
    local res10 = Res.err("")
    print("TC10: is_ok=" + res10.is_ok())             // → 0
    print("TC10: error_len=" + res10.error().length()) // → 0

    return 0
  }
}
```

**期待出力**:
```
TC1: is_ok=1
TC1: value=42
TC2: is_ok=0
TC2: error=error message
TC3: unwrap_or=100
TC4: unwrap_or=999
TC5: mapped=10
TC6: mapped=-1
TC7: error=original_modified
TC8: chained=13
TC9: fallback=999
TC10: is_ok=0
TC10: error_len=0
```

### 1.3 組み合わせテスト（5パターン）

```nyash
// apps/tests/test_option_result_combined.hako
using "apps/lib/boxes/option_std.hako" as Opt
using "apps/lib/boxes/result_std.hako" as Res

static box TestCombined {
  // ヘルパー: 0で除算エラー、それ以外は Ok(42/x)
  divide(x) {
    if x == 0 {
      return Res.err("division by zero")
    }
    return Res.ok(42 / x)
  }

  main() {
    // TC1: Option → Result変換（Some）
    local opt1 = Opt.some(6)
    local res1 = opt1.ok_or("no value")
    print("TC1: " + res1.unwrap_or(0))                // → 6

    // TC2: Option → Result変換（None）
    local opt2 = Opt.none()
    local res2 = opt2.ok_or("no value")
    print("TC2: " + res2.unwrap_or(-1))               // → -1
    print("TC2_err: " + res2.error())                 // → "no value"

    // TC3: Result → Option変換（Ok）
    local res3 = Res.ok(42)
    local opt3 = res3.ok()
    print("TC3: " + opt3.unwrap_or(0))                // → 42

    // TC4: Result → Option変換（Err）
    local res4 = Res.err("error")
    local opt4 = res4.ok()
    print("TC4: " + opt4.unwrap_or(-1))               // → -1

    // TC5: チェーニング（Option → Result → Option）
    local opt5 = Opt.some(7)
    local final = opt5.ok_or("fail").and_then(fn(x) {
      me.divide(x)
    }).ok()
    print("TC5: " + final.unwrap_or(0))               // → 6 (42/7)

    return 0
  }
}
```

**期待出力**:
```
TC1: 6
TC2: -1
TC2_err: no value
TC3: 42
TC4: -1
TC5: 6
```

---

## 🧪 2. スモークテスト統合

### 2.1 ファイル配置

**追加ファイル**:
```
tools/smokes/v2/profiles/quick/core/
├── option_basic_vm.sh          # Option基本テスト
└── result_basic_vm.sh          # Result基本テスト
```

### 2.2 スモークテスト実装例

#### option_basic_vm.sh
```bash
#!/bin/bash
# option_basic_vm.sh — Option<T> basic operations test

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/option_basic_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/test_option_basic.hako"

cat > "$SRC" << 'EOF'
using "apps/lib/boxes/option_std.hako" as Opt

static box Main {
  main() {
    // TC1-4: 基本操作
    local opt1 = Opt.some(42)
    print("is_some=" + opt1.is_some())
    print("value=" + opt1.value())

    local opt2 = Opt.none()
    print("is_none=" + opt2.is_none())
    print("unwrap_or=" + opt2.unwrap_or(999))

    return 0
  }
}
EOF

out=$(run_nyash_vm "$SRC")
want=$(cat << 'E'
is_some=1
value=42
is_none=1
unwrap_or=999
E
)

compare_outputs "$want" "$out" "option_basic_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
```

#### result_basic_vm.sh
```bash
#!/bin/bash
# result_basic_vm.sh — Result<T,E> basic operations test

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/result_basic_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/test_result_basic.hako"

cat > "$SRC" << 'EOF'
using "apps/lib/boxes/result_std.hako" as Res

static box Main {
  main() {
    // TC1-4: 基本操作
    local res1 = Res.ok(42)
    print("is_ok=" + res1.is_ok())
    print("value=" + res1.value())

    local res2 = Res.err("error")
    print("is_err=" + (1 - res2.is_ok()))
    print("unwrap_or=" + res2.unwrap_or(999))

    return 0
  }
}
EOF

out=$(run_nyash_vm "$SRC")
want=$(cat << 'E'
is_ok=1
value=42
is_err=1
unwrap_or=999
E
)

compare_outputs "$want" "$out" "result_basic_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
```

### 2.3 スモークテスト実行方法

```bash
# 個別実行
bash tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
bash tools/smokes/v2/profiles/quick/core/result_basic_vm.sh

# quick プロファイル全体実行
tools/smokes/v2/run.sh --profile quick
```

---

## 🎓 3. Phase 15.11成功事例の適用

### 3.1 成功パターン

**Phase 15.11 (StringHelpers統合)で成功した理由**:
1. ✅ **テスト先行作成**: `test_string_helpers.hako` を最初に作成
2. ✅ **包括的カバレッジ**: 7種類のヘルパー関数すべてテスト
3. ✅ **即座確認**: 各メソッド実装後、即座にテスト実行
4. ✅ **期待出力明確化**: すべてのテストケースに期待出力を明記

### 3.2 Option/Result への適用

**実行計画（3フェーズ）**:

#### Phase 1: テスト作成（1-2時間）
```bash
# ステップ1: テストファイル作成
apps/tests/test_option_basic.hako      # Option基本テスト
apps/tests/test_result_basic.hako      # Result基本テスト
apps/tests/test_option_result_combined.hako  # 組み合わせテスト
```

#### Phase 2: 実装（2-3時間）
```bash
# ステップ2: 実装ファイル作成
apps/lib/boxes/option_std.hako         # Option実装
apps/lib/boxes/result_std.hako         # Result実装（既存強化）

# 中間テストポイント:
# - OptionBox.some/none/is_some/is_none 実装 → テスト実行
# - OptionBox.unwrap_or 実装 → テスト実行
# - OptionBox.map/and_then 実装 → テスト実行
```

#### Phase 3: スモークテスト統合（1時間）
```bash
# ステップ3: スモークテスト追加
tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
tools/smokes/v2/profiles/quick/core/result_basic_vm.sh

# 最終確認:
tools/smokes/v2/run.sh --profile quick
```

---

## 🚨 4. 失敗事例からの学び

### 4.1 Phase 2.1 の失敗

**問題点**:
- ❌ テスト実行0回成功
- ❌ コンパイルエラー4回連続
- ❌ 動作確認できていない状態でcommit提案

**教訓**:
1. **中間テスト必須**: コード編集中に最低1回は動作確認
2. **構文事前確認**: Hakoruneの構文制約を確認してから実装
3. **調査優先**: エラーが出たら、試行錯誤より根本原因調査

### 4.2 Option/Result での対策

**具体的対策**:
```bash
# ✅ 対策1: 最小テストを最初に実行
cat > /tmp/minimal_option_test.hako << 'EOF'
static box OptionBox {
  birth() { }
}
static box Main {
  main() {
    local opt = new OptionBox()
    print("ok")
    return 0
  }
}
EOF
./target/release/hako /tmp/minimal_option_test.hako
# → 期待: "ok" と表示される

# ✅ 対策2: メソッド1つごとにテスト
# is_some() 実装 → テスト実行 → OK確認
# is_none() 実装 → テスト実行 → OK確認
# unwrap_or() 実装 → テスト実行 → OK確認

# ✅ 対策3: 構文エラー時は即座に調査
# セミコロン区切りエラー → Hakorune構文制約確認
# using文エラー → hako.toml/nyash.toml 確認
```

---

## 📊 5. テスト実装の優先順位

### 5.1 優先順位（高→低）

| 優先度 | 項目 | 理由 |
|--------|------|------|
| P0 | Option基本操作（some/none/is_some/is_none/unwrap_or） | 最も頻繁に使用 |
| P1 | Result基本操作（ok/err/is_ok/value/error/unwrap_or） | エラーハンドリングの核心 |
| P2 | Option.map/and_then | 関数型プログラミングの基本 |
| P3 | Result.map/map_err/and_then/or_else | エラー変換・チェーニング |
| P4 | Option/Result相互変換 | 実用的なユースケース |
| P5 | 境界条件テスト | 品質保証 |

### 5.2 最小実装（MVP）

**Phase 1: 最小動作確認（30分）**
```nyash
// apps/lib/boxes/option_std.hako（最小版）
box OptionBox {
  _val: Box
  _some: IntegerBox  // 1=some, 0=none

  birth() { me._val = null  me._some = 0 }

  is_some() { return me._some }
  is_none() { return 1 - me._some }
  value() { return me._val }
  unwrap_or(def) { if me._some == 1 { return me._val } return def }
}

static box Opt {
  some(v) {
    local opt = new OptionBox()
    opt._val = v
    opt._some = 1
    return opt
  }
  none() {
    local opt = new OptionBox()
    return opt
  }
}
```

**テスト実行**:
```bash
# 最小テストで即座確認
./target/release/hako apps/tests/test_option_minimal.hako
# → 期待: すべてのTC1-4がPASS
```

---

## 🔍 6. デバッグ支援

### 6.1 トレースログ

```bash
# Option/Result内部動作確認
export HAKO_VM_TRACE="op=boxcall,call;regs=1"
export NYASH_DISABLE_PLUGINS=1
./target/release/hakorune apps/tests/test_option_basic.hako 2>&1

# 出力例:
# [vm] bb=0 inst=2 call Opt.some args=[v%1(42)] dst=v%2
# [vm] bb=0 inst=3 boxcall OptionBox.is_some recv=v%2 dst=v%3 → 1
```

### 6.2 MIR出力確認

```bash
# MIR命令確認
./target/release/hako --dump-mir apps/tests/test_option_basic.hako

# 期待される命令パターン:
# - newbox %1 = OptionBox
# - boxcall %2 = OptionBox.is_some(%1)
# - const %3 = 1
# - compare %4 = Eq %2, %3
```

---

## 📝 7. チェックリスト

### 7.1 実装前チェックリスト

- [ ] Phase 15.11成功事例を確認
- [ ] Hakorune構文制約を確認（セミコロン、using文、lambda等）
- [ ] 既存のResultBox実装を確認
- [ ] StringStdなど既存の標準ライブラリパターンを確認

### 7.2 実装中チェックリスト

- [ ] 最小テスト（OptionBox birth）が動作
- [ ] 各メソッド実装後にテスト実行
- [ ] 構文エラー発生時は即座に調査
- [ ] 期待出力と実際の出力を比較

### 7.3 実装後チェックリスト

- [ ] すべてのテストケースがPASS
- [ ] スモークテストがPASS
- [ ] MIR出力が正常
- [ ] トレースログで内部動作確認
- [ ] ドキュメント更新

---

## 🎯 8. 成果物の評価基準

### 8.1 成功の定義

**✅ 最小成功**:
- Option基本操作（some/none/is_some/is_none/unwrap_or）が動作
- Result基本操作（ok/err/is_ok/value/error/unwrap_or）が動作
- スモークテスト2本がPASS

**✅ 完全成功**:
- 25テストケースすべてPASS
- スモークテスト統合完了
- ドキュメント更新完了
- Phase 15.11と同等の品質

### 8.2 失敗の定義

**❌ 失敗パターン**:
- テスト実行0回成功（Phase 2.1の再現）
- 構文エラー3回以上連続
- 根本原因調査なしの試行錯誤
- 見積もり精度18%以下（Phase 2.1の再現）

---

## 📖 9. 参考リソース

### 9.1 成功事例

- **Phase 15.11**: StringHelpers統合（335行削減、テスト駆動開発）
  - ファイル: `apps/selfhost/test_string_helpers.hako`
  - コミット: `6ba6b026`, `d07f3af3`

### 9.2 既存実装

- **ResultBox**: `apps/selfhost/vm/boxes/result_box.hako`
- **StringStd**: `apps/lib/boxes/string_std.hako`
- **ArrayStd**: `apps/lib/boxes/array_std.hako`
- **MapStd**: `apps/lib/boxes/map_std.hako`

### 9.3 スモークテスト例

- **ArrayBox**: `tools/smokes/v2/profiles/quick/core/wasm_std_array_resize_vm.sh`
- **Static Call**: `tools/smokes/v2/profiles/quick/core/core_static_add_call_vm.sh`

---

## 🚀 10. 次のステップ

### 10.1 即座実行（今日）

1. **最小テスト作成** (30分)
   - `apps/tests/test_option_minimal.hako`
   - OptionBox.birth/some/none/is_some/unwrap_or のみ

2. **最小実装** (30分)
   - `apps/lib/boxes/option_std.hako`（MVP版）

3. **動作確認** (10分)
   - `./target/release/hako apps/tests/test_option_minimal.hako`

### 10.2 明日以降

1. **完全テスト作成** (1-2時間)
   - `apps/tests/test_option_basic.hako`（10パターン）
   - `apps/tests/test_result_basic.hako`（10パターン）

2. **完全実装** (2-3時間)
   - OptionBox.map/and_then/filter 等
   - ResultBox拡張（map/map_err/and_then/or_else）

3. **スモークテスト統合** (1時間)
   - `option_basic_vm.sh`
   - `result_basic_vm.sh`

---

**注**: この戦略書は Phase 15.11 の成功事例と Phase 2.1 の失敗事例から学んだベストプラクティスを集約したものです。テスト駆動開発（TDD）を徹底し、中間確認を怠らないことが最も重要です。
