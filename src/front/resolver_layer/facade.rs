//! Resolver facade — stable entry for Phase 2
//! Responsibility: ParserOutput → ResolverInput (name/import resolution)
//! For now, this is a passthrough stub to keep boundaries explicit.

use crate::layers::FrontendError;

/// Passthrough resolver: returns input AST as-is (Phase 2 scaffold).
pub fn resolve_passthrough(ast: crate::ast::ASTNode) -> Result<crate::ast::ASTNode, FrontendError> {
    Ok(ast)
}

