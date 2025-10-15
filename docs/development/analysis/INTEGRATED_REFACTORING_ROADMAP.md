# 統合リファクタリングロードマップ - Hakorune Project

**作成日**: 2025-10-15
**対象**: Hakorune全体（Rust 99,439行 + Hakorune 13,417行）
**目的**: Phase 20.5以降の継続的な品質改善とコード削減

---

## 📊 Executive Summary

### 全体状況
- **Rustコードベース**: 99,439行（762ファイル）
- **Hakoruneコードベース**: 13,417行（165ファイル）
- **削減可能見込み**: 7,186行（Rust層の7.2%）
- **テスト成功率**: 170/185 (91.9%)
- **Phase 20.5変更**: 36週間 → 6週間（Hakorune VM発見により）

### 核心メッセージ

🎯 **Phase 20.5は実装フェーズではなく、検証・統合フェーズ**

**Critical Discovery (2025-10-14)**:
- ✅ Hakorune VMは**100%完成済み**（3,413行、22ハンドラー）
- ✅ VM実装期間: わずか8日間（2025-10-05 → 10-13）
- ✅ MIRカバレッジ: 16命令 + 6拡張 = 138%

**戦略変更**:
```
旧計画: 36週間（VM実装 8週間 + Dispatch 6週間 + ...）
新計画: 6週間（検証 2週間 + Golden Test 2週間 + CLI統合 2週間）
```

---

## 🎯 3フェーズ戦略（優先度順）

### Phase 1: Quick Wins（1-2週間）⚡ 最優先
**目標**: 即座に削減可能な低リスク項目を一掃

**削減見込み**: 2,221行（2.2%）
**工数**: 4-8時間
**リスク**: 極低（参照ゼロファイル）

### Phase 2: 構造改善（2-4週間）🏗️ 高優先
**目標**: Phase 20.5完了後の大規模リファクタリング

**削減見込み**: 4,145行（4.2%）
**工数**: 2-3週間
**リスク**: 中（Plugin安定化待ち）

### Phase 3: 長期最適化（4-8週間）🔧 中優先
**目標**: selfhost compiler整理 + Backend統合

**削減見込み**: 1,820行（1.8%）
**工数**: 4-6週間
**リスク**: 中-高（設計変更伴う）

---

## 🚀 Phase 1: Quick Wins（1-2週間）

### 1.1 即座削除可能ファイル（影響ゼロ）

#### 📝 Task 1-1: バックアップファイル削除
**ファイル**: `src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047`

```bash
# 削除コマンド
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047

# 確認
git status
```

- **削減**: 327行
- **工数**: 5分
- **リスク**: なし（Gitに履歴あり）
- **担当**: User/Claude（即実行可）

---

#### 📝 Task 1-2: BID Codegen実験コード削除
**ディレクトリ**:
- `src/bid-codegen-from-copilot/`
- `src/bid-converter-copilot/`

```bash
# 1. READMEを確認
cat src/bid-codegen-from-copilot/README.md
cat src/bid-converter-copilot/README.md

# 2. 有用なアイデアがあればdocs/proposals/ideas/へ移動

# 3. 削除
rm -rf src/bid-codegen-from-copilot
rm -rf src/bid-converter-copilot

# 4. Cargo.tomlから参照削除確認
grep -r "bid-codegen\|bid-converter" Cargo.toml
```

- **削減**: 1,894行
- **工数**: 30分（README確認含む）
- **リスク**: なし（参照ゼロ確認済み）
- **担当**: User/Claude（即実行可）

---

#### 📝 Task 1-3: Plugin Legacy Proxy削除
**ファイル**: `src/runtime/plugin_box_legacy.rs`

```bash
# 1. 参照確認（再確認）
grep -r "plugin_box_legacy" src --include="*.rs"

# 2. 参照ゼロなら削除
rm src/runtime/plugin_box_legacy.rs

# 3. mod.rsから削除
# src/runtime/mod.rs内の "pub mod plugin_box_legacy;" をコメントアウト
```

