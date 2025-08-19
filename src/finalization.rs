/*!
 * Nyash Finalization System - Memory Management
 * 
 * fini()によるメモリ管理システムの実装
 * - 解放済みBoxの追跡
 * - スコープベースの自動解放
 * - 再代入時の自動解放
 */

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::fmt;
use crate::box_trait::NyashBox;
use lazy_static::lazy_static;

lazy_static! {
    /// グローバルな解放済みBox ID管理
    static ref FINALIZED_BOXES: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
}

/// Boxが既に解放済みかチェック
pub fn is_finalized(box_id: u64) -> bool {
    FINALIZED_BOXES.lock().unwrap().contains(&box_id)
}

/// Boxを解放済みとして記録
pub fn mark_as_finalized(box_id: u64) {
    FINALIZED_BOXES.lock().unwrap().insert(box_id);
}

/// Box解放管理
pub struct BoxFinalizer {
    /// このスコープで作成されたBox ID
    created_boxes: Vec<(u64, Box<dyn NyashBox>)>,
    /// finalization除外対象のBox ID（関数の返り値など）
    excluded_boxes: HashSet<u64>,
}

impl BoxFinalizer {
    pub fn new() -> Self {
        Self {
            created_boxes: Vec::new(),
            excluded_boxes: HashSet::new(),
        }
    }
    
    /// 新しいBoxを追跡対象に追加
    pub fn track(&mut self, nyash_box: Box<dyn NyashBox>) {
        let box_id = nyash_box.box_id();
        self.created_boxes.push((box_id, nyash_box));
    }
    
    /// 指定したBoxを解放対象から除外（関数の返り値など）
    pub fn exclude_from_finalization(&mut self, nyash_box: &Box<dyn NyashBox>) {
        let box_id = nyash_box.box_id();
        self.excluded_boxes.insert(box_id);
    }
    
    /// スコープ終了時に全てのBoxを解放（除外対象を除く）
    pub fn finalize_all(&mut self) {
        // 作成順（古い順）に解放
        for (box_id, nyash_box) in &self.created_boxes {
            // 除外対象は解放しない
            if self.excluded_boxes.contains(box_id) {
                continue;
            }
            
            if !is_finalized(*box_id) {
                // fini()メソッドを呼び出す（存在する場合）
                if let Some(instance) = nyash_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let _ = instance.fini();
                }
                mark_as_finalized(*box_id);
            }
        }
        self.created_boxes.clear();
        self.excluded_boxes.clear();
    }
}

impl Drop for BoxFinalizer {
    fn drop(&mut self) {
        self.finalize_all();
    }
}

impl fmt::Debug for BoxFinalizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxFinalizer")
            .field("created_boxes_count", &self.created_boxes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_finalization_tracking() {
        let box_id = 12345;
        assert!(!is_finalized(box_id));
        
        mark_as_finalized(box_id);
        assert!(is_finalized(box_id));
        
        // 二重解放チェック
        mark_as_finalized(box_id); // 問題なし
        assert!(is_finalized(box_id));
    }
}