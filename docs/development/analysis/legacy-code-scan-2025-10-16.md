## レガシー経路スキャン結果

**実行日時**: 2025-10-16
**対象**: `src/mir/builder/` 配下のすべてのファイル（8,688行）

---

## 削除候補（優先度順）

### P0 - 即時削除可能（テスト後）

#### 1. Dead Code Helper Functions（`src/mir/builder/utils.rs`）
- **行数**: 72-83 (12行)
- **内容**: LocalSSA convenience helpers with `#[allow(dead_code)]`
  ```rust
  pub(crate) fn local_recv(&mut self, v: ValueId) -> ValueId
  pub(crate) fn local_arg(&mut self, v: ValueId) -> ValueId
  pub(crate) fn local_field_base(&mut self, v: ValueId) -> ValueId
  pub(crate) fn local_cond(&mut self, v: ValueId) -> ValueId
  ```
- **使用箇所**: utils.rs内で定義されているが、実際は`src/mir/builder/ssa/local.rs`の関数を直接呼ぶべき
- **理由**: readability helpersだが、直接`ssa::local::recv()`等を呼ぶ方が明確
- **削減見込み**: 12行削除

#### 2. WeakRef/Barrier Helper Functions（`src/mir/builder/utils.rs`）
- **行数**: 264-304 (41行)
- **内容**:
  ```rust
  #[allow(dead_code)]
  pub(super) fn emit_weak_new(...) -> Result<ValueId, String>

  #[allow(dead_code)]
  pub(super) fn emit_weak_load(...) -> Result<ValueId, String>

  #[allow(dead_code)]
  pub(super) fn emit_barrier_read(&mut self, ptr: ValueId) -> Result<(), String>

  #[allow(dead_code)]
  pub(super) fn emit_barrier_write(&mut self, ptr: ValueId) -> Result<(), String>
  ```
- **使用箇所**: `src/mir/builder/fields.rs:4箇所`で実際に使用されている
- **理由**: `fields.rs`がWeakRef/Barrierを使用中なので即時削除不可
- **優先度**: P1に降格（fields.rs WeakRef削除後に可能）

#### 3. collect_free_vars Function（`src/mir/builder/vars.rs`）
- **行数**: 6-149 (144行)
- **内容**:
  ```rust
  #[allow(dead_code)]
  pub(super) fn collect_free_vars(
      node: &ASTNode,
      used: &mut HashSet<String>,
      locals: &mut HashSet<String>,
  )
  ```
- **使用箇所**: `vars.rs`内で再帰的に自己参照のみ（外部から未使用）
- **理由**: Closure capture analysis用だったが、現在は使用されていない
- **削減見込み**: 144行削除可能

#### 4. record_kpi Function（`src/mir/builder/observe/resolve.rs`）
- **行数**: 41-43
- **内容**:
  ```rust
  #[allow(dead_code)]
  fn record_kpi(meta: &serde_json::Value) {
      if !kpi_enabled() { return; }
  }
  ```
- **使用箇所**: 0箇所（未使用）
- **削減見込み**: 10行削除

**P0 小計**: 12 + 144 + 10 = **166行削除可能**

---

### P1 - 検証後削除可能

#### 5. Legacy Call Bridge（`src/mir/builder/calls/legacy_bridge/mod.rs`）
- **行数**: 1-311 (311行)
- **ステータス**: "DEPRECATION (Phase‑in)" マーク付き
- **内容**:
  - Global/Extern/Method/Constructor call emission paths
  - 統一Call経路への移行期ブリッジ
- **削除条件**:
  1. `NYASH_UNIFIED_CALL=1`を標準化
  2. 全テストがunified call経路でPASS
  3. Router Policy完全移行
- **削減見込み**: 311行削除

#### 6. フェーズM削除済みコメント残骸（`src/mir/builder/phi.rs`）
- **行数**: 115
- **内容**: `// フェーズM: no_phi_mode分岐削除（常にPHI命令を使用）`
- **理由**: コメントのみ残っているが、実装は既に削除済み
- **削減見込み**: 1行削除（コメント整理）

**P1 小計**: 311 + 1 = **312行削除可能**

---

### P2 - リファクタ候補（統合可能）

#### 7. ParserBox特殊処理の重複（3箇所）
- **箇所**:
  1. `src/mir/builder/ssa/local.rs:84-94` (11行)
  2. `src/mir/builder/phi.rs:49-62` (14行)
  3. `src/mir/builder/phi.rs:142-153` (12行)
