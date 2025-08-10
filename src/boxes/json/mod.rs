//! JSONBox 📋 - JSON解析・生成
// Nyashの箱システムによるJSON解析・生成を提供します。
// 参考: 既存Boxの設計思想

use serde_json::{Value, Error};

pub struct JSONBox {
    pub value: Value,
}

impl JSONBox {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        let value = serde_json::from_str(s)?;
        Ok(JSONBox { value })
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}
