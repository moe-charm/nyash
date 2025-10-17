# Phase 2: ループ変数破損バグ完全根治計画

**作成日**: 2025-10-17
**優先度**: P0（最重要）
**所要時間**: 3-4日（調査完了済み、実装開始）
**背景**: 4回以上のPHIバグ再発歴 + 今回発見の3つのバグ → 構造的保証による完全根治が必要

---

## 📋 Executive Summary

**今回発見したバグ（Task先生4人並列調査）**:
1. **P0 - パラメータレジスタ上書きバグ**: MIR Builder が v%0-v%N を再利用
2. **P1 - メソッド降下の不安定性**: origin 推論失敗時に BoxCall/Extern で揺れる
3. **P2 - variable_map の ValueId 衝突**: メソッド結果が既存変数と衝突

**根治方針**: 3つのバグ修正 + LoopFormBox統合による**構造的保証**

**目標**: ループ構造を強制的に正規化し、PHIバグを**構造的に**防止する

**核心アイデア**:
```
Header = PHI群 + Branch のみ（構造強制）
条件式 = 別ブロックで構築（副作用隔離）
一時値 = pin スロット（__pin$...）のみ
```

**成果物**:
- P0修正: ParameterGuardBox（パラメータレジスタ保護）
- P1修正: OriginPropagationBox（origin推論強化）
- P2修正: ValueIdAllocatorBox（ValueId衝突回避）
- LoopFormBox（構造保証Box）
- LoopFormVerifierBox（構造強制Verifier）
- 回帰テスト30個
- 環境変数フラグ（戻せる実装）

---

## 🐛 今回発見したバグの詳細と修正方針

### P0: パラメータレジスタ上書きバグ

**症状**:
- `skip_ws(s, i, end)` でループ変数 j(v%4) がメソッド receiver copy で String に上書き
- MIR証拠: `%4 = copy %23` (v%4=Integer j → v%23=String s)
- エラー: "Type error: compare Lt on String("0") and Integer(1)"

**根本原因**:
- MIR Builder が関数パラメータレジスタ v%0-v%N をローカル変数で再利用

**修正内容**:
```rust
// src/mir/builder/guards/parameter_guard.rs（既存、Phase 2.4で実装済み）
pub struct ParameterGuardBox {
    param_count: usize,
    reserved_registers: HashSet<ValueId>,
}

// 修正: v%(N+1) からローカル変数開始
impl ParameterGuardBox {
    pub fn validate_no_overwrite(&self, vid: ValueId) -> Result<(), String> {
        if self.reserved_registers.contains(&vid) {
            return Err(format!("Attempted to overwrite parameter register {:?}", vid));
        }
        Ok(())
    }
}
```

**テスト**:
- `json_query_vm` - パラメータ参照が正しいか
- `param_register_protection_vm` - パラメータレジスタ保護を確認

---

### P1: メソッド降下の不安定性

**症状**:
- `s.substring(j, j+1)` が origin 推論失敗時に BoxCall/Extern で揺れる

**根本原因**:
- receiver の origin 情報が不明 → `infer_receiver()` が "UnknownBox" を返す
- **重要**: Extern 正規化は既に完全実装済み（Task 4確認）、問題は origin 推論の失敗

**修正内容**:
```rust
// src/mir/builder/origin/propagation.rs（新規）
pub struct OriginPropagationBox;

impl OriginPropagationBox {
    /// メソッド呼び出し結果に origin を伝播
    pub fn propagate_method_result(
        builder: &mut MirBuilder,
        receiver_origin: Option<&str>,
        method: &str,
        result: ValueId,
    ) -> Result<(), String> {
        match (receiver_origin, method) {
            (Some("StringBox"), "substring" | "indexOf" | "lastIndexOf") => {
                // String → String
                builder.origin_tracker.set_origin(result, "StringBox");
            }
            (Some("ArrayBox"), "slice") => {
                // Array → Array
                builder.origin_tracker.set_origin(result, "ArrayBox");
            }
            (Some("MapBox"), "keys" | "values") => {
                // Map → Array
                builder.origin_tracker.set_origin(result, "ArrayBox");
            }
            _ => {}
        }
        Ok(())
    }
}
```

**テスト**:
- `method_origin_propagation_vm` - substring の origin が正しく伝播
- `method_lowering_stability_vm` - Extern正規化が安定

