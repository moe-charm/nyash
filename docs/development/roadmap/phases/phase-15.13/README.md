# Phase 15.13: マクロ化綺麗綺麗大作戦（@enum/@match適用）

**完了日**: 2025-10-09
**目的**: @enum/@matchマクロを既存コードに適用し、コード削減＆可読性向上
**成果**: 52行純削減、全テストPASS

---

## 📊 **成果サマリー**

| Day | ファイル | 変更内容 | 削減行数 | テスト結果 |
|-----|---------|---------|----------|------------|
| **Day 1** | `json_inst_encode_box.hako` | 10段ネスト if-else-if → @match | **-28行** | 46/46 PASS |
| **Day 2** | `minivm_probe.hako` | 2重 if-else-if → @match（可読性向上） | +2行 | 1/1 PASS |
| **Day 3** | `result_box.hako` | 手動Result型 → @enum Result | **-26行** | 255/255 PASS |

**総削減**: 52行（純削減）
**予測精度**: 67%（予測78行 vs 実績52行）

---

## 🎯 **主な改善点**

### 1️⃣ **json_inst_encode_box.hako**: 命令エンコーダーの簡潔化

**変更前（54行）**:
```hako
encode(node) {
  if node == null { return "{}" }
  local op = node.get("op")
  local s = "{}"
  if op == "const" {
    s = "..."
  } else {
    if op == "compare" {
      s = "..."
    } else {
      if op == "binop" {
        // 10段ネスト...
      }
    }
  }
  return s
}
```

**変更後（26行）**:
```hako
encode(node) {
  if node == null { return "{}" }
  local op = node.get("op")
  return match op {
    "const" => "..."
    "compare" => "..."
    "binop" => "..."
    "branch" => "..."
    "jump" => "..."
    "ret" => "..."
    "copy" => "..."
    "call" => "..."
    "boxcall" => "..."
    "newbox" => "..."
    "mir_call" => "{\"op\":\"mir_call\"}"
    _ => "{}"
  }
}
```

**効果**:
- 削減: -28行（52%削減）
- 可読性: 深いネスト削除、一目で全命令種別が見える
- 保守性: 新規命令追加が容易

---

### 2️⃣ **minivm_probe.hako**: 命令ディスパッチの明示化

**変更前（50行）**:
```hako
if op == "const" {
  OpHandlersBox.handle_const(obj, regs)
} else if op == "binop" {
  OpHandlersBox.handle_binop(obj, regs)
} else if op == "compare" {
  local kind = ...
  if kind == "Eq" { if a == b { r = 1 } }
  else if kind == "Ne" { if a != b { r = 1 } }
  else if kind == "Lt" { if a < b { r = 1 } }
  // 6段ネスト...
}
```

**変更後（52行）**:
```hako
match op {
  "const" => { OpHandlersBox.handle_const(obj, regs) }
  "binop" => { OpHandlersBox.handle_binop(obj, regs) }
  "compare" => {
    local r = match kind {
      "Eq" => { if a == b { 1 } else { 0 } }
      "Ne" => { if a != b { 1 } else { 0 } }
      "Lt" => { if a < b { 1 } else { 0 } }
      "Gt" => { if a > b { 1 } else { 0 } }
      "Le" => { if a <= b { 1 } else { 0 } }
      "Ge" => { if a >= b { 1 } else { 0 } }
      _ => 0
    }
    return map({ a: a, b: b, r: r })
  }
  _ => {}
}
```

**効果**:
- 削減: +2行（行数増加だが可読性向上）
- 可読性: 2重ネスト削除、パターンマッチの明示化
- 保守性: 比較演算子が一覧で見える

---

### 3️⃣ **result_box.hako**: 手動Result型 → @enum Result

**変更前（34行）**:
```hako
box ResultBox {
  _val: Box
  _err: StringBox
  _ok: IntegerBox
  birth() { me._val = null  me._err = ""  me._ok = 0 }
  is_ok() { return me._ok }
  value() { return me._val }
  error() { return me._err }
  unwrap_or(def) { if me._ok == 1 { return me._val } return def }
}

static box Result {
  ok(v) {
    local r = new ResultBox()
    r._val = v
    r._ok = 1
    return r
  }
  err(msg) {
    local r = new ResultBox()
    r._err = msg
    r._ok = 0
    return r
  }
}
```

