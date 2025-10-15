# モジュール移行クイックリファレンス

**作成日**: 2025-10-15
**詳細計画**: [module_structure_optimization_plan.md](./module_structure_optimization_plan.md)

---

## 各Phase のチェックリスト

### Phase 1: 基盤統合 (Week 1-2) ⭐最優先

**目標**: `core/` モジュール確立

- [ ] **準備**
  - [ ] `feature/module-reorg-phase1` ブランチ作成
  - [ ] `core/` ディレクトリ作成
  - [ ] `core/hako_module.toml` 作成

- [ ] **ファイル移動** (6ファイル)
  - [ ] `vm/boxes/result_box.hako` → `core/result.hako`
  - [ ] `shared/common/string_helpers.hako` → `core/string_helpers.hako`
  - [ ] `shared/common/string_ops.hako` → `core/string_ops.hako`
  - [ ] `shared/json/json_cursor.hako` → `core/json_cursor.hako`
  - [ ] `shared/json/json_utils.hako` → `core/json_utils.hako`
  - [ ] `shared/json/json_canonical_box.hako` → `core/json_canonical.hako`

- [ ] **using 文更新** (60+箇所)
  - [ ] hakorune-vm/ 内の参照更新
  - [ ] compiler/ 内の参照更新
  - [ ] shared/ 内の参照更新
  - [ ] vm/ 内の参照更新 (旧VM、Phase 2で削除)

- [ ] **検証**
  - [ ] `cargo build --release`
  - [ ] スモークテスト実行
  - [ ] 全テストPASS確認

- [ ] **完了**
  - [ ] Phase 1 完了タグ作成 (`v1.0-phase1-core`)
  - [ ] ドキュメント更新

---

### Phase 2: VM統一 (Week 3-4) ⭐最優先

**目標**: `runtime/` モジュール確立、旧VM削除

- [ ] **準備**
  - [ ] `feature/module-reorg-phase2` ブランチ作成
  - [ ] `runtime/` ディレクトリ作成
  - [ ] `runtime/handlers/`, `runtime/guards/`, `runtime/locators/` 作成
  - [ ] `runtime/hako_module.toml` 作成