---

### P2: variable_map の ValueId 衝突

**症状**:
- メソッド呼び出し結果 dst が既存ループ変数と同じ ValueId を割り当て

**根本原因**:
- `value_gen.next()` が既存の variable_map を考慮しない
- ensure() 修正でカバー済み: receiver/arg/cond ✅
- 未カバー: メソッド結果、BinOp結果、Assignment RHS ❌

**修正内容**:
```rust
// src/mir/builder/value_allocator_box.rs（新規）
pub struct ValueIdAllocatorBox {
    next_id: usize,
    param_guard: ParameterGuardBox,
    in_use: HashSet<ValueId>,
}

impl ValueIdAllocatorBox {
    pub fn allocate_safe(&mut self) -> ValueId {
        loop {
            let candidate = ValueId::new(self.next_id);
            self.next_id += 1;

            // 3重チェック
            if !self.param_guard.is_parameter(candidate)
                && !self.in_use.contains(&candidate) {
                return candidate;
            }
        }
    }

    pub fn sync_in_use(&mut self, variable_map: &HashMap<String, ValueId>) {
        self.in_use = variable_map.values().copied().collect();
    }
}
```

**テスト**:
- `valueid_collision_avoidance_vm` - ValueId衝突が発生しない
- `method_result_allocation_vm` - メソッド結果が安全に割り当て

---

## 🏗️ アーキテクチャ設計（統合版）

### 全体構成

```
┌─────────────────────────────────────────────────────┐
│               MIR Builder                           │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌────────────────────┐  ┌──────────────────────┐  │
│  │ ParameterGuardBox  │  │ ValueIdAllocatorBox  │  │
│  │ (P0修正)           │  │ (P2修正)             │  │
│  │ - v%0-v%N保護      │  │ - 衝突回避           │  │
│  └────────────────────┘  └──────────────────────┘  │
│                                                     │
│  ┌────────────────────┐  ┌──────────────────────┐  │
│  │OriginPropagation   │  │   LoopFormBox        │  │
│  │ Box (P1修正)       │  │ (構造保証)           │  │
│  │ - origin伝播       │  │ - Header正規化       │  │
│  └────────────────────┘  └──────────────────────┘  │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │       LoopFormVerifierBox                    │  │
│  │       (構造強制)                             │  │
│  │       - PHI位置検証                          │  │
│  │       - Header構造検証                       │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

### 1. LoopFormBox（構造保証）

**責務**: ループ構造を正規化し、PHIバグを構造的に防止

**設計原則**:
1. **スコープ境界強制**: Header で変数束縛を禁止
2. **副作用隔離**: 条件式は別ブロックで構築
3. **構造検証**: Verifier で構造違反を即座検出

#### 1.1 データ構造

```rust
// src/mir/loop_builder/loopform_box.rs

/// LoopForm構造の保証Box
pub struct LoopFormBox {
    /// Builder参照（emit用）
    builder: *mut MirBuilder, // 循環参照回避のため raw pointer

    /// Header block ID
    header_bb: BlockId,

    /// Condition block ID（副作用隔離）
    condition_bb: Option<BlockId>,

    /// PHI nodes（搬送変数のみ）
    phi_nodes: Vec<PhiNode>,

    /// 搬送変数（キャリア）
    carrier_vars: HashSet<String>,

    /// Pin slots（一時値）
    pin_slots: Vec<ValueId>,

    /// Preheader block ID
    preheader_bb: BlockId,

    /// Latch block ID
    latch_bb: Option<BlockId>,

    /// Exit block ID
    exit_bb: Option<BlockId>,
}

/// PHI node descriptor
#[derive(Debug, Clone)]
pub struct PhiNode {
    pub var_name: String,
    pub phi_value: ValueId,
    pub preheader_input: ValueId,
    pub latch_input: Option<ValueId>, // Latch未確定時はNone
}

