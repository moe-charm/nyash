# リファクタリング調査レポート Task 3: レガシー経路

**調査日**: 2025-10-15
**調査者**: Claude Code (Task 3 Agent)
**対象**: `emit_legacy_call` および関連する legacy call path の削除計画

---

## 1. emit_legacy_call の使用箇所

### 1-1. MIR Builder 側の使用箇所

#### **使用箇所 1**: `src/mir/builder/ops.rs:52` - 演算子Box呼び出し (Add)
```rust
// AddOperator.apply/2(lhs, rhs)
self.emit_legacy_call(Some(dst), CallTarget::Global(name), vec![lhs, rhs])?;
```
- **呼び出し条件**: `NYASH_BUILDER_OPERATOR_BOX_ADD_CALL=1` または `NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1`
- **unified 移行可能性**: ✅ **可能**
- **理由**: `emit_unified_call` は Global target をサポート済み
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 2**: `src/mir/builder/ops.rs:86` - 演算子Box呼び出し (Sub/Mul/Div/Mod/Shl/Shr/BitAnd/BitOr/BitXor)
```rust
self.emit_legacy_call(Some(dst), CallTarget::Global(name.to_string()), vec![lhs, rhs])?;
```
- **呼び出し条件**: `NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1` (既定OFF)
- **unified 移行可能性**: ✅ **可能**
- **理由**: 同上
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 3**: `src/mir/builder/ops.rs:445` - 単項演算子Box呼び出し (Neg/Not/BitNot)
```rust
self.emit_legacy_call(Some(dst), CallTarget::Global(name.to_string()), vec![operand_val])?;
```
- **呼び出し条件**: `NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1` (既定OFF)
- **unified 移行可能性**: ✅ **可能**
- **理由**: 同上
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 4**: `src/mir/builder/builder_calls/emit.rs:20` - unified call のフォールバック
```rust
if !call_unified::is_unified_call_enabled() {
    return self.emit_legacy_call(dst, target, args);
}
```
- **呼び出し条件**: `NYASH_UNIFIED_CALL=0` (既定は有効)
- **unified 移行可能性**: ✅ **可能** (環境変数でゲート)
- **理由**: unified call が既定ONなので、このパスは通常使われない
- **移行方法**: フィーチャーフラグで切り替え可能にし、段階的に削除

#### **使用箇所 5**: `src/mir/builder/builder_calls/build.rs:452` - 静的メソッド呼び出しフォールバック
```rust
self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
```
- **呼び出し条件**: 静的メソッド index で一致するものが1つだけ見つかった場合
- **unified 移行可能性**: ✅ **可能**
- **理由**: `emit_unified_call` は Global target をサポート済み
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 6**: `src/mir/builder/builder_calls/build.rs:468` - 静的メソッド呼び出しフォールバック (qualified input)
```rust
self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
```
- **呼び出し条件**: Alias.Box.method 形式の呼び出しで method-name フォールバックが成功した場合
- **unified 移行可能性**: ✅ **可能**
- **理由**: 同上
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 7**: `src/mir/builder/builder_calls/build.rs:494` - tail-based 解決フォールバック
```rust
self.emit_legacy_call(Some(dst), CallTarget::Global(pick), arg_values)?;
```
- **呼び出し条件**: `NYASH_BUILDER_TAIL_RESOLVE=1` (既定OFF)
- **unified 移行可能性**: ✅ **可能**
- **理由**: 同上
- **移行方法**: `emit_legacy_call` → `emit_unified_call` に置き換え

#### **使用箇所 8**: `src/mir/builder/method_call_handlers.rs:193` - birth メソッド呼び出し
```rust
self.emit_legacy_call(
    Some(dst),
    CallTarget::Method { box_type: None, method, receiver: object_value },
    arg_values,
)?;
```
- **呼び出し条件**: instance.birth(args) 呼び出し
- **unified 移行可能性**: ⚠️ **要注意**
- **理由**: legacy path には birth メソッドの特別な処理 (ModuleFunction への変換) がある
- **移行方法**: unified call 側で birth メソッドの特別処理を実装後、移行可能

---

### 1-2. VM 側の使用箇所

#### **使用箇所 9**: `src/backend/mir_interpreter/handlers/calls/legacy/mod.rs:45`
```rust
self.execute_legacy_call(func, &args2)?
```
- **呼び出し条件**: `callee=None` (NameConst ベース) かつ `legacy-boxes` feature が有効
- **unified 移行可能性**: ✅ **可能** (builder 側で callee を必須にする)
- **理由**: builder 側で必ず `callee` を付与すれば、この経路は不要になる
- **移行方法**:
  1. builder 側で `callee=None` を生成しないようにする
  2. VM 側で `callee=None` の場合はエラーにする

