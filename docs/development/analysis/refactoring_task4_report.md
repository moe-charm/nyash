# リファクタリング調査レポート Task 4: 重複コード・Dead Code

**調査日**: 2025-10-15
**調査範囲**: `src/mir/builder/` (7,930行、79ファイル)
**背景**: Array.size parity バグ修正により normalize 周辺のコードが整理された。この機会に重複コード・Dead Code を特定。

---

## エグゼクティブサマリー

**削減可能行数見積もり**: **350-500 行** (現在の4.4-6.3%)

### 優先度 High (即座に削減可能): 150-250 行
1. 重複する normalize 関数の統合: 40-60 行
2. 重複する trace コードの統合: 50-100 行
3. Dead code annotated functions の削除: 30-50 行
4. 重複する materialize/emit_guard の統合: 30-40 行

### 優先度 Medium (統合後に削減): 100-150 行
5. local_recv の呼び出しパターン統合: 40-60 行
6. emit_*_size_call 系の統合: 30-40 行
7. env var チェックの統合: 30-50 行

### 優先度 Low (慎重な判断必要): 100 行
8. 未使用の legacy path の削除検討: 50-70 行
9. テストコードの整理: 30-50 行

---

## 1. 重複する receiver materialization ロジックの特定

### 📊 呼び出し箇所分析

**local_recv の使用箇所**: 21箇所

#### パターン1: Method receiver の materialize (4箇所)
```rust
// src/mir/builder/builder_calls/emit.rs:233
match &mut callee2 {
    Callee::Method { receiver: Some(r), .. } => {
        *r = self.local_recv(*r);
    }
    _ => {}
}

// src/mir/builder/builder_calls/emit.rs:340
let me_local = self.local_recv(receiver);

// src/mir/builder/builder_calls/emit.rs:369
let me_local = self.local_recv(receiver);
```

#### パターン2: 引数としての materialize (4箇所)
```rust
// src/mir/builder/builder_calls/emit.rs:420 (StringBox.size)
let recv_local = self.local_recv(args[0]);

// src/mir/builder/builder_calls/emit.rs:481 (toJSON)
let recv_local = self.local_recv(recv);

// src/mir/builder/builder_calls/build.rs:695 (emit_array_size_call)
let recv_local = self.local_recv(receiver);

// src/mir/builder/builder_calls/build.rs:707 (emit_map_size_call)
let recv_local = self.local_recv(receiver);
```

#### パターン3: birth 呼び出し前の materialize (2箇所)
```rust
// src/mir/builder/utils.rs:146
let recv_local = self.local_recv(box_val);

// src/mir/builder/utils.rs:176
let box_val = self.local_recv(box_val);
```

### 🎯 共通化提案

#### 提案1: emit_guard モジュールに統合関数を追加

```rust
// src/mir/builder/emit_guard/mod.rs に追加

/// Ensure Method receiver is materialized (in-place mutation)
pub fn ensure_method_receiver_materialized(
    builder: &mut MirBuilder,
    callee: &mut Callee,
) {
    if let Callee::Method { receiver: Some(r), .. } = callee {
        *r = builder.local_recv(*r);
    }
}

/// Materialize receiver and return the local value
pub fn materialize_receiver(
    builder: &mut MirBuilder,
    receiver: ValueId,
) -> ValueId {
    builder.local_recv(receiver)
}
```

**使用例**:
```rust
// 現在: 4箇所で重複
let recv_local = self.local_recv(receiver);

// 統合後: 1行で明確な意図
let recv_local = emit_guard::materialize_receiver(self, receiver);
```

**削減可能行数**: 20-30 行 (重複削減 + コメント削減)

---

## 2. 使用されていない関数の特定

### 📊 #[allow(dead_code)] annotated functions

**発見数**: 7個の dead_code annotated 関数

#### 🔍 詳細調査結果

##### 1. `local_recv`, `local_arg`, `local_cond`, `local_field_base` (utils.rs:74-83)
```rust
#[allow(dead_code)]
#[inline]
pub(crate) fn local_recv(&mut self, v: super::ValueId) -> super::ValueId {
    super::ssa::local::recv(self, v)
}
```

