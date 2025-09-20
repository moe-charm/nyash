/*!
 * Nyash Box Trait System - Everything is Box in Rust
 *
 * This module implements the core "Everything is Box" philosophy using Rust's
 * ownership system and trait system. Every value in Nyash is a Box that
 * implements the NyashBox trait.
 */

use std::any::Any;
use std::fmt::{Debug, Display};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// 🔥 新しい型エイリアス - 将来的にBox<dyn NyashBox>を全て置き換える
pub type SharedNyashBox = Arc<dyn NyashBox>;

/// 🔥 BoxBase + BoxCore革命 - 統一ID生成システム
/// CharmFlow教訓を活かした互換性保証の基盤
pub fn next_box_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定リスト
/// ユーザーは`pack`を一切意識せず、`from BuiltinBox()`で自動的に内部のpack機能が呼ばれる
pub const BUILTIN_BOXES: &[&str] = &[
    "StringBox",
    "IntegerBox",
    "BoolBox",
    "NullBox",
    "ArrayBox",
    "MapBox",
    "FileBox",
    "ResultBox",
    "FutureBox",
    "ChannelBox",
    "MathBox",
    "FloatBox",
    "TimeBox",
    "DateTimeBox",
    "TimerBox",
    "RandomBox",
    "SoundBox",
    "DebugBox",
    "MethodBox",
    "ConsoleBox",
    "BufferBox",
    "RegexBox",
    "JSONBox",
    "StreamBox",
    "HTTPClientBox",
    "IntentBox",
    "P2PBox",
    "SocketBox",
    "HTTPServerBox",
    "HTTPRequestBox",
    "HTTPResponseBox",
    "JitConfigBox",
];

/// 🔥 ビルトインBox判定関数 - pack透明化システムの核心
/// ユーザー側: `from StringBox()` → 内部的に `StringBox.pack()` 自動呼び出し
pub fn is_builtin_box(box_name: &str) -> bool {
    BUILTIN_BOXES.contains(&box_name)
}

/// 🏗️ BoxBase - 全てのBox型の共通基盤構造体
/// Phase 2: 統一的な基盤データを提供
/// 🔥 Phase 1: ビルトインBox継承システム - 最小限拡張
#[derive(Debug, Clone, PartialEq)]
pub struct BoxBase {
    pub id: u64,
    pub parent_type_id: Option<std::any::TypeId>, // ビルトインBox継承用
}

impl BoxBase {
    /// 新しいBoxBase作成 - 安全なID生成
    pub fn new() -> Self {
        Self {
            id: next_box_id(),
            parent_type_id: None, // ビルトインBox: 継承なし
        }
    }

    /// ビルトインBox継承用コンストラクタ
    pub fn with_parent_type(parent_type_id: std::any::TypeId) -> Self {
        Self {
            id: next_box_id(),
            parent_type_id: Some(parent_type_id),
        }
    }
}

/// 🎯 BoxCore - Box型共通メソッドの統一インターフェース
/// Phase 2: 重複コードを削減する中核トレイト
/// 🔥 Phase 2: ビルトインBox継承システム対応
pub trait BoxCore: Send + Sync {
    /// ボックスの一意ID取得
    fn box_id(&self) -> u64;

    /// 継承元の型ID取得（ビルトインBox継承用）
    fn parent_type_id(&self) -> Option<std::any::TypeId>;

    /// Display実装のための統一フォーマット
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    /// Any変換（ダウンキャスト用）
    fn as_any(&self) -> &dyn Any;

    /// Anyミュータブル変換（ダウンキャスト用）  
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The fundamental trait that all Nyash values must implement.
/// This embodies the "Everything is Box" philosophy with Rust's type safety.
pub trait NyashBox: BoxCore + Debug {
    /// Convert this box to a string representation (equivalent to Python's toString())
    fn to_string_box(&self) -> StringBox;

    /// Check equality with another box (equivalent to Python's equals())
    fn equals(&self, other: &dyn NyashBox) -> BoolBox;

    /// Get the type name of this box for debugging
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Clone this box (equivalent to Python's copy())
    fn clone_box(&self) -> Box<dyn NyashBox>;

    /// Share this box (state-preserving reference sharing)
    fn share_box(&self) -> Box<dyn NyashBox>;

    /// Identity hint: boxes that wrap external/stateful handles should override to return true.
    fn is_identity(&self) -> bool {
        false
    }

    /// Helper: pick share or clone based on identity semantics.
    fn clone_or_share(&self) -> Box<dyn NyashBox> {
        if self.is_identity() {
            self.share_box()
        } else {
            self.clone_box()
        }
    }

    /// Arc参照を返す新しいcloneメソッド（参照共有）
    fn clone_arc(&self) -> SharedNyashBox {
        Arc::from(self.clone_box())
    }

    // 🌟 TypeBox革命: Get type information as a Box
    // Everything is Box極限実現 - 型情報もBoxとして取得！
    // TODO: 次のステップで完全実装
    // fn get_type_box(&self) -> std::sync::Arc<crate::type_box::TypeBox>;
}

// ===== Basic Box Types =====

/// String values in Nyash - immutable and owned
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
        Box::new(BoolBox::new(self.value.contains(search)))
    }

    /// Check if string starts with prefix
    pub fn starts_with(&self, prefix: &str) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(self.value.starts_with(prefix)))
    }

    /// Check if string ends with suffix
    pub fn ends_with(&self, suffix: &str) -> Box<dyn NyashBox> {
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

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
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

/// Integer values in Nyash - 64-bit signed integers
#[derive(Debug, Clone, PartialEq)]
pub struct IntegerBox {
    pub value: i64,
    base: BoxBase,
}

impl IntegerBox {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            base: BoxBase::new(),
        }
    }

    pub fn zero() -> Self {
        Self::new(0)
    }
}

