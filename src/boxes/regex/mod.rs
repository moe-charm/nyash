//! RegexBox 🔍 - 正規表現
// Nyashの箱システムによる正規表現処理を提供します。
// 参考: 既存Boxの設計思想

use regex::Regex;
use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::boxes::array::ArrayBox;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RegexBox {
    regex: Arc<Regex>,
    pattern: Arc<String>,
    id: u64,
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
            regex: Arc::new(regex),
            pattern: Arc::new(pattern.to_string()),
            id,
        })
    }
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
    
    /// パターンマッチテスト
    pub fn test(&self, text: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let text_str = text.to_string_box().value;
        Box::new(BoolBox::new(self.is_match(&text_str)))
    }
    
    /// マッチ箇所を検索
    pub fn find(&self, text: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let text_str = text.to_string_box().value;
        if let Some(mat) = self.regex.find(&text_str) {
            Box::new(StringBox::new(mat.as_str()))
        } else {
            Box::new(crate::boxes::null_box::NullBox::new())
        }
    }
    
    /// すべてのマッチを検索
    pub fn find_all(&self, text: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let text_str = text.to_string_box().value;
        let array = ArrayBox::new();
        
        for mat in self.regex.find_iter(&text_str) {
            let _ = array.push(Box::new(StringBox::new(mat.as_str())));
        }
        
        Box::new(array)
    }
    
    /// 文字列置換
    pub fn replace(&self, text: Box<dyn NyashBox>, replacement: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let text_str = text.to_string_box().value;
        let replacement_str = replacement.to_string_box().value;
        let result = self.regex.replace_all(&text_str, replacement_str.as_str());
        Box::new(StringBox::new(&result))
    }
    
    /// 文字列分割
    pub fn split(&self, text: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let text_str = text.to_string_box().value;
        let array = ArrayBox::new();
        
        for part in self.regex.split(&text_str) {
            let _ = array.push(Box::new(StringBox::new(part)));
        }
        
        Box::new(array)
    }
}

impl NyashBox for RegexBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("RegexBox({})", **self.pattern))
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
            BoolBox::new(**self.pattern == **other_regex.pattern)
        } else {
            BoolBox::new(false)
        }
    }
}
