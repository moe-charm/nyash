//! Box Definition parser (scaffold)
#![allow(dead_code)]
//!
//! This module will progressively take over parsing of large `parse_box_declaration`
//! by splitting header and member parsing into focused units.
//! For now, it provides only type skeletons to stage the refactor safely.

use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};

pub mod header;
pub mod members;
pub mod validators;
pub mod interface;

/// Facade to host the staged migration.
pub(crate) struct BoxDefParserFacade;

impl BoxDefParserFacade {
    /// Entry planned: parse full box declaration (header + members).
    /// Not wired yet; use NyashParser::parse_box_declaration for now.
    pub(crate) fn parse_box(_p: &mut NyashParser) -> Result<ASTNode, ParseError> {
        Err(ParseError::UnexpectedToken {
            found: crate::tokenizer::TokenType::EOF,
            expected: "box declaration (facade not wired)".to_string(),
            line: 0,
        })
    }
}