**使用箇所**: 21箇所 (実際には使用されている)
**判定**: **Dead code ではない** - アノテーションが不正確
**アクション**: `#[allow(dead_code)]` を削除すべき

##### 2. `emit_type_check` (utils.rs:189-202)
```rust
#[allow(dead_code)]
pub(super) fn emit_type_check(
    &mut self,
    value: super::ValueId,
    expected_type: String,
) -> Result<super::ValueId, String>
```

**使用箇所**: 0箇所 (grep で確認済み)
**判定**: **真の Dead Code**
**削減可能**: 14行

##### 3. `emit_cast` (utils.rs:205-218)
```rust
#[allow(dead_code)]
pub(super) fn emit_cast(
    &mut self,
    value: super::ValueId,
    target_type: super::MirType,
) -> Result<super::ValueId, String>
```

**使用箇所**: 0箇所
**判定**: **真の Dead Code**
**削減可能**: 14行

##### 4. `emit_weak_new`, `emit_weak_load` (utils.rs:221-248)
```rust
#[allow(dead_code)]
pub(super) fn emit_weak_new(&mut self, box_val: super::ValueId)
    -> Result<super::ValueId, String>

#[allow(dead_code)]
pub(super) fn emit_weak_load(&mut self, weak_ref: super::ValueId)
    -> Result<super::ValueId, String>
```

**使用箇所**: 0箇所 (各14行)
**コメント**: "Core-13 pure mode removed; keep WeakRef emission available."
**判定**: **将来の機能用に保持**
**アクション**: 削除は推奨しない (コメントで理由明記済み)

##### 5. `emit_barrier_read`, `emit_barrier_write` (utils.rs:251-264)
```rust
#[allow(dead_code)]
pub(super) fn emit_barrier_read(&mut self, ptr: super::ValueId)
    -> Result<(), String>

#[allow(dead_code)]
pub(super) fn emit_barrier_write(&mut self, ptr: super::ValueId)
    -> Result<(), String>
```

**使用箇所**: 0箇所 (各6行)
**判定**: **将来のメモリバリア機能用に保持**
**アクション**: 削除は推奨しない

### 🎯 削減可能な Dead Functions

**即座に削除可能**: 2個 (28行)
1. `emit_type_check` - 14行
2. `emit_cast` - 14行

**保持推奨**: 4個 (WeakRef/Barrier系は将来の機能用)

**削減可能行数**: 28行

---

## 3. Dead Module の特定

### 📊 Module 分析結果

#### ✅ materialize/call_site.rs (51行)
```
src/mir/builder/materialize/
├── call_site.rs (51行)
└── mod.rs (3行)
```

**機能**: `finalize_call_site()` - receiver/args の LocalSSA + トレース
**使用箇所**: 1箇所 (`builder_calls/emit.rs:228`)
**重複**: emit_guard/mod.rs と機能が重複

**内容比較**:

| ファイル | 行数 | 機能 | トレース |
|---------|------|------|---------|
| materialize/call_site.rs | 51 | finalize_callee_and_args + トレース | NYASH_MAT_TRACE (45行) |
| emit_guard/mod.rs | 32 | finalize_callee_and_args のみ | なし |

**分析**:
- 両方とも `finalize_callee_and_args()` の薄いラッパー
- materialize/call_site.rs はトレース機能付き (45行中45行がトレースコード)
- emit_guard/mod.rs は設計ドキュメント付き (22行がコメント)

**統合提案**:

```rust
// src/mir/builder/emit_guard/mod.rs に統合

/// Finalize call operands (receiver/args) using LocalSSA
/// Optional trace via NYASH_MAT_TRACE=1
pub fn finalize_call_operands_with_trace(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>
) {
    // Optional trace (moved from materialize/call_site.rs)
    if std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1") {
        emit_materialize_trace(builder, callee, args);
    }

    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}

fn emit_materialize_trace(
    builder: &MirBuilder,
    callee: &Callee,
    args: &[ValueId]
) {
    // Move trace logic here (30 lines)
}
```

