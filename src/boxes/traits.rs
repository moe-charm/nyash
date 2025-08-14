/*!
 * Core Box Traits - Essential trait definitions for "Everything is Box"
 * 
 * This module contains the core trait definitions and base structures
 * that all Box types must implement.
 */

use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 🔥 新しい型エイリアス - 将来的にBox<dyn NyashBox>を全て置き換える
pub type SharedNyashBox = Arc<dyn NyashBox>;

/// 🔥 BoxBase + BoxCore革命 - 統一ID生成システム
/// CharmFlow教訓を活かした互換性保証の基盤
pub fn next_box_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
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

/// 🚀 BoxCore - 全てのBoxが持つ基盤的な操作
/// これはフォーマット、型特定、Anyキャストなど基本的な機能
pub trait BoxCore: Send + Sync {
    /// Boxの一意ID取得
    fn box_id(&self) -> u64;
    
    /// 継承元の型ID取得 (ビルトインBox継承)
    fn parent_type_id(&self) -> Option<std::any::TypeId>;
    
    /// フォーマッター用実装 - 内部でto_string_box().valueを使う
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
    
    /// Any型へのキャスト (ダウンキャスト用)
    fn as_any(&self) -> &dyn Any;
    
    /// Mutable Any型へのキャスト
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// 🌟 NyashBox - Nyashの全ての値が実装すべき主要trait
/// BoxCoreを継承し、さらにNyash固有の操作を追加
pub trait NyashBox: BoxCore + Debug {
    /// StringBoxへの変換 (全ての値は文字列表現を持つ)
    fn to_string_box(&self) -> super::string_box::StringBox;
    
    /// IntegerBoxへの変換 (可能な場合)
    fn to_integer_box(&self) -> super::integer_box::IntegerBox;
    
    /// BoolBoxへの変換 (真偽値としての評価)
    fn to_bool_box(&self) -> super::bool_box::BoolBox;
    
    /// 等価性比較
    fn equals(&self, other: &dyn NyashBox) -> bool;
    
    /// 型名取得
    fn type_name(&self) -> &'static str;
    
    /// クローン操作 (Box内での値コピー)
    fn clone_box(&self) -> Box<dyn NyashBox>;
}