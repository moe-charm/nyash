/*!
 * Nyash Box Trait System - Everything is Box in Rust
 *
 * This module implements the core "Everything is Box" philosophy using Rust's
 * ownership system and trait system. Every value in Nyash is a Box that
 * implements the NyashBox trait.
 */

use std::any::Any;
use std::fmt::Debug;
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

// ===== Basic Box Types (Re-exported from basic module) =====

// Re-export all basic box types from the dedicated basic module
pub use crate::boxes::basic::{
    BoolBox, ErrorBox, FileBox, IntegerBox, StringBox, VoidBox,
};







// Old Box implementations have been moved to separate files
// ArrayBox is now defined in boxes::array module
pub use crate::boxes::array::ArrayBox;

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