**変更後（8行）**:
```hako
@enum Result {
  Ok(value)
  Err(error)
}
```

**マクロ展開後の自動生成メソッド**:
- `Result.Ok(value)`: コンストラクタ
- `Result.Err(error)`: コンストラクタ
- `result.is_Ok()`: 判定メソッド
- `result.is_Err()`: 判定メソッド
- `result.as_Ok()`: 値抽出メソッド
- `result.as_Err()`: エラー抽出メソッド

**影響範囲の修正**:
- `mir_vm_min.hako`: `is_ok()` → `is_Ok()`, `value()` → `as_Ok()`
- `phi_decode_box.hako`: `Result.ok()` → `Result.Ok()`, `Result.err()` → `Result.Err()`

**効果**:
- 削減: -26行（76%削減）
- 標準化: @enumマクロによる統一的なsum type実装
- ボイラープレート削除: 手動実装が不要に

---

## 🧪 **テスト結果**

### Day 1: JSON関連テスト
```bash
tools/smokes/v2/run.sh --profile quick --filter "json"
```
- **結果**: 46/46 PASS ✅
- **時間**: 1.88秒

### Day 2: Probe VMテスト
```bash
bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_compare_neg_probe_vm.sh
```
- **結果**: 1/1 PASS ✅

### Day 3: Selfhost総合テスト
```bash
tools/smokes/v2/run.sh --profile quick --filter "selfhost"
```
- **結果**: 255/255 PASS ✅
- **時間**: 18.25秒

---

## 📈 **予測精度の分析**

| 項目 | 予測 | 実績 | 達成率 |
|------|------|------|--------|
| json_inst_encode_box | -32行 | -28行 | 88% |
| minivm_probe | -25行 | +2行 | - |
| result_box | -21行 | -26行 | 124% |
| **総計** | **-78行** | **-52行** | **67%** |

**予測との差異の原因**:
1. **minivm_probe**: 2重ネスト展開により行数増加（可読性は向上）
2. **match式の構文**: ブロック展開により予測より若干長くなった

---

## 💡 **学び**

### マクロ化が高効果なパターン
✅ **深いネストif-else-ifチェーン**（5段以上）
✅ **手動sum type実装**（_tag/_valueパターン）
✅ **命令ディスパッチ**（文字列判定の繰り返し）

### 行数削減より可読性優先すべきケース
⚠️ **2-3段のif-else-if**: @match適用で行数は増えるが、パターンの明示化で保守性向上
⚠️ **比較演算子分岐**: ネスト削除により全パターンが一覧で見える

---

## 🎯 **次のステップ候補**

### Phase 15.14候補: さらなる綺麗綺麗大作戦

Task先生による追加調査で発見した候補：
1. **mir_vm_min.hako（313行）**: 慎重な検討必要（最適化との兼ね合い）
2. **MapBox/ArrayBox初期化パターン**: 既に簡潔、マクロ化不要
3. **ファイルI/O重複**: Phase 2.1で `dep_tree_core.hako` に統合済み

---

## 📚 **関連ドキュメント**

- **@enumマクロ仕様**: [Phase 15.11 README](../phase-15.11/README.md)
- **マクロシステム**: [src/macro/engine.rs](../../../../src/macro/engine.rs)
- **マクロ化候補レポート**: [調査レポート](../../proposals/ideas/improvements/macro-cleanup-candidates.md)（作成予定）

---

## 🎓 **総括**

Phase 15.13では、完全実装済みの@enum/@matchマクロを既存コードに適用し、52行の純削減を達成しました。

**主な成果**:
- コード削減: 52行
- 可読性向上: 深いネスト削除、パターンマッチの明示化
- 保守性向上: 新規命令追加コストの削減
- テスト完全成功: 0エラー

**重要な発見**:
- 行数削減だけでなく、可読性・保守性の向上が本質的な価値
- @matchは3段以上のif-else-ifチェーンで特に効果的
- @enumは手動sum type実装の完全な代替として機能

次のフェーズでは、さらなる綺麗綺麗候補を探索し、段階的な改善を継続します。
