// StringBox implementation - String values in Nyash
use crate::box_trait::NyashBox;
use std::any::Any;
use std::fmt::Display;

/// String values in Nyash - UTF-8 encoded text
#[derive(Debug, Clone, PartialEq)]
pub struct StringBox {
    pub value: String,
    id: u64,
}

impl StringBox {
    pub fn new(value: impl Into<String>) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self {
            value: value.into(),
            id,
        }
    }
    
    pub fn empty() -> Self {
        Self::new("")
    }
    
    // ===== String Methods for Nyash =====
    
    /// Split string by delimiter and return ArrayBox
    pub fn split(&self, delimiter: &str) -> Box<dyn NyashBox> {
        use crate::box_trait::ArrayBox;
        let parts: Vec<String> = self.value.split(delimiter).map(|s| s.to_string()).collect();
        let array_elements: Vec<Box<dyn NyashBox>> = parts.into_iter()
            .map(|s| Box::new(StringBox::new(s)) as Box<dyn NyashBox>)
            .collect();
        Box::new(ArrayBox::new_with_elements(array_elements))
    }
    
    /// Find substring and return position (or -1 if not found)
    pub fn find(&self, search: &str) -> Box<dyn NyashBox> {
        use crate::boxes::IntegerBox;
        match self.value.find(search) {
            Some(pos) => Box::new(IntegerBox::new(pos as i64)),
            None => Box::new(IntegerBox::new(-1)),
        }
    }
    
    /// Replace all occurrences of old with new
    pub fn replace(&self, old: &str, new: &str) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.replace(old, new)))
    }
    
    /// Trim whitespace from both ends
    pub fn trim(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.trim()))
    }
    
    /// Convert to uppercase
    pub fn to_upper(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.to_uppercase()))
    }
    
    /// Convert to lowercase  
    pub fn to_lower(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.to_lowercase()))
    }
    
    /// Check if string contains substring
    pub fn contains(&self, search: &str) -> Box<dyn NyashBox> {
        use crate::boxes::BoolBox;
        Box::new(BoolBox::new(self.value.contains(search)))
    }
    
    /// Check if string starts with prefix
    pub fn starts_with(&self, prefix: &str) -> Box<dyn NyashBox> {
        use crate::boxes::BoolBox;
        Box::new(BoolBox::new(self.value.starts_with(prefix)))
    }
    
    /// Check if string ends with suffix
    pub fn ends_with(&self, suffix: &str) -> Box<dyn NyashBox> {
        use crate::boxes::BoolBox;
        Box::new(BoolBox::new(self.value.ends_with(suffix)))
    }
    
    /// Join array elements using this string as delimiter
    pub fn join(&self, array_box: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        use crate::box_trait::ArrayBox;
        if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
            let strings: Vec<String> = array.elements.lock().unwrap()
                .iter()
                .map(|element| element.to_string_box().value)
                .collect();
            Box::new(StringBox::new(strings.join(&self.value)))
        } else {
            // If not an ArrayBox, treat as single element
            Box::new(StringBox::new(array_box.to_string_box().value))
        }
    }
}

impl NyashBox for StringBox {
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        crate::box_trait::StringBox::new(self.value.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        use crate::box_trait::BoolBox;
        if let Some(other_string) = other.as_any().downcast_ref::<StringBox>() {
            BoolBox::new(self.value == other_string.value)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "StringBox"
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

impl Display for StringBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}