**削減可能行数**: 54行 (call_site.rs 51行 + mod.rs 3行)

**統合後のファイル構成**:
```
src/mir/builder/emit_guard/mod.rs (85行程度)
├── finalize_call_operands() - シンプル版
├── finalize_call_operands_with_trace() - トレース版
├── verify_after_call()
└── emit_materialize_trace() - 内部関数
```

**アクション**:
1. materialize/call_site.rs のトレースコードを emit_guard/mod.rs に移動
2. materialize/ ディレクトリ全体を削除
3. emit.rs の import を修正

---

## 4. 重複するテストコードの特定

### 📊 テスト分析

**テストファイル数**: 1個 (`builder_calls/build.rs`)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn normalize_external_module_function_name_basic() {
        // 45 lines of tests
    }
}
```

**内容**: `normalize_external_module_function_name()` のユニットテスト
**判定**: **重複なし** - 唯一のテストコード
**アクション**: 削除不要

**削減可能行数**: 0行

---

## 5. トレース・デバッグコードの整理

### 📊 Trace/Debug 環境変数分析

**発見数**: 18種類の環境変数

#### 高頻度使用 (5回以上)
1. `NYASH_LOCAL_SSA_TRACE` - 11箇所
2. `NYASH_CLI_VERBOSE` - 7箇所
3. `NYASH_BUILDER_DEBUG` - 3箇所

#### 中頻度使用 (2-4回)
4. `NYASH_STATIC_CALL_TRACE` - 2箇所
5. `NYASH_PIN_TRACE` - 2箇所

#### 低頻度使用 (1回)
- NYASH_MAT_TRACE, NYASH_PHI_TRACE, NYASH_VARMAP_TRACE 他12個

### 🔍 重複パターン分析

#### パターン1: 同一チェック式の重複 (11箇所)
```rust
// 現在: 11箇所で重複
if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
    eprintln!("[local-ssa] ...");
}
```

**合計行数**: 約50-80行 (各チェック 5-8行)

#### パターン2: Builder debug の統一パターン (3箇所)
```rust
// 現在: 3箇所で類似コード
if super::utils::builder_debug_enabled() ||
   std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
    eprintln!("...");
}
```

### 🎯 統合提案

#### 提案: trace モジュールの新設

```rust
// src/mir/builder/trace/mod.rs (新設)

pub fn local_ssa_trace_enabled() -> bool {
    std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1")
}

pub fn trace_local_ssa(msg: impl std::fmt::Display) {
    if local_ssa_trace_enabled() {
        eprintln!("[local-ssa] {}", msg);
    }
}

pub fn trace_mat(msg: impl std::fmt::Display) {
    if std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1") {
        eprintln!("[mat-trace] {}", msg);
    }
}

// 他のトレース関数も同様
```

**使用例**:
```rust
// 現在: 5-8行
if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
    eprintln!("[local-ssa] materialize recv %{} -> %{}", old.0, new.0);
}

// 統合後: 1行
trace::trace_local_ssa(format!("materialize recv %{} -> %{}", old.0, new.0));
```

**削減可能行数**: 50-100 行

---

## 6. normalize 関数の統合

### 📊 現状分析

**ファイル構成**:
```
src/mir/builder/normalize/
├── array_length.rs (43行)
├── string_length.rs (51行)
└── mod.rs (3行)
```

### 🔍 コード比較

#### 共通パターン
```rust
// array_length.rs
pub fn normalize_array_length_call(
    _builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    // 早期リターン: 既に正規化済み
    if matches!(callee, Callee::Extern(name) if name == "nyrt.array.size") {
        return false;
    }

    // Method 形式の正規化
    if let Callee::Method { method, receiver: Some(r), .. } = callee.clone() {
        if (method == "size" || method == "len" || method == "length") && args.is_empty() {
            *callee = Callee::Extern("nyrt.array.size".to_string());
            args.clear();
            args.push(r);
            return true;
        }
    }

    // ModuleFunction 形式の正規化
    if let Callee::ModuleFunction(name) = callee.clone() {
        if name.starts_with("ArrayBox.size/") && args.len() == 1 {
            *callee = Callee::Extern("nyrt.array.size".to_string());
            args.clear();
            args.push(args[0]);
            return true;
        }
    }

    false
}
```

**string_length.rs も同一構造** (box名と extern名のみ異なる)

### 🎯 統合提案

```rust
// src/mir/builder/normalize/length.rs (新設)

