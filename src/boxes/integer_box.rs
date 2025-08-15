/*! 🔢 IntegerBox - 整数計算Box
 * 
 * ## 📝 概要
 * 64ビット符号付き整数を扱うためのBox。
 * JavaScript Number型のように直感的な数値操作が可能。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `toString()` - 文字列変換
 * - `add(other)` - 加算 (演算子: +)
 * - `subtract(other)` - 減算 (演算子: -)
 * - `multiply(other)` - 乗算 (演算子: *)
 * - `divide(other)` - 除算 (演算子: /) 
 * - `modulo(other)` - 余り計算 (演算子: %)
 * - `equals(other)` - 等価比較 (演算子: ==)
 * - `abs()` - 絶対値
 * - `min(other)` - 最小値
 * - `max(other)` - 最大値
 * 
 * ## 💡 使用例
 * ```nyash
 * local num, result, text
 * num = 42
 * 
 * result = num + 8           // 50
 * result = num * 2           // 84
 * result = num / 3           // 14 (整数除算)
 * text = num.toString()      // "42"
 * 
 * // メソッド呼び出し形式も可能
 * result = num.add(10)       // 52
 * result = num.multiply(3)   // 126
 * ```
 * 
 * ## ⚠️ 注意
 * - ゼロ除算は実行時エラー
 * - オーバーフロー時は標準i64の動作に従う
 * - 小数点以下は切り捨て（整数除算）
 */

use crate::box_trait::{NyashBox, BoxCore, BoxBase};
use std::any::Any;
use std::fmt::Display;

/// Integer values in Nyash - 64-bit signed integers
#[derive(Debug, Clone, PartialEq)]
pub struct IntegerBox {
    pub value: i64,
    base: BoxBase,
}

impl IntegerBox {
    pub fn new(value: i64) -> Self {
        Self { 
            value, 
            base: BoxBase::new(),
        }
    }
    
    pub fn zero() -> Self {
        Self::new(0)
    }
}

impl NyashBox for IntegerBox {
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        crate::box_trait::StringBox::new(self.value.to_string())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        use crate::box_trait::BoolBox;
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            BoolBox::new(self.value == other_int.value)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "IntegerBox"
    }
    
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for IntegerBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for IntegerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}