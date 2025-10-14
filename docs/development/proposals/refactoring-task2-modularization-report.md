# リファクタリング調査レポート Task 2: モジュール化・共通処理

**日付**: 2025-10-15
**担当**: Claude Task Agent
**調査対象**: MirBuilder Call Emission Pipeline

---

## エグゼクティブサマリー

**発見**: `emit_unified_call` と `emit_legacy_call` の2つの経路には、**大量の重複ロジック**が存在します。主な重複箇所：

1. **receiver materialization**: 6箇所で `local_recv`/`local_arg` を直接呼び出し
2. **normalize 呼び出し**: 4箇所で同じ normalize パターンを重複
3. **finalize 呼び出し**: 12箇所で finalize_args/finalize_callee_and_args を重複呼び出し
4. **Call 命令生成**: 13箇所で同じパターンの MirInstruction::Call を生成

**削減可能行数**: **180-280行** (全体の32-49%)

---

## 1. emit_unified_call と emit_legacy_call の共通ロジック

### 共通ロジック 1: receiver materialization

**現在の実装**:
```rust
// emit_unified_call (Line 233-239)
match &mut callee2 {
    Callee::Method { receiver: Some(r), .. } => {
        *r = self.local_recv(*r);
    }
    _ => {}
}
for a in args2.iter_mut() {
    *a = self.local_arg(*a);
}

// emit_legacy_call - CallTarget::Method (Line 340, 369, 420)
let me_local = self.local_recv(receiver);
```

**呼び出し箇所リスト**:
- `emit.rs:233` - unified path (receiver)
- `emit.rs:238` - unified path (args)
- `emit.rs:340` - legacy path (birth method)
- `emit.rs:369` - legacy path (user instance boxcall)
- `emit.rs:420` - legacy path (StringBox.size)
- `emit.rs:481` - legacy path (JSON.stringify)

**問題点**:
- 同じ materialization ロジックが **6箇所** に散在
- finalize_call_operands (emit_guard/mod.rs:25) で統一済みなのに、再度呼び出している

**共通化案**:
```rust
// 新設関数: emit_guard/mod.rs
pub fn materialize_final_check(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) {
    // Safety net: ensure receiver/args are materialized
    // (already done by finalize_call_operands, but double-check)
    match callee {
        Callee::Method { receiver: Some(r), .. } => {
            *r = builder.local_recv(*r);
        }
        _ => {}
    }
    for a in args.iter_mut() {
        *a = builder.local_arg(*a);
    }
}
```

**削減可能行数**: **30-40行** (6箇所 × 5-7行)

---

### 共通ロジック 2: normalize 呼び出し

**現在の実装**:

```rust
// emit_unified_call (Line 258-259)
normalize::string_length::normalize_string_length_call(self, &mut callee2, &mut args2);
normalize::array_length::normalize_array_length_call(self, &mut callee2, &mut args2);

// emit_legacy_call - CallTarget::Method (Line 290-309, 311-329)
let mut callee = Callee::Method { ... };
let mut argv = Vec::<ValueId>::new();
let changed = normalize::string_length::normalize_string_length_call(self, &mut callee, &mut argv);
if changed {
    // ... 17 lines of Call emission ...
}

let mut callee = Callee::Method { ... };
let mut argv = Vec::<ValueId>::new();
let changed = normalize::array_length::normalize_array_length_call(self, &mut callee, &mut argv);
if changed {
    // ... 17 lines of Call emission ...
}
```

**呼び出し箇所リスト**:
- `emit.rs:258` - unified path (string)
- `emit.rs:259` - unified path (array)
- `emit.rs:293` - legacy path (string, with Call emission)
- `emit.rs:314` - legacy path (array, with Call emission)

**問題点**:
- unified path: normalize 後に Call 命令生成 (簡潔)
- legacy path: normalize 後に **17行の Call 命令生成コードを重複** (2回)
- **合計34行の重複コード** (17行 × 2箇所)