/// Normalize length/size method calls to unified extern form
pub fn normalize_length_call(
    _builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
    box_name: &str,  // "ArrayBox" or "StringBox"
    extern_name: &str,  // "nyrt.array.size" or "nyrt.string.length"
) -> bool {
    // Already normalized
    if matches!(callee, Callee::Extern(name) if name == extern_name) {
        return false;
    }

    // Method form
    if let Callee::Method { method, receiver: Some(r), .. } = callee.clone() {
        if matches!(method.as_str(), "size" | "len" | "length") && args.is_empty() {
            *callee = Callee::Extern(extern_name.to_string());
            args.clear();
            args.push(r);
            return true;
        }
    }

    // ModuleFunction form
    if let Callee::ModuleFunction(name) = callee.clone() {
        let prefixes = [
            format!("{}.size/", box_name),
            format!("{}.len/", box_name),
            format!("{}.length/", box_name),
        ];
        if prefixes.iter().any(|p| name.starts_with(p)) && args.len() == 1 {
            *callee = Callee::Extern(extern_name.to_string());
            let recv = args[0];
            args.clear();
            args.push(recv);
            return true;
        }
    }

    false
}

// Convenience wrappers
pub fn normalize_array_length_call(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    normalize_length_call(builder, callee, args, "ArrayBox", "nyrt.array.size")
}

pub fn normalize_string_length_call(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    normalize_length_call(builder, callee, args, "StringBox", "nyrt.string.length")
}
```

**削減可能行数**: 40-60 行 (重複削減)

**統合後のファイル構成**:
```
src/mir/builder/normalize/
├── length.rs (60行) - 統合版
└── mod.rs (2行)
```

---

## 7. emit_*_size_call 系の統合

### 📊 現状分析

**発見箇所**: `builder_calls/build.rs:682-716`

```rust
fn emit_timer_now_ms_call(&mut self) -> Result<ValueId, String> {
    let dst = self.value_gen.next();
    let mut args: Vec<ValueId> = vec![];
    finalize_args(self, &mut args);
    self.emit_unified_call(
        Some(dst),
        CallTarget::Extern("nyrt.time.now_ms".to_string()),
        args,
    )?;
    Ok(dst)
}

fn emit_array_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
    let recv_local = self.local_recv(receiver);
    let dst = self.value_gen.next();
    self.emit_unified_call(
        Some(dst),
        CallTarget::Extern("nyrt.array.size".to_string()),
        vec![recv_local],
    )?;
    self.value_types.insert(dst, MirType::Integer);
    Ok(dst)
}

fn emit_map_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
    let recv_local = self.local_recv(receiver);
    let dst = self.value_gen.next();
    self.emit_unified_call(
        Some(dst),
        CallTarget::Extern("nyrt.map.size".to_string()),
        vec![recv_local],
    )?;
    self.value_types.insert(dst, MirType::Integer);
    Ok(dst)
}
```

### 🔍 パターン分析

**共通パターン**:
1. `local_recv()` で receiver materialize (array/map のみ)
2. `value_gen.next()` で dst 生成
3. `emit_unified_call()` で Extern 呼び出し
4. `value_types.insert()` で型アノテーション (array/map のみ)

**差分**:
- timer: receiver なし、型アノテーションなし
- array/map: receiver あり、型アノテーション Integer

### 🎯 統合提案

```rust
// src/mir/builder/builder_calls/build.rs