- **削減**: 158行
- **工数**: 15分
- **リスク**: 低（参照ゼロ確認済み）
- **担当**: User/Claude（即実行可）

---

#### 📝 Task 1-4: 未使用警告修正
**場所**: 4箇所のコンパイル警告

```rust
// 1. src/runtime/type_registry.rs:92
// #[allow(dead_code)] 削除

// 2. src/runner/dispatch.rs:349
// use std::io::Write 削除

// 3. src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs:419
// box_type変数削除または使用

// 4. src/runner/mir_json_emit.rs:205
// entry_id_u32変数削除または使用
```

- **削減**: 4行（微小だが警告ゼロ化）
- **工数**: 15分
- **リスク**: なし
- **担当**: Claude（即実行可）

---

### Phase 1 成果物

✅ **削減合計**: 2,383行（2.4%）
✅ **コンパイル警告**: 4件 → 0件
✅ **クリーンアップ**: バックアップファイル完全削除
✅ **工数**: 4-8時間
✅ **リスク**: 極低

**テスト**:
```bash
# Phase 1完了後
cargo build --release
cargo test
tools/smokes/v2/run.sh --profile quick
```

**コミット例**:
```bash
git add -A
git commit -m "refactor(phase1): Quick Wins完了 - 即座削除可能ファイル一掃

- ✅ バックアップファイル削除: 327行
- ✅ BID Codegen実験コード削除: 1,894行
- ✅ Plugin Legacy Proxy削除: 158行
- ✅ 未使用警告修正: 4行
- ✅ 削減合計: 2,383行（2.4%）
- ✅ 全テストPASS

Phase 1完了。Phase 2（構造改善）へ。
"
git push
```

---

## 🏗️ Phase 2: 構造改善（2-4週間）

### 前提条件 ⚠️
**Phase 20.5完了を待つ** - Hakorune VM検証・CLI統合完了後に実施

### 2.1 Phase 20.5 - Hakorune VM検証・統合（6週間）

#### Week 1-2: VM検証・テスト拡充
**担当**: tomoaki + Claude

```bash
# 既存テストスイート実行
cd selfhost/hakorune-vm
for test in tests/*.hako; do
    echo "Testing: $test"
    NYASH_DISABLE_PLUGINS=1 ../../target/release/hako "$test"
done

# 期待: 26+ tests ALL PASS
```

**成果物**:
- [ ] 26個の既存テスト すべて PASS
- [ ] 22個のハンドラー動作確認
- [ ] エラーハンドリング（Result）動作確認
- [ ] @match dispatch 動作確認

---

#### Week 3-4: Golden Testing（Rust-VM vs Hako-VM）
**担当**: Claude + ChatGPT

**Golden Test実行スクリプト作成**:
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
        ((FAIL++))
    fi
done

echo "Golden Test Results: PASS=$PASS FAIL=$FAIL"
```

**テストケース**:
- [ ] 算術演算（10ケース）
- [ ] 制御フロー（10ケース）
- [ ] コレクション操作（10ケース）
- [ ] 再帰（5ケース）
- [ ] クロージャ（5ケース）
- [ ] 合計40ケース すべて一致

---

#### Week 5: CLI統合
**担当**: Claude

**実装**:
```rust
// src/backend/hakorune_vm_runner.rs (NEW)
pub fn run_hakorune_vm(mir_json: String) -> Result<i64> {
    // 1. selfhost/hakorune-vm/hakorune_vm_core.hako をロード
    // 2. Rust VMでHakoruneVmCoreBoxを実行
    // 3. HakoruneVmCoreBox.run(mir_json) を呼び出し
    // 4. 結果を返す
}

