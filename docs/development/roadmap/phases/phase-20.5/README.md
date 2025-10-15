# Phase 20.5 - Hakorune VM Validation & Adoption

**期間**: 2025-12-21 - 2026-01-31 (6週間)
**状態**: Planning ⚠️ **CRITICAL UPDATE** - VM already exists!
**前提**: Phase 15.77完了 (凍結EXE確定)
**重大発見**: Hakorune VM is **100% COMPLETE** (2025-10-14)

---

## 🎉 Critical Discovery (2025-10-14)

**Original Assumption**: Hakorune VM does not exist, needs 36 weeks to build.

**Actual Reality**: **Hakorune VM is COMPLETE** in `selfhost/hakorune-vm/`!
- ✅ 3,413 lines of Hakorune code
- ✅ 44 files including 22 instruction handlers
- ✅ 26+ comprehensive tests
- ✅ All 16 MIR instructions + 6 advanced handlers
- ✅ @match-based dispatch architecture
- ✅ Result-based error handling
- ✅ Implementation period: October 5-13, 2025 (8 days!)

**Impact**: Phase 20.5 changes from "implementation" to **"validation & adoption"**.

📖 **Full Discovery Report**: [HAKORUNE_VM_DISCOVERY.md](HAKORUNE_VM_DISCOVERY.md)

---

## 🎯 このフェーズで実現すること（更新版）

**"Hakorune VM → 検証 → CLI統合 → デフォルト化"**

1. **Hakorune VM検証**: 既存22ハンドラーの動作確認
2. **Golden Testing**: Rust-VM vs Hako-VM 完全一致保証
3. **CLI統合**: `--backend vm-hako` 実装
4. **ドキュメント整備**: アーキテクチャ・移行ガイド
5. **オプション: HostBridge API**: C-ABI境界（他の理由で必要な場合のみ）

**変更点**:
- ❌ **VM実装** → ✅ **VM検証**（既に完成）
- ❌ **36週間** → ✅ **6週間**
- ❌ **段階的実装** → ✅ **全機能検証**

---

## 💡 このフェーズの位置づけ（更新版）

```
Phase 15.77（凍結EXE確定）✅
├── hako-frozen-v1.exe (724KB MSVC)
├── NyRT関数呼び出し可能
└── MIR JSON → .o → EXE 導線確立

Hakorune VM実装完了（2025-10-05 → 10-13）✅
├── 3,413 lines / 44 files
├── 22 instruction handlers
├── 26+ comprehensive tests
└── selfhost/hakorune-vm/

Phase 20.5（VM検証・統合）← 今ここ
├── Week 1-2: VM検証・テスト拡充
├── Week 3-4: Golden Testing（Rust-VM vs Hako-VM）
├── Week 5: CLI統合（--backend vm-hako）
└── Week 6: ドキュメント・Phase 20.6計画

Phase 20.6（オプション: HostBridge + Rust最小化）
├── HostBridge API実装（必要な場合）
├── Rust VM非推奨化
└── Hakorune VM デフォルト化
```

**重要**: Phase 20.6以降は「必要に応じて」実施。Hakorune VM単体で動作可能。

---

## 🏆 成功基準（DoD）

### 1️⃣ Hakorune VM検証完了

```bash
# 既存テストスイート実行
cd selfhost/hakorune-vm
for test in tests/*.hako; do
    NYASH_DISABLE_PLUGINS=1 ../../target/release/hako "$test"
done

# 期待: 26+ tests ALL PASS
```

**チェックリスト**:
- [ ] 26個の既存テスト すべて PASS
- [ ] 22個のハンドラー動作確認
- [ ] エラーハンドリング（Result）動作確認
- [ ] @match dispatch 動作確認

### 2️⃣ Golden Testing完了

```bash
# Golden Test実行（Rust-VM vs Hako-VM）
bash tools/golden_test_hakorune_vm.sh

# 期待: 100% parity
```

**テストケース**:
- [ ] 算術演算（10ケース）
- [ ] 制御フロー（10ケース）
- [ ] コレクション操作（10ケース）
- [ ] 再帰（5ケース）
- [ ] クロージャ（5ケース）
- [ ] 合計40ケース すべて一致

### 3️⃣ CLI統合完了

