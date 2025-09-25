/*!
 * Exception Handling Statement Parsers
 *
 * Handles parsing of:
 * - try-catch statements
 * - throw statements
 * - cleanup (finally) blocks
 */

use crate::ast::{ASTNode, CatchClause, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse exception statement dispatch
    pub(super) fn parse_exception_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::TRY => self.parse_try_catch(),
            TokenType::THROW => self.parse_throw(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "exception statement".to_string(),
                line: self.current_token().line,
            }),
        }
    }

    /// Parse try-catch statement
    pub(super) fn parse_try_catch(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'try'
        let try_body = self.parse_block_statements()?;

        let mut catch_clauses = Vec::new();

        // Parse catch clauses
        while self.match_token(&TokenType::CATCH) {
            self.advance(); // consume 'catch'
            self.consume(TokenType::LPAREN)?;
            let (exception_type, exception_var) = self.parse_catch_param()?;
            self.consume(TokenType::RPAREN)?;
            let catch_body = self.parse_block_statements()?;

            catch_clauses.push(CatchClause {
                exception_type,
                variable_name: exception_var,
                body: catch_body,
                span: Span::unknown(),
            });
        }

        // Parse optional cleanup (finally) clause
        let finally_body = if self.match_token(&TokenType::CLEANUP) {
            self.advance(); // consume 'cleanup'
            Some(self.parse_block_statements()?)
        } else {
            None
        };

        Ok(ASTNode::TryCatch {
            try_body,
            catch_clauses,
            finally_body,
            span: Span::unknown(),
        })
    }

    /// Parse throw statement
    pub(super) fn parse_throw(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'throw'
        let value = Box::new(self.parse_expression()?);
        Ok(ASTNode::Throw {
            expression: value,
            span: Span::unknown(),
        })
    }

    /// Parse catch parameter: (ExceptionType varName) or (varName) or ()
    pub(crate) fn parse_catch_param(&mut self) -> Result<(Option<String>, Option<String>), ParseError> {
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(first) => {
                let first_str = first.clone();
                let two_idents = matches!(self.peek_token(), TokenType::IDENTIFIER(_));
                if two_idents {
                    self.advance(); // consume type identifier
                    if let TokenType::IDENTIFIER(var_name) = &self.current_token().token_type {
                        let var = var_name.clone();
                        self.advance();
                        Ok((Some(first_str), Some(var)))
                    } else {
                        Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "exception variable name".to_string(),
                            line: self.current_token().line,
                        })
                    }
                } else {
                    self.advance();
                    Ok((None, Some(first_str)))
                }
            }
            _ => {
                if self.match_token(&TokenType::RPAREN) {
                    Ok((None, None))
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: ") or identifier".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
        }
    }

    /// Parse postfix catch/cleanup error handler
    pub(super) fn parse_postfix_catch_cleanup_error(&mut self) -> Result<ASTNode, ParseError> {
        Err(ParseError::UnexpectedToken {
            found: self.current_token().token_type.clone(),
            expected: "catch/cleanup must follow a try block or standalone block".to_string(),
            line: self.current_token().line,
        })
    }
}