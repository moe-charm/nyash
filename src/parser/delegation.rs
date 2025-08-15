/*!
 * Nyash Parser - Delegation Module
 * 
 * from構文（デリゲーション呼び出し）の解析を担当するモジュール
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use super::{NyashParser, ParseError};

// Macro for infinite loop detection
macro_rules! must_advance {
    ($parser:expr, $fuel:expr, $location:literal) => {
        // デバッグ燃料がSomeの場合のみ制限チェック
        if let Some(ref mut limit) = $parser.debug_fuel {
            if *limit == 0 {
                eprintln!("🚨 PARSER INFINITE LOOP DETECTED at {}", $location);
                eprintln!("🔍 Current token: {:?} at line {}", $parser.current_token().token_type, $parser.current_token().line);
                eprintln!("🔍 Parser position: {}/{}", $parser.current, $parser.tokens.len());
                return Err(ParseError::InfiniteLoop { 
                    location: $location.to_string(),
                    token: $parser.current_token().token_type.clone(),
                    line: $parser.current_token().line,
                });
            }
            *limit -= 1;
        }
        // None の場合は無制限なのでチェックしない
    };
}

impl NyashParser {
    /// from構文をパース: from Parent.method(arguments)
    pub(super) fn parse_from_call(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'from'
        
        // Parent名を取得
        let parent = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "parent class name".to_string(),
                line,
            });
        };
        
        // DOT とmethod名は任意（pack透明化対応）
        let method = if self.match_token(&TokenType::DOT) {
            // DOTがある場合: from Parent.method() 形式
            self.advance(); // consume DOT
            
            // method名を取得 (IDENTIFIERまたはINITを受け入れ)
            match &self.current_token().token_type {
                TokenType::IDENTIFIER(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                TokenType::INIT => {
                    self.advance();
                    "init".to_string()
                }
                TokenType::PACK => {
                    self.advance();
                    "pack".to_string()
                }
                TokenType::BIRTH => {
                    self.advance();
                    "birth".to_string()
                }
                _ => {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "method name".to_string(),
                        line,
                    });
                }
            }
        } else {
            // DOTがない場合: from Parent() 形式 - 透明化システム廃止
            // Phase 8.9: 明示的birth()構文を強制
            let line = self.current_token().line;
            return Err(ParseError::TransparencySystemRemoved {
                suggestion: format!("Use 'from {}.birth()' instead of 'from {}()'", parent, parent),
                line,
            });
        };
        
        // 引数リストをパース
        self.consume(TokenType::LPAREN)?;
        let mut arguments = Vec::new();
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "from call argument parsing");
            
            arguments.push(self.parse_expression()?);
            
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                // カンマの後の trailing comma をチェック
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        Ok(ASTNode::FromCall {
            parent,
            method,
            arguments,
            span: Span::unknown(),
        })
    }
}