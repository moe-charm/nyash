/*!
 * Utility Functions Module
 * 
 * Extracted from expressions.rs lines 1033-1085 (~52 lines)
 * Handles utility functions for object identification and hash calculations
 * Core philosophy: "Everything is Box" with helper utilities
 */

use super::*;

impl NyashInterpreter {
    /// 🔄 循環参照検出: オブジェクトの一意IDを取得
    pub(super) fn get_object_id(&self, node: &ASTNode) -> Option<usize> {
        match node {
            ASTNode::Variable { name, .. } => {
                // 変数名のハッシュをIDとして使用
                Some(self.hash_string(name))
            }
            ASTNode::Me { .. } => {
                // 'me'参照の特別なID
                Some(usize::MAX) 
            }
            ASTNode::This { .. } => {
                // 'this'参照の特別なID  
                Some(usize::MAX - 1)
            }
            _ => None, // 他のノードタイプはID追跡しない
        }
    }
    
    /// 🔄 文字列のシンプルなハッシュ関数
    pub(super) fn hash_string(&self, s: &str) -> usize {
        let mut hash = 0usize;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }
    
    /// 🔗 Convert NyashBox to NyashValue for weak reference operations
    /// Note: Currently commented out due to complexity, to be implemented in future phases
    #[allow(dead_code)]
    fn box_to_nyash_value(&self, _box_val: &Box<dyn NyashBox>) -> Option<crate::value::NyashValue> {
        // This is a placeholder for future weak reference implementation
        // When implemented, this will convert Box types back to NyashValue
        // for proper weak reference storage and management
        None
    }
}