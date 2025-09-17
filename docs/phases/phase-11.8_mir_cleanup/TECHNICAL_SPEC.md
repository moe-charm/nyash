# Phase 11.8 技術仕様書：Core‑13 MIR命令セット（既定ON）

## 0. 変換スイッチとルーティング（Core‑13 既定ON）

推奨既定（nyash.toml の [env]）

- NYASH_MIR_CORE13=1 … Core‑13 一括ON（Array/Ref→BoxCall 等を内包）
- NYASH_OPT_DIAG_FORBID_LEGACY=1 … 旧命令が最終MIRに残ったらエラー

Builder/MIR 生成
- Builder は ArrayGet/ArraySet/RefGet/RefSet/PluginInvoke を emit せず、最初から BoxCall/Call/ExternCall に正規化する。
- Optimizer は保険として既存の正規化パスを維持（二重化で確実性を上げる）。

## 1. ArrayGet/ArraySet → BoxCall 統合仕様

### 1.1 変換規則

```rust
// MIR Optimizer での変換
match instruction {
    MirInstruction::ArrayGet { dst, array, index } => {
        MirInstruction::BoxCall {
            dst: Some(*dst),
            box_val: *array,
            method: "get".to_string(),
            method_id: Some(UNIVERSAL_GET_ID), // 予約ID: 4
            args: vec![*index],
            effects: EffectMask::READS_MEMORY,
        }
    }
    
    MirInstruction::ArraySet { array, index, value } => {
        MirInstruction::BoxCall {
            dst: None,
            box_val: *array,
            method: "set".to_string(),
            method_id: Some(UNIVERSAL_SET_ID), // 予約ID: 5
            args: vec![*index, *value],
            effects: EffectMask::WRITES_MEMORY | EffectMask::MAY_GC,
        }
    }
}
```

### 1.2 VM最適化

```rust
// VM execute_boxcall での特殊化
fn execute_boxcall(...) {
    // 高速パス：ArrayBoxの既知メソッド
    if let Some(method_id) = method_id {
        match (type_id, method_id) {
            (ARRAY_BOX_TYPE, UNIVERSAL_GET_ID) => {
                // 直接配列アクセス（BoxCall経由でも高速）
                return fast_array_get(receiver, args[0]);
            }
            (ARRAY_BOX_TYPE, UNIVERSAL_SET_ID) => {
                return fast_array_set(receiver, args[0], args[1]);
            }
            _ => {}
        }
    }
    
    // 通常パス
    plugin_invoke(...)
}
```

### 1.3 JIT最適化

```rust
// JIT Lowering での認識
fn lower_boxcall(builder: &mut IRBuilder, ...) {
    if is_known_array_type(receiver_type) {
        match method_id {
            Some(UNIVERSAL_GET_ID) => {
                // GEP + Load にインライン展開
                emit_array_bounds_check(...);
                emit_array_get_inline(...);
                return;
            }
            Some(UNIVERSAL_SET_ID) => {
                // Write barrier + GEP + Store
                emit_write_barrier(...);
                emit_array_set_inline(...);
                return;
            }
            _ => {}
        }
    }
    
    // 通常のBoxCall
    emit_plugin_invoke(...);
}
```

## 2. Load/Store 削減仕様（SSA最優先）

### 2.1 SSA変数活用の最大化

```mir
// Before（Load/Store使用）
bb0:
    Store %slot1, %x
    Branch %cond, bb1, bb2
bb1:
    Store %slot1, %y
    Jump bb3
bb2:
    // slot1 は x のまま
    Jump bb3
bb3:
    %result = Load %slot1
    Return %result

// After（Phi使用）
bb0:
    Branch %cond, bb1, bb2
bb1:
    Jump bb3(%y)
bb2:
    Jump bb3(%x)
bb3(%result):
    Return %result
```

### 2.2 フィールドアクセスの統合

```mir
// Before（RefGet/RefSet）
%field_val = RefGet %obj, "field"
RefSet %obj, "field", %new_val

// After（BoxCall）
%field_val = BoxCall %obj, "getField", ["field"]
BoxCall %obj, "setField", ["field", %new_val]
```

### 2.3 残すべきLoad/Store

- **スタックスロット**: JIT/AOTでの一時変数
- **C FFI境界**: 外部関数とのやり取り
- **最適化中間状態**: Phi導入前の一時的使用

## 3. Const統合仕様（設計）

### 3.1 統一表現

```rust
pub enum MirConst {
    // Before: 5種類
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    
    // After: 1種類
    Unified {
        ty: ConstType,
        bits: u64,  // i64/f64/bool/null はビット表現
        aux: Option<Arc<String>>, // 文字列用
    }
}

pub enum ConstType {
    I64, F64, Bool, Null, String, Handle
}
```

