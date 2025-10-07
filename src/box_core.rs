/*!
 * Core Box Trait Definitions
 *
 * This module contains the fundamental trait definitions for Nyash's "Everything is Box"
 * philosophy. These traits rarely change and form the stable foundation of the type system.
 *
 * Split from box_trait.rs to reduce compilation cascades - changes to concrete implementations
 * won't trigger recompilation of all files that only need these trait definitions.
 */

use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ===== Type Aliases =====

/// 🔥 新しい型エイリアス - 将来的にBox<dyn NyashBox>を全て置き換える
pub type SharedNyashBox = Arc<dyn NyashBox>;

// ===== ID Generation System =====

/// 🔥 BoxBase + BoxCore革命 - 統一ID生成システム
/// CharmFlow教訓を活かした互換性保証の基盤
pub fn next_box_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ===== Core Structs =====

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

// ===== Core Traits =====

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
    fn to_string_box(&self) -> crate::boxes::basic::StringBox;

    /// Check equality with another box (equivalent to Python's equals())
    fn equals(&self, other: &dyn NyashBox) -> crate::boxes::basic::BoolBox;

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
