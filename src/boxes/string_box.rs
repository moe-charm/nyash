/*! 🔤 StringBox - 文字列操作Box
 *
 * ## 📝 概要
 * UTF-8エンコード文字列を扱うためのBox。
 * JavaScript風のメソッドで直感的な文字列操作が可能。
 *
 * ## 🛠️ 利用可能メソッド
 * - `length()` - 文字列長を取得
 * - `concat(other)` - 文字列結合  
 * - `split(separator)` - 区切り文字で分割
 * - `substring(start, end)` - 部分文字列取得
 * - `toUpperCase()` - 大文字変換
 * - `toLowerCase()` - 小文字変換
 * - `trim()` - 前後の空白除去
 * - `indexOf(search)` - 文字列検索
 * - `replace(from, to)` - 文字列置換
 * - `charAt(index)` - 指定位置の文字取得
 *
 * ## 💡 使用例
 * ```nyash
 * local text, parts, result
 * text = "Hello, World!"
 *
 * print(text.length())        // 13
 * print(text.toUpperCase())   // "HELLO, WORLD!"
 * parts = text.split(",")     // ["Hello", " World!"]
 * result = text.concat(" Nyash")  // "Hello, World! Nyash"
 * ```
 */
use crate::box_trait::{BoxBase, BoxCore, NyashBox};
use std::any::Any;
use std::fmt::Display;

/// String values in Nyash - UTF-8 encoded text
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
        use crate::boxes::array::ArrayBox;
        let parts: Vec<String> = self.value.split(delimiter).map(|s| s.to_string()).collect();
        let array_elements: Vec<Box<dyn NyashBox>> = parts
            .into_iter()
            .map(|s| Box::new(StringBox::new(s)) as Box<dyn NyashBox>)
            .collect();
        let result = ArrayBox::new();
        for element in array_elements {
            result.push(element);
        }
        Box::new(result)
    }

    /// Find substring and return position (or -1 if not found)
    /// Env gate: NYASH_STR_CP=1 → return codepoint index; default is byte index
    pub fn find(&self, search: &str) -> Box<dyn NyashBox> {
        use crate::boxes::integer_box::IntegerBox;
        match self.value.find(search) {
            Some(byte_pos) => {
                let use_cp = std::env::var("NYASH_STR_CP").ok().as_deref() == Some("1");
                let idx = if use_cp {
                    self.value[..byte_pos].chars().count() as i64
                } else {
                    byte_pos as i64
                };
                Box::new(IntegerBox::new(idx))
            }
            None => Box::new(IntegerBox::new(-1)),
        }
    }

    /// Replace all occurrences of old with new
    pub fn replace(&self, old: &str, new: &str) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.replace(old, new)))
    }

    /// Return the last index of `search` or -1 if not found.
    /// Env gate: NYASH_STR_CP=1 → return codepoint index; default is byte index.
    pub fn lastIndexOf(&self, search: &str) -> Box<dyn NyashBox> {
        use crate::boxes::integer_box::IntegerBox;
        match self.value.rfind(search) {
            Some(byte_pos) => {
                let use_cp = std::env::var("NYASH_STR_CP").ok().as_deref() == Some("1");
                let idx = if use_cp {
                    self.value[..byte_pos].chars().count() as i64
                } else {
                    byte_pos as i64
                };
                Box::new(IntegerBox::new(idx))
            }
            None => Box::new(IntegerBox::new(-1)),
        }
    }

    /// Trim whitespace from both ends
    pub fn trim(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.value.trim()))
    }

    /// size() alias for length (codepoint-aware under NYASH_STR_CP)
    pub fn size(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::IntegerBox;
        let use_cp = std::env::var("NYASH_STR_CP").ok().as_deref() == Some("1");
        let n = if use_cp {
            self.value.chars().count() as i64
        } else {
            self.value.len() as i64
        };
        Box::new(IntegerBox::new(n))
    }

    /// isEmpty(): true when string length is zero
    pub fn isEmpty(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::BoolBox;
        Box::new(BoolBox::new(self.value.is_empty()))
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
        use crate::boxes::bool_box::BoolBox;
        Box::new(BoolBox::new(self.value.contains(search)))
    }

    /// Check if string starts with prefix
    pub fn starts_with(&self, prefix: &str) -> Box<dyn NyashBox> {
        use crate::boxes::bool_box::BoolBox;
        Box::new(BoolBox::new(self.value.starts_with(prefix)))
    }

    /// Check if string ends with suffix
    pub fn ends_with(&self, suffix: &str) -> Box<dyn NyashBox> {
        use crate::boxes::bool_box::BoolBox;
        Box::new(BoolBox::new(self.value.ends_with(suffix)))
    }

    /// Join array elements using this string as delimiter
    pub fn join(&self, array_box: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        use crate::boxes::array::ArrayBox;
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

    /// Convert string to integer (parse as i64)
    pub fn to_integer(&self) -> Box<dyn NyashBox> {
        use crate::boxes::integer_box::IntegerBox;
        match self.value.trim().parse::<i64>() {
            Ok(n) => Box::new(IntegerBox::new(n)),
            Err(_) => {
                // If parsing fails, return 0 (JavaScript-like behavior)
                Box::new(IntegerBox::new(0))
            }
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

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
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

impl Display for StringBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
