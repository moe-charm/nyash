# Hakorune テストベストプラクティス

**目的**: selfhost/ コードのテスト可能性を高めるためのガイドライン

**関連**:
- [テスト可能性分析レポート](../development/analysis/TESTABILITY_QUALITY_REPORT.md)
- [サマリー](../development/analysis/TESTABILITY_SUMMARY.md)

---

## 🎯 基本原則

### 1. Result型エラーハンドリング（必須）

**❌ 悪い例**:
```hakorune
box MyBox {
  process(value) {
    if value == null {
      print("Error: value is null")  // ✗ エラーが伝播しない
      return -1
    }
    return value * 2
  }
}
```

**✅ 良い例**:
```hakorune
using "selfhost/vm/boxes/result_box.hako" as Result

box MyBox {
  process(value) {
    if value == null {
      return Result.Err("value is null")  // ✅ Result型で伝播
    }
    return Result.Ok(value * 2)
  }
}

// 呼び出し側
static box Main {
  main() {
    local result = MyBox.process(null)
    if result.is_Err() {
      print("[ERROR] " + result.as_Err())
      return 1
    }
    return result.as_Ok()
  }
}
```

---

### 2. 依存注入パターン（推奨）

**❌ 悪い例**:
```hakorune
box MyBox {
  process() {
    local data = new MapBox()  // ✗ 直接生成（モック不可）
    data.set("key", "value")
    return me._transform(data)
  }
}
```

**✅ 良い例**:
```hakorune
box MyBox {
  data: MapBox

  // コンストラクタで依存を注入
  birth(data) {
    me.data = data
    if me.data == null { me.data = new MapBox() }  // デフォルト値
  }

  process() {
    return me._transform(me.data)
  }

  // テスト用: 状態を観測可能に
  get_data() { return me.data }
}

// テストコード
static box MyBoxTest {
  test_process_with_mock() {
    // Given: モックデータ
    local mock = new MapBox()
    mock.set("key", "test_value")

    // When: 処理実行
    local box = new MyBox(mock)
    local result = box.process()

    // Then: 結果検証
    if result != "expected" { return 1 }
    return 0
  }
}
```

---

### 3. 純粋関数の分離（重要）

**❌ 悪い例**:
```hakorune
box MyBox {
  data: MapBox

  // 副作用と計算が混在
  process(input) {
    me.data.set("input", input)  // ✗ 副作用
    local result = input * 2      // 計算
    me.data.set("result", result) // ✗ 副作用
    return result
  }
}
```

**✅ 良い例**:
```hakorune
box MyBox {
  data: MapBox

  // 純粋関数（テスト容易）
  _calculate(input) {
    return input * 2
  }

  // 副作用を分離
  process(input) {
    local result = me._calculate(input)  // ✅ 純粋関数
    me.data.set("input", input)          // 副作用
    me.data.set("result", result)        // 副作用
    return result
  }
}

// テストコード
static box MyBoxTest {
  test_calculate() {
    // 純粋関数は簡単にテスト可能
    local box = new MyBox(null)
    local result = box._calculate(21)
    if result != 42 { return 1 }
    return 0
  }
}
```

---

### 4. null チェックの徹底（必須）

**❌ 悪い例**:
```hakorune
box MyBox {
  get_size(data) {
    return data.size()  // ✗ data が null の場合クラッシュ
  }
}
```

**✅ 良い例**:
```hakorune
using "selfhost/vm/boxes/result_box.hako" as Result

box MyBox {
  get_size(data) {
    if data == null {
      return Result.Err("data is null")  // ✅ エラーハンドリング
    }
    if data.size == null {
      return Result.Err("data.size is not defined")
    }
    return Result.Ok(data.size())
  }
}
```

---

### 5. 関数サイズの制限（推奨）

**基準**:
- ✅ **0-50行**: 理想的（テスト容易）
- ⚠️ **51-100行**: 許容範囲（分割検討）
- ❌ **101+行**: 分割必須

**❌ 悪い例**:
```hakorune
box MyBox {
  process(data) {
    // ... 200行の複雑なロジック
    // （テスト困難）
  }
}
```

**✅ 良い例**:
```hakorune
box MyBox {
  process(data) {
    local validated = me._validate(data)  // 20行
    if validated.is_Err() { return validated }

    local normalized = me._normalize(validated.as_Ok())  // 30行
    if normalized.is_Err() { return normalized }

    return me._transform(normalized.as_Ok())  // 40行
  }

  _validate(data) { ... }    // 20行
  _normalize(data) { ... }   // 30行
  _transform(data) { ... }   // 40行
}
```

---

## 🧪 テストコード作成ガイド

### テストファイル配置

```
selfhost/
├── hakorune-vm/
│   ├── hakorune_vm_core.hako
│   └── tests/
│       └── test_vm_core.hako  ← テストファイル
│
├── compiler/pipeline_v2/
│   ├── pipeline.hako
│   └── tests/
│       └── test_pipeline.hako  ← テストファイル
```