/// LoopForm構造（出力）
pub struct LoopStructure {
    pub header_bb: BlockId,
    pub condition_bb: Option<BlockId>,
    pub body_bb: BlockId,
    pub latch_bb: BlockId,
    pub exit_bb: BlockId,
    pub phi_nodes: Vec<PhiNode>,
    pub carrier_vars: HashSet<String>,
}
```

#### 1.2 主要メソッド

```rust
impl LoopFormBox {
    /// LoopFormBox作成
    pub fn new(builder: &mut MirBuilder) -> Self {
        Self {
            builder: builder as *mut _,
            header_bb: BlockId::invalid(),
            condition_bb: None,
            phi_nodes: Vec::new(),
            carrier_vars: HashSet::new(),
            pin_slots: Vec::new(),
            preheader_bb: BlockId::invalid(),
            latch_bb: None,
            exit_bb: None,
        }
    }

    /// ループ構造構築（メインエントリーポイント）
    pub fn build_loop(
        &mut self,
        condition: &ast::Expr,
        preheader_vars: &HashMap<String, ValueId>,
        body: &[ast::Statement],
    ) -> Result<LoopStructure, String> {
        // Step 1: 搬送変数特定
        let carriers = self.identify_carriers(preheader_vars, body)?;

        // Step 2: Header block作成（PHIのみ）
        let header_bb = self.create_header(carriers)?;

        // Step 3: Condition block作成（副作用隔離）
        let cond_bb = self.create_condition_block(condition)?;

        // Step 4: Body/Latch/Exit作成
        let (body_bb, latch_bb, exit_bb) = self.create_body_latch_exit(body)?;

        // Step 5: PHI入力更新（Latch確定後）
        self.update_phi_inputs(latch_bb)?;

        // Step 6: 構造検証
        self.verify_structure()?;

        Ok(LoopStructure {
            header_bb,
            condition_bb: Some(cond_bb),
            body_bb,
            latch_bb,
            exit_bb,
            phi_nodes: self.phi_nodes.clone(),
            carrier_vars: self.carrier_vars.clone(),
        })
    }

    /// 搬送変数特定（preheader定義 ∩ body代入）
    fn identify_carriers(
        &mut self,
        preheader_vars: &HashMap<String, ValueId>,
        body: &[ast::Statement],
    ) -> Result<HashSet<String>, String> {
        // 既存のLoopCarrierAnalyzerBoxを使用
        let analyzer = LoopCarrierAnalyzerBox::new();
        let assigned = analyzer.collect_assigned_vars(body)?;

        let carriers: HashSet<String> = preheader_vars
            .keys()
            .filter(|k| assigned.contains(*k))
            .cloned()
            .collect();

        self.carrier_vars = carriers.clone();
        Ok(carriers)
    }

    /// Header block作成（PHI + Branch のみ）
    fn create_header(&mut self, carriers: HashSet<String>) -> Result<BlockId, String> {
        let builder = unsafe { &mut *self.builder };

        // Header block作成
        let header_bb = builder.create_block()?;
        self.header_bb = header_bb;
        builder.set_current_block(header_bb);

        // 搬送変数ごとにPHI作成
        for var_name in &carriers {
            let preheader_value = builder.variable_map.get(var_name)
                .ok_or_else(|| format!("Carrier variable '{}' not found in preheader", var_name))?;

            let phi_value = builder.value_gen.next();

            // PHI emit（Latch入力はNoneで仮作成）
            builder.emit_instruction(MirInstruction::Phi {
                dst: phi_value,
                inputs: vec![(*preheader_value, self.preheader_bb)],
            })?;

            // PHI node記録
            self.phi_nodes.push(PhiNode {
                var_name: var_name.clone(),
                phi_value,
                preheader_input: *preheader_value,
                latch_input: None,
            });

            // variable_map更新（Header内ではPHI値を使用）
            builder.variable_map.insert(var_name.clone(), phi_value);
        }

        Ok(header_bb)
    }

    /// Condition block作成（副作用隔離）
    fn create_condition_block(&mut self, condition: &ast::Expr) -> Result<BlockId, String> {
        let builder = unsafe { &mut *self.builder };

        // Condition block作成
        let cond_bb = builder.create_block()?;
        self.condition_bb = Some(cond_bb);
        builder.set_current_block(cond_bb);

        // 🔥 重要: variable_map スナップショット（副作用隔離）
        let saved_map = builder.variable_map.clone();

        // 条件式構築（pin スロットに格納）
        let cond_value = builder.build_expression(condition)?;

        // Pin slot記録
        self.pin_slots.push(cond_value);

        // 🔥 重要: variable_map リストア（副作用破棄）
        builder.variable_map = saved_map;

        // Branch emit（Header → Condition → Body/Exit）
        builder.set_current_block(self.header_bb);
        builder.emit_instruction(MirInstruction::Branch {
            condition: cond_value,
            true_target: BlockId::invalid(), // Body（後で設定）
            false_target: BlockId::invalid(), // Exit（後で設定）
        })?;

        Ok(cond_bb)
    }

