# Phase 20.5 - Escape from Rust (Pure Hakorune Strategy)

**期間**: 2025-12-21 - 2026-02-28 (10週間)
**状態**: Planning (Pure Hakorune Strategy Adopted)
**前提**: Phase 15.77完了 (凍結EXE確定)
**戦略変更**: C Code Generator → Pure Hakorune VM (2025-10-14)

---

## 🎯 このフェーズで実現すること

**"Rust=floor, Hakorune=house" - VM自体をHakoruneで実装**

1. **HostBridge API完成**: C-ABI境界の最小化（Rust↔Hakorune）
2. **op_eq移行**: 等価性ロジックをRust→Hakoruneへ（NoOperatorGuard実装）
3. **VM Foundations実装**: 命令ディスパッチの概念実証（5命令）
4. **Pure Hakorune Roadmap**: Phase 20.6-15.82への詳細計画

**注意**: C Code Generator実装は**中止**（Pure Hakorune戦略採用のため）

---

## 💡 このフェーズの位置づけ

### Pure Hakorune戦略（6フェーズ）のPhase A+B開始

```
Phase 15.77（凍結EXE確定）✅
├── hako-frozen-v1.exe (724KB MSVC)
├── NyRT関数呼び出し可能
└── MIR JSON → .o → EXE 導線確立

Phase 20.5（Pure Hakorune基盤）← 今ここ
├── Phase A: HostBridge API（Week 1-4）✅
├── Phase B開始: VM Foundations（Week 5-8）
│   ├── op_eq移行（Week 5-6）
│   └── 命令ディスパッチPoC（Week 7-8）
└── ドキュメント整備（Week 9-10）

Phase 20.6（VM Core完成）
├── Phase B完了: 16命令すべて実装
└── Phase C: ディスパッチ統一（Resolver）

Phase 20.7（コレクション）
└── Phase D: MapBox/ArrayBox in Hakorune

Phase 20.8（GC + Rust非推奨化）
├── Phase E: GC v0（Mark & Sweep）
└── Phase F: Rust VM互換モード

合計: 36週間（Phase 20.5-15.82）
```

---

## 🏆 成功基準（DoD）

### 1️⃣ HostBridge API完成

```bash
# C-ABI関数が動作
./test_hostbridge_abi  # Ubuntu
./test_hostbridge_abi.exe  # Windows

# 5コア関数実装
# - Hako_RunScriptUtf8
# - Hako_Retain / Hako_Release
# - Hako_ToUtf8
# - Hako_LastError
```

- [ ] 5コア関数実装完了
- [ ] Ubuntu ABIテスト PASS
- [ ] Windows ABIテスト PASS
- [ ] エラーハンドリング（TLS）動作確認

### 2️⃣ op_eq移行完了

```bash
# Hakorune側op_eqが動作
HAKO_USE_PURE_EQ=1 ./hako test.hako

# Golden Test: Rust vs Hako
./test_op_eq_golden.sh
# Expected: 100% parity
```

- [ ] NoOperatorGuard実装
- [ ] 8種類の比較実装（int/bool/null/string/array/map/enum/user）
- [ ] Golden Test: 20個すべて PASS
- [ ] パフォーマンス: Hako-VM ≥ 70% of Rust-VM

### 3️⃣ VM Foundations PoC

```bash
# 5命令が動作
./hako --backend vm-hako simple_program.hako
# 命令: const, binop, compare, jump, ret
```

- [ ] 5命令実装（const, binop, compare, jump, ret）
- [ ] MIR実行ループ（Hakorune実装）
- [ ] 統合テスト: 簡単なプログラム実行可能
- [ ] パフォーマンス: 測定可能

### 4️⃣ ドキュメント整備

- [ ] STRATEGY_RECONCILIATION.md（戦略変更の理由）
- [ ] HOSTBRIDGE_API_DESIGN.md（C-ABI仕様）
- [ ] OP_EQ_MIGRATION.md（等価性実装ガイド）
- [ ] PURE_HAKORUNE_ROADMAP.md（Phase 20.5-15.82計画）
- [ ] Phase 20.6計画書

---

## 📊 週次計画（Week 1-10）