---

## 2. legacy 経路の依存関係

### 2-1. 依存関係マップ

```
【Builder 側】
emit_legacy_call (emit.rs:277)
  ↓
  ├─ [特別処理1] StringBox.size/len/length → Extern("nyrt.string.length")
  ├─ [特別処理2] ArrayBox.size/len/length → Extern("nyrt.array.length")
  ├─ [特別処理3] instance.birth(args) → ModuleFunction(Class.birth/N)
  ├─ [特別処理4] user instance BoxCall 禁止時 → ModuleFunction 変換
  └─ [フォールバック] emit_box_or_plugin_call (BoxCall emission)

【VM 側】
execute_legacy_call (legacy_resolver.rs:11)
  ↓
  └─ callee_dispatcher (callee_dispatcher.rs)
       ↓
       ├─ handle_method_call_legacy (method_handler.rs:11)
       ├─ execute_global_function (function.rs)
       └─ handle_extern_call (extern_handler.rs)
```

### 2-2. legacy 経路が必要な理由

**歴史的経緯**:
1. **初期実装**: NameConst (文字列) ベースの関数呼び出し
2. **Phase 2**: 構造化 Callee の導入 (Global/Method/Extern/ModuleFunction 等)
3. **Phase 3**: unified call の導入 (emit_unified_call)

**現状の役割**:
- **後方互換**: NameConst ベース (callee=None) の既存 MIR を実行可能にする
- **特別処理**: birth メソッド、StringBox/ArrayBox の length/size 等の正規化
- **フォールバック**: unified call が無効な場合の代替経路

**unified 経路との違い**:
| 項目 | legacy 経路 | unified 経路 |
|------|------------|-------------|
| Method 解決 | 実行時 (VM) | コンパイル時 (builder) + 実行時 (VM) |
| normalize | emit.rs で実施 | emit.rs で実施 (同じ) |
| materialize | 個別実装 | 統一実装 (LocalSSA) |
| callee field | Optional (None 許容) | Required (必須) |
| feature gate | `legacy-boxes` | なし (既定ON) |

---

## 3. 段階的削除計画

### Phase 1: emit_legacy_call 呼び出しを unified に置き換え (2週間)

#### **Week 1-2**: Builder 側の置き換え

**Task 1.1**: 演算子Box呼び出しを unified に移行 (1日)
- **対象**: ops.rs の 3箇所 (Add, 他の演算子, 単項演算子)
- **方法**: `emit_legacy_call` → `emit_unified_call` に置き換え
- **テスト**: `NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1` で既存テスト実行

**Task 1.2**: build.rs のフォールバックを unified に移行 (2日)
- **対象**: build.rs の 3箇所 (静的メソッド index, qualified input, tail-based)
- **方法**: 同上
- **テスト**: 静的メソッド呼び出しテスト実行

**Task 1.3**: method_call_handlers.rs の birth メソッドを unified に移行 (2日)
- **対象**: method_call_handlers.rs:193
- **前提条件**: unified call 側で birth メソッドの特別処理を実装
- **方法**:
  1. emit.rs の emit_unified_call に birth メソッド特別処理を追加
  2. method_call_handlers.rs を `emit_unified_call` に置き換え
- **テスト**: birth メソッド呼び出しテスト実行

**Task 1.4**: emit.rs のフォールバックを削除準備 (2日)
- **対象**: emit.rs:20 (unified call のフォールバック)
- **方法**: フィーチャーフラグ `NYASH_UNIFIED_CALL_REQUIRED=1` を追加
- **テスト**: 既定OFF、明示的ONで動作確認

**Task 1.5**: 回帰テスト実施 (3日)
- **方法**:
  1. `cargo test` 全テスト実行
  2. `tools/smokes/v2/run.sh --profile quick` 実行
  3. selfhost compiler テスト実行
- **検証**: 170 PASS / 15 FAIL を維持

**影響範囲**:
- 修正ファイル数: 3-5 ファイル
- 削減可能行数: 50-100 行 (呼び出し箇所のみ、emit_legacy_call 本体は未削除)

**リスク**:
- **中**: 既存の Method 呼び出しが動作しなくなる可能性
- **軽減策**:
  1. フィーチャーフラグで切り替え可能にする
  2. 段階的にテストを追加
  3. ロールバック戦略を用意 (後述)

---

### Phase 2: emit_legacy_call 本体を削除 (1週間)

#### **Week 3**: emit_legacy_call 本体の削除

