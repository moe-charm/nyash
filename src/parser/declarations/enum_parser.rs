/*!
 * Enum Declaration Parser Module
 *
 * @enum マクロのパース処理
 * @enum Name { Variant1(field) Variant2 ... } → EnumDeclaration AST
 */

use crate::ast::{ASTNode, EnumVariant, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse @enum Name { Variant1(field) Variant2 ... }
    /// Called AFTER @ and "enum" tokens are consumed
    pub(crate) fn parse_enum_declaration(&mut self) -> Result<ASTNode, ParseError> {
        let start_line = self.current_token().line;

        // Parse enum name
        let name = match &self.current_token().token_type {
            TokenType::IDENTIFIER(n) => n.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "enum name".to_string(),
                    line: self.current_token().line,
                })
            }
        };
        self.advance(); // consume name

        // Consume {
        self.consume(TokenType::LBRACE)?;

        // Parse variants
        let mut variants = Vec::new();
        loop {
            // Skip newlines
            while self.match_token(&TokenType::NEWLINE) {
                self.advance();
            }

            // Check for }
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // Check for EOF
            if self.is_at_end() {
                return Err(ParseError::UnexpectedEOF);
            }

            // Parse variant
            let variant = self.parse_enum_variant()?;
            variants.push(variant);
        }

        // Consume }
        self.consume(TokenType::RBRACE)?;

        // Validate non-empty
        if variants.is_empty() {
            return Err(ParseError::InvalidStatement { line: start_line });
        }

        Ok(ASTNode::EnumDeclaration {
            name,
            variants,
            span: Span::unknown(),
        })
    }

    /// Parse Variant(field1, field2) or Variant
    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
        // Parse variant name
        let name = match &self.current_token().token_type {
            TokenType::IDENTIFIER(n) => n.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "variant name".to_string(),
                    line: self.current_token().line,
                })
            }
        };
        self.advance(); // consume variant name

        // Check for (
        if !self.match_token(&TokenType::LPAREN) {
            // Zero-field variant
            return Ok(EnumVariant {
                name,
                fields: Vec::new(),
                span: Span::unknown(),
            });
        }

        // Consume (
        self.advance();

        // Parse field list
        let mut fields = Vec::new();
        loop {
            // Check for )
            if self.match_token(&TokenType::RPAREN) {
                break;
            }

            // Parse field name
            match &self.current_token().token_type {
                TokenType::IDENTIFIER(f) => {
                    fields.push(f.clone());
                    self.advance();
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "field name or ')'".to_string(),
                        line: self.current_token().line,
                    })
                }
            }

            // Check for comma
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                // Allow trailing comma
                if self.match_token(&TokenType::RPAREN) {
                    break;
                }
            } else {
                // No comma = must be end of list
                break;
            }
        }

        // Consume )
        self.consume(TokenType::RPAREN)?;

        Ok(EnumVariant {
            name,
            fields,
            span: Span::unknown(),
        })
    }
}