### Week 1-2（2025-12-21 - 2026-01-03）HostBridge API設計・実装

**目標**: C-ABI境界の確立

#### タスク
- [ ] HostBridge API設計（5コア関数）
- [ ] HandleRegistry実装（Rust側）
- [ ] C header生成（hakorune_hostbridge.h）
- [ ] 基本実装（RunScriptUtf8, Retain/Release）

#### 成果物
```
src/hostbridge/
├── mod.rs                     # C-ABI exports
├── handle_registry.rs         # Handle管理
└── tests.rs                   # Unit tests

include/
└── hakorune_hostbridge.h      # C header
```

### Week 3-4（2026-01-04 - 01-17）HostBridge API完成・テスト

**目標**: Ubuntu/Windows ABIテスト PASS

#### タスク
- [ ] ToUtf8, LastError実装
- [ ] エラーハンドリング（TLS）
- [ ] ABIテスト作成（C言語）
- [ ] Ubuntu/Windows検証

#### 成果物
```
tests/
├── hostbridge_abi_test.c      # C ABIテスト
└── hostbridge_integration/    # 統合テスト
    └── test_hostbridge_call.hako

tools/
└── test_hostbridge_abi.sh     # テストスクリプト
```

### Week 5-6（2026-01-18 - 01-31）op_eq移行

**目標**: 等価性ロジックをRust→Hakoruneへ

#### タスク
- [ ] NoOperatorGuard実装（再帰防止）
- [ ] 8種類の比較実装（int/bool/null/string/array/map/enum/user）
- [ ] Golden Test作成（20ケース）
- [ ] Rust-VM vs Hako-VM パリティ検証

#### 成果物
```
apps/hakorune-vm/
├── op_eq_box.hako             # 等価性実装
├── no_operator_guard_box.hako # Guard実装
└── tests/                     # Unit tests

tests/golden/op_eq/            # Golden tests
├── primitives.hako
├── arrays.hako
├── maps.hako
├── recursion.hako
└── user_defined.hako
```

### Week 7-8（2026-02-01 - 02-14）VM Foundations PoC

**目標**: 命令ディスパッチの概念実証（5命令）

#### タスク
- [ ] MIR実行ループ（Hakorune実装）
- [ ] 5命令実装（const, binop, compare, jump, ret）
- [ ] 統合テスト（簡単なプログラム実行）
- [ ] パフォーマンス測定

#### 成果物
```
apps/hakorune-vm/
├── mini_vm_box.hako           # VM実行ループ
├── instruction_dispatch.hako  # ディスパッチ
└── tests/                     # 統合テスト

tests/vm_poc/                  # VM PoC tests
├── hello.hako                 # Hello World
├── arithmetic.hako            # 算術演算
└── control_flow.hako          # 制御フロー
```

### Week 9（2026-02-15 - 02-21）統合テスト・パフォーマンス測定

**目標**: 全コンポーネント統合動作確認

#### タスク
- [ ] HostBridge + op_eq + VM PoC 統合
- [ ] E2Eテスト（10ケース）
- [ ] パフォーマンス測定・比較
- [ ] 問題点の特定・修正

#### 検証スクリプト
```bash
# tools/phase_15_79_verify.sh

#!/bin/bash
set -e

echo "Test 1: HostBridge API"
./test_hostbridge_abi

echo "Test 2: op_eq Golden Tests"
./test_op_eq_golden.sh

echo "Test 3: VM PoC"
./hako --backend vm-hako tests/vm_poc/arithmetic.hako

echo "✅ PASS: Phase 20.5 統合テスト"
```

### Week 10（2026-02-22 - 02-28）ドキュメント・Phase 20.6計画

**目標**: ドキュメント整備、次フェーズ計画

#### タスク
- [ ] 戦略変更文書（STRATEGY_RECONCILIATION.md）
- [ ] HostBridge API仕様（HOSTBRIDGE_API_DESIGN.md）
- [ ] op_eq移行ガイド（OP_EQ_MIGRATION.md）
- [ ] Pure Hakorune Roadmap（PURE_HAKORUNE_ROADMAP.md）
- [ ] Phase 20.6計画書作成
- [ ] 完了報告書作成