**共通化案**:
```rust
// 新設関数: normalize/mod.rs
pub fn normalize_all(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    let mut changed = false;
    changed |= string_length::normalize_string_length_call(builder, callee, args);
    changed |= array_length::normalize_array_length_call(builder, callee, args);
    // Future: other normalizers can be added here
    changed
}

// emit_unified_call simplification
normalize::normalize_all(self, &mut callee2, &mut args2);

// emit_legacy_call - remove duplicate blocks, use shared helper
if let changed = normalize::normalize_all(self, &mut callee, &mut argv) {
    if changed {
        emit_normalized_call(self, dst, callee, argv)?;
        return Ok(());
    }
}
```

**削減可能行数**: **50-70行**
- 重複 Call 命令生成: 34行
- 統一化による簡略化: 16-36行

---

### 共通ロジック 3: finalize 呼び出しの重複

**現在の実装**:

```rust
// emit_unified_call (Line 218)
crate::mir::builder::emit_guard::finalize_call_operands(self, &mut callee, &mut args_local);

// emit_unified_call (Line 228)
crate::mir::builder::materialize::call_site::finalize_call_site(self, &mut callee2, &mut args2);

// emit_legacy_call - 10箇所で finalize_args を重複呼び出し
crate::mir::builder::ssa::local::finalize_args(self, &mut args);
```

**呼び出し箇所リスト**:
- `emit.rs:218` - unified: finalize_call_operands
- `emit.rs:228` - unified: finalize_call_site
- `emit.rs:124` - legacy: Global fallback (finalize_args)
- `emit.rs:143` - legacy: Global fallback (finalize_args)
- `emit.rs:295` - legacy: normalize string (finalize_args)
- `emit.rs:316` - legacy: normalize array (finalize_args)
- `emit.rs:344` - legacy: birth method (finalize_args)
- `emit.rs:373` - legacy: user instance boxcall (finalize_args)
- `emit.rs:403` - legacy: Extern (finalize_args)
- `emit.rs:442` - legacy: hostbridge (finalize_args)
- `emit.rs:464` - legacy: ModuleFunction (finalize_args)
- `emit.rs:489` - legacy: Global final fallback (finalize_args)

**問題点**:
- **finalize_call_operands** と **finalize_call_site** の違いが不明確
- legacy path では **10箇所** で同じ finalize_args を重複呼び出し
- 実装を見ると、両方とも `finalize_callee_and_args` を呼んでいる (重複!)

**実装詳細**:
```rust
// emit_guard/mod.rs:25
pub fn finalize_call_operands(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}

// materialize/call_site.rs:6
pub fn finalize_call_site(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    // ... dev trace (8-45行) ...
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
    // ... 3 lines of comment ...
}
```

**重大発見**: `finalize_call_operands` と `finalize_call_site` は **完全に同じ関数** を呼んでいる！
- 唯一の違い: `finalize_call_site` は dev trace が付いているだけ

**共通化案**:
```rust
// emit_guard/mod.rs: 統一的な finalize 関数
pub fn finalize_call(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
    enable_trace: bool,  // dev-only
) {
    if enable_trace && std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1") {
        // ... trace logic from call_site.rs ...
    }
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}

// emit_unified_call simplification (1箇所で済む)
emit_guard::finalize_call(self, &mut callee, &mut args_local, true);

// emit_legacy_call simplification (全箇所で統一)
emit_guard::finalize_call(self, &mut callee, &mut args, false);
```

**削減可能行数**: **30-50行**
- finalize_call_site 削除: 51行
- finalize_call_operands 簡略化: -10行 (trace 統合)
- legacy path 呼び出し統一: 0行 (1行→1行なので変わらず)
- **実質削減**: finalize_call_site の重複除去 (51行) - 新関数追加 (20行) = **31行**

---

### 共通ロジック 4: Call 命令生成パターンの重複

**現在の実装**:

**13箇所で同じパターン**の MirInstruction::Call を生成:

