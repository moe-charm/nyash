/*!
 * Nyash Null Box - Null value representation
 * 
 * null値を表現するBox型
 * Everything is Box哲学に基づくnull実装
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::fmt::{Debug, Display};
use std::any::Any;

/// null値を表現するBox
#[derive(Debug, Clone)]
pub struct NullBox {
    id: u64,
}

impl NullBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { id }
    }
    
    /// null値かどうかを判定
    pub fn is_null(&self) -> bool {
        true  // NullBoxは常にnull
    }
    
    /// 値がnullでないかを判定
    pub fn is_not_null(&self) -> bool {
        false  // NullBoxは常にnull
    }
    
    /// 他の値がnullかどうかを判定
    pub fn check_null(value: &dyn NyashBox) -> bool {
        value.as_any().downcast_ref::<NullBox>().is_some()
    }
    
    /// 他の値がnullでないかを判定
    pub fn check_not_null(value: &dyn NyashBox) -> bool {
        !Self::check_null(value)
    }
    
    /// null安全な値の取得
    pub fn get_or_default(
        value: &dyn NyashBox, 
        default: Box<dyn NyashBox>
    ) -> Box<dyn NyashBox> {
        if Self::check_null(value) {
            default
        } else {
            value.clone_box()
        }
    }
}

impl NyashBox for NullBox {
    fn type_name(&self) -> &'static str {
        "NullBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("null")
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        // すべてのNullBoxは等しい
        BoolBox::new(other.as_any().downcast_ref::<NullBox>().is_some())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for NullBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

// グローバルnullインスタンス用の関数
pub fn null() -> Box<dyn NyashBox> {
    Box::new(NullBox::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::IntegerBox;
    
    #[test]
    fn test_null_creation() {
        let null_box = NullBox::new();
        assert!(null_box.is_null());
        assert!(!null_box.is_not_null());
        assert_eq!(null_box.to_string_box().value, "null");
    }
    
    #[test]
    fn test_null_check() {
        let null_box = null();
        let int_box = Box::new(IntegerBox::new(42));
        
        assert!(NullBox::check_null(null_box.as_ref()));
        assert!(!NullBox::check_null(int_box.as_ref()));
        
        assert!(!NullBox::check_not_null(null_box.as_ref()));
        assert!(NullBox::check_not_null(int_box.as_ref()));
    }
    
    #[test]
    fn test_null_equality() {
        let null1 = NullBox::new();
        let null2 = NullBox::new();
        let int_box = IntegerBox::new(42);
        
        assert!(null1.equals(&null2).value);
        assert!(!null1.equals(&int_box).value);
    }
    
    #[test]
    fn test_get_or_default() {
        let null_box = null();
        let default_value = Box::new(IntegerBox::new(100));
        let actual_value = Box::new(IntegerBox::new(42));
        
        // nullの場合はデフォルト値を返す
        let result1 = NullBox::get_or_default(null_box.as_ref(), default_value.clone());
        assert_eq!(result1.to_string_box().value, "100");
        
        // null以外の場合は元の値を返す
        let result2 = NullBox::get_or_default(actual_value.as_ref(), default_value);
        assert_eq!(result2.to_string_box().value, "42");
    }
}