// src/cli.rs (MODIFY)
match backend {
    Backend::Vm => run_rust_vm(mir),
    Backend::VmHako => run_hakorune_vm(mir),  // NEW!
    Backend::Llvm => run_llvm(mir),
    Backend::Wasm => run_wasm(mir),
}
```

**成果物**:
- [ ] `--backend vm-hako` フラグ実装
- [ ] `HAKO_USE_HAKORUNE_VM=1` 環境変数対応
- [ ] エラーメッセージ整備

---

#### Week 6: ドキュメント整備
**担当**: Claude + ChatGPT

**ドキュメント**:
```
selfhost/hakorune-vm/
├── README.md                         # アーキテクチャ概要
├── DESIGN.md                         # 設計パターン詳解
├── TESTING.md                        # テスト戦略
└── CHANGELOG.md                      # 実装履歴（Oct 5-13）

docs/guides/
└── hakorune-vm-migration.md          # ユーザー移行ガイド

docs/development/roadmap/phases/phase-20.5/
├── VALIDATION_REPORT.md              # 検証レポート
└── COMPLETION_REPORT.md              # 完了報告
```

---

### 2.2 Legacy Code削除（Phase 20.5完了後）

#### 📝 Task 2-1: Legacy VM Handlers削除
**前提条件**: Phase 20.5完了 + Plugin安定化

**場所**:
- `src/backend/mir_interpreter/handlers/calls/legacy/`
- `src/backend/mir_interpreter/handlers/boxes/legacy/`

**削除手順**:
```bash
# 1. Plugin安定性確認（1週間テスト）
tools/smokes/v2/run.sh --profile integration
# 期待: 全PASS、1週間連続

# 2. Legacy handlers削除
rm -rf src/backend/mir_interpreter/handlers/calls/legacy
rm -rf src/backend/mir_interpreter/handlers/boxes/legacy

# 3. mod.rsから参照削除
# src/backend/mir_interpreter/handlers/calls/mod.rs
# src/backend/mir_interpreter/handlers/boxes/mod.rs

# 4. テスト
cargo build --release
tools/smokes/v2/run.sh --profile quick
```

- **削減**: 1,145行
- **工数**: 2-3日（テスト期間含む）
- **リスク**: 中（Plugin安定性依存）
- **担当**: User + ChatGPT（Phase 15.6完了後）

---

#### 📝 Task 2-2: src/boxes/ 完全削除
**前提条件**: Phase 15.6完了（Everything is Plugin）

**戦略**: 段階的削除（core系 → 拡張系）

**Phase 1（core系移行確認）**:
```bash
# core系Boxがplugins/に移行済みか確認
ls plugins/ | grep -E "array|map|string|integer|bool"

# スモークテスト
tools/smokes/v2/run.sh --profile quick
```

**Phase 2（拡張系移行確認）**:
```bash
# 拡張系Boxがplugins/に移行済みか確認
ls plugins/ | grep -E "file|net|json|math"

# 統合テスト
tools/smokes/v2/run.sh --profile integration
```

**Phase 3（削除実行）**:
```bash
# src/boxes/削除
rm -rf src/boxes

# mod.rsから参照削除
# src/runtime/mod.rs

# Cargo.toml確認
grep -r "src/boxes" Cargo.toml

# テスト
cargo build --release
tools/smokes/v2/run.sh --profile quick
```

- **削減**: 約3,000行（推定）
- **工数**: 1-2週間（段階的削除 + テスト）
- **リスク**: 高（実行基盤への影響大）
- **担当**: ChatGPT + User（Phase 15.6完了後）

---

#### 📝 Task 2-3: MIR Builder Legacy削除
**ファイル**: `src/mir/builder/exprs_legacy.rs`

```bash
# 1. 参照調査
grep -r "build_expression_impl_legacy\|exprs_legacy" src --include="*.rs"

# 2. 呼び出し箇所の移行
# exprs.rsのlegacy呼び出しを新実装に置き換え

