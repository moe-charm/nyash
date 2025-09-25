/*!
 * Statement Parser Helper Functions
 *
 * Common utility functions used across statement parsers
 */

use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::parser::cursor::TokenCursor;
use crate::tokenizer::TokenType;

/// Check if token cursor is enabled
pub(super) fn cursor_enabled() -> bool {
    std::env::var("NYASH_PARSER_TOKEN_CURSOR").ok().as_deref() == Some("1")
}

impl NyashParser {

    /// Thin adapter: when Cursor route is enabled, align statement start position
    /// by letting TokenCursor apply its statement-mode newline policy
    pub(super) fn with_stmt_cursor<F>(&mut self, f: F) -> Result<ASTNode, ParseError>
    where
        F: FnOnce(&mut Self) -> Result<ASTNode, ParseError>,
    {
        if cursor_enabled() {
            let mut cursor = TokenCursor::new(&self.tokens);
            cursor.set_position(self.current);
            cursor.with_stmt_mode(|c| {
                // Allow cursor to collapse any leading NEWLINEs in stmt mode
                c.skip_newlines();
            });
            self.current = cursor.position();
        }
        f(self)
    }

    /// Map a starting token into a grammar keyword string used by GRAMMAR_DIFF tracing
    pub(super) fn grammar_keyword_for(start: &TokenType) -> Option<&'static str> {
        match start {
            TokenType::BOX => Some("box"),
            TokenType::GLOBAL => Some("global"),
            TokenType::FUNCTION => Some("function"),
            TokenType::STATIC => Some("static"),
            TokenType::IF => Some("if"),
            TokenType::LOOP => Some("loop"),
            TokenType::BREAK => Some("break"),
            TokenType::RETURN => Some("return"),
            TokenType::PRINT => Some("print"),
            TokenType::NOWAIT => Some("nowait"),
            TokenType::LOCAL => Some("local"),
            TokenType::OUTBOX => Some("outbox"),
            TokenType::TRY => Some("try"),
            TokenType::THROW => Some("throw"),
            TokenType::USING => Some("using"),
            TokenType::FROM => Some("from"),
            _ => None,
        }
    }

    /// Small helper: build UnexpectedToken with current token and line
    pub(super) fn err_unexpected<S: Into<String>>(&self, expected: S) -> ParseError {
        ParseError::UnexpectedToken {
            found: self.current_token().token_type.clone(),
            expected: expected.into(),
            line: self.current_token().line,
        }
    }

    /// Expect an identifier and advance. Returns its string or an UnexpectedToken error
    pub(super) fn expect_identifier(&mut self, what: &str) -> Result<String, ParseError> {
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let out = name.clone();
            self.advance();
            Ok(out)
        } else {
            Err(self.err_unexpected(what))
        }
    }
}