- [ ] **ファイル移動** (67ファイル)
  - [ ] hakorune-vm/*.hako → runtime/
  - [ ] hakorune-vm/tests/ → runtime/tests/

- [ ] **サブディレクトリ整理**
  - [ ] *_handler.hako → runtime/handlers/
  - [ ] *_guard.hako → runtime/guards/
  - [ ] *_locator.hako → runtime/locators/

- [ ] **旧VM削除**
  - [ ] `vm/` ディレクトリ削除 (30ファイル)
  - [ ] `shared/common/mini_vm_*.hako` 削除 (3ファイル)
  - [ ] `shared/hako_module.toml` から `mini_vm_*` exports削除

- [ ] **using 文更新** (50+箇所)
  - [ ] runtime/ 内部の相互参照更新
  - [ ] compiler/ からの参照更新 (あれば)

- [ ] **検証**
  - [ ] `cargo build --release`
  - [ ] runtime/tests/ の全22テスト実行
  - [ ] スモークテスト実行
  - [ ] 全テストPASS確認

- [ ] **完了**
  - [ ] Phase 2 完了タグ作成 (`v1.0-phase2-runtime`)
  - [ ] ドキュメント更新

---

### Phase 3: MIR集約 (Week 5)

**目標**: `mir/` モジュール確立

- [ ] **準備**
  - [ ] `feature/module-reorg-phase3` ブランチ作成
  - [ ] `mir/` ディレクトリ作成
  - [ ] `mir/hako_module.toml` 作成

- [ ] **ファイル移動** (7ファイル)
  - [ ] `shared/mir/mir_schema_box.hako` → `mir/schema.hako`
  - [ ] `shared/mir/block_builder_box.hako` → `mir/block_builder.hako`
  - [ ] `shared/mir/mir_io_box.hako` → `mir/io.hako`
  - [ ] `shared/json/mir_builder_min.hako` → `mir/builder_min.hako`
  - [ ] `shared/json/mir_builder2.hako` → `mir/builder_v2.hako`
  - [ ] `shared/json/mir_v1_adapter.hako` → `mir/v1_adapter.hako`
  - [ ] `shared/json/json_inst_encode_box.hako` → `mir/inst_encoder.hako`

- [ ] **using 文更新** (10+箇所)
  - [ ] compiler/ からの参照更新

- [ ] **検証**
  - [ ] `cargo build --release`
  - [ ] スモークテスト実行
  - [ ] 全テストPASS確認

- [ ] **完了**
  - [ ] Phase 3 完了タグ作成 (`v1.0-phase3-mir`)
  - [ ] ドキュメント更新

---

### Phase 4: コンパイラー整理 (Week 6)

**目標**: `compiler/pipeline_v2/` サブディレクトリ整理

- [ ] **準備**
  - [ ] `feature/module-reorg-phase4` ブランチ作成
  - [ ] `compiler/pipeline_v2/emitters/` 作成
  - [ ] `compiler/pipeline_v2/extractors/` 作成

- [ ] **ファイル移動** (12ファイル)
  - [ ] emit_*.hako → emitters/ (8ファイル)
  - [ ] *_extract_*.hako → extractors/ (4ファイル)

- [ ] **using 文更新** (12箇所)
  - [ ] pipeline.hako からの相対パス更新

- [ ] **検証**
  - [ ] `cargo build --release`
  - [ ] スモークテスト実行
  - [ ] 全テストPASS確認

- [ ] **完了**
  - [ ] Phase 4 完了タグ作成 (`v1.0-phase4-compiler`)
  - [ ] ドキュメント更新

---

### Phase 5: Backend分離 (Week 7、オプショナル)

**目標**: `backend/` モジュール確立

- [ ] **準備**
  - [ ] `feature/module-reorg-phase5` ブランチ作成
  - [ ] `backend/` ディレクトリ作成
  - [ ] `backend/hako_module.toml` 作成

- [ ] **ファイル移動** (3ファイル)
  - [ ] `shared/backend/llvm_backend_box.hako` → `backend/llvm_backend.hako`
  - [ ] `shared/host_bridge/host_bridge_box.hako` → `backend/host_bridge.hako`
  - [ ] `shared/adapters/map_kv_*.hako` → `backend/adapters/`

- [ ] **検証**
  - [ ] `cargo build --release`
  - [ ] スモークテスト実行
  - [ ] 全テストPASS確認

- [ ] **完了**
  - [ ] Phase 5 完了タグ作成 (`v1.0-phase5-backend`)
  - [ ] ドキュメント更新

---

## using 文の変換ルール

### Phase 1: core への移行

| 旧 | 新 |
|----|-----|
| `using "selfhost/vm/boxes/result_box.hako" as Result` | `using selfhost.core.result as Result` |
| `using "selfhost/shared/common/string_helpers.hako" as StringHelpers` | `using selfhost.core.string_helpers as StringHelpers` |
| `using "selfhost/shared/common/string_ops.hako" as StringOps` | `using selfhost.core.string_ops as StringOps` |
| `using "selfhost/shared/json/json_cursor.hako" as JsonCursorBox` | `using selfhost.core.json_cursor as JsonCursorBox` |

### Phase 2: runtime への移行

| 旧 | 新 |
|----|-----|
| `using "selfhost/hakorune-vm/hakorune_vm_core.hako" as VmCore` | `using selfhost.runtime.vm_core as VmCore` |
| `using "selfhost/hakorune-vm/instruction_dispatcher.hako" as Dispatcher` | `using selfhost.runtime.instruction_dispatcher as Dispatcher` |
| `using "selfhost/hakorune-vm/binop_handler.hako" as BinOpHandler` | `using selfhost.runtime.handlers.binop_handler as BinOpHandler` |
| `using "selfhost/hakorune-vm/args_guard.hako" as ArgsGuard` | `using selfhost.runtime.guards.args_guard as ArgsGuard` |

### Phase 3: mir への移行

| 旧 | 新 |
|----|-----|
| `using selfhost.common.json.mir_builder_min as MirBuilderMin` | `using selfhost.mir.builder_min as MirBuilderMin` |
| `using "selfhost/shared/mir/mir_schema_box.hako" as MirSchema` | `using selfhost.mir.schema as MirSchema` |
| `using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder` | `using selfhost.mir.block_builder as BlockBuilder` |

---

## 便利なスクリプト

### 一括置換スクリプト (Phase 1用)

```bash
#!/bin/bash
# tools/migrate_phase1_using.sh

cd selfhost/

# result_box 置換
find . -name "*.hako" -exec sed -i \
  's|using "selfhost/vm/boxes/result_box.hako"|using selfhost.core.result|g' {} +

# string_helpers 置換
find . -name "*.hako" -exec sed -i \
  's|using "selfhost/shared/common/string_helpers.hako"|using selfhost.core.string_helpers|g' {} +

# string_ops 置換
find . -name "*.hako" -exec sed -i \
  's|using "selfhost/shared/common/string_ops.hako"|using selfhost.core.string_ops|g' {} +

# json_cursor 置換
find . -name "*.hako" -exec sed -i \
  's|using "selfhost/shared/json/json_cursor.hako"|using selfhost.core.json_cursor|g' {} +

echo "Phase 1 using 文の置換完了"
```

### 検証スクリプト

```bash
#!/bin/bash
# tools/verify_migration.sh

echo "=== 旧パスが残っていないか確認 ==="
grep -r "selfhost/vm/boxes/result_box" selfhost/ || echo "✅ result_box OK"
grep -r "selfhost/shared/common/string_helpers" selfhost/ || echo "✅ string_helpers OK"
grep -r "selfhost/hakorune-vm/" selfhost/ || echo "✅ hakorune-vm OK"

echo -e "\n=== ビルド確認 ==="
cargo build --release

echo -e "\n=== スモークテスト ==="
tools/smokes/v2/run.sh --profile quick
```

---

## トラブルシューティング

### 問題: using 文の解決失敗

**症状**:
```
error: Cannot resolve module 'selfhost.core.result'
```

**原因**: `core/hako_module.toml` の exports が不足

**解決**:
```toml
[exports]
result = "result.hako"  # この行を追加
```

---

### 問題: 循環依存エラー

**症状**:
```
error: Circular dependency detected: A -> B -> A
```

**原因**: `core` が他のモジュールに依存している

**解決**: `core/` 内のファイルから他モジュールへの `using` を削除

---

### 問題: テスト失敗

**症状**:
```
test runtime::tests::test_binop ... FAILED
```

**原因**: 移動後のパスが正しくない

**解決**:
1. `runtime/tests/` 内のテストファイルの `using` 文を確認
2. 相対パスから絶対パス (`selfhost.runtime.*`) に変更

---

## 完了確認チェックリスト (Phase 3完了時点)

### ディレクトリ構造

- [ ] `selfhost/core/` が存在 (15-20ファイル)
- [ ] `selfhost/runtime/` が存在 (70-80ファイル)
  - [ ] `runtime/handlers/` が存在 (22ファイル)
  - [ ] `runtime/guards/` が存在 (5-6ファイル)
  - [ ] `runtime/locators/` が存在 (3-4ファイル)
  - [ ] `runtime/tests/` が存在 (22ファイル)
- [ ] `selfhost/mir/` が存在 (10ファイル)
- [ ] `selfhost/vm/` が削除されている ✅
- [ ] `selfhost/hakorune-vm/` が削除されている ✅

### hako_module.toml

- [ ] `core/hako_module.toml` が存在 (依存なし)
- [ ] `runtime/hako_module.toml` が存在 (依存: core)
- [ ] `mir/hako_module.toml` が存在 (依存: core)
- [ ] `compiler/hako_module.toml` が更新 (依存: core, mir)

### using 文

- [ ] `using "selfhost/vm/boxes/result_box.hako"` が存在しない
- [ ] `using "selfhost/hakorune-vm/*"` が存在しない
- [ ] `using selfhost.core.*` が正しく使用されている
- [ ] `using selfhost.runtime.*` が正しく使用されている
- [ ] `using selfhost.mir.*` が正しく使用されている

### テスト

- [ ] `cargo build --release` が成功
- [ ] `tools/smokes/v2/run.sh --profile quick` が全PASS
- [ ] `runtime/tests/` の全22テストが実行可能
- [ ] 既存の全テストが PASS (170+)

### ドキュメント

- [ ] `docs/development/analysis/module_structure_optimization_plan.md` が最新
- [ ] `CLAUDE.md` が更新
- [ ] `README.md` が更新 (using 文の例など)

---

## ロールバック手順

### Phase 1でロールバック

```bash
# Phase 1 開始前のコミットに戻る
git checkout <phase0-commit-hash>

# または Phase 1 タグに戻る
git checkout v1.0-phase0
```

### Phase 2でロールバック

```bash
# Phase 1 完了時点に戻る
git checkout v1.0-phase1-core

# または最新のmainに戻る
git checkout main
```

---

## 次のアクション

### 今すぐ実施
1. このレポートをレビュー
2. `feature/module-reorg-phase1` ブランチ作成
3. Phase 1 のチェックリストを印刷/手元に準備

### Week 1 開始
1. Phase 1 の **準備** セクションを実施
2. Phase 1 の **ファイル移動** セクションを実施
3. 毎日スモークテストを実行

---

**End of Quick Reference**