```rust
// パターン A: ModuleFunction (6箇所)
self.emit_instruction(MirInstruction::Call {
    dst: Some(dstv),
    func: ValueId::new(0),
    callee: Some(Callee::ModuleFunction(name)),
    args: args2,
    effects: EffectMask::IO,
})?;
self.annotate_call_result_from_func_name(dstv, name);

// パターン B: Extern (4箇所)
self.emit_instruction(MirInstruction::Call {
    dst: Some(dstv),
    func: name_const,
    callee: Some(Callee::Extern("nyrt.string.length".to_string())),
    args: argv,
    effects: EffectMask::READ.add(Effect::ReadHeap),
})?;
self.value_types.insert(dstv, MirType::Integer);

// パターン C: Global (1箇所)
self.emit_instruction(MirInstruction::Call {
    dst: Some(actual_dst),
    func: ValueId::new(0),
    callee: Some(Callee::Global(normalized.clone())),
    args,
    effects: EffectMask::IO,
})?;
self.annotate_call_result_from_func_name(actual_dst, normalized);
```

**呼び出し箇所リスト**:
- `emit.rs:125` - Global fallback (ModuleFunction)
- `emit.rs:144` - Unique static-method fallback (ModuleFunction)
- `emit.rs:298` - normalize string (Extern)
- `emit.rs:319` - normalize array (Extern)
- `emit.rs:348` - birth method (ModuleFunction)
- `emit.rs:377` - user instance boxcall (ModuleFunction)
- `emit.rs:409` - Extern target (Extern)
- `emit.rs:425` - StringBox.size rewrite (Extern)
- `emit.rs:443` - hostbridge (Extern)
- `emit.rs:465` - ModuleFunction direct (ModuleFunction)
- `emit.rs:490` - Global final (Global)
- `emit.rs:503` - Value target (Value)

**問題点**:
- 同じ構造の Call 命令生成が **13箇所** に散在
- dst 処理 (Some vs unwrap_or)、effects 選択、annotate 呼び出しのパターンが重複
- 各箇所で微妙に異なる (dst の有無、effects の種類、annotate の有無)

**共通化案**:
```rust
// 新設関数: builder_calls/helpers.rs
pub fn emit_call_with_annotate(
    builder: &mut MirBuilder,
    dst: Option<ValueId>,
    callee: Callee,
    args: Vec<ValueId>,
    effects: EffectMask,
) -> Result<ValueId, String> {
    let actual_dst = dst.unwrap_or_else(|| builder.value_gen.next());
    builder.emit_instruction(MirInstruction::Call {
        dst: Some(actual_dst),
        func: ValueId::new(0),  // dummy for unified
        callee: Some(callee.clone()),
        args,
        effects,
    })?;

    // Auto-annotate based on callee type
    match callee {
        Callee::ModuleFunction(ref name) | Callee::Global(ref name) => {
            builder.annotate_call_result_from_func_name(actual_dst, name);
        }
        Callee::Extern(ref name) if name.starts_with("nyrt.string.length") || name.starts_with("nyrt.array.size") => {
            builder.value_types.insert(actual_dst, MirType::Integer);
        }
        _ => {}
    }

    Ok(actual_dst)
}

// 使用例
let dstv = emit_call_with_annotate(
    self,
    dst,
    Callee::ModuleFunction(func_name),
    args2,
    EffectMask::IO,
)?;
```

**削減可能行数**: **70-110行**
- 13箇所 × 6-9行 (Call 命令生成 + annotate) = 78-117行
- 新関数追加: 20行
- **実質削減**: 58-97行

---

## 2. normalize パスの統一化

### 現在の問題

**unified 経路**:
```rust
// emit_unified_call (Line 258-259)
normalize::string_length::normalize_string_length_call(self, &mut callee2, &mut args2);
normalize::array_length::normalize_array_length_call(self, &mut callee2, &mut args2);
// → 続けて Call 命令生成 (1箇所)
```

