// BoolBox implementation - Boolean values in Nyash
use crate::box_trait::NyashBox;
use std::any::Any;
use std::fmt::Display;

/// Boolean values in Nyash - true/false
#[derive(Debug, Clone, PartialEq)]
pub struct BoolBox {
    pub value: bool,
    id: u64,
}

impl BoolBox {
    pub fn new(value: bool) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { value, id }
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
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for BoolBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.value { "true" } else { "false" })
    }
}