**Task 2.1**: emit_legacy_call 関数を削除 (1日)
- **対象**: emit.rs:277-253 (277行の関数)
- **前提条件**: Phase 1 完了 (すべての呼び出し箇所を unified に置き換え)
- **方法**:
  1. `emit_legacy_call` 関数を削除
  2. 関連する import を削除
- **テスト**: コンパイルが通ることを確認

**Task 2.2**: VM 側 legacy handler の削除 (2日)
- **対象**: `src/backend/mir_interpreter/handlers/calls/legacy/` ディレクトリ全体
  - `callee_dispatcher.rs` (60行)
  - `extern_handler.rs` (252行)
  - `legacy_resolver.rs` (335行)
  - `method_handler.rs` (199行)
  - `mod.rs` (57行)
- **方法**:
  1. ディレクトリ全体を削除
  2. `handlers/calls/mod.rs` から legacy re-export を削除
- **テスト**:
  1. コンパイルが通ることを確認
  2. VM テスト実行

**Task 2.3**: VM 側で callee=None を reject (1日)
- **対象**: `handlers/calls/mod.rs:43-50`
- **方法**:
  ```rust
  let call_result = if let Some(callee_type) = callee {
      self.execute_callee_call(callee_type, &args2)?
  } else {
      return Err(VMError::InvalidInstruction(
          "Call instruction requires callee field (legacy NameConst path removed)".into()
      ));
  };
  ```
- **テスト**: callee=None の MIR を実行してエラーになることを確認

**Task 2.4**: feature gate `legacy-boxes` の削除 (1日)
- **対象**:
  - `Cargo.toml:36` (`legacy-boxes` feature 削除)
  - `Cargo.toml:15` (default features から `legacy-boxes` 削除)
  - 関連する `#[cfg(feature = "legacy-boxes")]` を削除
- **影響範囲**: 20-30箇所
- **方法**: grep で検索し、段階的に削除
- **テスト**: `cargo build --no-default-features --features cli` でビルド確認

**Task 2.5**: 回帰テスト実施 (2日)
- **方法**: Phase 1 と同様
- **検証**: 170 PASS / 15 FAIL を維持

**影響範囲**:
- 修正ファイル数: 10-15 ファイル
- 削減可能行数: 900-1000 行

**リスク**:
- **低**: Phase 1 完了後なら依存なし
- **軽減策**: Phase 1 で十分にテストしておく

---

### Phase 3: legacy テストコードの削除 (1週間)

#### **Week 4**: legacy テストコードの削除

**Task 3.1**: legacy テストコードを検索 (1日)
- **方法**:
  ```bash
  grep -rn "legacy" tests/ --include="*.rs" | grep -i test
  grep -rn "emit_legacy_call" tests/ --include="*.rs"
  grep -rn "NameConst" tests/ --include="*.rs" | grep -i call
  ```
- **期待**: 10-20個のテストファイルを特定

**Task 3.2**: legacy テストを削除または更新 (2日)
- **方法**:
  1. 完全に legacy 経路のみをテストしているものは削除
  2. unified 経路でも有効なテストは更新 (emit_legacy_call → emit_unified_call)
- **テスト**: 削除後のテストスイートを実行

**Task 3.3**: ドキュメント更新 (1日)
- **対象**:
  - `CLAUDE.md`: legacy 経路の削除を記録
  - `docs/development/architecture/*.md`: legacy 経路の記述を削除
  - `docs/guides/*.md`: emit_legacy_call の記述を削除
- **方法**: grep で検索し、段階的に更新

**Task 3.4**: CHANGELOG.md に記録 (1日)
- **内容**:
  ```markdown
  ## Phase 15.XX: Legacy Call Path Elimination

  - **削除**: `emit_legacy_call` および VM 側 legacy handler (900+ lines)
  - **移行**: すべての呼び出し箇所を `emit_unified_call` に統一
  - **影響**:
    - NameConst (callee=None) ベースの MIR は実行不可 → エラーメッセージを表示
    - `legacy-boxes` feature 削除
  - **互換性**: Phase 15.75 以降の MIR との互換性を維持
  ```

**Task 3.5**: 最終回帰テスト (2日)
- **方法**: Phase 1/2 と同様
- **検証**: 170 PASS / 15 FAIL を維持

**影響範囲**:
- 修正ファイル数: 15-25 ファイル
- 削減可能行数: 100-200 行

**リスク**:
- **低**: テストコードのみなので本番コードに影響なし

---

## 4. 削除可能行数見積もり

### 4-1. Builder 側

