# FloatBox実装計画

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🟡 中（JSON処理に必要）
**影響範囲**: コアBox・JSON処理

## 🎯 問題

FloatBox未実装により2箇所でコメントアウト：

### 該当箇所

#### 1. `src/boxes/json/mod.rs:213`
```rust
// TODO: FloatBoxが実装されたら有効化
// LiteralValue::Float(f) => {
//     let float_box = FloatBox::new(*f);
//     Ok(Arc::new(float_box))
// }
```

#### 2. `src/boxes/json/mod.rs:250`
```rust
// TODO: FloatBoxが実装されたら有効化
// "FloatBox" => {
//     if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
//         LiteralValue::Float(float_box.value())
//     } else {
//         LiteralValue::Null
//     }
// }
```

### 現状の問題点
- JSON浮動小数点数が正しく処理できない
- LiteralValue::Floatが使用不可
- 回避策: IntegerBoxで代用（精度損失）

## 💡 解決策案

### Option A: コアビルトインBox実装（推奨）

```rust
// src/box_factory/builtin_impls/float_box.rs
use crate::box_trait::NyashBox;
use std::any::Any;
use std::sync::Arc;

/// FloatBox - 浮動小数点数を保持するBox
#[derive(Debug, Clone)]
pub struct FloatBox {
    value: f64,
}

impl FloatBox {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }
}

impl NyashBox for FloatBox {
    fn type_name(&self) -> &str {
        "FloatBox"
    }

    fn call_method(&self, method: &str, args: &[Arc<dyn NyashBox>]) -> Result<Arc<dyn NyashBox>, String> {
        match method {
            // 算術演算
            "add" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.add requires FloatBox argument")?;
                Ok(Arc::new(FloatBox::new(self.value + other.value)))
            }
            "sub" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.sub requires FloatBox argument")?;
                Ok(Arc::new(FloatBox::new(self.value - other.value)))
            }
            "mul" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.mul requires FloatBox argument")?;
                Ok(Arc::new(FloatBox::new(self.value * other.value)))
            }
            "div" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.div requires FloatBox argument")?;
                if other.value == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Arc::new(FloatBox::new(self.value / other.value)))
            }

            // 比較演算
            "eq" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.eq requires FloatBox argument")?;
                Ok(Arc::new(BoolBox::new((self.value - other.value).abs() < f64::EPSILON)))
            }
            "lt" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.lt requires FloatBox argument")?;
                Ok(Arc::new(BoolBox::new(self.value < other.value)))
            }
            "gt" => {
                let other = args.get(0)
                    .and_then(|b| b.as_any().downcast_ref::<FloatBox>())
                    .ok_or("FloatBox.gt requires FloatBox argument")?;
                Ok(Arc::new(BoolBox::new(self.value > other.value)))
            }

            // 型変換
            "to_int" => {
                Ok(Arc::new(IntegerBox::new(self.value as i64)))
            }
            "to_string" => {
                Ok(Arc::new(StringBox::new(self.value.to_string())))
            }

            // 数学関数
            "abs" => Ok(Arc::new(FloatBox::new(self.value.abs()))),
            "sqrt" => Ok(Arc::new(FloatBox::new(self.value.sqrt()))),
            "ceil" => Ok(Arc::new(FloatBox::new(self.value.ceil()))),
            "floor" => Ok(Arc::new(FloatBox::new(self.value.floor()))),
            "round" => Ok(Arc::new(FloatBox::new(self.value.round()))),

            _ => Err(format!("Unknown method: FloatBox.{}", method))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Arc<dyn NyashBox> {
        Arc::new(self.clone())
    }
}
```

**利点**:
- コアBox（builtin）として実装
- プラグイン不要
- 高性能（直接実装）

**実装時間**: 3-4時間

### Option B: プラグインBox実装

```bash
# plugins/floatbox/
plugins/floatbox/
├── Cargo.toml
├── plugin.nyashbox.toml
└── src/
    └── lib.rs
```

**利点**:
- プラグインシステムの実証
- 独立したビルド

**欠点**:
- コアBoxとして必要（builtin推奨）
- ローディングオーバーヘッド

**実装時間**: 4-5時間

### Option C: Python FloatBox（PyVM専用）

```python
# src/llvm_py/boxes/float_box.py
class FloatBox:
    def __init__(self, value):
        self.value = float(value)

    def add(self, other):
        return FloatBox(self.value + other.value)
    # ...
```

**利点**:
- PyVM統合容易
- Python浮動小数点精度

**欠点**:
- VM/LLVMバックエンドで使用不可

**実装時間**: 1-2時間（PyVMのみ）

## 🚀 実装ステップ（推奨: Option A）

### Step 1: FloatBox実装 - 3時間
1. `src/box_factory/builtin_impls/float_box.rs`作成
2. 算術演算・比較演算実装
3. 型変換・数学関数実装

### Step 2: JSON統合 - 1時間
1. `src/boxes/json/mod.rs`でTODO解除
2. LiteralValue::Float処理有効化
3. テストケース追加

### Step 3: MIRビルダー統合 - 1時間
1. FloatBox生成サポート
2. Floatリテラル（`3.14`）の処理
3. 型推論・キャスト対応

## 📊 影響範囲

### 新規追加ファイル
- `src/box_factory/builtin_impls/float_box.rs` - FloatBox実装
- `tests/floatbox_basic.rs` - 基本動作テスト
- `tests/floatbox_json.rs` - JSON統合テスト

### 修正必要ファイル
- `src/boxes/json/mod.rs` - TODO解除（2箇所）
- `src/box_factory/builtin_impls/mod.rs` - FloatBox追加
- `src/mir/builder/exprs.rs` - FloatリテラルLowering
- `src/ast.rs` - LiteralValue::Float有効化

### テスト追加
- FloatBox算術演算テスト
- FloatBox比較演算テスト
- JSON Float roundtripテスト
- スモークテスト: Float処理全般

## 🎯 成功基準

- ✅ FloatBox基本動作（算術・比較）
- ✅ JSON Float処理が正しく動作
- ✅ MIRビルダーでFloatリテラル処理
- ✅ IEEE 754浮動小数点精度
- ✅ 既存のすべてのスモークテストがPASS

## 🔗 関連資料

- [Core Box設計](../../../../reference/boxes-system/core-boxes.md)
- [JSON処理](../../../../reference/boxes-system/json-box.md)
- IEEE 754仕様: https://en.wikipedia.org/wiki/IEEE_754

## 📝 補足

**優先度判断**:
- JSON処理で浮動小数点必須
- 多くの言語でFloat型は基本型
- **Phase 15.6で実装推奨**

**実装タイミング**: Phase 15.5完了後、JSON処理強化時

**代替手段（現状）**:
- IntegerBoxで代用（精度損失）
- JSON Floatをstringとして処理（回避策）

**メリット**:
- JSON処理完全対応
- 科学計算・数学処理可能
- Nyash言語の基本型完成（Integer/Float/Bool/String/Null/Void）

**注意点**:
- 浮動小数点比較（`==`）は`abs(a - b) < EPSILON`で実装必須
- NaN/Infinity処理の明確化
- JSON roundtrip精度保証（IEEE 754準拠）