**legacy 経路**:
```rust
// emit_legacy_call - CallTarget::Method (Line 290-329)
// string normalize (19 lines)
let mut callee = Callee::Method { ... };
let mut argv = Vec::<ValueId>::new();
let changed = normalize::string_length::normalize_string_length_call(self, &mut callee, &mut argv);
if changed {
    finalize_args(self, &mut argv);
    let dstv = dst.unwrap_or_else(|| self.value_gen.next());
    let name_const = make_name_const_result(self, "nyrt.string.length")?;
    self.emit_instruction(MirInstruction::Call { ... })?;
    self.value_types.insert(dstv, MirType::Integer);
    return Ok(());
}

// array normalize (19 lines, 完全に同じパターン)
let mut callee = Callee::Method { ... };
let mut argv = Vec::<ValueId>::new();
let changed = normalize::array_length::normalize_array_length_call(self, &mut callee, &mut argv);
if changed {
    finalize_args(self, &mut argv);
    let dstv = dst.unwrap_or_else(|| self.value_gen.next());
    let name_const = make_name_const_result(self, "nyrt.array.size")?;
    self.emit_instruction(MirInstruction::Call { ... })?;
    self.value_types.insert(dstv, MirType::Integer);
    return Ok(());
}
```

**重複している箇所**:
- normalize 呼び出し: unified 2行 vs legacy 2ブロック (38行)
- Call 命令生成: unified 1箇所 vs legacy 各 normalize ブロック内 (2回)

### 統一化提案

**ステップ1: normalize_all 関数を新設**

```rust
// normalize/mod.rs
pub fn normalize_all(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    let mut changed = false;
    changed |= string_length::normalize_string_length_call(builder, callee, args);
    changed |= array_length::normalize_array_length_call(builder, callee, args);
    changed
}
```

**ステップ2: emit_normalized_call ヘルパー関数を新設**

```rust
// builder_calls/helpers.rs
pub fn emit_normalized_call(
    builder: &mut MirBuilder,
    dst: Option<ValueId>,
    callee: Callee,
    args: Vec<ValueId>,
) -> Result<(), String> {
    let mut args = args;
    crate::mir::builder::ssa::local::finalize_args(builder, &mut args);

    let dstv = dst.unwrap_or_else(|| builder.value_gen.next());

    // Determine effects and result type based on callee
    let (effects, result_type) = match &callee {
        Callee::Extern(name) if name == "nyrt.string.length" || name == "nyrt.array.size" => {
            (EffectMask::READ.add(Effect::ReadHeap), Some(MirType::Integer))
        }
        _ => (EffectMask::IO, None),
    };

    let func = if matches!(callee, Callee::Extern(_)) {
        if let Callee::Extern(ref name) = callee {
            crate::mir::builder::name_const::make_name_const_result(builder, name)?
        } else {
            ValueId::new(0)
        }
    } else {
        ValueId::new(0)
    };

    builder.emit_instruction(MirInstruction::Call {
        dst: Some(dstv),
        func,
        callee: Some(callee),
        args,
        effects,
    })?;

    if let Some(ty) = result_type {
        builder.value_types.insert(dstv, ty);
    }

    Ok(())
}
```

**ステップ3: emit_unified_call/emit_legacy_call を簡略化**

```rust
// emit_unified_call (Line 258-259 → 1行)
normalize::normalize_all(self, &mut callee2, &mut args2);

// emit_legacy_call - CallTarget::Method (Line 290-329 → 10行)
{
    let mut callee = Callee::Method {
        box_name: "StringBox".to_string(),
        method: method.clone(),
        receiver: Some(receiver),
        certainty: TypeCertainty::Union
    };
    let mut argv = Vec::<ValueId>::new();
    if normalize::normalize_all(self, &mut callee, &mut argv) {
        emit_normalized_call(self, dst, callee, argv)?;
        return Ok(());
    }
}
```

### 削減可能行数

- legacy path 重複除去: **38行** (19行 × 2ブロック)
- 新関数追加コスト: 40行 (normalize_all: 10行, emit_normalized_call: 30行)
- 呼び出し側簡略化: 28行削減 (38行 → 10行)