    /// PHI入力更新（Latch確定後）
    fn update_phi_inputs(&mut self, latch_bb: BlockId) -> Result<(), String> {
        let builder = unsafe { &mut *self.builder };

        for phi_node in &mut self.phi_nodes {
            // Latch blockでの変数値を取得
            let latch_value = builder.variable_map.get(&phi_node.var_name)
                .ok_or_else(|| format!("Carrier '{}' not found in latch", phi_node.var_name))?;

            phi_node.latch_input = Some(*latch_value);

            // PHI命令更新
            builder.set_current_block(self.header_bb);
            // 既存PHIを検索して更新（実装省略）
        }

        Ok(())
    }

    /// 構造検証（LoopFormVerifierBox呼び出し）
    fn verify_structure(&self) -> Result<(), String> {
        let builder = unsafe { &*self.builder };
        let header_block = builder.get_block(self.header_bb)?;

        LoopFormVerifierBox::verify_loop_header(header_block)?;
        Ok(())
    }
}
```

---

### 2. LoopFormVerifierBox（構造強制）

**責務**: LoopForm構造ルールを強制

```rust
// src/mir/verification/loopform.rs

pub struct LoopFormVerifierBox;

impl LoopFormVerifierBox {
    /// Loop Header構造検証
    pub fn verify_loop_header(block: &MirBlock) -> Result<(), String> {
        // Rule 1: PHI はブロック先頭のみ
        Self::verify_phi_at_start(block)?;

        // Rule 2: Header = PHI + Branch のみ
        Self::verify_header_minimal(block)?;

        // Rule 3: PHI以外の命令でvariable_map更新禁止
        Self::verify_no_rebinding(block)?;

        Ok(())
    }

    /// Rule 1: PHI はブロック先頭のみ
    fn verify_phi_at_start(block: &MirBlock) -> Result<(), String> {
        let mut phi_section = true;

        for (i, inst) in block.instructions.iter().enumerate() {
            match inst {
                MirInstruction::Phi{..} => {
                    if !phi_section {
                        return Err(format!(
                            "PHI instruction at position {} not at block start (block: bb{})",
                            i, block.id
                        ));
                    }
                }
                _ => {
                    phi_section = false;
                }
            }
        }

        Ok(())
    }

    /// Rule 2: Header = PHI + Branch のみ
    fn verify_header_minimal(block: &MirBlock) -> Result<(), String> {
        let non_phi_instructions: Vec<_> = block.instructions.iter()
            .filter(|i| !matches!(i, MirInstruction::Phi{..}))
            .collect();

        if non_phi_instructions.len() > 1 {
            return Err(format!(
                "Loop header bb{} has {} non-PHI instructions (expected 1 Branch only)",
                block.id, non_phi_instructions.len()
            ));
        }

        // 最後の命令がBranchであること確認
        if let Some(last) = non_phi_instructions.last() {
            if !matches!(last, MirInstruction::Branch{..}) {
                return Err(format!(
                    "Loop header bb{} must end with Branch (found: {:?})",
                    block.id, last
                ));
            }
        }

        Ok(())
    }