# 3. 削除
rm src/mir/builder/exprs_legacy.rs

# 4. テスト
cargo test
```

- **削減**: 52行
- **工数**: 2-3時間
- **リスク**: 低（移行済み想定）
- **担当**: Claude

---

### Phase 2 成果物

✅ **削減合計**: 4,197行（4.2%）
✅ **Legacy完全削除**: calls/boxes/mir全て
✅ **src/boxes/削除**: Plugin system統一完了
✅ **工数**: 2-3週間（Phase 20.5含む）
✅ **リスク**: 中（Plugin安定化が鍵）

**コミット例**:
```bash
git add -A
git commit -m "refactor(phase2): 構造改善完了 - Legacy削除 + src/boxes統合

- ✅ Phase 20.5完了: Hakorune VM検証・CLI統合
- ✅ Legacy VM handlers削除: 1,145行
- ✅ src/boxes/削除: 3,000行
- ✅ MIR Builder legacy削除: 52行
- ✅ 削減合計: 4,197行（4.2%）
- ✅ Plugin system統一完了
- ✅ 全テストPASS

Phase 2完了。Phase 3（長期最適化）へ。
"
git push
```

---

## 🔧 Phase 3: 長期最適化（4-8週間）

### 3.1 Selfhost Compiler整理

#### 📝 Task 3-1: 重複ファイル統一（.nyash → .hako）
**参照**: [selfhost-super-refactoring Master Plan](selfhost-super-refactoring/reports/refactoring_master_plan.md)

**対象**: 14組のファイルペア

```bash
# 各ファイルペアごとに:
# 1. 差分確認
diff -u file.nyash file.hako

# 2. .hakoに統合
# 3. .nyash削除
git rm file.nyash

# 4. テスト実行
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"
```

**優先順**:
1. interfaces.nyash → interfaces.hako
2. parser/lexer.nyash → parser/lexer.hako
3. parser/parser.nyash → parser/parser.hako
4. ... (14組)

- **削減**: 約300行（重複解消）
- **工数**: 2-3日
- **リスク**: 中（統合ミスの可能性）
- **担当**: Claude + User

---

#### 📝 Task 3-2: parser_box.hako分割（921行 → 3箱）

**分割戦略**:
```
parser_box.hako (921行)
  ↓
├─ lexer_box.hako (~300行)          // 字句解析専用
├─ parser_core_box.hako (~400行)    // 構文解析コア
└─ ast_builder_box.hako (~250行)    // AST構築
```

**手順**:
1. 責務分析（30分）
2. インターフェース設計（30分）
3. 分割実装（90分）
4. テスト（30分）

- **削減**: 0行（分割のみ、保守性向上）
- **工数**: 3時間
- **リスク**: 中（責務分離の設計）
- **担当**: Claude

---

#### 📝 Task 3-3: pipeline_v2/ 構造整理

**現状**: 17箱が並列配置

**整理後**:
```
pipeline_v2/
├─ core/              // コア箱
├─ extractors/        // 抽出系
├─ emitters/          // 発行系
├─ flows/             // フロー系
├─ utils/             // ユーティリティ
└─ ssa/               // SSA関連
```

- **削減**: 0行（構造化のみ）
- **工数**: 1-2時間（ファイル移動 + using更新）
- **リスク**: 低
- **担当**: Claude

---

### 3.2 Backend統合・整理

#### 📝 Task 3-4: Cranelift JIT削除判断
**ファイル**: `src/runner/modes/cranelift.rs`

**判断基準**:
```bash
# 計画確認
grep -r "cranelift" docs/development/roadmap --include="*.md"

