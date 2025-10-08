# Result<T,E> Box 完全設計（既存実装との互換性）

**作成日**: 2025-10-08
**設計者**: Claude Code
**状態**: 設計完了（実装前）

---

## 📋 目次

1. [既存ResultBox分析](#1-既存resultbox分析)
2. [新Result<T,E> Box設計](#2-新resultte-box設計)
3. [完全実装コード](#3-完全実装コード)
4. [エラーハンドリング戦略](#4-エラーハンドリング戦略)
5. [段階移行計画](#5-段階移行計画)
6. [VariantBoxベース版（将来）](#6-variantboxベース版将来)

---

## 1. 既存ResultBox分析

### 1.1 現在の実装（34行）

**場所**: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/vm/boxes/result_box.hako`

```hakorune
box ResultBox {
  _val: Box
  _err: StringBox
  _ok: IntegerBox  // 1=ok, 0=err（BoolBox 非依存）

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

### 1.2 長所

✅ **シンプル・軽量**: 34行で基本機能実現
✅ **BoolBox非依存**: IntegerBoxのみで判定（依存関係最小）
✅ **実証済み**: 5箇所で実用使用中
✅ **読みやすいAPI**: `Result.ok(v)` / `Result.err(msg)`

### 1.3 短所・不足機能

❌ **unwrap()なし**: panicする基本メソッドがない
❌ **expect()なし**: カスタムメッセージ付きpanicがない
❌ **is_err()なし**: エラー判定が `!r.is_ok()` しかない
❌ **map系なし**: `map()`, `and_then()`, `or_else()` 等の関数型メソッドなし
❌ **エラー型固定**: エラーが常に `StringBox` のみ（`Result<T,E>` の `E` が固定）

### 1.4 実際の使用箇所（5箇所確認）

#### phi_decode_box.hako（2箇所）
```hakorune
// Line 109-122
decode_result(seg, prev_bb) {
  local arr = me._decode_array(seg, prev_bb)
  if arr != null {
    local typ = arr.get("type")
    if typ == "ok" { return Result.ok(arr.get("pair")) }
    return Result.err(arr.get("code"))
  }
  local one = me._decode_single(seg)
  local typ2 = one.get("type")
  if typ2 == "ok" { return Result.ok(one.get("pair")) }
  return Result.err(one.get("code"))
}
```

#### selfhost_utils_result_box_vm.sh（テスト）
```hakorune
local a = Result.ok(7)
local b = Result.err("oops")
local s = a.unwrap_or(0) + b.unwrap_or(5)
print(""+s)  // → 12
```

**使用パターン分析**:
- ✅ `Result.ok(value)` / `Result.err(msg)` - 生成に使用
- ✅ `result.unwrap_or(default)` - 安全な値取り出し
- ❌ `unwrap()`, `expect()` - **使われていない（存在しない）**
- ❌ `map()`, `and_then()` - **使われていない（存在しない）**

---

## 2. 新Result<T,E> Box設計

### 2.1 設計原則

1. **後方互換性**: 既存の4メソッド（`is_ok/value/error/unwrap_or`）は**完全互換**
2. **Rust風API**: Rustの `Result<T,E>` に準拠したメソッド追加
3. **段階導入**: MVP → 基本拡張 → 関数型拡張の3段階
4. **VariantBox対応**: Phase 20.6以降、VariantBoxベース版への移行パス確保

### 2.2 API完全リスト

#### 現在の実装（4メソッド）
| メソッド | 型 | 説明 | 互換性 |
|---------|-----|------|--------|
| `is_ok()` | `-> IntegerBox` | 成功判定（1=ok, 0=err） | ✅ 完全互換 |
| `value()` | `-> Box` | 成功値取得（エラー時null） | ✅ 完全互換 |
| `error()` | `-> StringBox` | エラーメッセージ取得 | ✅ 完全互換 |
| `unwrap_or(def)` | `-> Box` | 値取得、エラー時はデフォルト | ✅ 完全互換 |

#### 追加すべきメソッド（Phase 1: 基本拡張）
| メソッド | 型 | 説明 | 優先度 |
|---------|-----|------|--------|
| `is_err()` | `-> IntegerBox` | エラー判定（1=err, 0=ok） | **P0** |
| `unwrap()` | `-> Box` | 値取得、エラー時panic | **P0** |
| `expect(msg)` | `-> Box` | 値取得、エラー時カスタムメッセージでpanic | **P1** |
| `unwrap_err()` | `-> StringBox` | エラー取得、成功時panic | **P2** |

#### 追加すべきメソッド（Phase 2: 関数型拡張）
| メソッド | 型 | 説明 | 優先度 |
|---------|-----|------|--------|
| `map(fn)` | `-> ResultBox` | 成功値を変換 | **P1** |
| `map_err(fn)` | `-> ResultBox` | エラー値を変換 | **P2** |
| `and_then(fn)` | `-> ResultBox` | 成功時に次の処理（flatMap） | **P1** |
| `or_else(fn)` | `-> ResultBox` | エラー時に代替処理 | **P2** |

### 2.3 エラーメッセージの改善

#### 現在の問題
- `value()` がエラー時に `null` を返す → **サイレント失敗**
- エラーが起きた場所の情報なし

#### 改善案
```hakorune
unwrap() {
  if me._ok == 0 {
    local msg = "Result.unwrap() called on Err: " + me._err
    panic(msg)  // ← Hakoruneにpanic()があると仮定
  }
  return me._val
}

expect(custom_msg) {
  if me._ok == 0 {
    local msg = custom_msg + ": " + me._err
    panic(msg)
  }
  return me._val
}
```

**問題**: Hakoruneに `panic()` がない場合の代替案
→ `print("[PANIC] " + msg)` + `return null` で疑似panic

---

## 3. 完全実装コード

### 3.1 MVP版（既存互換 + 基本拡張）

```hakorune
// result_box.hako — Result<T,E> Box (MVP + Phase 1)
// 責務: 処理結果の統一表現（成功値 or エラーメッセージ）
// Version: 2.0 (Phase 1: 基本拡張完了)

box ResultBox {
  _val: Box
  _err: StringBox
  _ok: IntegerBox  // 1=ok, 0=err（BoolBox 非依存）

  // === 初期化 ===
  birth() {
    me._val = null
    me._err = ""
    me._ok = 0
  }

  // === 基本判定（既存互換） ===
  is_ok() {
    return me._ok
  }

  is_err() {
    if me._ok == 1 { return 0 }
    return 1
  }

  // === 値アクセス（既存互換） ===
  value() {
    return me._val
  }

  error() {
    return me._err
  }

  // === unwrap系（新規） ===
  unwrap() {
    if me._ok == 0 {
      local msg = "[PANIC] Result.unwrap() called on Err: " + me._err
      print(msg)
      return null  // Hakoruneにpanic()がないため疑似panic
    }
    return me._val
  }

  expect(custom_msg) {
    if me._ok == 0 {
      local msg = "[PANIC] " + custom_msg + ": " + me._err
      print(msg)
      return null
    }
    return me._val
  }

  unwrap_err() {
    if me._ok == 1 {
      local msg = "[PANIC] Result.unwrap_err() called on Ok: " + me._val
      print(msg)
      return ""
    }
    return me._err
  }

  unwrap_or(def) {
    if me._ok == 1 { return me._val }
    return def
  }

  // === デバッグ ===
  debug() {
    if me._ok == 1 {
      return "Ok(" + me._val + ")"
    }
    return "Err(" + me._err + ")"
  }
}

// === コンストラクタ（既存互換） ===
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

**行数**: 84行（現在34行 → **+50行**）

### 3.2 Phase 2版（関数型拡張）

```hakorune
// result_box.hako — Result<T,E> Box (Phase 2: 関数型拡張)
// Version: 3.0

box ResultBox {
  // ... (上記MVP版のすべてのコード) ...

  // === 関数型メソッド ===

  // map: Ok(v) -> Ok(fn(v)), Err(e) -> Err(e)
  map(fn) {
    if me._ok == 0 {
      return me  // Errはそのまま伝播
    }
    local new_val = fn.call(me._val)  // ← Hakoruneに関数オブジェクトがあると仮定
    return Result.ok(new_val)
  }

  // map_err: Ok(v) -> Ok(v), Err(e) -> Err(fn(e))
  map_err(fn) {
    if me._ok == 1 {
      return me  // Okはそのまま
    }
    local new_err = fn.call(me._err)
    return Result.err(new_err)
  }

  // and_then: Ok(v) -> fn(v), Err(e) -> Err(e)
  and_then(fn) {
    if me._ok == 0 {
      return me
    }
    return fn.call(me._val)  // fn は ResultBox を返す想定
  }

  // or_else: Ok(v) -> Ok(v), Err(e) -> fn(e)
  or_else(fn) {
    if me._ok == 1 {
      return me
    }
    return fn.call(me._err)  // fn は ResultBox を返す想定
  }
}
```

**問題**: Hakoruneに **第一級関数（クロージャ）** がない
→ Phase 2は **Phase 21（関数型機能）** まで保留

**行数**: 120行見込み（MVP 84行 + 関数型 36行）

---

## 4. エラーハンドリング戦略

### 4.1 panic動作の実装

#### 問題
Hakoruneには `panic()` 組み込み関数がない（2025-10-08時点）

#### 解決策（優先順）

**Option 1: 疑似panic（現在）**
```hakorune
unwrap() {
  if me._ok == 0 {
    print("[PANIC] Result.unwrap() called on Err: " + me._err)
    return null  // サイレント失敗
  }
  return me._val
}
```
- ✅ 即座に実装可能
- ❌ プログラムが継続してしまう（真のpanicではない）

**Option 2: PanicBox利用（推奨）**
```hakorune
using "apps/lib/boxes/panic_box.hako" as PanicBox

unwrap() {
  if me._ok == 0 {
    PanicBox.panic("Result.unwrap() called on Err: " + me._err)
  }
  return me._val
}
```
- ✅ 統一的なpanic処理
- ⚠️ PanicBoxの実装が必要（別途実装）

**Option 3: MIR命令拡張（将来）**
```
MIR命令: panic <msg>
```
- ✅ 最も正確
- ❌ MIR16維持原則に反する（Phase 30以降）

### 4.2 エラーメッセージ詳細化

#### 改善前
```
Result.unwrap() called on Err: file not found
```

#### 改善後（スタックトレース風）
```
[PANIC] Result.unwrap() called on Err
  Error: file not found
  Location: apps/selfhost/vm/boxes/phi_decode_box.hako:115
  Suggestion: Use unwrap_or() or expect() for safer error handling
```

**実装**: Phase 25（デバッグ情報）で実現

---

## 5. 段階移行計画

### 5.1 Phase 1: MVP版実装（優先度P0）

**目標**: 既存互換性100% + 基本拡張

**タスク**:
1. ✅ 設計完了（本ドキュメント）
2. ⬜ `is_err()` 実装
3. ⬜ `unwrap()` 実装（疑似panic版）
4. ⬜ `expect(msg)` 実装
5. ⬜ `unwrap_err()` 実装
6. ⬜ `debug()` 実装
7. ⬜ テスト追加（`apps/tests/result_box_extended.hako`）
8. ⬜ スモークテスト追加

**見積もり**: 2-3時間

**成果物**:
- `apps/selfhost/vm/boxes/result_box.hako` (84行、現在34行 → +50行)
- `apps/tests/result_box_extended.hako` (新規、30行)
- `tools/smokes/v2/profiles/quick/selfhost/result_box_extended_vm.sh` (新規)

### 5.2 Phase 2: 関数型拡張（優先度P2）

**前提条件**: Phase 21（関数型機能）完了

**タスク**:
1. ⬜ `map(fn)` 実装
2. ⬜ `map_err(fn)` 実装
3. ⬜ `and_then(fn)` 実装
4. ⬜ `or_else(fn)` 実装
5. ⬜ テスト追加

**見積もり**: 1-2時間（関数型機能実装済み前提）

### 5.3 既存コードへの影響（互換性確認）

#### 影響なしコード（そのまま動作）
```hakorune
// 既存の5箇所すべて互換性維持
local a = Result.ok(7)
local b = Result.err("oops")
local s = a.unwrap_or(0) + b.unwrap_or(5)  // ← 変更不要
```

#### 推奨移行パターン
```hakorune
// 移行前（nullチェック必須）
local r = Result.ok(42)
local v = r.value()
if v == null {
  print("Error!")
}

// 移行後（unwrap使用）
local r = Result.ok(42)
local v = r.unwrap()  // エラー時は自動でパニック
```

### 5.4 ドキュメント更新

#### 必要な更新
1. ⬜ [docs/reference/boxes-system/result-box.md](../../boxes-system/) (新規作成)
2. ⬜ [docs/reference/language/error-handling.md](../../language/) (新規セクション)
3. ⬜ [docs/guides/best-practices.md](../../guides/) (Resultの使い方追加)

---

## 6. VariantBoxベース版（将来）

### 6.1 Phase 20.6以降の統合計画

**VariantBox設計書**: [DESIGN.md](./DESIGN.md)

#### VariantBoxベースのResult<T,E>

```hakorune
// Phase 20.6: VariantBoxベースの新実装
@enum Result {
  Ok(value)
  Err(error)
}

// 自動生成されるコード
static box Result {
  Ok(value) {
    local v = new VariantBox("Ok")
    v.fields.push(value)
    return v
  }

  Err(error) {
    local v = new VariantBox("Err")
    v.fields.push(error)
    return v
  }

  // 判定ヘルパー
  is_ok(r: VariantBox) -> BoolBox {
    return r.is_tag("Ok")
  }

  is_err(r: VariantBox) -> BoolBox {
    return r.is_tag("Err")
  }

  // unwrap系
  unwrap(r: VariantBox) -> Box {
    if !me.is_ok(r) {
      panic("Result.unwrap() called on Err: " + r.field(0))
    }
    return r.field(0)
  }
}
```

### 6.2 移行パス

#### Step 1: 並行運用（Phase 20.6）
```hakorune
// 旧版: apps/selfhost/vm/boxes/result_box.hako
using "apps/selfhost/vm/boxes/result_box.hako" as ResultLegacy

// 新版: @enum Result（VariantBoxベース）
@enum Result {
  Ok(value)
  Err(error)
}

// 旧コードは ResultLegacy.ok() を使用
// 新コードは Result.Ok() を使用
```

#### Step 2: 段階移行（Phase 20.7）
1. 新規コードは `@enum Result` を使用
2. 既存コード5箇所を順次移行
3. テスト追加

#### Step 3: 旧版廃止（Phase 20.8）
1. `result_box.hako` を `result_box.hako.legacy` にリネーム
2. すべてのコードが `@enum Result` 使用
3. レガシー版削除

### 6.3 VariantBox版の利点

| 項目 | 現在のResultBox | VariantBox版 | 判定 |
|------|----------------|--------------|------|
| **型安全性** | ⚠️ 手動判定 | ✅ 型検証 | VariantBox勝利 |
| **網羅性チェック** | ❌ なし | ✅ @matchで可能 | VariantBox勝利 |
| **拡張性** | ⚠️ 固定2状態 | ✅ 任意Variant追加 | VariantBox勝利 |
| **コード量** | ✅ 84行 | ⚠️ 120行見込み | 現在版勝利 |
| **実装済み** | ✅ 動作中 | ❌ Phase 20.6待ち | 現在版勝利 |

---

## 7. まとめ

### 7.1 実装優先順位

#### Phase 1: MVP版（即座実装推奨）
- **優先度**: P0
- **工数**: 2-3時間
- **成果**: 既存互換 + 基本拡張（`is_err/unwrap/expect`）
- **リスク**: 低（既存機能に影響なし）

#### Phase 2: 関数型拡張（Phase 21後）
- **優先度**: P2
- **工数**: 1-2時間（関数型機能前提）
- **成果**: `map/and_then/or_else` 等
- **リスク**: 中（関数型機能の実装必要）

#### Phase 3: VariantBox統合（Phase 20.6後）
- **優先度**: P1
- **工数**: 3-5時間
- **成果**: 型安全性向上、パターンマッチング対応
- **リスク**: 中（5箇所の既存コード移行）

### 7.2 推奨実装順序

```
1. ⬜ Phase 1 MVP版実装（今すぐ）
   ├─ is_err() 追加
   ├─ unwrap() 追加
   ├─ expect(msg) 追加
   ├─ debug() 追加
   └─ テスト追加

2. ⬜ ドキュメント整備
   ├─ result-box.md 作成
   └─ error-handling.md 作成

3. ⬜ Phase 20.6: VariantBox実装待機

4. ⬜ Phase 20.7: VariantBoxベースResult移行
   ├─ @enum Result 実装
   ├─ 既存5箇所を段階移行
   └─ レガシー版廃止

5. ⬜ Phase 21: 関数型拡張（オプション）
   ├─ map/and_then 追加
   └─ テスト追加
```

### 7.3 成功の鍵

✅ **後方互換性100%**: 既存5箇所は一切変更不要
✅ **段階導入**: MVP → 拡張 → VariantBox統合
✅ **80/20ルール**: Phase 1で80%の価値（unwrap/expectが最重要）
✅ **テスト駆動**: 各Phaseでテスト追加
✅ **ドキュメント先行**: 実装前に使い方を明確化

---

**次のステップ**: Phase 1 MVP版の実装（2-3時間）

**承認**: 設計完了、実装開始可能