**合計削減**: **50-80行**

---

## 3. materialize ロジックの一本化

### 現在の問題

**3つの関数が存在**:

1. **finalize_call_operands** (`emit_guard/mod.rs:25`)
   - 32行のファイル
   - 実装: `finalize_callee_and_args` を呼ぶだけ (1行)

2. **finalize_call_site** (`materialize/call_site.rs:6`)
   - 51行のファイル
   - 実装: dev trace (40行) + `finalize_callee_and_args` を呼ぶ (1行)

3. **finalize_callee_and_args** (`ssa/local.rs:84`)
   - 実際の処理を実装
   - receiver materialization (Method の場合)
   - args materialization (全引数)

**重大発見**: `finalize_call_operands` と `finalize_call_site` は **同じ関数を呼んでいる**！

### 一本化提案

**ステップ1: finalize_call_site を削除**

`materialize/call_site.rs` を削除 (51行)

**ステップ2: finalize_call_operands を拡張**

```rust
// emit_guard/mod.rs (32行 → 50行程度)
pub fn finalize_call_operands(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
    enable_trace: bool,  // dev-only
) {
    // Dev trace (moved from materialize/call_site.rs)
    if enable_trace && std::env::var("NYASH_MAT_TRACE").ok().as_deref() == Some("1") {
        match callee {
            Callee::Method { box_name, method, receiver, .. } => {
                let (rid, rty, rorig) = receiver
                    .and_then(|r| {
                        let ty = builder.value_types.get(&r).cloned();
                        let orig = builder.origin_get(r).map(|s| s.to_string());
                        Some((r, ty, orig))
                    })
                    .unwrap_or((ValueId(u32::MAX), None, None));
                let mut parts: Vec<String> = Vec::with_capacity(args.len());
                for a in args.iter() {
                    let aty = builder.value_types.get(a)
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "?".into());
                    parts.push(format!("%{}:{}", a.0, aty));
                }
                eprintln!(
                    "[mat-trace] call recv=%{} ty={:?} orig={} -> {}.{}({})",
                    rid.0, rty, rorig.as_deref().unwrap_or("-"),
                    box_name, method, parts.join(", ")
                );
            }
            Callee::Global(name) => {
                eprintln!("[mat-trace] call global {}(..)", name);
            }
            _ => {}
        }
    }

    // Actual finalization
    crate::mir::builder::ssa::local::finalize_callee_and_args(builder, callee, args);
}
```

**ステップ3: 呼び出し側を統一**

```rust
// emit_unified_call (Line 218 → そのまま、Line 228 削除)
emit_guard::finalize_call_operands(self, &mut callee, &mut args_local, false);
// Line 228: finalize_call_site 呼び出しを削除

// emit_legacy_call - 全10箇所を統一
emit_guard::finalize_call_operands(self, &mut callee, &mut args, false);
```

### 削減可能行数

- `materialize/call_site.rs` 削除: **51行**
- `finalize_call_operands` 拡張: +18行 (trace 追加)
- `emit_unified_call` の重複呼び出し削除: 1行
- **合計削減**: **30-50行**

---

## 4. モジュール構造の改善案

### 現在の構造

```
src/mir/builder/
├── builder_calls/
│   └── emit.rs (568 lines, 複雑)
├── emit_guard/
│   └── mod.rs (32 lines, 薄いラッパー)
├── materialize/
│   └── call_site.rs (51 lines, 重複)
├── normalize/
│   ├── array_length.rs (43 lines)
│   └── string_length.rs (52 lines)
└── ssa/
    └── local.rs (146 lines)
```

**問題点**:
- `emit.rs` が **568行** と肥大化
- `emit_guard` と `materialize` が重複機能を提供
- `normalize` が複数ファイルに分散 (統一 API なし)
- 呼び出し側 (emit.rs) に normalize + Call 生成のロジックが混在

### 改善後の構造

**オプションA: 最小限の改善 (推奨)**