```bash
# Hakorune VM経由で実行可能
./target/release/hako --backend vm-hako program.hako

# 環境変数でもOK
HAKO_USE_HAKORUNE_VM=1 ./target/release/hako program.hako
```

**チェックリスト**:
- [ ] `--backend vm-hako` フラグ実装
- [ ] `HAKO_USE_HAKORUNE_VM=1` 環境変数対応
- [ ] エラーメッセージ整備
- [ ] パフォーマンス測定

### 4️⃣ ドキュメント整備完了

**必須ドキュメント**:
- [ ] `selfhost/hakorune-vm/README.md` - アーキテクチャ概要
- [ ] `selfhost/hakorune-vm/DESIGN.md` - 設計パターン
- [ ] `selfhost/hakorune-vm/TESTING.md` - テスト戦略
- [ ] `docs/guides/hakorune-vm-migration.md` - 移行ガイド
- [ ] Phase 20.6計画書（オプション: HostBridge）

---

## 📊 週次計画（Week 1-6）

### Week 1（2025-12-21 - 12-27）VM検証・テスト実行

**目標**: 既存実装の動作確認

#### タスク
```bash
# 1. 既存テスト全実行
cd selfhost/hakorune-vm
for test in tests/*.hako; do
    echo "Testing: $test"
    NYASH_DISABLE_PLUGINS=1 ../../target/release/hako "$test" || echo "FAIL"
done

# 2. ハンドラーカバレッジ確認
ls *_handler.hako | wc -l  # 期待: 22

# 3. エラーケーステスト
# - 不正なMIR JSON
# - 未定義命令
# - 型エラー
```

**成果物**:
- [ ] テスト実行レポート（26+ tests）
- [ ] カバレッジレポート（22 handlers）
- [ ] バグ修正リスト（あれば）

### Week 2（2025-12-28 - 2026-01-03）テスト拡充

**目標**: テストカバレッジ拡大

#### タスク
```bash
# 新規テストケース追加
selfhost/hakorune-vm/tests/
├── test_edge_cases/
│   ├── test_large_numbers.hako
│   ├── test_deep_recursion.hako
│   ├── test_long_strings.hako
│   └── test_complex_control_flow.hako
└── test_stress/
    ├── test_1000_instructions.hako
    └── test_nested_calls_100_deep.hako
```

**成果物**:
- [ ] 10個の新規テストケース
- [ ] エッジケース網羅
- [ ] ストレステスト実施

### Week 3-4（2026-01-04 - 01-17）Golden Testing

**目標**: Rust-VM vs Hako-VM 完全一致保証

#### Golden Testスイート作成
```bash
tests/golden/hakorune-vm/
├── arithmetic/
│   ├── test_add.hako
│   ├── test_mul.hako
│   └── test_div.hako (10 tests)
├── control_flow/
│   ├── test_if_else.hako
│   ├── test_loop.hako
│   └── test_branch.hako (10 tests)
├── collections/
│   ├── test_array_ops.hako
│   ├── test_map_ops.hako
│   └── test_string_ops.hako (10 tests)
├── recursion/
│   ├── test_factorial.hako
│   └── test_fibonacci.hako (5 tests)
└── closures/
    ├── test_capture.hako
    └── test_nested.hako (5 tests)
```

#### Golden Test実行スクリプト
```bash
# tools/golden_test_hakorune_vm.sh
#!/bin/bash
set -e

PASS=0
FAIL=0

for test in tests/golden/hakorune-vm/**/*.hako; do
    echo "Testing: $test"

    # Rust VM実行
    ./target/release/hako --backend vm "$test" > /tmp/rust_out.txt 2>&1
    rust_exit=$?

    # Hakorune VM実行
    ./target/release/hako --backend vm-hako "$test" > /tmp/hako_out.txt 2>&1
    hako_exit=$?

    # 出力比較
    if diff /tmp/rust_out.txt /tmp/hako_out.txt && [ $rust_exit -eq $hako_exit ]; then
        echo "  ✅ PASS"
        ((PASS++))
    else
        echo "  ❌ FAIL"
        echo "  Rust output:"
        cat /tmp/rust_out.txt
        echo "  Hako output:"
        cat /tmp/hako_out.txt
        ((FAIL++))
    fi
done

echo ""
echo "Golden Test Results:"
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  Total: $((PASS + FAIL))"

if [ $FAIL -eq 0 ]; then
    echo "✅ All Golden Tests PASSED!"
    exit 0
else
    echo "❌ Some Golden Tests FAILED"
    exit 1
fi
```

