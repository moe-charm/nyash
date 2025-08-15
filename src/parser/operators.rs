/*!
 * Nyash Parser - Operator Parsing Module
 * 
 * 演算子の解析を担当するモジュール
 * 演算子の優先順位に従った再帰下降パーサー実装
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, BinaryOperator, UnaryOperator, Span};
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
    /// OR演算子をパース: ||
    pub(super) fn parse_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&TokenType::OR) {
            let operator = BinaryOperator::Or;
            self.advance();
            let right = self.parse_and()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// AND演算子をパース: &&
    pub(super) fn parse_and(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&TokenType::AND) {
            let operator = BinaryOperator::And;
            self.advance();
            let right = self.parse_equality()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 等値演算子をパース: == !=
    pub(super) fn parse_equality(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_comparison()?;
        
        while self.match_token(&TokenType::EQUALS) || self.match_token(&TokenType::NotEquals) {
            let operator = match &self.current_token().token_type {
                TokenType::EQUALS => BinaryOperator::Equal,
                TokenType::NotEquals => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 比較演算子をパース: < <= > >=
    pub(super) fn parse_comparison(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_term()?;
        
        while self.match_token(&TokenType::LESS) || 
              self.match_token(&TokenType::LessEquals) ||
              self.match_token(&TokenType::GREATER) ||
              self.match_token(&TokenType::GreaterEquals) {
            let operator = match &self.current_token().token_type {
                TokenType::LESS => BinaryOperator::Less,
                TokenType::LessEquals => BinaryOperator::LessEqual,
                TokenType::GREATER => BinaryOperator::Greater,
                TokenType::GreaterEquals => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 項をパース: + - >>
    pub(super) fn parse_term(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_factor()?;
        
        while self.match_token(&TokenType::PLUS) || self.match_token(&TokenType::MINUS) || self.match_token(&TokenType::ARROW) {
            if self.match_token(&TokenType::ARROW) {
                // >> Arrow演算子
                self.advance();
                let right = self.parse_factor()?;
                expr = ASTNode::Arrow {
                    sender: Box::new(expr),
                    receiver: Box::new(right),
                    span: Span::unknown(),
                };
            } else {
                let operator = match &self.current_token().token_type {
                    TokenType::PLUS => BinaryOperator::Add,
                    TokenType::MINUS => BinaryOperator::Subtract,
                    _ => unreachable!(),
                };
                self.advance();
                let right = self.parse_factor()?;
                expr = ASTNode::BinaryOp {
                    operator,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span: Span::unknown(),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// 因子をパース: * /
    pub(super) fn parse_factor(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_unary()?;
        
        while self.match_token(&TokenType::MULTIPLY) || self.match_token(&TokenType::DIVIDE) || self.match_token(&TokenType::MODULO) {
            let operator = match &self.current_token().token_type {
                TokenType::MULTIPLY => BinaryOperator::Multiply,
                TokenType::DIVIDE => BinaryOperator::Divide,
                TokenType::MODULO => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 単項演算子をパース
    pub(super) fn parse_unary(&mut self) -> Result<ASTNode, ParseError> {
        if self.match_token(&TokenType::MINUS) {
            self.advance(); // consume '-'
            let operand = self.parse_unary()?; // 再帰的に単項演算をパース
            return Ok(ASTNode::UnaryOp {
                operator: UnaryOperator::Minus,
                operand: Box::new(operand),
                span: Span::unknown(),
            });
        }
        
        if self.match_token(&TokenType::NOT) {
            self.advance(); // consume 'not'
            let operand = self.parse_unary()?; // 再帰的に単項演算をパース
            return Ok(ASTNode::UnaryOp {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
                span: Span::unknown(),
            });
        }
        
        if self.match_token(&TokenType::AWAIT) {
            return self.parse_await();
        }
        
        self.parse_call()
    }
}