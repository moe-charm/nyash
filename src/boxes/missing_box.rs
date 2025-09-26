/*! ❓ MissingBox — 欠損/未設定の値を表す Box（Null と区別）
 *
 * 目的: JSON の欠損キー、未設定のフィールド、未初期化参照などを表現するための一級オブジェクト。
 * 設計: Null（明示的な無）とは意味を分離。演算・比較・呼び出し境界に現れたら原則エラー。
 * 既定: 本実装は型の導入のみ（生成はしない）。既定挙動は従来どおり不変。
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone)]
pub struct MissingBox {
    base: BoxBase,
}

impl MissingBox {
    pub fn new() -> Self {
        Self { base: BoxBase::new() }
    }

    pub fn is_missing(&self) -> bool { true }
}

impl BoxCore for MissingBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 開発時の可視性向上のための文字列表現。prod では基本的に表面化させない想定。
        write!(f, "(missing)")
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl NyashBox for MissingBox {
    fn type_name(&self) -> &'static str { "MissingBox" }
    fn to_string_box(&self) -> StringBox { StringBox::new("(missing)") }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        // 欠損どうしは論理同値とみなすが、通常の等価比較は境界で禁止される想定。
        BoolBox::new(other.as_any().downcast_ref::<MissingBox>().is_some())
    }
}

impl Display for MissingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) }
}

