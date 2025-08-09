// IntegerBox implementation - Integer values in Nyash
use crate::box_trait::NyashBox;
use std::any::Any;
use std::fmt::Display;

/// Integer values in Nyash - 64-bit signed integers
#[derive(Debug, Clone, PartialEq)]
pub struct IntegerBox {
    pub value: i64,
    id: u64,
}

impl IntegerBox {
    pub fn new(value: i64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { value, id }
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
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for IntegerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}