**成果物**:
- [ ] 40個のGolden Testケース
- [ ] Golden Test実行スクリプト
- [ ] CI統合（GitHub Actions）
- [ ] パフォーマンス比較レポート

### Week 5（2026-01-18 - 01-24）CLI統合

**目標**: `--backend vm-hako` 実装

#### 実装ファイル
```rust
// src/backend/hakorune_vm_runner.rs (NEW)
use std::path::PathBuf;
use crate::error::Result;

pub fn run_hakorune_vm(mir_json: String) -> Result<i64> {
    // 1. selfhost/hakorune-vm/hakorune_vm_core.hako をロード
    let vm_core_path = PathBuf::from("selfhost/hakorune-vm/hakorune_vm_core.hako");

    // 2. Rust VMでHakoruneVmCoreBoxを実行
    let vm_instance = load_and_run_box(&vm_core_path)?;

    // 3. HakoruneVmCoreBox.run(mir_json) を呼び出し
    let result = vm_instance.call_method("run", vec![Box::new(mir_json)])?;

    // 4. 結果を返す
    Ok(result.as_int()?)
}

// src/cli.rs (MODIFY)
match backend {
    Backend::Vm => run_rust_vm(mir),
    Backend::VmHako => run_hakorune_vm(mir),  // NEW!
    Backend::Llvm => run_llvm(mir),
    Backend::Wasm => run_wasm(mir),
}
```

**テスト**:
```bash
# 基本実行
./target/release/hako --backend vm-hako apps/tests/hello.hako
# 期待: Hello World出力

# 環境変数版
HAKO_USE_HAKORUNE_VM=1 ./target/release/hako apps/tests/hello.hako

# デバッグモード
HAKO_VM_TRACE=1 ./target/release/hako --backend vm-hako test.hako
```

**成果物**:
- [ ] `src/backend/hakorune_vm_runner.rs` 実装
- [ ] CLI統合完了
- [ ] 環境変数対応
- [ ] エラーメッセージ整備

### Week 6（2026-01-25 - 01-31）ドキュメント・Phase 20.6計画

**目標**: ドキュメント整備、次フェーズ計画

#### ドキュメント作成
```
selfhost/hakorune-vm/
├── README.md                         # アーキテクチャ概要
├── DESIGN.md                         # 設計パターン詳解
├── TESTING.md                        # テスト戦略
└── CHANGELOG.md                      # 実装履歴（Oct 5-13）

docs/guides/
└── hakorune-vm-migration.md          # ユーザー移行ガイド

docs/development/roadmap/phases/phase-20.5/
├── HAKORUNE_VM_DISCOVERY.md          # 発見レポート
├── README.md                         # このファイル（更新済み）
├── VALIDATION_REPORT.md              # 検証レポート
└── COMPLETION_REPORT.md              # 完了報告

docs/development/roadmap/phases/phase-20.6/ (オプション)
└── README.md                         # HostBridge計画（必要な場合）
```

**成果物**:
- [ ] 5個のドキュメント完成
- [ ] Phase 20.5完了報告書
- [ ] Phase 20.6計画書（オプション）
- [ ] tomoakiさんへの報告

---

## 🎯 Hakorune VM アーキテクチャ（既存実装）

### ファイル構成