- **内容**: "VarMapGuard" - ParserBox.* 内で`me`のValueIdを保護
- **重複ロジック**:
  ```rust
  if fun.signature.name.starts_with("ParserBox.") && name != "me" {
      if let Some(&me_vid) = self.variable_map.get("me") {
          if then_v == me_vid || else_v == me_vid {
              let loc = self.value_gen.next();
              self.emit_instruction(MirInstruction::Copy { dst: loc, src: merged })?;
              // ...
          }
      }
  }
  ```
- **提案**: 共通ヘルパー `apply_varmap_guard(builder, name, value) -> ValueId` を作成
- **削減見込み**: 20-25行削減（重複排除）

#### 8. variable_map操作の散在
- **箇所**: 12ファイルで`variable_map`を直接操作
- **提案**:
  - `VarMapBox` trait/struct化
  - `insert_with_guard()`, `merge_with_phi()` 等のAPI化
- **削減見込み**: 50-80行削減（統合・抽象化）

**P2 小計**: 25 + 65 = **90行削減可能**

---

## パラメータ保護関連の既存コード

### ParserBox特殊処理（VarMapGuard）

**目的**: ParserBox.* メソッド内で`me`パラメータの識別性を保護

**実装箇所**:
1. **`src/mir/builder/ssa/local.rs:84-94`**
   - finalize_callee_and_args内
   - Method calleeのreceiverを強制的にfirst_paramに修正

2. **`src/mir/builder/phi.rs:49-62`**
   - merge_modified_vars内
   - PHI merge時に`me`を他変数に直接束縛しない

3. **`src/mir/builder/phi.rs:142-153`**
   - normalize_if_else_phi内
   - if/else PHI結果に対して同様の保護

**コメント**: "dev-only concept; 挙動不変" - 開発安定性のための防御的コード

**今回のバグとの関連**:
- 今回のバグは**パラメータレジスタがNullになる**問題
- VarMapGuardは**Copyを挟んで識別性を保つ**処理
- **関連性**: 両方とも「パラメータの特別扱い」だが、異なる層
  - VarMapGuard: variable_map層での保護
  - 今回のバグ: MIR命令emission層でのレジスタ割り当て

**削除可否**:
- 現状維持推奨（ParserBox安定性のため）
- ただし、P2でヘルパー化して集約すべき

---

## 総削減見込み

| 優先度 | 削減行数 | 内容 |
|--------|---------|------|
| P0 | 166行 | Dead code即時削除 |
| P1 | 312行 | Legacy bridge削除 |
| P2 | 90行 | リファクタ統合 |
| **合計** | **568行** | **6.5%削減**（8,688行→8,120行） |

---

## 重大発見: レガシー経路は想定より少ない

**仮説**: 「大量のレガシーコードが残っている」
**現実**:
- フェーズM削除は**完了済み**（コメント1行のみ残存）
- Legacy bridge以外に大規模なレガシーコードなし
- 重複コードも限定的（ParserBox特殊処理3箇所のみ）

**結論**:
- Hakorune MIR Builderは**既にかなり綺麗**
- 今回のバグは「レガシー経路」ではなく「新実装の穴」
- 削除より**テスト・検証の強化**が優先

---

## 次のアクション（推奨順）

### 即座（Task 4完了後）
1. **P0削除実施**: collect_free_vars (144行) + record_kpi (10行)
2. **テスト実行**: 170 PASS維持確認

### Phase 1-3期間中
3. **P1検証開始**: Legacy bridge削除準備
   - NYASH_UNIFIED_CALL=1でfull test suite実行
   - Router Policy完全移行確認
4. **P2リファクタ**: ParserBox VarMapGuard統合

### 長期（Phase 15.76以降）
5. **fields.rs WeakRef削除**: emit_weak_*/emit_barrier_* 削除可能に

---

## 参考: 最大ファイル（リファクタ候補）

```
786行 src/mir/builder/builder_calls/build.rs
499行 src/mir/builder/ops.rs
455行 src/mir/builder/builder_calls/emit.rs
423行 src/mir/builder/lifecycle.rs
348行 src/mir/builder/utils.rs
333行 src/mir/builder/exprs.rs
310行 src/mir/builder/calls/legacy_bridge/mod.rs
300行 src/mir/builder/stmts.rs
```

**提案**: build.rs (786行) を優先的に分割・整理
