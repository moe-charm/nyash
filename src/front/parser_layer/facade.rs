//! Parser facade — stable entry for Phase 2
//! Responsibility: source text → AST (ParserOutput)

use crate::layers::FrontendError;

/// Parse Nyash/Hakorune source into AST using the existing parser.
pub fn parse_source_to_ast(src: &str) -> Result<crate::ast::ASTNode, FrontendError> {
    crate::parser::NyashParser::parse_from_string(src)
        .map_err(|e| FrontendError::new(format!("{}", e)))
}

