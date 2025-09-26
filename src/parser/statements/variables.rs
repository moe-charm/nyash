/*!
 * Variable Declaration and Assignment Parsers
 *
 * Handles parsing of:
 * - local variable declarations
 * - outbox variable declarations
 * - assignment statements
 */

use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::parser::cursor::TokenCursor;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse variable declaration statement dispatch
    pub(super) fn parse_variable_declaration_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::LOCAL => self.parse_local(),
            TokenType::OUTBOX => self.parse_outbox(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "variable declaration".to_string(),
                line: self.current_token().line,
            }),
        }
    }

    /// Parse local variable declaration: local var1, var2, var3 or local x = 10
    pub(super) fn parse_local(&mut self) -> Result<ASTNode, ParseError> {
        if super::helpers::cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| c.skip_newlines());
            self.current = cursor.position();
        }
        self.advance(); // consume 'local'

        let mut names = Vec::new();
        let mut initial_values = Vec::new();

        // Get first variable name
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            names.push(name.clone());
            self.advance();

            // Check for initialization
            if self.match_token(&TokenType::ASSIGN) {
                self.advance(); // consume '='
                initial_values.push(Some(Box::new(self.parse_expression()?)));

                // With initialization, only single variable allowed
                Ok(ASTNode::Local {
                    variables: names,
                    initial_values,
                    span: Span::unknown(),
                })
            } else {
                // Without initialization, comma-separated variables allowed
                initial_values.push(None);

                // Parse additional comma-separated variables
                while self.match_token(&TokenType::COMMA) {
                    self.advance(); // consume ','

                    if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                        names.push(name.clone());
                        initial_values.push(None);
                        self.advance();
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "identifier".to_string(),
                            line: self.current_token().line,
                        });
                    }
                }

                Ok(ASTNode::Local {
                    variables: names,
                    initial_values,
                    span: Span::unknown(),
                })
            }
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line: self.current_token().line,
            })
        }
    }

    /// Parse outbox variable declaration: outbox var1, var2, var3
    pub(super) fn parse_outbox(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'outbox'

        let mut names = Vec::new();

        // Get first variable name
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            names.push(name.clone());
            self.advance();

            // Parse additional comma-separated variables
            while self.match_token(&TokenType::COMMA) {
                self.advance(); // consume ','

                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    names.push(name.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "identifier".to_string(),
                        line: self.current_token().line,
                    });
                }
            }

            let len = names.len();
            Ok(ASTNode::Outbox {
                variables: names,
                initial_values: vec![None; len],
                span: Span::unknown(),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line: self.current_token().line,
            })
        }
    }
}