### テストテンプレート

```hakorune
// tests/test_my_box.hako

using "path/to/my_box.hako" as MyBox
using "selfhost/shared/common/string_helpers.hako" as StringHelpers

static box MyBoxTest {
  main() {
    print("=== MyBox Tests ===")

    local test1 = me._test_basic_case()
    if test1 != 0 {
      print("[FAIL] Test 1")
      return test1
    }

    local test2 = me._test_error_case()
    if test2 != 0 {
      print("[FAIL] Test 2")
      return test2
    }

    print("✅ All tests PASSED")
    return 0
  }

  _test_basic_case() {
    // Given: 入力データ
    local input = 42

    // When: 処理実行
    local box = new MyBox(null)
    local result = box.process(input)

    // Then: 結果検証
    if result.is_Err() {
      print("[ERROR] Expected Ok, got Err: " + result.as_Err())
      return 1
    }

    local value = result.as_Ok()
    if value != 84 {
      print("[ERROR] Expected 84, got: " + StringHelpers.int_to_str(value))
      return 1
    }

    return 0
  }

  _test_error_case() {
    // Given: null 入力
    local input = null

    // When: 処理実行
    local box = new MyBox(null)
    local result = box.process(input)

    // Then: エラー検証
    if result.is_Ok() {
      print("[ERROR] Expected Err, got Ok")
      return 1
    }

    local err = result.as_Err()
    if err.indexOf("null") < 0 {
      print("[ERROR] Expected null error, got: " + err)
      return 1
    }

    return 0
  }
}
```

---

## 🏗️ リファクタリングパターン

### パターン #1: 巨大関数の分割

**Before**: 500行の巨大関数
```hakorune
flow PipelineV2 {
  lower_stage1_to_mir(ast_json, prefer_cfg) {
    // ... 500行の複雑な分岐
  }
}
```

**After**: パターン検出 + ハンドラー分離
```hakorune
static box PatternDetectorBox {
  detect(ast_json) {
    if ast_json.indexOf("\"type\":\"Compare\"") >= 0 { return "compare" }
    if ast_json.indexOf("\"type\":\"Call\"") >= 0 { return "call" }
    return "unknown"
  }
}

flow PipelineV2 {
  lower_stage1_to_mir(ast_json, prefer_cfg) {
    local pattern = PatternDetectorBox.detect(ast_json)

    if pattern == "compare" { return me._handle_compare(ast_json, prefer_cfg) }
    if pattern == "call" { return me._handle_call(ast_json, prefer_cfg) }
    return me._handle_default(ast_json, prefer_cfg)
  }

  _handle_compare(ast_json, prefer_cfg) { ... }  // 50行
  _handle_call(ast_json, prefer_cfg) { ... }     // 50行
}
```

---

### パターン #2: 状態の外部化

**Before**: 内部状態が観測不可
```hakorune
box MyBox {
  process() {
    local state = new MapBox()
    // ... 状態を内部で変更
    return state.get("result")
  }
}
```

**After**: 状態を観測可能に
```hakorune
box MyBox {
  state: MapBox

  birth(state) {
    me.state = state
    if me.state == null { me.state = new MapBox() }
  }

  process() {
    // ... me.state を変更
    return me.state.get("result")
  }

  get_state() { return me.state }  // テスト用
}
```

---

## 📊 品質チェックリスト

コードレビュー時に確認すべき項目:

### ✅ エラーハンドリング
- [ ] すべてのエラーがResult型で伝播している
- [ ] null チェックが適切に行われている
- [ ] エラーメッセージが具体的（デバッグ可能）

### ✅ テスト可能性
- [ ] 関数が50行以下（分割検討）
- [ ] 副作用が分離されている（純粋関数化）
- [ ] 依存が注入可能（モック可能）
- [ ] 状態が観測可能（get_xxx メソッド）

### ✅ コード品質
- [ ] 循環的複雑度が20以下（分岐が少ない）
- [ ] 深いネストなし（3階層以下）
- [ ] 責務が単一（関数名が明確）

---

## 🔗 参考リソース

### ドキュメント
- [テスト可能性分析レポート](../development/analysis/TESTABILITY_QUALITY_REPORT.md)
- [Phase 20.5 README](../development/roadmap/phases/phase-20.5/README.md)

### コード例
- **優秀な例**: `selfhost/hakorune-vm/hakorune_vm_core.hako`
  - Result型エラーハンドリング
  - 関数分割（各50行以下）
  - null チェック徹底

- **テスト例**: `selfhost/hakorune-vm/tests/test_boxcall.hako`
  - 9テストケース
  - Given-When-Then構造
  - エラーケース網羅

---

**最終更新**: 2025-10-15
**次回レビュー**: Phase 20.6開始時