# 結果:
# - 計画あり → docs/proposals/ideas/へ移動
# - 計画なし → 削除
```

- **削減**: 45行
- **工数**: 30分（計画確認）
- **リスク**: 低
- **担当**: User判断 + Claude実行

---

#### 📝 Task 3-5: AOT Backend統合判断
**ファイル**: `src/backend/aot/`

**判断基準**:
- Phase 9 AOT計画の現状確認
- WASM backendとの統合計画確認

- **削減**: 約350行
- **工数**: 1日（計画確認 + 判断）
- **リスク**: 中
- **担当**: User判断 + ChatGPT

---

#### 📝 Task 3-6: LLVM Legacy削除
**前提条件**: Python/llvmlite完全移行

**場所**:
- `src/backend/llvm/` (deprecated shim)
- `src/backend/llvm_legacy/`

```bash
# 1. Python版カバレッジ確認
grep -r "llvm_py" src --include="*.rs"

# 2. Legacy使用箇所確認
grep -r "llvm_legacy\|inkwell" src --include="*.rs"

# 3. feature flag確認
grep "llvm-inkwell-legacy" Cargo.toml

# 4. 削除判断
```

- **削減**: 約500行（推定）
- **工数**: 2-3日（カバレッジ確認）
- **リスク**: 中-高（LLVM backend依存）
- **担当**: ChatGPT + User

---

### 3.3 ドキュメント・品質向上

#### 📝 Task 3-7: INTERFACES.md完全更新
**参照**: [selfhost-super-refactoring Master Plan](selfhost-super-refactoring/reports/refactoring_master_plan.md) Phase 3

**追加内容**:
1. 全箱のインターフェース定義
2. 依存関係マトリックス
3. 契約（Contracts）強化

- **削減**: 0行（ドキュメント追加）
- **工数**: 2時間
- **リスク**: 低
- **担当**: Claude

---

#### 📝 Task 3-8: TODO/FIXME整理
**場所**: 44箇所（30ファイル）

**手順**:
1. 緊急度でトリアージ
2. Issueトラッカーへ移行
3. 完了済みコメント削除

- **削減**: 約50行（コメント削減）
- **工数**: 2-3時間
- **リスク**: 低
- **担当**: Claude

---

### Phase 3 成果物

✅ **削減合計**: 1,245行（1.2%）
✅ **Selfhost compiler整理**: 重複削除 + 構造化
✅ **Backend統合**: Cranelift/AOT/LLVM legacy判断
✅ **ドキュメント完備**: INTERFACES.md v2.0
✅ **工数**: 4-6週間
✅ **リスク**: 中-高（設計判断含む）

**コミット例**:
```bash
git add -A
git commit -m "refactor(phase3): 長期最適化完了 - Selfhost整理 + Backend統合

- ✅ Selfhost重複ファイル統一: 300行削減
- ✅ parser_box分割: 3箱化（保守性向上）
- ✅ pipeline_v2構造化: 6ディレクトリ
- ✅ Backend統合判断: Cranelift/AOT/LLVM
- ✅ INTERFACES.md v2.0完成
- ✅ TODO/FIXME整理: 50行削減
- ✅ 削減合計: 1,245行（1.2%）
- ✅ 全テストPASS

