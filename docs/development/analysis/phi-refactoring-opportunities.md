# PHI関連リファクタリング調査レポート

**調査日**: 2025-10-17
**調査範囲**: PHI生成・検証・実行コード全域
**目的**: 重複コード削減・箱化候補の特定

---

## エグゼクティブサマリー

**発見**: PHI処理は既に**高度に箱化・共通化済み** ✅

- **phi_core/**: if/loop PHI生成ロジックが既に分離済み
- **PhiMergeHelper**: 到達可能predecessor判定が統一済み
- **LoopCarrierAnalyzerBox**: ループ変数検出が箱化済み
- **PhiHandlerBox (Python/Hakorune)**: 各バックエンドで箱化実装済み

**結論**: 大規模リファクタリングの必要性は**低い**。小規模な改善機会のみ。

---

## 1. 重複コード分析

### 1.1 Predecessor判定の重複 ✅ **解決済み**

**重複箇所**:
- `src/mir/builder/phi_merge_helper.rs:27-45` - `compute_if_merge_preds()`
- `src/mir/phi_core/common.rs:15-19` - `is_unreachable_pred()`
- `src/mir/verification/phi_inputs.rs:36-38` - 同じロジック

**現状**: 既に`PhiMergeHelper`に統一されている ✅

**難易度**: N/A（既に解決済み）

---

### 1.2 PHI入力検証の重複 ⚠️ **軽微な重複**

**重複箇所**:
1. **ビルド時検証** (`src/mir/builder/phi_merge_helper.rs:107`)
   ```rust
   crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
   ```

2. **事後検証** (`src/mir/verification/phi_inputs.rs:6-52`)
   ```rust
   pub fn check_phi_inputs_cover_predecessors(function: &MirFunction) -> Result<...>
   ```

**現状**: 同じ検証ロジックが2箇所に存在

**共通化の難易度**: **Medium**
- ビルド時 = debug_assertions下でのassert
- 事後検証 = Verifier passでの完全検証
- 責務が異なるため、共通化よりも「同じ検証関数を呼び出す」形が適切

**提案**: `phi_core::common::validate_phi_inputs()` を作成し、両者から呼び出す

**削減行数見積もり**: ~20行（中程度の価値）

---

### 1.3 PHI命令処理の統一 ⚠️ **バックエンド間の類似**

**類似箇所**:

| バックエンド | ファイル | 処理内容 |
|-------------|---------|---------|
| **Rust VM** | `src/backend/mir_interpreter/exec.rs:152-228` | predecessor判定 + reg代入 |
| **LLVM** | `src/llvm_py/builders/phi_handler.py:80-247` | PHI生成 + incoming配線 |
| **Hakorune VM** | `selfhost/hakorune-vm/phi_handler.hako:10-123` | predecessor判定 + JSON解析 |

**現状**: 各バックエンドごとに独立実装（意図的分離）

**共通化の難易度**: **Hard**
- 各バックエンドの実装言語・データ構造が異なる
- 責務が明確に分離されているため、共通化のメリットが低い

**推奨**: 現状維持（箱理論：境界を作る）

---

## 2. 箱化候補

### 2.1 PhiInputValidatorBox ⚠️ **検討候補**

**責務**:
- PHI入力の完全性検証（全predecessorカバー）
- 重複predecessor検出
- unreachable predecessor除外

**現状**: `phi_core::common::debug_verify_phi_inputs()` + `verification/phi_inputs.rs`

**箱化後の設計**:
```rust
pub struct PhiInputValidatorBox;

impl PhiInputValidatorBox {
    /// ビルド時検証（debug_assertions下）
    pub fn debug_validate(
        function: &MirFunction,
        merge_bb: BasicBlockId,
        inputs: &[(BasicBlockId, ValueId)],
    ) {
        #[cfg(debug_assertions)]
        Self::validate_internal(function, merge_bb, inputs).unwrap();
    }

    /// 事後検証（Verifier pass）
    pub fn validate(
        function: &MirFunction,
        merge_bb: BasicBlockId,
        inputs: &[(BasicBlockId, ValueId)],
    ) -> Result<(), VerificationError> {
        Self::validate_internal(function, merge_bb, inputs)
    }

    fn validate_internal(...) -> Result<(), ...> {
        // 共通検証ロジック
    }
}
```

**削減行数見積もり**: ~30行

**実装難易度**: **Easy**

**優先度**: **P1**（Phase完了後）

---

### 2.2 PhiTraceBox ✅ **既存機能で十分**

**責務**:
- PHI命令のトレース出力
- predecessor/value情報の整形

**現状**:
- Rust VM: `NYASH_PHI_TRACE=1` 環境変数
- LLVM: verbose フラグ
- Hakorune VM: デバッグ出力なし（今後追加可能）

**提案**: 箱化不要（環境変数で統一済み）

---

### 2.3 LoopCarrierDetectorBox ✅ **既に実装済み**

**現状**: `src/mir/loop_builder/carrier_analyzer.rs` で既に箱化済み ✅

**特徴**:
- 単一責任（loop-carried変数検出のみ）
- Pure Function設計
- テスト完備（3テストケース）

**評価**: 箱理論の模範実装 ⭐

---

## 3. 命名統一

### 3.1 PHI関連の命名パターン ✅ **概ね統一済み**

| 概念 | 統一名 | 使用箇所 |
|-----|-------|---------|
| PHI命令 | `Phi { dst, inputs }` | MIR, VM, LLVM, Hakorune |
| Predecessor | `pred` / `predecessor` | 統一 ✅ |
| 到達可能性 | `unreachable_pred` | 統一 ✅ |
| Loop-carried変数 | `loop_carried_vars` | 統一 ✅ |
| 不完全PHI | `IncompletePhi` | Rust/LLVM で統一 ✅ |

**問題点**: なし（既に統一済み）

---

### 3.2 PHI生成関数の命名 ⚠️ **軽微な不統一**

| 機能 | Rust | LLVM | Hakorune |
|-----|------|------|----------|
| PHI生成 | `merge_var_value()` | `process_phi_instructions()` | `handle_phi_instructions()` |
| PHI確定 | `seal_block()` | `complete_incomplete_phis()` | N/A |

**影響度**: 低（各バックエンドで独立している）

**推奨**: 現状維持（バックエンド間の統一は不要）

---

## 4. 優先度付けリファクタリング

### P0（即座実施推奨）

**なし** ✅

現状のコードは既に高品質で、緊急性のあるリファクタリングは不要。

---

### P1（Phase完了後）

#### 1. PhiInputValidatorBox の箱化

**目的**: PHI入力検証の統一化

**対象ファイル**:
- `src/mir/phi_core/common.rs` - debug_verify_phi_inputs()
- `src/mir/verification/phi_inputs.rs` - check_phi_inputs_cover_predecessors()

**作業内容**:
1. `src/mir/phi_core/validator.rs` を新規作成
2. `PhiInputValidatorBox` 構造体を定義
3. `debug_validate()` / `validate()` 関数を実装
4. 既存コードを新関数に置き換え

**見積もり**:
- 工数: 2-3時間
- 削減行数: ~30行
- テスト追加: 5テストケース

**価値**: Medium（重複削減 + テスタビリティ向上）

---

### P2（アイデアとして保持）

#### 1. PHI検証の段階的強化

**現状**:
- ビルド時: `debug_assertions` 下でのみ検証
- Verifier: 全MIR関数を事後検証

**提案**: `NYASH_MIR_STRICT_PHI=1` で実行時検証を追加

**目的**: 開発中のPHIバグを即座に発見

**実装箇所**: `src/mir/builder/phi_merge_helper.rs:107`

**見積もり**: 1時間（環境変数追加のみ）

---

#### 2. Hakorune VM PHIトレースの追加

**現状**: Hakorune VMにはPHIトレース機能がない

**提案**: `selfhost/hakorune-vm/phi_handler.hako` に環境変数トレース追加

```hakorune
if EnvGateBox.bool("HAKO_PHI_TRACE") {
  ConsoleBox.log("[phi-trace] dst=" + dst_id + " pred=" + predecessor)
}
```

**見積もり**: 1時間

**価値**: Low（開発ツールとしての利便性向上）

---

## 5. 結論と推奨アクション

### 5.1 全体評価 ⭐⭐⭐⭐⭐

**PHI関連コードは既に高品質** ✅

- 箱化: LoopCarrierAnalyzerBox, PhiMergeHelper 等で実装済み
- 共通化: phi_core/ モジュールで統一済み
- テスト: 26+ PHI関連テストが存在
- 文書化: phi_invariants.md, INSTRUCTION_SET.md で仕様明記

**評価**: 箱理論の模範実装 ⭐

---

### 5.2 推奨アクション

#### 即座実施（P0）

**なし** - 現状のコードで十分

---

#### Phase完了後（P1）

1. **PhiInputValidatorBox の箱化** (2-3時間)
   - 重複削減: ~30行
   - テスタビリティ向上
   - 価値: Medium

---

#### アイデアとして保持（P2）

1. **PHI検証の段階的強化** (1時間)
   - `NYASH_MIR_STRICT_PHI=1` で実行時検証
   - 価値: Low（開発ツール）

2. **Hakorune VM PHIトレース** (1時間)
   - デバッグ利便性向上
   - 価値: Low

---

### 5.3 リファクタリングしない理由

**既に十分に綺麗** ✅

- PHI生成: if_phi.rs, loop_phi.rs で分離済み
- PHI検証: verification/phi_inputs.rs で独立済み
- PHI実行: 各バックエンドで箱化済み（PhiHandler）

**箱理論の原則**:
- ✅ 箱にする: 既に実装済み
- ✅ 境界を作る: phi_core/ で責務分離済み
- ✅ 戻せる: 各機能が独立
- ✅ 見える化: 文書化・テスト完備

**結論**: **80/20ルール適用 - 現状で十分（80%完成）**

---

## 6. 参考：PHI関連ファイル一覧

### MIR Builder (Rust)

**PHI生成**:
- `src/mir/builder/phi.rs` - if/else PHI生成（メイン）
- `src/mir/loop_builder/phi.rs` - loop PHI生成（LoopBuilder専用）
- `src/mir/builder/phi_merge_helper.rs` - predecessor判定・PHI統一処理
- `src/mir/phi_core/if_phi.rs` - if PHI共通ロジック
- `src/mir/phi_core/loop_phi.rs` - loop PHI共通ロジック
- `src/mir/phi_core/common.rs` - 共通検証・型定義

**PHI検証**:
- `src/mir/verification/phi_inputs.rs` - PHI入力完全性検証
- `src/mir/verification/cfg.rs` - CFG検証（predecessorチェック）
- `src/mir/verification/dom.rs` - 支配関係検証

**PHI解析**:
- `src/mir/loop_builder/carrier_analyzer.rs` - ループ変数検出 ⭐箱化済み

---

### Rust VM

**PHI実行**:
- `src/backend/mir_interpreter/exec.rs:152-228` - `apply_phi_nodes()`
- 環境変数: `NYASH_PHI_TRACE=1` でトレース

---

### LLVM Backend (Python)

**PHI生成**:
- `src/llvm_py/builders/phi_handler.py` - PhiHandler クラス ⭐箱化済み
- `src/llvm_py/phi_wiring/registry.py` - PhiRegistry（単一インスタンス管理）
- `src/llvm_py/phi_wiring/wiring.py` - incoming配線処理
- `src/llvm_py/dispatch/phi_dispatch.py` - PHI命令ディスパッチ

---

### Hakorune VM

**PHI実行**:
- `selfhost/hakorune-vm/phi_handler.hako` - PhiHandlerBox ⭐箱化済み
- `selfhost/vm/boxes/phi_decode_box.hako` - PHI JSON解析
- `selfhost/vm/boxes/phi_apply_box.hako` - PHI適用処理

---

### 文書

- `docs/reference/mir/phi_invariants.md` - PHI不変条件仕様
- `docs/reference/mir/INSTRUCTION_SET.md` - PHI命令仕様
- `docs/reference/mir/verification.md` - PHI検証仕様

---

## 7. 今後の監視ポイント

### 7.1 重複が発生しやすい箇所

1. **新しいバックエンド追加時**
   - WASM/C等で新しいPHI実行機構を追加する際、既存パターンを参照すること
   - 参考: `PhiHandlerBox` (Rust VM / LLVM / Hakorune)

2. **PHI最適化パス追加時**
   - PHI削除・簡約化等の最適化を追加する際、検証ロジックを再利用すること
   - 参考: `PhiInputValidatorBox` (提案中)

---

### 7.2 命名規約の維持

**新しいPHI関連コードを追加する際のチェックリスト**:

- [ ] predecessor判定は `PhiMergeHelper::compute_if_merge_preds()` を使用
- [ ] PHI検証は `phi_core::common::debug_verify_phi_inputs()` を使用
- [ ] Loop-carried変数検出は `LoopCarrierAnalyzerBox::analyze()` を使用
- [ ] 環境変数トレースは `NYASH_PHI_TRACE=1` で統一

---

## 8. リファクタリング実施時の注意点

### 8.1 破壊的変更の回避

**PHI生成ロジックは極めて繊細**

- 変更前に必ず全PHIテストを実行すること
- 特に以下のテストは必須:
  - `tools/smokes/v2/run_phi.sh` - PHI専用スモークテスト
  - `tools/smokes/curated_phi_invariants.sh` - PHI不変条件テスト
  - `apps/benchmarks/micro/10_phi_stress_if_bench_v2.hako` - PHIストレステスト

---

### 8.2 段階的移行の原則

**一度に全てを変更しない**

1. **Phase 1**: 新しい箱（PhiInputValidatorBox）を作成
2. **Phase 2**: 既存コードを並行動作させる（フラグで切替可能に）
3. **Phase 3**: テスト完了後、旧コードを削除

**箱理論：戻せる** - 常にロールバック可能な状態を維持

---

## 9. 結論

**PHI関連コードは既に高品質で、大規模リファクタリングは不要** ✅

- 箱化: 完了済み（LoopCarrierAnalyzerBox, PhiMergeHelper等）
- 共通化: phi_core/ モジュールで統一済み
- 文書化: 仕様・不変条件が明記済み

**推奨**: 現状維持 + 小規模改善（P1: PhiInputValidatorBox）のみ実施

**箱理論の模範実装** ⭐ - 他のコードベースの参考事例として活用可能