```
selfhost/hakorune-vm/ (3,413 lines, 44 files)
├── hakorune_vm_core.hako (225 lines)       # Entry point
├── instruction_dispatcher.hako (72 lines)   # @match dispatch
├── blocks_locator.hako                     # Control flow
├── error_builder.hako                      # Error messages
├── args_guard.hako                         # Argument validation
├── json_normalize_box.hako                 # JSON processing
│
├── [22 handler files]                      # Instruction handlers:
│   ├── barrier_handler.hako                # GC barrier
│   ├── binop_handler.hako                  # Binary operations
│   ├── boxcall_handler.hako                # Box method calls
│   ├── closure_call_handler.hako           # Closure invocation
│   ├── compare_handler.hako                # Comparisons
│   ├── const_handler.hako                  # Constants
│   ├── constructor_call_handler.hako       # Constructor calls
│   ├── copy_handler.hako                   # Copy operation
│   ├── extern_call_handler.hako            # External calls
│   ├── global_call_handler.hako            # Global functions
│   ├── load_handler.hako                   # Memory load
│   ├── method_call_handler.hako            # Method calls
│   ├── mircall_handler.hako                # Unified MIR call
│   ├── module_function_call_handler.hako   # Module functions
│   ├── newbox_handler.hako                 # Box creation
│   ├── nop_handler.hako                    # No-op
│   ├── phi_handler.hako                    # PHI nodes
│   ├── safepoint_handler.hako              # GC safepoint
│   ├── store_handler.hako                  # Memory store
│   ├── terminator_handler.hako             # Control flow (jump/branch/ret)
│   ├── typeop_handler.hako                 # Type operations
│   └── unaryop_handler.hako                # Unary operations
│
└── tests/ (26+ files)                      # Comprehensive tests
    ├── test_phase1_minimal.hako
    ├── test_phase2_day4.hako
    ├── test_boxcall.hako
    ├── test_mircall_*.hako (5 files)
    ├── test_mapbox_*.hako (3 files)
    └── ... (16 more)
```

### 設計パターン

#### 1. @match-Based Dispatch
```hakorune
// instruction_dispatcher.hako
dispatch(inst_json, regs, mem) {
    local op = extract_op(inst_json)

    return match op {
        "const" => ConstHandlerBox.handle(inst_json, regs)
        "binop" => BinOpHandlerBox.handle(inst_json, regs)
        "compare" => CompareHandlerBox.handle(inst_json, regs)
        "load" => LoadHandlerBox.handle(inst_json, regs, mem)
        "store" => StoreHandlerBox.handle(inst_json, regs, mem)
        "mir_call" => MirCallHandlerBox.handle(inst_json, regs, mem)
        "boxcall" => BoxCallHandlerBox.handle(inst_json, regs)
        "newbox" => NewBoxHandlerBox.handle(inst_json, regs)
        "phi" => PhiHandlerBox.handle(inst_json, regs)
        "copy" => CopyHandlerBox.handle(inst_json, regs)
        "typeop" => TypeOpHandlerBox.handle(inst_json, regs, mem)
        "nop" => NopHandlerBox.handle(inst_json, regs, mem)
        "safepoint" => SafepointHandlerBox.handle(inst_json, regs, mem)
        "barrier" => BarrierHandlerBox.handle(inst_json, regs, mem)
        "jump" | "branch" | "ret" => TerminatorHandlerBox.handle(inst_json, regs)
        _ => Result.Err("Unsupported instruction: " + op)
    }
}
```

**特徴**:
- ✅ すべての命令を1箇所で管理
- ✅ 新規命令の追加が容易
- ✅ Fail-Fast（未知命令→即エラー）

#### 2. Result-Based Error Handling
```hakorune
// Every handler returns Result<Value>
static box BinOpHandlerBox {
    handle(inst_json, regs) {
        // Validation
        local lhs_result = get_register(regs, lhs_id)
        if lhs_result.is_Err() {
            return Result.Err("BinOp: lhs register not found")
        }

        // Compute
        local result = compute_binop(kind, lhs, rhs)
        if result.is_Err() {
            return result  // Propagate error
        }

        // Store result
        set_register(regs, dst, result.as_Ok())
        return Result.Ok(result.as_Ok())
    }
}
```

**特徴**:
- ✅ 型安全なエラー処理
- ✅ エラー伝播が明示的
- ✅ Rustと同じパターン

#### 3. Block-Based Execution
```hakorune
// hakorune_vm_core.hako
_execute_blocks(mir_json, regs, mem) {
    local current_block = "entry"

    loop(current_block != null) {
        // Get block instructions
        local block = BlocksLocatorBox.locate(mir_json, current_block)
        if block.is_Err() { return block }

        // Execute instructions
        local result = me._execute_block(block.as_Ok(), regs, mem)
        if result.is_Err() { return result }

        // Update current block (from terminator)
        current_block = result.as_Ok()
    }

    return Result.Ok(get_register(regs, "return_value"))
}
```

**特徴**:
- ✅ MIR Block構造に忠実
- ✅ Control flowが明示的
- ✅ TerminatorでBlock遷移

---

## 🔄 Phase 20.6以降（オプション）

### Option A: Pure Hakorune Path（推奨）