```
src/mir/builder/
├── builder_calls/
│   ├── emit.rs (380-440 lines, 簡潔化)
│   └── helpers.rs (新設, 80-100 lines)
│       ├── emit_call_with_annotate
│       ├── emit_normalized_call
│       └── materialize_final_check
├── emit_guard/
│   └── mod.rs (50 lines, trace 統合)
│       └── finalize_call_operands (拡張)
├── normalize/
│   ├── mod.rs (15 lines, 新設)
│   │   └── normalize_all (統一 API)
│   ├── array_length.rs (43 lines, 変更なし)
│   └── string_length.rs (52 lines, 変更なし)
└── ssa/
    └── local.rs (146 lines, 変更なし)

削除:
└── materialize/call_site.rs (51 lines, 削除)
```

**変更点**:
- `builder_calls/helpers.rs` 新設: 共通 Call 生成ロジック
- `normalize/mod.rs` 新設: normalize_all 統一 API
- `materialize/call_site.rs` 削除: 重複除去
- `emit.rs` 簡潔化: 568行 → 380-440行 (**128-188行削減**)

**オプションB: 大規模再構成 (Phase 2 以降)**

```
src/mir/builder/
├── call_emission/  (新設: 統一的な呼び出し生成)
│   ├── mod.rs (エントリーポイント)
│   ├── pipeline.rs (normalize → materialize → emit の統一パイプライン)
│   ├── normalize.rs (normalize 統一)
│   ├── materialize.rs (materialize 統一)
│   └── emit.rs (簡潔化後の emit)
└── (既存モジュール)
```

**メリット**:
- Call emission の全フェーズを1箇所に集約
- パイプライン型の設計で拡張性向上

**デメリット**:
- 大規模な移動 (リスク高)
- 既存コードへの影響大

**推奨**: **オプションA** を採用 (最小限の改善、リスク低)

---

## 5. 実装ロードマップ

### Phase 1: 基盤整備 (Week 1-2)

**Week 1: normalize 統一 + finalize 一本化**

1. `normalize/mod.rs` 新設 → `normalize_all` 関数追加 (1時間)
2. `emit_guard/mod.rs` 拡張 → trace 統合 (2時間)
3. `materialize/call_site.rs` 削除 (30分)
4. 呼び出し側修正 (emit_unified_call, emit_legacy_call) (2時間)
5. テスト実行 (smoke tests) (1時間)

**Week 2: Call 生成ヘルパー実装**

1. `builder_calls/helpers.rs` 新設 (2時間)
2. `emit_call_with_annotate` 実装 (2時間)
3. `emit_normalized_call` 実装 (1時間)
4. emit.rs の13箇所を helpers 呼び出しに置き換え (3時間)
5. テスト実行 (regression tests) (2時間)

### Phase 2: emit.rs 簡潔化 (Week 3)

1. materialize_final_check ヘルパー追加 (1時間)
2. emit.rs の6箇所を helpers 呼び出しに置き換え (2時間)
3. 重複コメント・デッドコード削除 (2時間)
4. 最終テスト (smoke + integration) (3時間)

### Phase 3: ドキュメント整備 (Week 4)

1. アーキテクチャドキュメント更新 (2時間)
2. Call emission パイプライン図作成 (1時間)
3. 削減行数レポート作成 (1時間)

---

## 6. 削減可能行数見積もり

### 詳細内訳

| 項目 | 削減行数 | 根拠 |
|------|---------|------|
| **normalize 統一** | 50-80行 | legacy path の重複除去 (38行) + 簡略化 (12-42行) |
| **materialize 統一** | 30-50行 | call_site.rs 削除 (51行) - 拡張コスト (18行) - 呼び出し側 (1行) |
| **Call 生成ヘルパー** | 70-110行 | 13箇所の重複除去 (78-117行) - 新関数コスト (20行) + 簡略化 (12-13行) |
| **receiver materialization** | 30-40行 | 6箇所の重複除去 (30-42行) - 新関数コスト (0行, 既存拡張) |