Phase 3完了。統合リファクタリング完遂！🎉
"
git push
```

---

## 📊 総合成果指標（KPI）

### 定量指標

| 指標 | 現状 | Phase 1後 | Phase 2後 | Phase 3後 | 削減率 |
|-----|------|-----------|-----------|-----------|--------|
| **Rust総行数** | 99,439 | 97,056 | 92,859 | 91,614 | **7.9%** |
| **削除ファイル数** | - | 3 | 60+ | 63+ | - |
| **Legacy残存** | Yes | Yes | **No** | **No** | - |
| **src/boxes/** | 57ファイル | 57 | **0** | **0** | **100%** |
| **テスト成功率** | 91.9% | 91.9% | **95%+** | **95%+** | +3.1% |
| **コンパイル警告** | 4件 | **0件** | **0件** | **0件** | **100%** |

### 削減内訳

| Phase | 削減行数 | 削減率 | 主要項目 |
|-------|---------|--------|---------|
| **Phase 1** | 2,383 | 2.4% | バックアップ、BID Codegen、Plugin Legacy |
| **Phase 2** | 4,197 | 4.2% | Legacy handlers、src/boxes/、MIR legacy |
| **Phase 3** | 1,245 | 1.2% | Selfhost重複、Backend統合、TODO整理 |
| **合計** | **7,825** | **7.9%** | - |

### 定性指標

- ✅ **モジュール結合度**: Low → 箱間依存が明示的
- ✅ **コードの凝集度**: High → 1箱1責務（Phase 3後）
- ✅ **保守性**: 高 → INTERFACES.md完備
- ✅ **テストカバレッジ**: 十分 → Golden Test追加
- ✅ **Plugin安定性**: 100% → Legacy完全削除
- ✅ **Hakorune VM**: 統合完了 → `--backend vm-hako`動作

---

## ⏱️ 工数見積もり

### フェーズ別工数

| Phase | 内容 | 最短 | 最長 | 平均 |
|-------|------|------|------|------|
| **Phase 1** | Quick Wins | 4h | 8h | **6h** |
| **Phase 2** | 構造改善（Phase 20.5含む） | 3週間 | 5週間 | **4週間** |
| **Phase 3** | 長期最適化 | 4週間 | 8週間 | **6週間** |
| **合計** | | **8週間** | **14週間** | **11週間** |

### 推奨実行スケジュール

```
Week 1:
  Day 1-2: Phase 1（Quick Wins） - 6時間

Week 2-5: Phase 2（構造改善）
  Week 2-4: Phase 20.5（Hakorune VM検証・統合） - 3週間
  Week 5: Legacy削除 - 1週間

Week 6-11: Phase 3（長期最適化）
  Week 6-7: Selfhost compiler整理 - 2週間
  Week 8-9: Backend統合・判断 - 2週間
  Week 10-11: ドキュメント・品質向上 - 2週間
