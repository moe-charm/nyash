/*! ✅ BoolBox - 真偽値Box
 * 
 * ## 📝 概要
 * true/false値を扱うためのBox。
 * JavaScript Boolean型のように直感的な論理演算が可能。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `toString()` - 文字列変換 ("true" / "false")
 * - `not()` - 論理NOT (演算子: not)
 * - `and(other)` - 論理AND (演算子: and)
 * - `or(other)` - 論理OR (演算子: or)
 * - `equals(other)` - 等価比較 (演算子: ==)
 * 
 * ## 💡 使用例
 * ```nyash
 * local flag, result, text
 * flag = true
 * 
 * result = not flag          // false
 * result = flag and true     // true
 * result = flag or false     // true
 * text = flag.toString()     // "true"
 * 
 * // 条件分岐での利用
 * if (flag) {
 *     print("Flag is true!")
 * }
 * ```
 * 
 * ## 🔄 型変換
 * - 数値への変換: true → 1, false → 0
 * - 文字列への変換: "true" / "false"
 * - 空文字・null・0は false として扱われる
 * 
 * ## ⚡ 論理演算子実装済み
 * - `not condition` - NOT演算子
 * - `a and b` - AND演算子
 * - `a or b` - OR演算子
 */

use crate::box_trait::{NyashBox, BoxCore, BoxBase};
use std::any::Any;
use std::fmt::Display;

/// Boolean values in Nyash - true/false
#[derive(Debug, Clone, PartialEq)]
pub struct BoolBox {
    pub value: bool,
    base: BoxBase,
}

impl BoolBox {
    pub fn new(value: bool) -> Self {
        Self { 
            value, 
            base: BoxBase::new(),
        }
    }
    
    pub fn true_box() -> Self {
        Self::new(true)
    }
    
    pub fn false_box() -> Self {
        Self::new(false)
    }
}

impl NyashBox for BoolBox {
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        crate::box_trait::StringBox::new(if self.value { "true" } else { "false" })
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            crate::box_trait::BoolBox::new(self.value == other_bool.value)
        } else {
            crate::box_trait::BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "BoolBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    
}

impl BoxCore for BoolBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", if self.value { "true" } else { "false" })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for BoolBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}