### 合計見積もり

- **最小**: 180行 (50 + 30 + 70 + 30)
- **最大**: 280行 (80 + 50 + 110 + 40)
- **平均**: **230行** (32-49% 削減、元 568行 → 338-388行)

---

## 7. リスク評価

### 低リスク項目 (即座に実施可能)

- ✅ `normalize_all` 統一 API (既存関数のラッパー)
- ✅ `finalize_call_operands` trace 統合 (機能追加)
- ✅ `materialize/call_site.rs` 削除 (重複除去)

### 中リスク項目 (慎重に実施)

- ⚠️ `emit_call_with_annotate` 導入 (13箇所の置き換え)
- ⚠️ `emit_normalized_call` 導入 (2箇所の置き換え + 既存ロジック削除)

### 高リスク項目 (Phase 2 以降)

- 🔴 モジュール再構成 (call_emission/ フォルダ新設)
- 🔴 emit.rs の大規模分割

---

## 8. 推奨アクション

### 即座に実施 (Week 1-2)

1. **normalize 統一**: `normalize_all` API 追加 → legacy path 簡略化
2. **finalize 一本化**: `materialize/call_site.rs` 削除 → trace 統合

**期待効果**: 80-130行削減、テストリスク低

### Phase 2 実施 (Week 3-4)

1. **Call 生成ヘルパー**: `emit_call_with_annotate` + `emit_normalized_call`
2. **receiver materialization**: 既存ヘルパーに統合

**期待効果**: 100-150行削減、moderate risk

### 延期項目

- モジュール再構成 (call_emission/ フォルダ) → **Phase 3 以降で検討**

---

## 9. 補足: 発見された設計上の問題点

### 問題1: finalize の二重呼び出し

**現在**:
```rust
// emit_unified_call
finalize_call_operands(self, &mut callee, &mut args_local);    // Line 218
// ... 10 lines ...
finalize_call_site(self, &mut callee2, &mut args2);            // Line 228
// ... 5 lines ...
*r = self.local_recv(*r);  // Line 233 (再度 materialize!)
*a = self.local_arg(*a);   // Line 238
```

**問題**: 同じ receiver/args を **3回** materialize している！

**原因**: 歴史的経緯で finalize 関数が重複追加された

**解決策**: finalize 呼び出しを **1箇所** に統一

---

### 問題2: normalize の位置が不統一

**unified path**: finalize **後** に normalize (Line 258-259)
**legacy path**: normalize **前** に finalize (Line 295, 316)

**問題**: normalize が receiver を使うのに、finalize 前に呼ぶと undefined 参照の可能性

**解決策**: 全経路で **finalize → normalize → emit** の順序を統一

---

### 問題3: Call 命令生成の散在

**現在**: 13箇所で同じパターンの Call 命令生成

**問題**:
- dst 処理 (Some vs unwrap_or) がバラバラ
- effects 選択がハードコード
- annotate 呼び出しの有無が不統一

**解決策**: `emit_call_with_annotate` ヘルパーで統一

---

## 10. まとめ

### 主要発見

1. **重複ロジック**: normalize (4箇所)、finalize (12箇所)、Call 生成 (13箇所) が散在
2. **設計上の問題**: finalize の二重/三重呼び出し、normalize の位置不統一
3. **削減可能行数**: **180-280行** (32-49% 削減)

### 推奨実装順序

1. **Week 1-2**: normalize 統一 + finalize 一本化 (低リスク、80-130行削減)
2. **Week 3**: Call 生成ヘルパー (中リスク、100-150行削減)
3. **Week 4**: ドキュメント整備

### 期待効果

- **コード行数**: 568行 → 338-388行 (230行削減)
- **保守性**: 重複除去により変更箇所が1箇所に
- **テスト性**: 統一 API により単体テストが容易に

---

**調査完了日**: 2025-10-15
**調査時間**: 約2時間
**次のステップ**: User 確認 → Week 1-2 実装開始