```

### 並行作業可能性

**Phase 1**: 独立実行可能（今すぐ開始可）
**Phase 2**: Phase 20.5が前提（Week 2-4待機）
**Phase 3**: Phase 2完了後（Week 6-開始）

---

## 🚨 リスク評価と緩和策

### リスク1: Phase 20.5 Hakorune VM統合失敗
**確率**: 低（VM実装済み）
**影響**: 高（Phase 2ブロック）

**軽減策**:
- ✅ Golden Testing徹底（40ケース）
- ✅ 段階的統合（検証 → CLI → デフォルト化）
- ✅ Rust VM互換モード維持（ロールバック可能）

---

### リスク2: Plugin安定性不足
**確率**: 中
**影響**: 高（Legacy削除不可）

**軽減策**:
- ✅ 1週間連続テスト（Phase 2前）
- ✅ プラグイン個別テスト拡充
- ✅ Legacy handlers一時保持（緊急時復旧）

---

### リスク3: Selfhost compiler分割ミス
**確率**: 中
**影響**: 中（責務不明確化）

**軽減策**:
- ✅ 責務分析を慎重に実施（Phase 3）
- ✅ INTERFACES.md完全同期
- ✅ 各分割後にテスト実行

---

### リスク4: Backend統合判断ミス
**確率**: 中
**影響**: 中-高（将来計画への影響）

**軽減策**:
- ✅ User判断を仰ぐ（Cranelift/AOT/LLVM）
- ✅ docs/proposals/ideas/へ移動（削除前保存）
- ✅ Git履歴保持（復元可能）

---

### リスク5: 工数超過（Phase 3）
**確率**: 高
**影響**: 低（Phase 3は優先度低い）

**軽減策**:
- ✅ Phase毎に完了判定（80/20ルール）
- ✅ 優先度低いタスクはスキップ可
- ✅ Phase 1/2完了が最重要

---

## 🎯 成功の定義

### 必須条件（Must Have）

#### Phase 1完了:
- ✅ バックアップファイル 0個
- ✅ BID Codegen実験コード削除
- ✅ コンパイル警告 0件
- ✅ 全テストPASS

#### Phase 2完了:
- ✅ Phase 20.5完了（Hakorune VM統合）
- ✅ Legacy handlers削除（calls/boxes/mir）
- ✅ src/boxes/削除（Plugin統一）
- ✅ 全テストPASS（成功率95%+）

#### Phase 3完了:
- ✅ Selfhost重複ファイル 0個
- ✅ parser_box分割完了（3箱）
- ✅ INTERFACES.md v2.0完成
- ✅ 全テストPASS

### 推奨条件（Should Have）

- ✅ Hakorune VMデフォルト化
- ✅ Backend統合判断完了
- ✅ ドキュメント完備（5+ドキュメント）

### 理想条件（Nice to Have）

- ✅ パフォーマンス改善 10%+
- ✅ テストカバレッジ 100%（Golden Test含む）
- ✅ Rust層 10,000行削減

---

## 📚 関連リソース

### 重要ドキュメント

#### Phase 20.5関連:
- **[Phase 20.5 README](../roadmap/phases/phase-20.5/README.md)** ⭐全面改訂済み
- **[HAKORUNE_VM_DISCOVERY](../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md)** ⭐発見レポート
- **[STRATEGY_RECONCILIATION](../roadmap/phases/phase-20.5/STRATEGY_RECONCILIATION.md)** - 戦略比較

#### リファクタリング計画:
- **[Selfhost Super Refactoring Master Plan](proposals/ideas/refactoring/selfhost-super-refactoring/reports/refactoring_master_plan.md)**
- **[Legacy Code Detection Report](legacy-code-detection-report.md)**

#### テスト・品質:
- **[TEST_COMPLEXITY_REPORT](TEST_COMPLEXITY_REPORT.md)**
- **[00_MASTER_ROADMAP](../roadmap/phases/00_MASTER_ROADMAP.md)**

#### 開発原則:
- **[CLAUDE.md](../../../CLAUDE.md)** - 箱理論・Fail-Fast原則

---

## 🔄 次のアクション（優先順）

### 🚨 今すぐ実行可能（Phase 1）

```bash
# 1. バックアップファイル削除（5分）
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047

# 2. BID Codegen実験コード削除（30分）
cat src/bid-codegen-from-copilot/README.md  # 確認
rm -rf src/bid-codegen-from-copilot
rm -rf src/bid-converter-copilot

# 3. Plugin Legacy Proxy削除（15分）
grep -r "plugin_box_legacy" src --include="*.rs"  # 再確認
rm src/runtime/plugin_box_legacy.rs

# 4. 警告修正（15分）
# type_registry.rs, dispatch.rs, ffi_bridge.rs, mir_json_emit.rs

# 5. テスト
cargo build --release
tools/smokes/v2/run.sh --profile quick