    /// Rule 3: PHI以外の命令でvariable_map更新禁止
    fn verify_no_rebinding(block: &MirBlock) -> Result<(), String> {
        // この検証は実行時に行う（静的には困難）
        // 環境変数 HAKO_LOOPFORM_TRACE=1 でトレース可能
        Ok(())
    }
}
```

---

## 📅 実装ステップ（週次計画、統合版）

### Day 1（今日、4時間）: P0修正（緊急パッチ）+ P2基盤

**午前（2時間）: P0修正**
1. ParameterGuardBox強化
   - 場所: `src/mir/builder/guards/parameter_guard.rs` （既存）
   - 修正: VarTracker に統合、v%(N+1) からローカル変数開始
   - ファイル: `src/mir/builder/var_tracker.rs` 新規メソッド追加

2. ビルド＆テスト
   - `json_query_vm` 復活確認 ✅

**午後（2時間）: P2基盤実装**
3. ValueIdAllocatorBox骨格作成
   - 場所: `src/mir/builder/value_allocator_box.rs` （新規）
   - データ構造定義
   - `new()`, `allocate_safe()`, `sync_in_use()` 実装

4. 初回テスト
   - 最小ケースでValueId衝突回避を確認

**成果物**:
- ✅ json_query_vm PASS（P0修正）
- ✅ ValueIdAllocatorBox基盤完成（P2）

---

### Day 2（明日、8時間）: P1修正 + P2完成 + LoopFormBox骨格

**午前（4時間）: P1修正**
1. OriginPropagationBox実装
   - 場所: `src/mir/builder/origin/propagation.rs` （新規）
   - `propagate_method_result()` 実装
   - substring/slice/keys/values の origin 伝播

2. MirBuilder統合
   - `build_method_call()` 内で `OriginPropagationBox` 呼び出し
   - テスト: `method_origin_propagation_vm` 作成・実行

**午後（4時間）: P2完成 + LoopFormBox骨格**
3. ValueIdAllocatorBox完成
   - MirBuilder統合（`safe_next_value()` メソッド追加）
   - フラグ制御（`HAKO_USE_VALUE_ALLOCATOR_BOX=1`）
   - テスト: `valueid_collision_avoidance_vm` 作成・実行

4. LoopFormBox骨格作成
   - 場所: `src/mir/loop_builder/loopform_box.rs` （新規）
   - データ構造定義（LoopFormBox, PhiNode, LoopStructure）
   - `new()`, `identify_carriers()` 実装

**成果物**:
- ✅ P1修正完了（origin伝播）
- ✅ P2修正完了（ValueId衝突回避）
- ✅ LoopFormBox骨格

---

### Day 3（2日後、8時間）: LoopFormBox実装

**午前（4時間）**:
1. Header block作成
   - `create_header()` 実装
   - PHI作成ロジック
   - variable_map更新

2. Condition block作成
   - `create_condition_block()` 実装
   - variable_map スナップショット/リストア
   - pin スロット管理

**午後（4時間）**:
3. Body/Latch/Exit作成
   - `create_body_latch_exit()` 実装
   - PHI入力更新（`update_phi_inputs()`）

4. 初回テスト
   - 最小ループ（単一搬送変数）で動作確認
   - MIRダンプで構造確認

**成果物**:
- ✅ LoopFormBox基本実装完了

---

### Day 4（3日後、8時間）: LoopFormVerifierBox + 統合

**午前（4時間）**:
1. LoopFormVerifierBox実装
   - 場所: `src/mir/verification/loopform.rs` （新規）
   - Rule 1: PHI位置検証
   - Rule 2: Header構造検証
   - Rule 3: 変数束縛禁止検証

2. Verifierテスト
   - 構造違反を検出できるか確認
   - エラーメッセージの分かりやすさ確認

**午後（4時間）**:
3. MirBuilder統合
   - 場所: `src/mir/builder.rs` 修正
   - 環境変数フラグ（`HAKO_USE_LOOPFORM_BOX=1`）
   - Fallback実装（フラグ無効時は既存動作）

4. 統合テスト
   - quick profile で動作確認
   - P0/P1/P2 すべての修正が有効か確認

**成果物**:
- ✅ LoopFormVerifierBox完成
- ✅ MirBuilder統合完了
- ✅ 環境変数フラグで切り替え可能

---

### Day 5（4日後、8時間）: テスト＆ドキュメント

**午前（4時間）**:
1. 回帰テスト作成（30個）
   - **P0テスト（10個）**: パラメータレジスタ保護
     - `param_register_protection_vm` - 基本保護
     - `json_query_vm` - 実戦ケース
     - `param_overwrite_detection_vm` - 上書き検出
   - **P1テスト（5個）**: origin伝播
     - `method_origin_propagation_vm` - substring/slice/keys/values
     - `method_lowering_stability_vm` - Extern正規化安定性
   - **P2テスト（5個）**: ValueId衝突回避
     - `valueid_collision_avoidance_vm` - 基本衝突回避
     - `method_result_allocation_vm` - メソッド結果安全割り当て
   - **LoopFormテスト（10個）**: 構造保証
     - 単一搬送変数（i, acc）
     - 複数搬送変数（i, j, sum）
     - 条件式複雑化（&&, ||, 短絡評価）
     - メソッド呼び出し（s.substring）
     - continue/break、入れ子ループ

2. エラーケーステスト（10個）
   - 構造違反検出（PHI位置違反）
   - variable_map汚染検出
   - 一時変数の過大PHI化

**午後（4時間）**:
3. スモークテスト統合
   - quick profile でLoopFormBox有効化
   - 全スモーク再実行（170テスト）
   - 差分確認

4. ドキュメント作成
   - 使用ガイド作成
   - トラブルシューティングガイド作成
   - CURRENT_TASK.md更新

**成果物**:
- ✅ 30個のユニットテスト PASS
- ✅ 170個のスモークテスト PASS
- ✅ ドキュメント完成

---

## 🧪 テスト戦略（統合版）

### 1. ユニットテスト（40個）

#### カテゴリA: P0 - パラメータレジスタ保護（10個）

```rust
// tests/parameter_guard_unit.rs