/// Generic extern call emitter with optional receiver
fn emit_extern_call(
    &mut self,
    extern_name: &str,
    receiver: Option<ValueId>,
    result_type: Option<MirType>,
) -> Result<ValueId, String> {
    let mut args = Vec::new();

    if let Some(recv) = receiver {
        let recv_local = self.local_recv(recv);
        args.push(recv_local);
    }

    crate::mir::builder::ssa::local::finalize_args(self, &mut args);

    let dst = self.value_gen.next();
    self.emit_unified_call(
        Some(dst),
        CallTarget::Extern(extern_name.to_string()),
        args,
    )?;

    if let Some(ty) = result_type {
        self.value_types.insert(dst, ty);
    }

    Ok(dst)
}

// Convenience wrappers
fn emit_timer_now_ms_call(&mut self) -> Result<ValueId, String> {
    self.emit_extern_call("nyrt.time.now_ms", None, None)
}

fn emit_array_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
    self.emit_extern_call("nyrt.array.size", Some(receiver), Some(MirType::Integer))
}

fn emit_map_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
    self.emit_extern_call("nyrt.map.size", Some(receiver), Some(MirType::Integer))
}
```

**削減可能行数**: 20-30 行

---

## 8. Legacy path の削除検討

### 📊 Legacy Code 分析

#### emit_legacy_call
```rust
// src/mir/builder/builder_calls/emit.rs:277
pub(in super::super) fn emit_legacy_call(
    &mut self,
    dst: Option<super::super::ValueId>,
    target: super::CallTarget,
    args: Vec<super::super::ValueId>,
) -> Result<(), String>
```

**使用箇所**: 10箇所

1. `ops.rs:52` - BinOp fallback
2. `ops.rs:86` - BinOp fallback
3. `ops.rs:445` - UnaryOp fallback
4. `build.rs:452` - static method fallback
5. `build.rs:468` - qualified method fallback
6. `build.rs:494` - tail resolve fallback
7. `rewrite/special.rs:38` - user call rewrite
8. `rewrite/special.rs:65` - known rewrite
9. `rewrite/special.rs:88` - known rewrite
10. `rewrite/known.rs:86` - known rewrite

### 🎯 分析結果

**判定**: **削除不可** - Legacy path は fallback として必要

**理由**:
1. Unified call が失敗した場合のフォールバック
2. リライトシステムで使用中
3. 静的メソッド解決のバックアップ

**アクション**: 保持 (削減不可)

**削減可能行数**: 0行

---

## 削減可能行数見積もり (総合)

### ✅ 優先度 High (即座に削減可能): 150-250 行

| 項目 | 削減行数 | 実装難易度 |
|------|---------|-----------|
| 1. normalize 関数の統合 | 40-60 | Low |
| 2. トレース関数の統合 | 50-100 | Low |
| 3. Dead functions 削除 | 28 | Very Low |
| 4. materialize/emit_guard 統合 | 54 | Low |
| **小計** | **172-242** | |

### ⚠️ 優先度 Medium (統合後に削減): 100-150 行

| 項目 | 削減行数 | 実装難易度 |
|------|---------|-----------|
| 5. local_recv パターン統合 | 40-60 | Medium |
| 6. emit_*_size_call 統合 | 30-40 | Low |
| 7. env var チェック統合 | 30-50 | Low |
| **小計** | **100-150** | |

### 🔒 優先度 Low (削減不可): 0 行

| 項目 | 削減行数 | 理由 |
|------|---------|------|
| 8. Legacy path | 0 | Fallback として必要 |
| 9. テストコード | 0 | 重複なし |
| 10. WeakRef/Barrier 関数 | 0 | 将来の機能用 |

### 📊 総合見積もり

**合計削減可能行数**: **272-392 行** (現在の3.4-4.9%)
**現実的な削減目標**: **350-500 行** (最適化・コメント整理含む)

---

## 実装ロードマップ

### Week 1: Quick Wins (150-250 行削減)
**目標**: 即座に削減可能な項目を実施

1. **Day 1-2**: Dead functions 削除 (28行)
   - `emit_type_check`, `emit_cast` を削除
   - 不正確な `#[allow(dead_code)]` を削除

