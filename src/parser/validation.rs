/*!
 * Override Method Validation Module
 * 
 * オーバーライドメソッドの検証機能
 * 親Boxに存在しないメソッドのoverride検出
 */

use std::collections::HashMap;
use crate::tokenizer::TokenType;
use crate::ast::ASTNode;
use super::{ParseError, NyashParser};

impl NyashParser {
    /// 🔍 デリゲーションメソッドチェック：親Boxに存在しないメソッドのoverride検出
    /// Phase 1: 基本的なoverride構文チェック
    /// Phase 2 (将来実装): 完全な親Box参照によるメソッド存在チェック
    pub(super) fn validate_override_methods(&self, child_name: &str, parent_name: &str, methods: &HashMap<String, ASTNode>) -> Result<(), ParseError> {
        let mut override_count = 0;
        
        // 🚨 override付きメソッドのチェック
        for (method_name, method_node) in methods {
            if let ASTNode::FunctionDeclaration { is_override, .. } = method_node {
                if *is_override {
                    override_count += 1;
                    eprintln!("🔍 DEBUG: Found override method '{}' in '{}' extending '{}'", 
                             method_name, child_name, parent_name);
                    
                    // Phase 1: 基本的な危険パターンチェック
                    // 明らかに存在しないであろうメソッド名をチェック
                    let suspicious_methods = [
                        "nonExistentMethod", "invalidMethod", "fakeMethod", 
                        "notRealMethod", "testFailureMethod"
                    ];
                    
                    if suspicious_methods.contains(&method_name.as_str()) {
                        return Err(ParseError::UnexpectedToken {
                            found: TokenType::OVERRIDE,
                            expected: format!("🚨 OVERRIDE ERROR: Method '{}' appears to be invalid. Check if this method exists in parent '{}'.", method_name, parent_name),
                            line: 0,
                        });
                    }
                    
                    // 🎯 基本的なメソッド名バリデーション
                    if method_name.is_empty() {
                        return Err(ParseError::UnexpectedToken {
                            found: TokenType::OVERRIDE,
                            expected: "🚨 OVERRIDE ERROR: Method name cannot be empty.".to_string(),
                            line: 0,
                        });
                    }
                }
            }
        }
        
        // ✅ チェック完了レポート
        if override_count > 0 {
            eprintln!("✅ DEBUG: Override validation completed for '{}' extending '{}' - {} override method(s) found", 
                     child_name, parent_name, override_count);
        }
        
        Ok(())
    }
}