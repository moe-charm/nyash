//! RegexBox 🔍 - 正規表現
// Nyashの箱システムによる正規表現処理を提供します。
// 参考: 既存Boxの設計思想

use regex::Regex;
use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RegexBox {
    pub regex: Regex,
    id: u64,
    pattern: String,
}

impl RegexBox {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        static mut COUNTER: u64 = 0;
        let regex = Regex::new(pattern)?;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        Ok(RegexBox {
            regex,
            id,
            pattern: pattern.to_string(),
        })
    }
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl NyashBox for RegexBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("RegexBox({})", self.pattern))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "RegexBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_regex) = other.as_any().downcast_ref::<RegexBox>() {
            BoolBox::new(self.pattern == other_regex.pattern)
        } else {
            BoolBox::new(false)
        }
    }
}