# 6. コミット
git add -A
git commit -m "refactor(phase1): Quick Wins完了"
git push
```

**所要時間**: 1-2時間
**削減**: 2,383行（2.4%）

---

### ⏳ Phase 2開始前（Phase 20.5待機）

**Week 2-4（Phase 20.5実施）**:
1. Hakorune VM検証（Week 1-2）
2. Golden Testing（Week 3-4）
3. CLI統合（Week 5）
4. ドキュメント整備（Week 6）

**Week 5（Legacy削除）**:
1. Plugin安定性確認（1週間連続テスト）
2. Legacy handlers削除
3. src/boxes/削除

**所要時間**: 4週間
**削減**: 4,197行（4.2%）

---

### 📝 Phase 3開始（Phase 2完了後）

**Week 6-11（長期最適化）**:
1. Selfhost compiler整理（Week 6-7）
2. Backend統合判断（Week 8-9）
3. ドキュメント・品質向上（Week 10-11）

**所要時間**: 6週間
**削減**: 1,245行（1.2%）

---

## 🎊 完了後の展望

### 即座に得られる効果（Phase 2完了後）

1. **開発速度向上**: Legacy削除 → コードナビゲーション高速化
2. **バグ減少**: Plugin統一 → フォールバック経路排除
3. **Hakorune VM統合**: `--backend vm-hako` → Pure Hakorune実行
4. **テスト安定性**: Golden Test → Rust-VM/Hako-VM完全一致保証

### 中長期的効果（Phase 3完了後）

1. **保守性向上**: INTERFACES.md完備 → 新規参入容易化
2. **モジュール性**: Selfhost整理 → 責務明確化
3. **Backend統合**: 計画明確化 → 将来拡張容易
4. **技術的負債削減**: 7,825行削減 → クリーンなコードベース

### Phase 20.6以降の可能性

#### Option A: Pure Hakorune Path（推奨）

**前提**: Hakorune VM単体で完結

**Phase 20.6以降は不要**:
- ✅ VM実装完了
- ✅ すべての命令サポート済み
- ✅ CLI統合可能

**次のステップ**:
1. Hakorune VMをデフォルトに
2. Rust VMを `--backend vm-rust` (互換モード)
3. パフォーマンス最適化

#### Option B: HostBridge Path（必要な場合のみ）

**条件**: C-ABI境界が他の理由で必要

**Phase 20.6（8週間）**:
- Week 1-4: HostBridge API実装
- Week 5-6: Rust最小化（~100行）
- Week 7-8: ドキュメント・検証

---

## 📞 サポート・相談

### 迷ったら

- **Phase 1**: 即座実行（リスク極低）
- **Phase 2**: Phase 20.5完了待ち → tomoaki確認
- **Phase 3**: User判断を仰ぐ（Backend統合等）

### エラー時

- Fail-Fast原則に従い、エラーメッセージを精査
- git logで前回動作時のコミットを確認
- 必要ならgit revertでロールバック

### 改善提案

- アイデアは `docs/development/proposals/ideas/` に記録
- 80/20ルール適用（すべてを今やらない）

---

## 🎯 最終メッセージ

この統合リファクタリングロードマップは、**Phase 20.5の重大発見**（Hakorune VM完成）を受けて、**実行可能な3フェーズ戦略**に再編されました。

### 核心原則

1. **段階的実行**: Phase 1 → Phase 2 → Phase 3（各Phase完了判定あり）
2. **Fail-Fast**: エラーは隠さず即座に失敗（フォールバック禁止）
3. **Box-First**: すべてを箱で分離・固定（いつでも戻せる）
4. **80/20ルール**: 完璧より進捗（Phase 3は柔軟に調整可）

### 今日から始められること

✅ **Phase 1を今すぐ実行**（1-2時間で2,383行削減）

```bash
# Step 1: バックアップ削除（5分）
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047

# Step 2: BID Codegen削除（30分）
rm -rf src/bid-codegen-from-copilot src/bid-converter-copilot

# Step 3: Plugin Legacy削除（15分）
rm src/runtime/plugin_box_legacy.rs

# Step 4: 警告修正（15分）
# type_registry.rs, dispatch.rs, ffi_bridge.rs, mir_json_emit.rs

# Step 5: テスト＆コミット（30分）
cargo build --release && tools/smokes/v2/run.sh --profile quick
git add -A && git commit -m "refactor(phase1): Quick Wins完了" && git push
```

---

**🚀 Let's refactor with Box-First philosophy and Fail-Fast culture!**

**📅 作成日**: 2025-10-15
**👤 作成者**: Claude (Task 8 - Integration)
**📊 分析基盤**: Task 1-7結果 + Phase 20.5 Discovery + Master Roadmap
**🎯 目標**: 実行可能・段階的・後戻り可能なリファクタリング計画
