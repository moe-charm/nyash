//! ResultBox ⚠️ - エラー処理（ResultBox推奨）
// Nyashの箱システムによるエラー処理を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;

#[derive(Debug)]
pub enum NyashResultBox {
    Ok(Box<dyn NyashBox>),
    Err(Box<dyn NyashBox>),
}

impl NyashResultBox {
    pub fn new_ok(value: Box<dyn NyashBox>) -> Self {
        NyashResultBox::Ok(value)
    }
    
    pub fn new_err(error: Box<dyn NyashBox>) -> Self {
        NyashResultBox::Err(error)
    }
    
    pub fn is_ok(&self) -> bool {
        matches!(self, NyashResultBox::Ok(_))
    }
    
    pub fn is_err(&self) -> bool {
        matches!(self, NyashResultBox::Err(_))
    }
    
    pub fn unwrap(self) -> Box<dyn NyashBox> {
        match self {
            NyashResultBox::Ok(val) => val,
            NyashResultBox::Err(_) => panic!("called `unwrap()` on an `Err` value"),
        }
    }
}

impl NyashBox for NyashResultBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        match self {
            NyashResultBox::Ok(val) => Box::new(NyashResultBox::Ok(val.clone_box())),
            NyashResultBox::Err(err) => Box::new(NyashResultBox::Err(err.clone_box())),
        }
    }

    fn to_string_box(&self) -> StringBox {
        match self {
            NyashResultBox::Ok(val) => StringBox::new(format!("Ok({})", val.to_string_box().value)),
            NyashResultBox::Err(err) => StringBox::new(format!("Err({})", err.to_string_box().value)),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "NyashResultBox"
    }

    fn box_id(&self) -> u64 {
        // For enum variants, we use the contained value's ID
        match self {
            NyashResultBox::Ok(val) => val.box_id(),
            NyashResultBox::Err(err) => err.box_id(),
        }
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_result) = other.as_any().downcast_ref::<NyashResultBox>() {
            match (self, other_result) {
                (NyashResultBox::Ok(a), NyashResultBox::Ok(b)) => a.equals(b.as_ref()),
                (NyashResultBox::Err(a), NyashResultBox::Err(b)) => a.equals(b.as_ref()),
                _ => BoolBox::new(false),
            }
        } else {
            BoolBox::new(false)
        }
    }
}

// Export NyashResultBox as ResultBox for compatibility
pub type ResultBox = NyashResultBox;

impl ResultBox {
    /// is_ok()の実装
    pub fn is_ok(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(matches!(self, NyashResultBox::Ok(_))))
    }
    
    /// getValue()の実装 - Ok値を取得
    pub fn get_value(&self) -> Box<dyn NyashBox> {
        match self {
            NyashResultBox::Ok(val) => val.clone_box(),
            NyashResultBox::Err(_) => Box::new(StringBox::new("Error: Result is Err")),
        }
    }
    
    /// getError()の実装 - Err値を取得
    pub fn get_error(&self) -> Box<dyn NyashBox> {
        match self {
            NyashResultBox::Ok(_) => Box::new(StringBox::new("Error: Result is Ok")),
            NyashResultBox::Err(err) => err.clone_box(),
        }
    }
}