| ファイル | 削除可能行数 | 備考 |
|---------|------------|------|
| `builder_calls/emit.rs` | 277行 | emit_legacy_call 関数本体 |
| `ops.rs` | 10行 | 呼び出し箇所 (3箇所) |
| `builder_calls/build.rs` | 15行 | 呼び出し箇所 (3箇所) |
| `method_call_handlers.rs` | 10行 | 呼び出し箇所 (1箇所) |
| **合計 (Builder)** | **312行** | |

### 4-2. VM 側

| ファイル | 削除可能行数 | 備考 |
|---------|------------|------|
| `legacy/callee_dispatcher.rs` | 60行 | 削除 |
| `legacy/extern_handler.rs` | 252行 | 削除 |
| `legacy/legacy_resolver.rs` | 335行 | 削除 |
| `legacy/method_handler.rs` | 199行 | 削除 |
| `legacy/mod.rs` | 57行 | 削除 |
| `calls/mod.rs` | 10行 | legacy re-export 削除 |
| **合計 (VM)** | **913行** | |

### 4-3. テスト・ドキュメント

| カテゴリ | 削除可能行数 | 備考 |
|---------|------------|------|
| テストコード | 100-200行 | 見積もり |
| ドキュメント | 50-100行 | 見積もり |
| **合計 (その他)** | **150-300行** | |

### 4-4. 総計

- **Phase 1** (呼び出し箇所置き換え): 35行削減
- **Phase 2** (本体削除): 1,190行削減
- **Phase 3** (テスト・ドキュメント): 150-300行削減
- **総計**: **1,375-1,525行削減**

---

## 5. ロールバック戦略

### 5-1. フィーチャーフラグ

**環境変数**: `NYASH_UNIFIED_CALL_REQUIRED`
- **既定値**: `0` (OFF) - legacy call を許可
- **Phase 1 後**: `1` (ON) - legacy call を禁止、エラーメッセージ表示
- **Phase 2 後**: 環境変数削除 (常にエラー)

**実装例** (emit.rs):
```rust
pub fn emit_unified_call(
    &mut self,
    dst: Option<ValueId>,
    target: CallTarget,
    args: Vec<ValueId>,
) -> Result<(), String> {
    // Phase 1: フィーチャーフラグでゲート
    let require_unified = std::env::var("NYASH_UNIFIED_CALL_REQUIRED")
        .ok()
        .as_deref() == Some("1");

    if !call_unified::is_unified_call_enabled() {
        if require_unified {
            return Err(format!(
                "Legacy call path is disabled (NYASH_UNIFIED_CALL_REQUIRED=1). \
                 Target: {:?}. Enable unified call with NYASH_UNIFIED_CALL=1",
                target
            ));
        }
        // Fallback to legacy (Phase 1 のみ)
        return self.emit_legacy_call(dst, target, args);
    }

    // ... unified call 実装 ...
}
```

### 5-2. ロールバック手順

**Phase 1 でのロールバック**:
1. `NYASH_UNIFIED_CALL_REQUIRED=0` で legacy 経路に戻す
2. 問題のあるテストケースを特定
3. unified 経路の修正
4. 再度 `NYASH_UNIFIED_CALL_REQUIRED=1` でテスト

**Phase 2 でのロールバック**:
1. Git で Phase 1 完了時点に戻す (`git revert` または `git reset`)
2. 問題を修正
3. Phase 1 から再実施

**Phase 3 でのロールバック**:
- テストコード・ドキュメントのみなので、個別に `git revert` で戻せる

---

## 6. リスク評価

| リスク | 深刻度 | 発生確率 | 軽減策 |
|--------|--------|----------|--------|
| Method 解決失敗 | 高 | 中 | フィーチャーフラグで切り替え |
| birth メソッド動作不良 | 高 | 中 | 専用テストを追加、段階的移行 |
| テスト失敗 | 中 | 高 | 段階的にテスト追加 |
| パフォーマンス低下 | 低 | 低 | ベンチマーク実施 |
| ロールバック困難 | 中 | 低 | Git タグでマイルストーン管理 |

### 6-1. 深刻度の定義

- **高**: 本番コンパイラの動作に影響 (selfhost compiler が動かない等)
- **中**: 一部機能の動作不良 (特定のテストが失敗)
- **低**: ドキュメント・テストコードのみの問題

### 6-2. 軽減策の詳細

**Method 解決失敗への対策**:
1. **フィーチャーフラグ**: `NYASH_UNIFIED_CALL_REQUIRED` で段階的移行
2. **詳細ログ**: `NYASH_CLI_VERBOSE=1` で Method 解決過程を出力
3. **回帰テスト**: 既存テストスイートを Phase 1 完了時点で全実行