impl BoxCore for IntegerBox {
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

impl NyashBox for IntegerBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.value.to_string())
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
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

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for IntegerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Boolean values in Nyash - true/false
#[derive(Debug, Clone, PartialEq)]
pub struct BoolBox {
    pub value: bool,
    base: BoxBase,
}

impl BoolBox {
    pub fn new(value: bool) -> Self {
        Self {
            value,
            base: BoxBase::new(),
        }
    }

    pub fn true_box() -> Self {
        Self::new(true)
    }

    pub fn false_box() -> Self {
        Self::new(false)
    }
}

impl BoxCore for BoolBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", if self.value { "true" } else { "false" })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for BoolBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(if self.value { "true" } else { "false" })
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            BoolBox::new(self.value == other_bool.value)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "BoolBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for BoolBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Void/null values in Nyash - represents empty or null results
#[derive(Debug, Clone, PartialEq)]
pub struct VoidBox {
    base: BoxBase,
}

impl VoidBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl Default for VoidBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxCore for VoidBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "void")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for VoidBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("void")
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<VoidBox>())
    }

    fn type_name(&self) -> &'static str {
        "VoidBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for VoidBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// ArrayBox is now defined in boxes::array module
pub use crate::boxes::array::ArrayBox;

/// File values in Nyash - file system operations
#[derive(Debug, Clone)]
pub struct FileBox {
    pub path: String,
    base: BoxBase,
}

impl FileBox {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            base: BoxBase::new(),
        }
    }

    // ===== File Methods for Nyash =====

    /// Read file contents as string
    pub fn read(&self) -> Box<dyn NyashBox> {
        match fs::read_to_string(&self.path) {
            Ok(content) => Box::new(StringBox::new(content)),
            Err(_) => Box::new(VoidBox::new()), // Return void on error for now
        }
    }

    /// Write content to file
    pub fn write(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let content_str = content.to_string_box().value;
        match fs::write(&self.path, content_str) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
        }
    }

    /// Check if file exists
    pub fn exists(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(Path::new(&self.path).exists()))
    }

    /// Delete file
    pub fn delete(&self) -> Box<dyn NyashBox> {
        match fs::remove_file(&self.path) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
        }
    }

    /// Copy file to destination
    pub fn copy(&self, dest_path: &str) -> Box<dyn NyashBox> {
        match fs::copy(&self.path, dest_path) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
        }
    }
}

impl BoxCore for FileBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<FileBox: {}>", self.path)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for FileBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<FileBox: {}>", self.path))
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBox>() {
            BoolBox::new(self.path == other_file.path)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "FileBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for FileBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// Error values in Nyash - represents error information
#[derive(Debug, Clone)]
pub struct ErrorBox {
    pub error_type: String,
    pub message: String,
    base: BoxBase,
}

impl ErrorBox {
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for ErrorBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for ErrorBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("{}: {}", self.error_type, self.message))
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_error) = other.as_any().downcast_ref::<ErrorBox>() {
            BoolBox::new(
                self.error_type == other_error.error_type && self.message == other_error.message,
            )
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "ErrorBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for ErrorBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// FutureBox is now implemented in src/boxes/future/mod.rs using RwLock pattern
// and re-exported from src/boxes/mod.rs as both NyashFutureBox and FutureBox

// Re-export operation boxes from the dedicated operations module
pub use crate::box_arithmetic::{
    AddBox, CompareBox, DivideBox, ModuloBox, MultiplyBox, SubtractBox,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_box_creation() {
        let s = StringBox::new("Hello, Rust!");
        assert_eq!(s.value, "Hello, Rust!");
        assert_eq!(s.type_name(), "StringBox");
        assert_eq!(s.to_string_box().value, "Hello, Rust!");
    }

    #[test]
    fn test_integer_box_creation() {
        let i = IntegerBox::new(42);
        assert_eq!(i.value, 42);
        assert_eq!(i.type_name(), "IntegerBox");
        assert_eq!(i.to_string_box().value, "42");
    }

    #[test]
    fn test_bool_box_creation() {
        let b = BoolBox::new(true);
        assert_eq!(b.value, true);
        assert_eq!(b.type_name(), "BoolBox");
        assert_eq!(b.to_string_box().value, "true");
    }

    #[test]
    fn test_box_equality() {
        let s1 = StringBox::new("test");
        let s2 = StringBox::new("test");
        let s3 = StringBox::new("different");

        assert!(s1.equals(&s2).value);
        assert!(!s1.equals(&s3).value);
    }

    #[test]
    fn test_add_box_integers() {
        let left = Box::new(IntegerBox::new(5)) as Box<dyn NyashBox>;
        let right = Box::new(IntegerBox::new(3)) as Box<dyn NyashBox>;
        let add = AddBox::new(left, right);

        let result = add.execute();
        let result_int = result.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(result_int.value, 8);
    }

    #[test]
    fn test_add_box_strings() {
        let left = Box::new(StringBox::new("Hello, ")) as Box<dyn NyashBox>;
        let right = Box::new(StringBox::new("Rust!")) as Box<dyn NyashBox>;
        let add = AddBox::new(left, right);

        let result = add.execute();
        let result_str = result.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(result_str.value, "Hello, Rust!");
    }

    #[test]
    fn test_box_ids_unique() {
        let s1 = StringBox::new("test");
        let s2 = StringBox::new("test");

        // Same content but different IDs
        assert_ne!(s1.box_id(), s2.box_id());
    }

    #[test]
    fn test_void_box() {
        let v = VoidBox::new();
        assert_eq!(v.type_name(), "VoidBox");
        assert_eq!(v.to_string_box().value, "void");
    }
}