#### 成果物
```
docs/development/roadmap/phases/phase-20.5/
├── STRATEGY_RECONCILIATION.md # 戦略変更理由
├── HOSTBRIDGE_API_DESIGN.md   # C-ABI仕様
├── OP_EQ_MIGRATION.md         # 等価性ガイド
├── PURE_HAKORUNE_ROADMAP.md   # 全体計画
├── COMPLETION_REPORT.md       # 完了報告
└── LESSONS_LEARNED.md         # 学び

docs/development/roadmap/phases/phase-20.6/
└── README.md                  # Phase 20.6計画
```

---

## 🎯 実装戦略の決定

### 戦略変更: C Code Generator → Pure Hakorune（2025-10-14）

**原案（Task Agent）**:
- C Code Generator実装（500行）
- MIR JSON → Cコード変換
- 10週間でBootstrap達成

**新戦略（ChatGPT Pro + User承認）**:
- VM自体をHakoruneで実装
- Rust = 薄い橋（C-ABI のみ）
- 36週間（Phase 20.5-15.82）で完全実現

### Pure Hakorune戦略の優位性

**アーキテクチャの美しさ**:
```
C Generator:  Hakorune→MIR→C→EXE（複数パス）
Pure Hakorune: Hakorune→MIR→Hako-VM実行（単一パス）
```

**長期保守性**:
- ✅ 実行パス単一（Hakorune VMのみ）
- ✅ VM意味論を一箇所で定義（Hakorune内）
- ✅ 外部コンパイラ依存なし（clang/gcc不要）
- ✅ デバッグ容易（すべてHakorune内）

**過去の学びの反映**:
- Equals/== 再帰バグ → NoOperatorGuard で構造的に解決
- Handle管理複雑性 → 最小C-ABI（Retain/Release）
- ディスパッチ断片化 → 単一解決パス（Resolver）

**究極のBox理論実現**:
> Everything is Box、VMもBox。

### Phase 20.5での実現範囲

**✅ 10週間で完成**:
1. HostBridge API（C-ABI境界）
2. op_eq移行（等価性をHakoruneへ）
3. VM Foundations PoC（5命令）

**⏸️ Phase 20.6+へ延期**:
4. VM Core完成（16命令すべて）
5. Dispatch統一（Resolver経由）
6. Collections in Hakorune
7. GC v0
8. Rust VM非推奨化

---

## 🎯 Pure Hakorune 3つの柱

ChatGPT Pro提案の"三位一体"（不変原則）:

### 1. 唯一の境界 = C-ABI（HostBridge）

```
Rust (floor)          ┃          Hakorune (house)
OS/FFI/File/Process   ┃   VM/Collections/Dispatch/etc.
     ~100 lines       ┃        Everything Else
                      ┃
                  HostBridge (C-ABI)
              ┃
        5 functions only:
        - Hako_RunScriptUtf8
        - Hako_Retain / Release
        - Hako_ToUtf8
        - Hako_LastError
```

### 2. 唯一の呼び出し = MethodHandle

```hakorune
// すべての呼び出しはこれ1つ
ExecBox.call_by_handle(handle, args, NoOperatorGuard)

// handle の取得元:
Resolver.lookup(type_id, method, arity) -> MethodHandle
```

**重要**: NoOperatorGuard で equals/== 再帰を構造的に防止

### 3. 唯一の解決 = Resolver

```hakorune
// 特別扱いなし、すべてResolver経由
Resolver.lookup(type_id, method, arity) -> MethodHandle

// 例: Array.push
local handle = Resolver.lookup(arr_type_id, :push, 1)
ExecBox.call_by_handle(handle, [arr, value], NoOperatorGuard)
```

---

## ⚠️ リスク & 対策

### リスク1: タイムラインの長期化（36週間）

**問題**: 原案10週間→新戦略36週間

**影響**: HIGH（完全自己ホスト達成の遅延）

**対策**:
- Phase 20.5で独立価値提供（HostBridge, op_eq, VM PoC）
- 各フェーズ独立（中断・再開可能）
- 段階的リスク低減（簡単→難しい順）
- Phase 20.6以降の優先度再評価可能

