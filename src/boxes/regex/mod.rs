//! RegexBox 🔍 - 正規表現
// Nyashの箱システムによる正規表現処理を提供します。
// 参考: 既存Boxの設計思想

use regex::Regex;

pub struct RegexBox {
    pub regex: Regex,
}

impl RegexBox {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        Ok(RegexBox { regex })
    }
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}