#[test]
fn test_param_register_protection_basic() {
    // function(p1, p2) で v%0=me, v%1=p1, v%2=p2
    // local i = 0 は v%3 から開始
    // v%0-v%2 への代入は禁止
}

#[test]
fn test_param_overwrite_detection() {
    // ParameterGuardBox が v%0-v%N への代入を検出
    // Fail-Fast エラー発生
}

#[test]
fn test_json_query_skip_ws() {
    // 実戦ケース: skip_ws(s, i, end)
    // ループ内メソッド呼び出しでパラメータ保護
}
```

#### カテゴリB: P1 - origin伝播（5個）

```rust
// tests/origin_propagation_unit.rs

#[test]
fn test_substring_origin_propagation() {
    // local s = "hello"
    // local sub = s.substring(0, 3)
    // origin(sub) == "StringBox"
}

#[test]
fn test_map_keys_values_origin() {
    // local map = new MapBox()
    // local keys = map.keys()
    // origin(keys) == "ArrayBox"
}
```

#### カテゴリC: P2 - ValueId衝突回避（5個）

```rust
// tests/valueid_allocator_unit.rs

#[test]
fn test_valueid_collision_avoidance() {
    // local i = 0
    // loop(i < 10) {
    //   local temp = arr.get(i)  // temp は v%i と異なる
    //   i = i + 1
    // }
}

#[test]
fn test_method_result_safe_allocation() {
    // メソッド呼び出し結果が既存変数と衝突しない
}
```

#### カテゴリD: LoopForm - 基本ループ（10個）

```rust
// tests/loopform_unit.rs

#[test]
fn test_single_carrier_loop() {
    // local i = 0
    // loop(i < 10) { i = i + 1 }
    // 搬送変数: i のみ
    // Header: PHI(i) + Branch のみ
}

#[test]
fn test_double_carrier_loop() {
    // local i = 0, local acc = 0
    // loop(i < 10) { acc = acc + i; i = i + 1 }
    // 搬送変数: i, acc
}

#[test]
fn test_loop_with_non_carrier() {
    // local i = 0
    // loop(i < 10) { local temp = i * 2; i = i + 1 }
    // 搬送変数: i のみ（temp は一時変数）
}
```

#### カテゴリB: 条件式複雑化（5個）

```rust
#[test]
fn test_loop_condition_short_circuit() {
    // loop(i < end && is_valid(arr.get(i))) { ... }
    // Condition block で短絡評価処理
    // variable_map汚染なし確認
}

#[test]
fn test_loop_condition_method_call() {
    // loop(i < s.size()) { ... }
    // Condition block で s.size() 呼び出し
    // variable_map汚染なし確認
}
```

#### カテゴリC: 制御フロー（5個）

```rust
#[test]
fn test_loop_with_break() {
    // loop(i < 10) { if condition { break }; i = i + 1 }
    // Exit block への分岐確認
}

#[test]
fn test_loop_with_continue() {
    // loop(i < 10) { if condition { continue }; i = i + 1 }
    // Latch block への分岐確認
}

#[test]
fn test_nested_loop() {
    // loop(i < 10) { loop(j < 5) { ... } }
    // 各ループのHeader独立性確認
}
```

#### カテゴリD: エラー検出（5個）

```rust
#[test]
fn test_verify_phi_at_start_violation() {
    // 手動でPHI位置違反を作成
    // LoopFormVerifierBox がエラー検出
}

