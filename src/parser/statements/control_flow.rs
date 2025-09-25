/*!
 * Control Flow Statement Parsers
 *
 * Handles parsing of control flow statements:
 * - if/else statements
 * - loop statements
 * - break/continue statements
 * - return statements
 */

use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::parser::cursor::TokenCursor;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse control flow statement dispatch
    pub(super) fn parse_control_flow_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::IF => self.parse_if(),
            TokenType::LOOP => self.parse_loop(),
            TokenType::BREAK => self.parse_break(),
            TokenType::CONTINUE => self.parse_continue(),
            TokenType::RETURN => self.parse_return(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "control flow statement".to_string(),
                line: self.current_token().line,
            }),
        }
    }

    /// Parse if statement: if (condition) { body } else if ... else { body }
    pub(super) fn parse_if(&mut self) -> Result<ASTNode, ParseError> {
        // Thin-adapt statement start when Cursor route is enabled
        if super::helpers::cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| c.skip_newlines());
            self.current = cursor.position();
        }
        self.advance(); // consume 'if'

        // Parse condition
        let condition = Box::new(self.parse_expression()?);

        // Parse then body
        let then_body = self.parse_block_statements()?;

        // Parse else if/else
        let else_body = if self.match_token(&TokenType::ELSE) {
            self.advance(); // consume 'else'

            if self.match_token(&TokenType::IF) {
                // else if - parse as nested if
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                // plain else
                Some(self.parse_block_statements()?)
            }
        } else {
            None
        };

        Ok(ASTNode::If {
            condition,
            then_body,
            else_body,
            span: Span::unknown(),
        })
    }

    /// Parse loop statement
    pub(super) fn parse_loop(&mut self) -> Result<ASTNode, ParseError> {
        if super::helpers::cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| c.skip_newlines());
            self.current = cursor.position();
        }
        self.advance(); // consume 'loop'

        // Parse optional condition: loop(condition) or loop { ... }
        let condition = if self.match_token(&TokenType::LPAREN) {
            self.advance(); // consume '('
            let cond = Box::new(self.parse_expression()?);
            self.consume(TokenType::RPAREN)?;
            cond
        } else {
            // default: true for infinite loop
            Box::new(ASTNode::Literal {
                value: crate::ast::LiteralValue::Bool(true),
                span: Span::unknown(),
            })
        };

        // Parse body
        let body = self.parse_block_statements()?;

        Ok(ASTNode::Loop {
            condition,
            body,
            span: Span::unknown(),
        })
    }

    /// Parse break statement
    pub(super) fn parse_break(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'break'
        Ok(ASTNode::Break {
            span: Span::unknown(),
        })
    }

    /// Parse continue statement
    pub(super) fn parse_continue(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'continue'
        Ok(ASTNode::Continue {
            span: Span::unknown(),
        })
    }

    /// Parse return statement
    pub(super) fn parse_return(&mut self) -> Result<ASTNode, ParseError> {
        if super::helpers::cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| c.skip_newlines());
            self.current = cursor.position();
        }
        self.advance(); // consume 'return'

        // Check if there's a return value
        let value = if self.is_at_end() || self.match_token(&TokenType::RBRACE) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        Ok(ASTNode::Return {
            value,
            span: Span::unknown(),
        })
    }
}