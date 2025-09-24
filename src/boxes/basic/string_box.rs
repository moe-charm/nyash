//! StringBox - String values in Nyash
//!
//! Implements the core StringBox type with all string manipulation methods.

use crate::box_trait::{NyashBox, BoxCore, BoxBase};
use crate::boxes::ArrayBox;
use std::any::Any;
use std::fmt::{Debug, Display};

/// String values in Nyash - UTF-8 strings with rich method support
#[derive(Debug, Clone, PartialEq)]
pub struct StringBox {
    pub value: String,
    base: BoxBase,
}

impl StringBox {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            base: BoxBase::new(),
        }
    }

    pub fn empty() -> Self {
        Self::new("")
    }

    // ===== String Methods for Nyash =====

    /// Split string by delimiter and return ArrayBox
    pub fn split(&self, delimiter: &str) -> Box<dyn NyashBox> {
        let parts: Vec<String> = self.value.split(delimiter).map(|s| s.to_string()).collect();
        let array_elements: Vec<Box<dyn NyashBox>> = parts
            .into_iter()
            .map(|s| Box::new(StringBox::new(s)) as Box<dyn NyashBox>)
            .collect();
        Box::new(ArrayBox::new_with_elements(array_elements))
    }

    /// Find substring and return position (or -1 if not found)
    pub fn find(&self, search: &str) -> Box<dyn NyashBox> {
        use crate::box_trait::IntegerBox;
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
        use crate::box_trait::BoolBox;
        Box::new(BoolBox::new(self.value.contains(search)))
    }

    /// Check if string starts with prefix
    pub fn starts_with(&self, prefix: &str) -> Box<dyn NyashBox> {
        use crate::box_trait::BoolBox;
        Box::new(BoolBox::new(self.value.starts_with(prefix)))
    }

    /// Check if string ends with suffix
    pub fn ends_with(&self, suffix: &str) -> Box<dyn NyashBox> {
        use crate::box_trait::BoolBox;
        Box::new(BoolBox::new(self.value.ends_with(suffix)))
    }

    /// Join array elements using this string as delimiter
    pub fn join(&self, array_box: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
            let strings: Vec<String> = array
                .items
                .read()
                .unwrap()
                .iter()
                .map(|element| element.to_string_box().value)
                .collect();
            Box::new(StringBox::new(strings.join(&self.value)))
        } else {
            // If not an ArrayBox, treat as single element
            Box::new(StringBox::new(array_box.to_string_box().value))
        }
    }

    /// Get string length
    ///
    /// Env gate: NYASH_STR_CP=1 → count Unicode scalar values (chars),
    /// otherwise use UTF-8 byte length (legacy/default).
    pub fn length(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::IntegerBox;
        let use_cp = std::env::var("NYASH_STR_CP").ok().as_deref() == Some("1");
        let n = if use_cp {
            self.value.chars().count() as i64
        } else {
            self.value.len() as i64
        };
        Box::new(IntegerBox::new(n))
    }

    /// Convert string to integer (parse as i64)
    pub fn to_integer(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::IntegerBox;
        match self.value.trim().parse::<i64>() {
            Ok(n) => Box::new(IntegerBox::new(n)),
            Err(_) => {
                // If parsing fails, return 0 (JavaScript-like behavior)
                Box::new(IntegerBox::new(0))
            }
        }
    }

    /// Get character at index
    pub fn get(&self, index: usize) -> Option<Box<dyn NyashBox>> {
        if let Some(ch) = self.value.chars().nth(index) {
            Some(Box::new(StringBox::new(ch.to_string())))
        } else {
            None
        }
    }

    /// Get substring from start to end (exclusive)
    pub fn substring(&self, start: usize, end: usize) -> Box<dyn NyashBox> {
        let chars: Vec<char> = self.value.chars().collect();
        let actual_end = end.min(chars.len());
        let actual_start = start.min(actual_end);
        let substring: String = chars[actual_start..actual_end].iter().collect();
        Box::new(StringBox::new(substring))
    }
}

impl BoxCore for StringBox {
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

impl NyashBox for StringBox {
    fn to_string_box(&self) -> StringBox {
        self.clone()
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

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for StringBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}