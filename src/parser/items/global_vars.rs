/*!
 * Global variable parsing
 */

use crate::ast::{ASTNode, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// グローバル変数をパース: global name = value
    pub fn parse_global_var(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::GLOBAL)?;

        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };

        self.consume(TokenType::ASSIGN)?;
        let value = Box::new(self.parse_expression()?);

        Ok(ASTNode::GlobalVar {
            name,
            value,
            span: Span::unknown(),
        })
    }
}
