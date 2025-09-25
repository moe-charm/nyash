/*!
 * I/O and Async Statement Parsers
 *
 * Handles parsing of:
 * - print statements
 * - nowait statements
 */

use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::parser::cursor::TokenCursor;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse I/O and module-related statement dispatch
    pub(super) fn parse_io_module_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::PRINT => self.parse_print(),
            TokenType::NOWAIT => self.parse_nowait(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "io/module statement".to_string(),
                line: self.current_token().line,
            }),
        }
    }

    /// Parse print statement
    pub(super) fn parse_print(&mut self) -> Result<ASTNode, ParseError> {
        if super::helpers::cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| c.skip_newlines());
            self.current = cursor.position();
        }
        self.advance(); // consume 'print'
        self.consume(TokenType::LPAREN)?;
        let value = Box::new(self.parse_expression()?);
        self.consume(TokenType::RPAREN)?;

        Ok(ASTNode::Print {
            expression: value,
            span: Span::unknown(),
        })
    }

    /// Parse nowait statement: nowait variable = expression
    pub(super) fn parse_nowait(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'nowait'

        // Get variable name
        let variable = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let var = name.clone();
            self.advance();
            var
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "variable name".to_string(),
                line: self.current_token().line,
            });
        };

        self.consume(TokenType::ASSIGN)?;
        let expression = Box::new(self.parse_expression()?);

        Ok(ASTNode::Nowait {
            variable,
            expression,
            span: Span::unknown(),
        })
    }
}