### 3.2 エンコーディング

```rust
impl MirConst {
    fn encode_i64(val: i64) -> Self {
        Self::Unified {
            ty: ConstType::I64,
            bits: val as u64,
            aux: None,
        }
    }
    
    fn encode_f64(val: f64) -> Self {
        Self::Unified {
            ty: ConstType::F64,
            bits: val.to_bits(),
            aux: None,
        }
    }
    
    fn encode_bool(val: bool) -> Self {
        Self::Unified {
            ty: ConstType::Bool,
            bits: val as u64,
            aux: None,
        }
    }
}
```

## 4. パフォーマンス保証（CI基準）

### 4.1 ベンチマーク項目

```yaml
必須ベンチマーク:
  - array_access_sequential: 配列順次アクセス
  - array_access_random: 配列ランダムアクセス
  - field_access: フィールド読み書き
  - local_variables: ローカル変数操作
  - arithmetic_loop: 算術演算ループ

許容範囲:
  - 速度: ベースライン ±5%
  - メモリ: ベースライン ±10%
  - MIRサイズ: -20%以上の削減
```

### 4.2 最適化保証

```rust
// 必須最適化パス
const REQUIRED_OPTIMIZATIONS: &[&str] = &[
    "array_bounds_elim",      // 配列境界チェック除去
    "boxcall_devirt",        // BoxCall脱仮想化
    "const_fold",            // 定数畳み込み
    "dead_store_elim",       // 不要Store除去
    "phi_simplify",          // Phi簡約
];
```

## 5. 移行戦略（段階→固定）

### 5.1 段階的有効化

```rust
// 環境変数による制御
// 実装上は env トグルを残しつつ、CI/既定は CORE13=1 / FORBID_LEGACY=1 とする。
```

### 5.2 互換性レイヤー

```rust
// Rewrite パス
pub fn rewrite_legacy_mir(module: &mut MirModule) {
    for (_, func) in &mut module.functions {
        for (_, block) in &mut func.blocks {
            let mut new_instructions = vec![];
            
            for inst in &block.instructions {
                match inst {
                    // ArrayGet/ArraySet → BoxCall
                    MirInstruction::ArrayGet { .. } => {
                        new_instructions.push(convert_array_get(inst));
                    }
                    MirInstruction::ArraySet { .. } => {
                        new_instructions.push(convert_array_set(inst));
                    }
                    
                    // RefGet/RefSet → BoxCall
                    MirInstruction::RefGet { .. } => {
                        new_instructions.push(convert_ref_get(inst));
                    }
                    MirInstruction::RefSet { .. } => {
                        new_instructions.push(convert_ref_set(inst));
                    }
                    
                    // そのまま
                    _ => new_instructions.push(inst.clone()),
                }
            }
            
            block.instructions = new_instructions;
        }
    }
}
```

## 6. 検証項目

### 6.1 正当性検証

```rust
#[cfg(test)]
mod core13_tests {
    // 各変換の意味保存を検証
    #[test]
    fn test_array_get_conversion() {
        let before = MirInstruction::ArrayGet { ... };
        let after = convert_to_boxcall(before);
        assert_semantic_equivalence(before, after);
    }
    
    // SSA形式の保持を検証
    #[test]
    fn test_ssa_preservation() {
        let module = build_test_module();
        eliminate_load_store(&mut module);
        assert_is_valid_ssa(&module);
    }
}
```

### 6.2 性能検証

```rust
// ベンチマークハーネス
pub fn benchmark_core13_migration() {
    let scenarios = vec![
        "array_intensive",
        "field_intensive",
        "arithmetic_heavy",
        "mixed_workload",
    ];
    
    for scenario in scenarios {
        let baseline = run_with_core15(scenario);
        let core13 = run_with_core13(scenario);
        
        assert!(
            (core13.time - baseline.time).abs() / baseline.time < 0.05,
            "Performance regression in {}", scenario
        );
    }
}
```

## 7. エラーハンドリング

### 7.1 診断メッセージ

```rust
pub enum Core13Error {
    UnsupportedInstruction(String),
    ConversionFailed { from: String, to: String },
    PerformanceRegression { metric: String, delta: f64 },
}

impl Core13Error {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            Self::UnsupportedInstruction(inst) => {
                Diagnostic::error()
                    .with_message(format!("Instruction '{}' not supported in Core-13", inst))
                    .with_note("Consider using BoxCall for this operation")
                    .with_help("Set NYASH_MIR_LEGACY=1 for compatibility mode")
            }
            // ...
        }
    }
}
```

---

*この仕様に従い、MIRを「最小の接着剤」として純化し、Boxに「無限の可能性」を委ねる*