**前提**: Hakorune VM単体で完結

**Phase 20.6以降は不要**:
- ✅ VM実装完了
- ✅ すべての命令サポート済み
- ✅ テストカバレッジ十分
- ✅ CLI統合可能

**次のステップ**:
1. Hakorune VMをデフォルトに
2. Rust VMを `--backend vm-rust` (互換モード)
3. パフォーマンス最適化

### Option B: HostBridge Path（必要な場合のみ）

**条件**: C-ABI境界が他の理由で必要

**Phase 20.6（8週間）**:
- Week 1-4: HostBridge API実装
- Week 5-6: Rust最小化（~100行）
- Week 7-8: ドキュメント・検証

---

## ⚠️ リスク & 対策（更新版）

### リスク1: Hakorune VM のバグ

**問題**: 既存実装に未発見のバグがある可能性

**影響**: MEDIUM（Golden Testで検出可能）

**対策**:
- Golden Testで Rust-VM との一致を検証
- エッジケーステスト拡充
- ストレステスト実施
- バグ発見時は即座に修正

### リスク2: パフォーマンス

**問題**: Hakorune VM が Rust VM より遅い可能性

**影響**: LOW（機能的には問題なし）

**対策**:
- ベンチマーク測定
- ボトルネック特定
- 最適化は Phase 20.7以降
- 「動作正しさ」が最優先

### リスク3: CLI統合の複雑性

**問題**: Rust VM経由で Hakorune VM を呼び出す必要

**影響**: LOW（標準的なパターン）

**対策**:
- Rust VM の既存実行パスを再利用
- Box loading機構を活用
- エラーハンドリング整備

---

## 📚 関連リソース

### Phase 20.5ドキュメント
- **[HAKORUNE_VM_DISCOVERY.md](HAKORUNE_VM_DISCOVERY.md)** ⭐ 重大発見レポート
- **[STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md)** - 戦略変更の理由
- **[PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md)** - 全体計画（更新予定）
- [C_CODE_GENERATOR_DESIGN.md](C_CODE_GENERATOR_DESIGN.md) - ❌ 中止（参考用）

### Hakorune VM実装
- **Location**: `selfhost/hakorune-vm/` (3,413 lines)
- **Entry Point**: `hakorune_vm_core.hako`
- **Dispatcher**: `instruction_dispatcher.hako`
- **Tests**: `tests/*.hako` (26+ files)

### 前フェーズ
- [Phase 15.77 - 凍結EXE確定](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)

---

## 💬 開発体制

### 実装担当
- **tomoaki**: Hakorune VM実装完了 ✅（Oct 5-13, 2025）
- **ChatGPT**: Golden Test設計・CLI統合支援
- **Claude**: ドキュメント整備・検証スクリプト作成

### レビュー方針
- Week 2, 4, 6終了時にレビュー
- Golden Test すべて PASS が必須
- バグ発見時は即座に修正

---

## 🎉 成功後の世界

### Phase 20.5完了後（6週間）:

1. **Hakorune VM検証完了**: 22ハンドラーすべて動作確認
2. **Golden Testing完了**: Rust-VM vs Hako-VM 100%一致
3. **CLI統合完了**: `--backend vm-hako` で実行可能
4. **ドキュメント完備**: アーキテクチャ・移行ガイド
5. **次フェーズ判断**: HostBridge必要性の決定

### Option A: Pure Hakorune Path（推奨）

6. **Hakorune VMデフォルト化**: `--backend vm` で Hakorune VM 使用
7. **Rust VM互換モード化**: `--backend vm-rust` でRust VM（旧来）
8. **完全自己ホスト達成**: Hakorune IS Hakorune ✅

### Option B: HostBridge Path（必要な場合）

6. **HostBridge API実装**: C-ABI境界確立
7. **Rust最小化**: ~100行（C-ABI橋のみ）
8. **Phase 20.7以降**: パフォーマンス最適化、GC改良

---

**作成日**: 2025-10-14
**重大更新**: 2025-10-14（Hakorune VM発見による全面改訂）
**Phase開始予定**: 2025-12-21（Phase 15.77完了後）
**想定期間**: 6週間（36週間 → 6週間に短縮！）
**戦略**: 検証・統合（実装ではなく）
**成果**: Pure Hakorune VM 実現 ✅
