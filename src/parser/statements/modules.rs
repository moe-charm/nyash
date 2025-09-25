/*!
 * Module System Statement Parsers
 *
 * Handles parsing of:
 * - import statements
 * - using statements (namespace)
 * - from statements (delegation)
 */

use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse import statement: import "path" (as Alias)?
    pub(super) fn parse_import(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'import'

        let path = if let TokenType::STRING(s) = &self.current_token().token_type {
            let v = s.clone();
            self.advance();
            v
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "string literal".to_string(),
                line: self.current_token().line,
            });
        };

        // Optional: 'as' Alias
        let mut alias: Option<String> = None;
        if let TokenType::IDENTIFIER(w) = &self.current_token().token_type {
            if w == "as" {
                self.advance();
                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    alias = Some(name.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "alias name".to_string(),
                        line: self.current_token().line,
                    });
                }
            }
        }

        Ok(ASTNode::ImportStatement {
            path,
            alias,
            span: Span::unknown(),
        })
    }

    /// Parse using statement: using namespace_name
    pub(super) fn parse_using(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'using'

        // Get namespace name
        if let TokenType::IDENTIFIER(namespace_name) = &self.current_token().token_type {
            let name = namespace_name.clone();
            self.advance();

            // Phase 0 only allows "nyashstd"
            if name != "nyashstd" {
                return Err(ParseError::UnsupportedNamespace {
                    name,
                    line: self.current_token().line,
                });
            }

            Ok(ASTNode::UsingStatement {
                namespace_name: name,
                span: Span::unknown(),
            })
        } else {
            Err(ParseError::ExpectedIdentifier {
                line: self.current_token().line,
            })
        }
    }

    /// Parse from statement: from Parent.method(args)
    /// Delegates to the existing parse_from_call() expression parser
    pub(super) fn parse_from_call_statement(&mut self) -> Result<ASTNode, ParseError> {
        // Use existing parse_from_call() to create FromCall AST node
        let from_call_expr = self.parse_from_call()?;

        // FromCall can be used as both expression and statement
        // Example: from Animal.constructor() (return value unused)
        Ok(from_call_expr)
    }
}