**birth メソッド動作不良への対策**:
1. **専用テスト追加**: birth メソッド呼び出しの統合テストを追加
2. **段階的移行**:
   - Step 1: emit_unified_call に birth 特別処理を追加
   - Step 2: テストで動作確認
   - Step 3: method_call_handlers.rs を置き換え

**テスト失敗への対策**:
1. **段階的テスト追加**: Phase 1 の各 Task 完了時に回帰テスト実施
2. **CI 統合**: GitHub Actions で自動テスト実行
3. **スモークテスト**: `tools/smokes/v2/run.sh --profile quick` で基本動作確認

---

## 7. 期待される効果

### 7-1. コードベース削減

- **削減行数**: 1,375-1,525行 (全体の約1.5%)
- **削減ファイル数**: 15-25ファイル
- **削減ディレクトリ**: `legacy/` (VM 側)

### 7-2. 保守性向上

**Before** (legacy 経路あり):
```
emit_unified_call
  ↓ [fallback]
emit_legacy_call
  ↓
execute_legacy_call (VM)
  ↓
callee_dispatcher
  ↓
method_handler / function_handler / extern_handler
```

**After** (unified のみ):
```
emit_unified_call
  ↓
execute_callee_call (VM)
  ↓
[統一された handler]
```

**改善点**:
- **経路数**: 2経路 → 1経路
- **特別処理の重複**: なし (emit_unified_call に集約)
- **テストの複雑さ**: 2倍 → 1倍

### 7-3. パフォーマンス影響

- **コンパイル時間**: 変化なし (経路統一による最適化の余地はあり)
- **実行時間**: 変化なし (VM 側の handler は同じロジック)
- **バイナリサイズ**: 若干削減 (1-2%)

---

## 8. マイルストーン管理

### 8-1. Git タグ

- **Phase 1 完了**: `refactor/legacy-call-phase1-complete`
- **Phase 2 完了**: `refactor/legacy-call-phase2-complete`
- **Phase 3 完了**: `refactor/legacy-call-phase3-complete`

### 8-2. 進捗管理

**Week 1-2** (Phase 1):
- [ ] Task 1.1: 演算子Box呼び出し移行
- [ ] Task 1.2: build.rs フォールバック移行
- [ ] Task 1.3: birth メソッド移行
- [ ] Task 1.4: emit.rs フォールバック削除準備
- [ ] Task 1.5: 回帰テスト実施
- [ ] Git タグ作成: `refactor/legacy-call-phase1-complete`

**Week 3** (Phase 2):
- [ ] Task 2.1: emit_legacy_call 本体削除
- [ ] Task 2.2: VM 側 legacy handler 削除
- [ ] Task 2.3: callee=None reject
- [ ] Task 2.4: feature gate 削除
- [ ] Task 2.5: 回帰テスト実施
- [ ] Git タグ作成: `refactor/legacy-call-phase2-complete`

**Week 4** (Phase 3):
- [ ] Task 3.1: legacy テストコード検索
- [ ] Task 3.2: legacy テスト削除/更新
- [ ] Task 3.3: ドキュメント更新
- [ ] Task 3.4: CHANGELOG 記録
- [ ] Task 3.5: 最終回帰テスト
- [ ] Git タグ作成: `refactor/legacy-call-phase3-complete`

---

## 9. 結論

### 9-1. 実施推奨度: ⭐⭐⭐⭐⭐ (5/5)

**理由**:
1. **削減効果が大きい**: 1,375-1,525行削減 (全体の約1.5%)
2. **保守性向上**: 経路統一により複雑さが半減
3. **リスクが低い**: フィーチャーフラグで段階的移行可能
4. **技術的負債の解消**: legacy 経路は歴史的経緯で残存しているだけ

### 9-2. 優先度: **高** (Phase 15.XX として実施推奨)

**次のステップ**:
1. **Phase 1 開始**: 2週間以内 (Week 1-2)
2. **Phase 2 開始**: Phase 1 完了後 1週間以内 (Week 3)
3. **Phase 3 開始**: Phase 2 完了後 1週間以内 (Week 4)

### 9-3. 期待される効果まとめ

| 項目 | Before | After | 改善率 |
|------|--------|-------|--------|
| 総行数 | 100,000行 | 98,500行 | -1.5% |
| call 経路数 | 2経路 | 1経路 | -50% |
| legacy handler 行数 | 903行 | 0行 | -100% |
| feature gate | `legacy-boxes` | なし | - |
| テストの複雑さ | 2倍 | 1倍 | -50% |

---

**調査完了日**: 2025-10-15
**推定実施期間**: 4週間 (Week 1-4)
**推定削減行数**: 1,375-1,525行