### リスク2: 実装複雑性の増大

**問題**: VM in Hakoruneは C Generator より難しい

**影響**: MEDIUM（実装工数増加）

**対策**:
- Rust VMを参考実装として活用
- Golden Test で早期バグ検出
- 段階的実装（5命令→16命令）
- ChatGPT/Claude協調開発

### リスク3: Rust VM二重保守

**問題**: 移行期間中の二重保守負担

**影響**: MEDIUM（保守工数）

**対策**:
- Phase 20.5後 Rust VM凍結（新機能なし）
- 新規開発はHakorune VMのみ
- 明確な非推奨化タイムライン（Phase 20.8）
- Golden Test で乖離防止

### リスク4: C-ABI安定性

**問題**: HostBridge API変更の可能性

**影響**: LOW（十分理解された境界）

**対策**:
- 最小API（5関数のみ）
- 実績あるデザイン（Lua/Python C API参考）
- バージョン関数（Hako_ApiVersion）
- 包括的ABIテスト（Ubuntu/Windows）

---

## 📚 関連リソース

### Phase 20.5ドキュメント
- **[STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md)** - 戦略変更の理由・2戦略比較
- **[HOSTBRIDGE_API_DESIGN.md](HOSTBRIDGE_API_DESIGN.md)** - C-ABI仕様・実装ガイド
- **[OP_EQ_MIGRATION.md](OP_EQ_MIGRATION.md)** - 等価性実装ガイド・NoOperatorGuard
- **[PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md)** - Phase 20.5-15.82 全体計画
- **[CHATGPT_PURE_HAKORUNE_STRATEGY.md](CHATGPT_PURE_HAKORUNE_STRATEGY.md)** - ChatGPT Pro原案
- [C_CODE_GENERATOR_DESIGN.md](C_CODE_GENERATOR_DESIGN.md) - ❌ 中止（参考用）
- [BOOTSTRAP_CHAIN_ANALYSIS.md](BOOTSTRAP_CHAIN_ANALYSIS.md) - ❌ 旧戦略（参考用）

### 前フェーズ
- [Phase 15.77 - 凍結EXE確定](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)

### 参考実装
- [apps/selfhost-compiler/](../../../../apps/selfhost-compiler/) - Hakoruneコンパイラ（参考）
- Rust VM実装 - `src/backend/mir_interpreter/` （Pure Hakorune参考実装）

### 業界パターン
- **Lua**: C-APIミニマル（5-10関数）
- **Python**: C-API + Pure Python stdlib
- **Rust**: stage0（凍結）→ stage1（ブートストラップ）→ stage2（検証）
- **Go**: Go 1.4 frozen → Go 1.5 self-hosted

---

## 💬 開発体制

### 実装担当
- **ChatGPT**: HostBridge API・op_eq移行 実装主導
- **Claude**: レビュー・ドキュメント整備・統合テスト
- **tomoaki**: 戦略決定・最終承認

### レビュー方針
- 各Week終了時にレビュー
- Golden Test で Rust-VM パリティ維持
- 問題発生時は即座にロールバック可能

---

## 🎉 成功後の世界

### Phase 20.5完了後（10週間）:

1. **HostBridge API確立**: Rust↔Hakorune境界明確化
2. **op_eq in Hakorune**: 等価性の正しさ向上（NoOperatorGuard）
3. **VM PoC成功**: Hakorune-VM実現可能性実証
4. **Phase 20.6への道筋**: 詳細計画完成

### Phase 20.8完了後（36週間、2026-09-30）:

5. **完全自己ホスト達成**: Hakorune IS Hakorune
6. **Rust最小化**: ~100行（C-ABI橋のみ）
7. **究極のBox理論**: VMもBoxで実装
8. **長期保守性**: 単一パス実行、単一コードベース

---

**作成日**: 2025-10-14
**戦略変更日**: 2025-10-14（Pure Hakorune採用）
**Phase開始予定**: 2025-12-21（Phase 15.77完了後）
**想定期間**: 10週間（Phase 20.5のみ）
**完全実現**: 36週間（Phase 20.5-15.82）
**戦略**: Pure Hakorune（"Rust=floor, Hakorune=house"）
