//! Static Box Definition (staged split)
#![allow(dead_code)]

use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};

pub mod header;
pub mod members;
pub mod validators;

/// Facade placeholder for static box parsing (to be wired gradually).
pub(crate) struct StaticDefFacade;

impl StaticDefFacade {
    pub(crate) fn parse_box(_p: &mut NyashParser) -> Result<ASTNode, ParseError> {
        Err(ParseError::UnexpectedToken {
            found: crate::tokenizer::TokenType::EOF,
            expected: "static box declaration (facade not wired)".to_string(),
            line: 0,
        })
    }
}

