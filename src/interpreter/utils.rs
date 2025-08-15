/*!
 * Utilities Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles utility functions for expressions and operations
 * Core philosophy: "Everything is Box" with helper utilities
 */

use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl NyashInterpreter {
    /// Get object ID from AST node for debugging/tracking
    fn get_object_id(&self, node: &ASTNode) -> Option<usize> {
        match node {
            ASTNode::Variable { name, .. } => {
                Some(self.hash_string(name))
            }
            _ => None,
        }
    }

    /// Hash string to consistent usize for object tracking
    fn hash_string(&self, s: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }
}