#[test]
fn test_verify_header_minimal_violation() {
    // Header に PHI + Branch 以外の命令を挿入
    // LoopFormVerifierBox がエラー検出
}
```

---

### 2. 統合テスト（スモークテスト）

```bash
# quick profile で LoopFormBox 有効化
HAKO_USE_LOOPFORM_BOX=1 tools/smokes/v2/run.sh --profile quick

# 期待: 170 PASS / 0 FAIL
```

---

### 3. 回帰テスト（既存バグの再発防止）

| テストケース | 検証内容 | 関連バグ |
|------------|---------|---------|
| `json_query_vm` | パラメータレジスタ上書きなし | 2025-10-17 |
| `loop_carrier_overphiification_vm` | 一時変数の過大PHI化なし | 2025-10-18 |
| `loop_condition_var_map_pollution_vm` | 条件式による変数マップ汚染なし | 2025-10-16 |
| `loop_localssa_collision_vm` | LocalSSA衝突なし | 2025-10-16 |

---

## 🚨 リスク分析

### リスク1: 既存コードとの互換性 ⚠️⚠️

**リスク**: LoopFormBox 有効化で既存テストが失敗

**軽減策**:
- 環境変数フラグで段階的導入（`HAKO_USE_LOOPFORM_BOX=1`）
- Fallback実装（フラグ無効時は既存動作）
- quick → plugins → full の段階的スモークテスト

**Contingency Plan**:
- フラグ無効でロールバック
- 問題箇所を特定して個別修正

---

### リスク2: パフォーマンス劣化 ⚠️

**リスク**: Condition block 分離でブロック数増加 → 最適化パフォーマンス低下

**軽減策**:
- ベンチマーク測定（`bench_unified.sh`）
- 許容範囲: 5%以内の劣化

**Contingency Plan**:
- Optimizer Pass で Condition block をインライン化

---

### リスク3: 実装バグ ⚠️⚠️⚠️

**リスク**: LoopFormBox 自体にバグ → 新しい問題発生

**軽減策**:
- 20個のユニットテスト
- LoopFormVerifierBox で構造検証
- 段階的導入（quick → plugins → full）

**Contingency Plan**:
- フラグ無効でロールバック
- Task先生調査 → バグ修正

---

## 🔄 Fallback戦略

### フラグ制御

```rust
// src/mir/builder.rs