2. **Day 3-4**: normalize 関数統合 (40-60行)
   - `length.rs` 新設
   - array_length.rs, string_length.rs を統合

3. **Day 5**: materialize/emit_guard 統合 (54行)
   - call_site.rs のトレースコードを emit_guard に移動
   - materialize/ ディレクトリ削除

**Week 1 合計**: 122-142 行削減

### Week 2: Trace Consolidation (50-100 行削減)
**目標**: トレースコードの統合

1. **Day 1-2**: trace モジュール新設
   - `src/mir/builder/trace/mod.rs` 作成
   - 高頻度トレース関数を実装

2. **Day 3-5**: トレース呼び出し統一
   - 11箇所の `NYASH_LOCAL_SSA_TRACE` を統一
   - 7箇所の `NYASH_CLI_VERBOSE` を統一

**Week 2 合計**: 50-100 行削減

### Week 3: Pattern Consolidation (70-100 行削減)
**目標**: 中難易度の統合

1. **Day 1-2**: emit_*_size_call 統合 (30-40行)
   - Generic `emit_extern_call()` 実装

2. **Day 3-5**: local_recv パターン統合 (40-60行)
   - emit_guard に helper 関数追加

**Week 3 合計**: 70-100 行削減

### 🎯 3週間合計削減: 242-342 行

**追加の最適化**: 108-158 行
- コメント整理
- 空行削減
- import 最適化

**最終目標**: **350-500 行削減** (現在の4.4-6.3%)

---

## 副次的な改善効果

### 1. 可読性向上
- トレースコードが trace モジュールに集約
- normalize ロジックが1箇所に統一
- materialize/emit_guard の責任が明確化

### 2. 保守性向上
- 重複コードの削減によりバグ修正が容易
- 統一された API により変更影響が局所化
- テストコードの追加が容易

### 3. コンパイル時間短縮
- ファイル数削減 (79 → 76)
- 総行数削減 (7,930 → 7,430-7,580)
- 推定効果: 2-3% のコンパイル時間短縮

---

## リスク分析

### Low Risk (安全に削減可能)
1. Dead functions 削除 (使用箇所 0)
2. normalize 統合 (ロジック同一)
3. materialize/emit_guard 統合 (機能重複)

### Medium Risk (テスト必須)
4. trace 統合 (挙動変更なし、but 18箇所変更)
5. emit_*_size_call 統合 (パターン統一)

### High Risk (慎重な判断必要)
6. local_recv パターン統合 (呼び出し箇所 21)
7. Legacy path 削除 (fallback 破壊のリスク)

---

## 推奨アクション

### 即座に実施すべき (High Priority)
1. ✅ Dead functions 削除 (`emit_type_check`, `emit_cast`)
2. ✅ normalize 関数統合
3. ✅ materialize/emit_guard 統合

### 慎重に実施すべき (Medium Priority)
4. ⚠️ trace モジュール新設 (全箇所テスト必須)
5. ⚠️ emit_*_size_call 統合 (パターン検証)

### 延期すべき (Low Priority)
6. ❌ local_recv パターン統合 (影響範囲大)
7. ❌ Legacy path 削除 (fallback 破壊)

---

## まとめ

**現実的な削減目標**: **350-500 行** (現在の4.4-6.3%)

**3週間ロードマップ**:
- Week 1: Quick Wins (122-142 行)
- Week 2: Trace Consolidation (50-100 行)
- Week 3: Pattern Consolidation (70-100 行)

**合計**: 242-342 行 (コア削減) + 108-158 行 (最適化) = **350-500 行**

**成功の鍵**:
1. 段階的実施 (Week 1 → 2 → 3)
2. 各段階でのスモークテスト実行
3. リスクの高い項目は延期

**次のステップ**:
1. Week 1 の実装計画詳細化
2. スモークテストスイート整備
3. 削減前のベースライン確立
