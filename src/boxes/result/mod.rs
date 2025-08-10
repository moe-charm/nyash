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

// Keep the original generic ResultBox for compatibility
pub enum ResultBox<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> ResultBox<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, ResultBox::Ok(_))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, ResultBox::Err(_))
    }
    pub fn unwrap(self) -> T {
        match self {
            ResultBox::Ok(val) => val,
            ResultBox::Err(_) => panic!("called `unwrap()` on an `Err` value"),
        }
    }
}