impl MirBuilder {
    fn build_loop(&mut self, ...) -> Result<(), String> {
        // 環境変数チェック
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_USE_LOOPFORM_BOX",
            "NYASH_USE_LOOPFORM_BOX",
        ]) {
            // LoopFormBox経由（新実装）
            if let Some(ref mut loopform) = self.loopform {
                return loopform.build_loop(...);
            }
        }

        // Fallback: 既存動作
        self.build_loop_legacy(...)
    }
}
```

### ロールバック手順

1. **即座の無効化**:
   ```bash
   export HAKO_USE_LOOPFORM_BOX=0
   cargo build --release
   ```

2. **問題箇所の特定**:
   ```bash
   HAKO_TRACE_LOOPFORM=1 ./target/release/hakorune --backend vm <test_case>
   ```

3. **バグ修正 or フラグ維持**:
   - バグが小さい → 修正してリトライ
   - バグが大きい → フラグ無効のまま Phase 3 へ延期

---

## ✅ 成功基準（統合版）

### Phase 2 完了の定義

| 項目 | 基準 | 確認方法 |
|-----|------|---------|
| **P0修正** | パラメータレジスタ保護完了 | `json_query_vm` PASS |
| **P1修正** | origin伝播完了 | `method_origin_propagation_vm` PASS |
| **P2修正** | ValueId衝突回避完了 | `valueid_collision_avoidance_vm` PASS |
| **LoopFormBox実装** | 完了 | コードレビュー |
| **LoopFormVerifierBox** | 完了 | 構造違反検出テスト PASS |
| **ユニットテスト** | 40個すべて PASS | `cargo test parameter_guard loopform origin valueid` |
| **統合テスト** | quick profile 170 PASS | `HAKO_USE_LOOPFORM_BOX=1 tools/smokes/v2/run.sh --profile quick` |
| **回帰テスト** | 4つの既存バグすべて再発なし | 個別スモーク実行 |
| **パフォーマンス** | 5%以内の劣化 | `bench_unified.sh` |
| **ドキュメント** | 設計書・使用ガイド・トラブルシューティング完成 | レビュー |

### 各修正の個別成功基準

#### P0: パラメータレジスタ保護

| テスト | 期待結果 |
|-------|---------|
| `json_query_vm` | PASS（パラメータ参照が正しい） |
| `param_register_protection_vm` | PASS（v%0-v%N 保護） |
| `param_overwrite_detection_vm` | FAIL（Fail-Fastエラー検出） |

#### P1: origin伝播

| テスト | 期待結果 |
|-------|---------|
| `method_origin_propagation_vm` | PASS（substring/slice/keys/valuesの origin 伝播） |
| `method_lowering_stability_vm` | PASS（Extern正規化が安定） |

#### P2: ValueId衝突回避

| テスト | 期待結果 |
|-------|---------|
| `valueid_collision_avoidance_vm` | PASS（ValueId衝突なし） |
| `method_result_allocation_vm` | PASS（メソッド結果が安全割り当て） |

#### LoopForm: 構造保証

| テスト | 期待結果 |
|-------|---------|
| LoopFormテスト10個 | すべて PASS |
| LoopFormVerifierテスト10個 | 構造違反を正しく検出 |

---

## 📚 関連ドキュメント

### 既存ドキュメント
- [ループ変数破損バグ調査](../../issues/loop-variable-corruption-investigation.md) - 背景
- [LoopCarrierAnalyzerBox](../../../mir/loop_builder/carrier_analyzer.rs) - 既存実装
- [Phase 31 INDEX](./phase-31-box-Normalization/INDEX_JA.md) - PHI配置

### 新規作成予定
- LoopFormBox設計書（このドキュメント）
- LoopFormBox使用ガイド（Day 5作成）
- LoopFormBoxトラブルシューティング（Day 5作成）

---

## 🎯 次のステップ（統合版）

### 今日（Day 1、4時間）
**午前（2時間）**:
1. ✅ ドキュメント作成完了
2. ✅ Commit完了

**午後（2-4時間）**:
3. P0実装開始（パラメータレジスタ保護）
4. P2基盤実装（ValueIdAllocatorBox骨格）

### 明日以降（Day 2-5、32時間）
**Day 2（8時間）**: P1修正 + P2完成 + LoopFormBox骨格
**Day 3（8時間）**: LoopFormBox実装
**Day 4（8時間）**: LoopFormVerifierBox + 統合
**Day 5（8時間）**: テスト＆ドキュメント

### 5日後（完了時）
- ✅ P0/P1/P2 すべての修正完了
- ✅ LoopFormBox + Verifier完成
- ✅ 40個のユニットテスト PASS
- ✅ 170個のスモークテスト PASS
- ✅ **PHIバグ完全根治達成**

---

## 📊 実装サマリー

| 項目 | 内容 | 所要時間 |
|-----|------|---------|
| **P0修正** | ParameterGuardBox強化 | 2時間 |
| **P1修正** | OriginPropagationBox実装 | 4時間 |
| **P2修正** | ValueIdAllocatorBox実装 | 6時間 |
| **LoopFormBox** | 構造保証Box実装 | 12時間 |
| **Verifier** | LoopFormVerifierBox実装 | 4時間 |
| **テスト** | ユニット40個+スモーク統合 | 8時間 |
| **ドキュメント** | 使用ガイド+トラブルシューティング | 4時間 |
| **合計** | - | **40時間（5日）** |

---

## 🏆 期待される効果

### 短期効果（Day 1完了時）
- ✅ `json_query_vm` 復活
- ✅ パラメータレジスタ破壊バグ修正

### 中期効果（Day 3完了時）
- ✅ P0/P1/P2 すべてのバグ修正完了
- ✅ ValueId衝突なし
- ✅ origin推論安定

### 長期効果（Day 5完了時）
- ✅ **PHIバグの完全根治**（構造的保証）
- ✅ 将来のPHIバグを**理論的に防止**
- ✅ Hakoruneスクリプト化準備（Phase 4で移植可能）

---

**作成日**: 2025-10-17
**最終更新**: 2025-10-17（統合計画作成完了）
**作成者**: Claude Code + ChatGPT協調
**次回更新**: Phase 2 Day 1完了時
